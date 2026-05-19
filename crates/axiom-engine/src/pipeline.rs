//! Execute compiled AQL pipelines over event batches.

use avm_bytecode::module::AxcModule;
use avm_runtime::{Interpreter, RunError, RunResult, Value};
use std::collections::HashMap;

pub struct PipelineRunner {
    module: AxcModule,
}

impl PipelineRunner {
    pub fn new(module: AxcModule) -> Self {
        Self { module }
    }

    pub fn run_event(&self, event: Value) -> Result<RunResult, RunError> {
        let mut vm = Interpreter::new(self.module.clone());
        vm.push_event(event);
        vm.run()
    }

    pub fn run_batch(&self, events: Vec<Value>) -> Result<Vec<Value>, RunError> {
        let mut out = Vec::new();
        for ev in events {
            let result = self.run_event(ev)?;
            out.extend(result.emitted);
        }
        Ok(out)
    }

    pub fn json_to_event(value: &serde_json::Value) -> Value {
        let mut map = HashMap::new();
        if let Some(obj) = value.as_object() {
            for (k, v) in obj {
                map.insert(k.clone(), Value::from(v.clone()));
            }
        }
        Value::Struct(map)
    }
}
