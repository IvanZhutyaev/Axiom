use std::net::SocketAddr;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

pub mod pb {
    tonic::include_proto!("axiom.v1");
}

use pb::axiom_jobs_server::{AxiomJobs, AxiomJobsServer};
use pb::{JobRequest, JobResponse};

pub struct JobsSvc;

#[tonic::async_trait]
impl AxiomJobs for JobsSvc {
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
}

pub async fn serve_grpc(addr: SocketAddr) -> anyhow::Result<()> {
    Server::builder()
        .add_service(AxiomJobsServer::new(JobsSvc))
        .serve(addr)
        .await?;
    Ok(())
}
