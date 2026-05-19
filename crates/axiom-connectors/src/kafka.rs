//! Kafka connectors with optional `rdkafka` backend.

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
    #[cfg(feature = "kafka")]
    consumer: Option<rdkafka::consumer::StreamConsumer>,
}

impl KafkaSource {
    pub fn new(config: KafkaConfig) -> Self {
        Self {
            config,
            offset: 0,
            buffer: Vec::new(),
            #[cfg(feature = "kafka")]
            consumer: None,
        }
    }

    pub fn inject_events(&mut self, events: Vec<Vec<u8>>) {
        self.buffer.extend(events);
    }

    #[cfg(feature = "kafka")]
    pub fn connect_rdkafka(&mut self) -> Result<(), ConnectorError> {
        use rdkafka::config::ClientConfig;
        use rdkafka::consumer::{Consumer, StreamConsumer};
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &self.config.brokers)
            .set("group.id", &self.config.group_id)
            .set("enable.auto.commit", "false")
            .create()
            .map_err(|e| ConnectorError::Msg(e.to_string()))?;
        consumer
            .subscribe(&[&self.config.topic])
            .map_err(|e| ConnectorError::Msg(e.to_string()))?;
        self.consumer = Some(consumer);
        Ok(())
    }
}

#[async_trait]
impl SourceConnector for KafkaSource {
    async fn poll(&mut self) -> Result<Option<Vec<u8>>, ConnectorError> {
        #[cfg(feature = "kafka")]
        if let Some(ref consumer) = self.consumer {
            use futures::StreamExt;
            use rdkafka::consumer::Consumer;
            let mut stream = consumer.stream();
            if let Some(Ok(msg)) = stream.next().await {
                if let Some(payload) = msg.payload() {
                    self.offset = msg.offset().unwrap_or(self.offset) as u64;
                    return Ok(Some(payload.to_vec()));
                }
            }
        }
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
    #[cfg(feature = "kafka")]
    producer: Option<rdkafka::producer::FutureProducer>,
}

impl KafkaSink {
    pub fn new(config: KafkaConfig) -> Self {
        Self {
            config,
            dedup: HashSet::new(),
            delivered: Vec::new(),
            #[cfg(feature = "kafka")]
            producer: None,
        }
    }

    #[cfg(feature = "kafka")]
    pub fn connect_rdkafka(&mut self) -> Result<(), ConnectorError> {
        use rdkafka::config::ClientConfig;
        use rdkafka::producer::FutureProducer;
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &self.config.brokers)
            .create()
            .map_err(|e| ConnectorError::Msg(e.to_string()))?;
        self.producer = Some(producer);
        Ok(())
    }
}

#[async_trait]
impl SinkConnector for KafkaSink {
    async fn write(&mut self, payload: Vec<u8>, idempotency_key: &str) -> Result<(), ConnectorError> {
        if !self.dedup.insert(idempotency_key.to_string()) {
            return Ok(());
        }
        #[cfg(feature = "kafka")]
        if let Some(ref producer) = self.producer {
            use rdkafka::producer::{FutureRecord, Producer};
            let record = FutureRecord::to(&self.config.topic)
                .key(idempotency_key)
                .payload(&payload);
            producer
                .send(record, std::time::Duration::from_secs(5))
                .await
                .map_err(|(e, _)| ConnectorError::Internal(e.to_string()))?;
        }
        self.delivered
            .push((idempotency_key.to_string(), payload));
        Ok(())
    }
}
