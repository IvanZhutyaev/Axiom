//! Physical pipeline execution with watermark and operator chain.

use avm_bytecode::module::AxcModule;
use crate::interpreter::{Interpreter, RunError};
use crate::value::Value;
use crate::window::WindowBuffer;
use aql_syntax::ast::{Aggregate, Stage, WindowKind};
use aql_syntax::Program;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct WatermarkState {
    pub max_event_time_ms: i64,
    pub watermark_ms: i64,
    pub delay_ms: u64,
    pub field: Option<String>,
}

pub struct PipelineExecutor {
    module: AxcModule,
    watermark: WatermarkState,
    window: Option<WindowBuffer>,
    window_aggs: Vec<Aggregate>,
    time_field: String,
}

impl PipelineExecutor {
    pub fn new(module: AxcModule) -> Self {
        Self {
            module,
            watermark: WatermarkState::default(),
            window: None,
            window_aggs: Vec::new(),
            time_field: "timestamp".into(),
        }
    }

    pub fn configure_from_program(&mut self, program: &Program) {
        for stage in &program.stages {
            match stage {
                Stage::Watermark { field, delay_ms } => {
                    self.watermark.field = Some(field.clone());
                    self.watermark.delay_ms = *delay_ms;
                }
                Stage::Window {
                    kind,
                    size_ms,
                    aggregates,
                    ..
                } if *kind == WindowKind::Tumbling => {
                    self.window = Some(WindowBuffer::new(*size_ms));
                    self.window_aggs = aggregates.clone();
                }
                _ => {}
            }
        }
    }

    pub fn set_watermark(&mut self, field: String, delay_ms: u64) {
        self.watermark.field = Some(field);
        self.watermark.delay_ms = delay_ms;
    }

    pub fn process_event(&mut self, event: Value) -> Result<Vec<Value>, RunError> {
        let event_time = self.event_time_ms(&event);
        self.advance_watermark(event_time);

        let mut vm = Interpreter::new(self.module.clone());
        vm.push_event(event.clone());
        let mut result = vm.run()?;

        if let Some(ref mut win) = self.window {
            win.add(event_time, event);
            if win.should_fire(self.watermark.watermark_ms) {
                result
                    .emitted
                    .push(win.fire(&self.window_aggs));
            }
        }

        Ok(result.emitted)
    }

    fn event_time_ms(&self, event: &Value) -> i64 {
        if let Some(ref field) = self.watermark.field {
            if let Some(Value::Int(ts)) = event.field(field) {
                return ts;
            }
        }
        if let Some(Value::Int(ts)) = event.field(&self.time_field) {
            return ts;
        }
        0
    }

    fn advance_watermark(&mut self, event_time_ms: i64) {
        self.watermark.max_event_time_ms = self.watermark.max_event_time_ms.max(event_time_ms);
        self.watermark.watermark_ms =
            self.watermark.max_event_time_ms - self.watermark.delay_ms as i64;
    }

    pub fn watermark(&self) -> i64 {
        self.watermark.watermark_ms
    }
}

pub fn run_batch(module: AxcModule, events: Vec<Value>) -> Result<Vec<Value>, RunError> {
    let mut exec = PipelineExecutor::new(module);
    let mut out = Vec::new();
    for ev in events {
        out.extend(exec.process_event(ev)?);
    }
    Ok(out)
}

pub fn run_batch_with_program(
    program: &Program,
    module: AxcModule,
    events: Vec<Value>,
) -> Result<Vec<Value>, RunError> {
    let mut exec = PipelineExecutor::new(module);
    exec.configure_from_program(program);
    let mut out = Vec::new();
    for ev in events {
        out.extend(exec.process_event(ev)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aql_compile::compile;
    use std::collections::HashMap;

    #[test]
    fn filter_batch() {
        let c = compile(
            r#"source "s"
|> filter(x > 1.0)
|> sink "o""#,
        )
        .unwrap();
        let mut ev = HashMap::new();
        ev.insert("x".into(), Value::Float(2.0));
        let out = run_batch(c.module, vec![Value::Struct(ev)]).unwrap();
        assert!(!out.is_empty());
    }
}
