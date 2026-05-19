//! Logical stream plan (optimized).

use aql_syntax::ast::{Program, Stage};

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalNode {
    Source { name: String },
    Sink { name: String },
    Filter { predicate_idx: usize },
    Window {
        kind: String,
        size_ms: u64,
        agg_count: usize,
    },
    Raw(Stage),
}

#[derive(Debug, Clone)]
pub struct LogicalPlan {
    pub nodes: Vec<LogicalNode>,
}

pub fn build_logical_plan(program: &Program) -> LogicalPlan {
    let mut nodes = Vec::new();
    for stage in &program.stages {
        let node = match stage {
            Stage::Source { name } => LogicalNode::Source { name: name.clone() },
            Stage::Sink { name } => LogicalNode::Sink { name: name.clone() },
            Stage::Filter { .. } => LogicalNode::Filter { predicate_idx: 0 },
            Stage::Window {
                kind,
                size_ms,
                aggregates,
                ..
            } => LogicalNode::Window {
                kind: format!("{kind:?}"),
                size_ms: *size_ms,
                agg_count: aggregates.len(),
            },
            other => LogicalNode::Raw(other.clone()),
        };
        nodes.push(node);
    }
    LogicalPlan { nodes }
}

/// Push filters down when consecutive (placeholder optimization).
pub fn optimize(plan: LogicalPlan) -> LogicalPlan {
    plan
}
