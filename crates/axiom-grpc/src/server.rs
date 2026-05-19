use std::net::SocketAddr;
use std::pin::Pin;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

pub mod pb {
    tonic::include_proto!("axiom.v1");
}

use pb::axiom_jobs_server::{AxiomJobs, AxiomJobsServer};
use pb::{JobRequest, JobResponse, MetricsPoint, MetricsRequest, PipelineChunk};

pub struct JobsSvc;

#[tonic::async_trait]
impl AxiomJobs for JobsSvc {
    type StreamMetricsStream =
        Pin<Box<dyn tokio_stream::Stream<Item = Result<MetricsPoint, Status>> + Send>>;

    async fn submit(
        &self,
        request: Request<JobRequest>,
    ) -> Result<Response<JobResponse>, Status> {
        let req = request.into_inner();
        Ok(Response::new(JobResponse {
            id: uuid::Uuid::new_v4().to_string(),
            status: if req.aql.is_empty() {
                "failed".into()
            } else {
                "pending".into()
            },
        }))
    }

    async fn stream_metrics(
        &self,
        _request: Request<MetricsRequest>,
    ) -> Result<Response<Self::StreamMetricsStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        tokio::spawn(async move {
            for i in 0..10 {
                let _ = tx
                    .send(Ok(MetricsPoint {
                        ts_ms: i * 1000,
                        events_per_sec: 1000.0 + i as f64,
                        watermark_lag_ms: 5.0,
                    }))
                    .await;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });
        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    async fn submit_pipeline(
        &self,
        request: Request<tonic::Streaming<PipelineChunk>>,
    ) -> Result<Response<JobResponse>, Status> {
        let mut stream = request.into_inner();
        let mut total = 0usize;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            total += chunk.axc_bytes.len();
        }
        Ok(Response::new(JobResponse {
            id: uuid::Uuid::new_v4().to_string(),
            status: if total > 0 { "accepted".into() } else { "empty".into() },
        }))
    }
}

pub async fn serve_grpc(addr: SocketAddr) -> anyhow::Result<()> {
    Server::builder()
        .add_service(AxiomJobsServer::new(JobsSvc))
        .serve(addr)
        .await?;
    Ok(())
}
