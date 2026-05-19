use crate::plugin::{ConnectorError, SinkConnector, SourceConnector};
use async_trait::async_trait;

pub struct HttpSource {
    pub url: String,
    pub client: reqwest::Client,
    pub offset: u64,
    pub pending: Vec<Vec<u8>>,
}

impl HttpSource {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: reqwest::Client::new(),
            offset: 0,
            pending: Vec::new(),
        }
    }

    pub async fn fetch_batch(&mut self) -> Result<(), ConnectorError> {
        let body = self
            .client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| ConnectorError::Http(e.to_string()))?
            .bytes()
            .await
            .map_err(|e| ConnectorError::Http(e.to_string()))?;
        self.pending.push(body.to_vec());
        Ok(())
    }
}

#[async_trait]
impl SourceConnector for HttpSource {
    async fn poll(&mut self) -> Result<Option<Vec<u8>>, ConnectorError> {
        if self.pending.is_empty() {
            let _ = self.fetch_batch().await;
        }
        Ok(self.pending.pop())
    }

    async fn seek(&mut self, offset: u64) -> Result<(), ConnectorError> {
        self.offset = offset;
        Ok(())
    }

    fn offset(&self) -> u64 {
        self.offset
    }
}

pub struct HttpSink {
    pub url: String,
    pub client: reqwest::Client,
    pub dedup: std::collections::HashSet<String>,
}

impl HttpSink {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: reqwest::Client::new(),
            dedup: std::collections::HashSet::new(),
        }
    }
}

#[async_trait]
impl SinkConnector for HttpSink {
    async fn write(&mut self, payload: Vec<u8>, idempotency_key: &str) -> Result<(), ConnectorError> {
        if !self.dedup.insert(idempotency_key.to_string()) {
            return Ok(());
        }
        self.client
            .post(&self.url)
            .header("X-Axiom-Idempotency-Key", idempotency_key)
            .body(payload)
            .send()
            .await
            .map_err(|e| ConnectorError::Http(e.to_string()))?
            .error_for_status()
            .map_err(|e| ConnectorError::Http(e.to_string()))?;
        Ok(())
    }
}
