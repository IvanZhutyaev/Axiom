use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub id: TaskId,
    pub pipeline_axc: Vec<u8>,
    pub key_groups: Vec<KeyGroup>,
    pub parallelism: u32,
    pub preferred_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyGroup {
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub assigned_node: Option<String>,
}

#[derive(Debug, Default)]
pub struct Scheduler {
    tasks: HashMap<String, TaskSpec>,
    node_load: HashMap<String, u32>,
}

impl Scheduler {
    pub fn submit(&mut self, axc: Vec<u8>, parallelism: u32) -> TaskSpec {
        let id = TaskId(Uuid::new_v4().to_string());
        let key_groups = (0..parallelism)
            .map(|i| KeyGroup {
                from: vec![i as u8],
                to: vec![i as u8],
                assigned_node: None,
            })
            .collect();
        let spec = TaskSpec {
            id: id.clone(),
            pipeline_axc: axc,
            key_groups,
            parallelism,
            preferred_nodes: Vec::new(),
        };
        self.tasks.insert(id.0.clone(), spec.clone());
        spec
    }

    pub fn assign_tasks(&mut self, nodes: &[String]) {
        if nodes.is_empty() {
            return;
        }
        for task in self.tasks.values_mut() {
            for (i, kg) in task.key_groups.iter_mut().enumerate() {
                let node = &nodes[i % nodes.len()];
                kg.assigned_node = Some(node.clone());
                *self.node_load.entry(node.clone()).or_insert(0) += 1;
            }
        }
    }

    pub fn rebalance_on_failure(&mut self, failed: &str, nodes: &[String]) {
        let alive: Vec<_> = nodes.iter().filter(|n| *n != failed).cloned().collect();
        self.assign_tasks(&alive);
    }

    pub fn get(&self, id: &str) -> Option<&TaskSpec> {
        self.tasks.get(id)
    }

    pub fn list(&self) -> Vec<&TaskSpec> {
        self.tasks.values().collect()
    }
}
