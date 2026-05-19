//! Tumbling/sliding window aggregation state.

use crate::value::Value;
use aql_syntax::ast::{AggFunc, Aggregate};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct WindowBuffer {
    pub size_ms: u64,
    pub events: Vec<(i64, Value)>,
    pub window_start_ms: i64,
}

impl WindowBuffer {
    pub fn new(size_ms: u64) -> Self {
        Self {
            size_ms,
            ..Default::default()
        }
    }

    pub fn add(&mut self, event_time_ms: i64, event: Value) {
        if self.events.is_empty() {
            self.window_start_ms = event_time_ms;
        }
        self.events.push((event_time_ms, event));
    }

    pub fn should_fire(&self, watermark_ms: i64) -> bool {
        !self.events.is_empty() && watermark_ms >= self.window_start_ms + self.size_ms as i64
    }

    pub fn fire(&mut self, aggregates: &[Aggregate]) -> Value {
        let mut out = HashMap::new();
        for agg in aggregates {
            let v = compute_agg(agg, &self.events);
            out.insert(agg.name.clone(), v);
        }
        self.events.clear();
        self.window_start_ms = 0;
        Value::Struct(out)
    }
}

fn compute_agg(agg: &Aggregate, events: &[(i64, Value)]) -> Value {
    let nums: Vec<f64> = events
        .iter()
        .filter_map(|(_, ev)| field_as_f64(ev, &agg.arg))
        .collect();
    match agg.func {
        AggFunc::Count => Value::Int(events.len() as i64),
        AggFunc::Sum => Value::Float(nums.iter().sum()),
        AggFunc::Avg if !nums.is_empty() => Value::Float(nums.iter().sum::<f64>() / nums.len() as f64),
        AggFunc::Min => Value::Float(nums.iter().copied().fold(f64::INFINITY, f64::min)),
        AggFunc::Max => Value::Float(nums.iter().copied().fold(f64::NEG_INFINITY, f64::max)),
        AggFunc::First => events
            .first()
            .map(|(_, v)| v.clone())
            .unwrap_or(Value::Null),
        AggFunc::Last => events
            .last()
            .map(|(_, v)| v.clone())
            .unwrap_or(Value::Null),
        _ => Value::Float(0.0),
    }
}

fn field_as_f64(ev: &Value, arg: &Option<aql_syntax::ast::Expr>) -> Option<f64> {
    match arg {
        Some(aql_syntax::ast::Expr::Ident(name)) => ev.field(name).map(|v| match v {
            Value::Int(i) => i as f64,
            Value::Float(f) => f,
            _ => 0.0,
        }),
        _ => Some(0.0),
    }
}
