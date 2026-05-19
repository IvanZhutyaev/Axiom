//! Distributed barrier: coordinator inject → operator ack → checkpoint id.

use crate::checkpoint::{BarrierCoordinator, CheckpointId};
use crate::exactly_once::{EventKey, IdempotencyStore};
use uuid::Uuid;

pub struct DistributedBarrier {
    pub coordinator: BarrierCoordinator,
    pub idempotency: IdempotencyStore,
    pub sequence: u64,
}

impl DistributedBarrier {
    pub fn new(operator_count: usize) -> Self {
        Self {
            coordinator: BarrierCoordinator::new(operator_count),
            idempotency: IdempotencyStore::new(),
            sequence: 0,
        }
    }

    pub fn inject(&mut self) -> u64 {
        self.coordinator.inject_barrier()
    }

    pub fn operator_ack(&mut self, barrier: u64, operator_id: &str) -> Option<CheckpointId> {
        self.coordinator.ack(barrier, operator_id);
        self.coordinator.finalize(barrier)
    }

    pub fn process_at_checkpoint(
        &mut self,
        checkpoint: Uuid,
        payload: &[u8],
    ) -> bool {
        self.sequence += 1;
        let key = EventKey {
            checkpoint_id: checkpoint,
            sequence: self.sequence,
        };
        self.idempotency.should_process(key)
            && !payload.is_empty()
    }
}
