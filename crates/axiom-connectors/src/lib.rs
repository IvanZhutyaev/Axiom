//! Connector plugin framework and built-in sources/sinks.

pub mod http;
pub mod kafka;
pub mod memory;
pub mod plugin;

pub use http::{HttpSink, HttpSource};
pub use kafka::{KafkaSink, KafkaSource};
pub use memory::{MemorySink, MemorySource};
pub use plugin::{ConnectorError, SinkConnector, SourceConnector};
