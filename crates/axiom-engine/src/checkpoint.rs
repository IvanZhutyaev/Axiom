use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CheckpointId(pub Uuid);

#[derive(Debug, Default)]
pub struct BarrierCoordinator {
    pub barrier_seq: u64,
    pending_acks: HashMap<u64, HashMap<String, bool>>,
    operator_count: usize,
}

impl BarrierCoordinator {
    pub fn new(operator_count: usize) -> Self {
        Self {
            operator_count,
            ..Default::default()
        }
    }

    pub fn inject_barrier(&mut self) -> u64 {
        self.barrier_seq += 1;
        self.pending_acks
            .insert(self.barrier_seq, HashMap::new());
        self.barrier_seq
    }

    pub fn ack(&mut self, barrier: u64, operator_id: &str) -> bool {
        if let Some(map) = self.pending_acks.get_mut(&barrier) {
            map.insert(operator_id.to_string(), true);
            return map.len() >= self.operator_count;
        }
        false
    }

    pub fn all_acked(&self, barrier: u64) -> bool {
        self.pending_acks
            .get(&barrier)
            .is_some_and(|m| m.len() >= self.operator_count)
    }

    pub fn finalize(&mut self, barrier: u64) -> Option<CheckpointId> {
        if self.all_acked(barrier) {
            self.pending_acks.remove(&barrier);
            Some(CheckpointId(Uuid::new_v4()))
        } else {
            None
        }
    }
}
