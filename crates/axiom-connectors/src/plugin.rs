use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConnectorError {
    #[error("connector error: {0}")]
    Msg(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("http: {0}")]
    Http(String),
}

#[async_trait]
pub trait SourceConnector: Send + Sync {
    async fn poll(&mut self) -> Result<Option<Vec<u8>>, ConnectorError>;
    async fn seek(&mut self, offset: u64) -> Result<(), ConnectorError>;
    fn offset(&self) -> u64;
}

#[async_trait]
pub trait SinkConnector: Send + Sync {
    async fn write(&mut self, payload: Vec<u8>, idempotency_key: &str) -> Result<(), ConnectorError>;
}
