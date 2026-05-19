use crate::plugin::{ConnectorError, SinkConnector, SourceConnector};
use async_trait::async_trait;
use std::collections::HashSet;

pub struct MemorySource {
    pub events: Vec<Vec<u8>>,
    pub offset: usize,
}

#[async_trait]
impl SourceConnector for MemorySource {
    async fn poll(&mut self) -> Result<Option<Vec<u8>>, ConnectorError> {
        if self.offset < self.events.len() {
            let ev = self.events[self.offset].clone();
            self.offset += 1;
            Ok(Some(ev))
        } else {
            Ok(None)
        }
    }

    async fn seek(&mut self, offset: u64) -> Result<(), ConnectorError> {
        self.offset = offset as usize;
        Ok(())
    }

    fn offset(&self) -> u64 {
        self.offset as u64
    }
}

pub struct MemorySink {
    pub written: Vec<(String, Vec<u8>)>,
    dedup: HashSet<String>,
}

impl MemorySink {
    pub fn new() -> Self {
        Self {
            written: Vec::new(),
            dedup: HashSet::new(),
        }
    }
}

#[async_trait]
impl SinkConnector for MemorySink {
    async fn write(&mut self, payload: Vec<u8>, idempotency_key: &str) -> Result<(), ConnectorError> {
        if !self.dedup.insert(idempotency_key.to_string()) {
            return Ok(());
        }
        self.written.push((idempotency_key.to_string(), payload));
        Ok(())
    }
}
