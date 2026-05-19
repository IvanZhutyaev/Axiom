//! Kafka connectors (phase 1: offset-tracking stub; swap for rdkafka in production).

use crate::plugin::{ConnectorError, SinkConnector, SourceConnector};
use async_trait::async_trait;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct KafkaConfig {
    pub brokers: String,
    pub topic: String,
    pub group_id: String,
}

pub struct KafkaSource {
    pub config: KafkaConfig,
    pub offset: u64,
    pub buffer: Vec<Vec<u8>>,
}

impl KafkaSource {
    pub fn new(config: KafkaConfig) -> Self {
        Self {
            config,
            offset: 0,
            buffer: Vec::new(),
        }
    }

    pub fn inject_events(&mut self, events: Vec<Vec<u8>>) {
        self.buffer.extend(events);
    }
}

#[async_trait]
impl SourceConnector for KafkaSource {
    async fn poll(&mut self) -> Result<Option<Vec<u8>>, ConnectorError> {
        if let Some(ev) = self.buffer.pop() {
            self.offset += 1;
            return Ok(Some(ev));
        }
        Ok(None)
    }

    async fn seek(&mut self, offset: u64) -> Result<(), ConnectorError> {
        self.offset = offset;
        Ok(())
    }

    fn offset(&self) -> u64 {
        self.offset
    }
}

pub struct KafkaSink {
    pub config: KafkaConfig,
    dedup: HashSet<String>,
    pub delivered: Vec<(String, Vec<u8>)>,
}

impl KafkaSink {
    pub fn new(config: KafkaConfig) -> Self {
        Self {
            config,
            dedup: HashSet::new(),
            delivered: Vec::new(),
        }
    }
}

#[async_trait]
impl SinkConnector for KafkaSink {
    async fn write(&mut self, payload: Vec<u8>, idempotency_key: &str) -> Result<(), ConnectorError> {
        if !self.dedup.insert(idempotency_key.to_string()) {
            return Ok(());
        }
        self.delivered
            .push((idempotency_key.to_string(), payload));
        Ok(())
    }
}
