use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventKey {
    pub checkpoint_id: Uuid,
    pub sequence: u64,
}

#[derive(Debug, Default)]
pub struct IdempotencyStore {
    seen: HashSet<EventKey>,
}

impl IdempotencyStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn should_process(&mut self, key: EventKey) -> bool {
        if self.seen.contains(&key) {
            return false;
        }
        self.seen.insert(key);
        true
    }

    pub fn record_sink_write(&mut self, key: EventKey) -> bool {
        self.should_process(key)
    }

    pub fn len(&self) -> usize {
        self.seen.len()
    }
}
