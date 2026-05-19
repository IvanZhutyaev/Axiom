//! REST API `/api/v1/*` with JWT auth skeleton.

mod auth;

use aql_compile::compile;
use auth::{authorize, AuthConfig, Claims};
use axiom_engine::PipelineRunner;
use axiom_metrics::Metrics;
use axum::{
    extract::{Path, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use auth::mint_token;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub id: String,
    pub status: JobStatus,
    pub aql: String,
    pub events_processed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMemberView {
    pub id: String,
    pub addr: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRecord {
    pub id: u32,
    pub json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorRecord {
    pub name: String,
    pub kind: String,
    pub config: serde_json::Value,
}

#[derive(Debug)]
pub struct ApiState {
    pub jobs: HashMap<String, JobRecord>,
    pub node_id: String,
    pub members: Vec<ClusterMemberView>,
    pub metrics: Metrics,
    pub schemas: HashMap<u32, SchemaRecord>,
    pub connectors: HashMap<String, ConnectorRecord>,
    pub auth: AuthConfig,
    pub next_schema_id: u32,
}

impl Default for ApiState {
    fn default() -> Self {
        Self {
            jobs: HashMap::new(),
            node_id: "local".into(),
            members: Vec::new(),
            metrics: Metrics::default(),
            schemas: HashMap::new(),
            connectors: HashMap::new(),
            auth: AuthConfig::dev_default(),
            next_schema_id: 1,
        }
    }
}

#[derive(Deserialize)]
pub struct SubmitJobRequest {
    pub aql: String,
    pub name: Option<String>,
    #[serde(default)]
    pub sample_events: Vec<serde_json::Value>,
}

#[derive(Serialize)]
pub struct SubmitJobResponse {
    pub id: String,
    pub status: JobStatus,
    pub events_processed: u64,
}

#[derive(Deserialize)]
pub struct RegisterSchemaRequest {
    pub json: String,
}

#[derive(Deserialize)]
pub struct CreateConnectorRequest {
    pub name: String,
    pub kind: String,
    pub config: serde_json::Value,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub node_id: String,
}

#[derive(Serialize)]
pub struct ClusterStatusResponse {
    pub node_id: String,
    pub members: Vec<ClusterMemberView>,
    pub alive: usize,
}

pub async fn serve(bind: &str, state: Arc<RwLock<ApiState>>) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(prometheus_metrics))
        .route("/api/v1/cluster", get(cluster_status))
        .route("/api/v1/jobs", post(submit_job).get(list_jobs))
        .route("/api/v1/jobs/:id", get(get_job))
        .route("/api/v1/schemas", post(register_schema).get(list_schemas))
        .route("/api/v1/schemas/:id", get(get_schema))
        .route(
            "/api/v1/connectors",
            post(create_connector).get(list_connectors),
        )
        .route("/api/v1/auth/token", post(dev_token))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!("REST API listening on http://{bind}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn check_auth(state: &ApiState, headers: &HeaderMap) -> Result<Claims, StatusCode> {
    let hdr = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let token = hdr.strip_prefix("Bearer ").ok_or(StatusCode::UNAUTHORIZED)?;
    authorize(&state.auth, token).map_err(|_| StatusCode::UNAUTHORIZED)
}

async fn health(State(state): State<Arc<RwLock<ApiState>>>) -> Json<HealthResponse> {
    let s = state.read().await;
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        node_id: s.node_id.clone(),
    })
}

async fn cluster_status(
    State(state): State<Arc<RwLock<ApiState>>>,
    headers: HeaderMap,
) -> Result<Json<ClusterStatusResponse>, StatusCode> {
    let s = state.read().await;
    let _ = check_auth(&s, &headers)?;
    let alive = s.members.iter().filter(|m| m.state == "alive").count();
    Ok(Json(ClusterStatusResponse {
        node_id: s.node_id.clone(),
        members: s.members.clone(),
        alive,
    }))
}

async fn prometheus_metrics(State(state): State<Arc<RwLock<ApiState>>>) -> String {
    state.read().await.metrics.render_prometheus()
}

async fn dev_token(State(state): State<Arc<RwLock<ApiState>>>) -> Json<serde_json::Value> {
    let s = state.read().await;
    let token = mint_token(&s.auth, "developer", "developer");
    Json(serde_json::json!({ "access_token": token, "token_type": "Bearer" }))
}

async fn submit_job(
    State(state): State<Arc<RwLock<ApiState>>>,
    headers: HeaderMap,
    Json(req): Json<SubmitJobRequest>,
) -> Result<Json<SubmitJobResponse>, StatusCode> {
    let mut s = state.write().await;
    let _ = check_auth(&s, &headers)?;
    let compiled = compile(&req.aql).map_err(|_| StatusCode::BAD_REQUEST)?;
    let runner = PipelineRunner::new(compiled.module);
    let mut processed = 0u64;
    for ev in &req.sample_events {
        let result = runner
            .run_event(PipelineRunner::json_to_event(ev))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        processed += result.emitted.len() as u64;
    }
    let id = uuid::Uuid::new_v4().to_string();
    let status = if req.sample_events.is_empty() {
        JobStatus::Pending
    } else {
        JobStatus::Completed
    };
    s.metrics.inc_events(processed);
    s.jobs.insert(
        id.clone(),
        JobRecord {
            id: id.clone(),
            status: status.clone(),
            aql: req.aql,
            events_processed: processed,
        },
    );
    Ok(Json(SubmitJobResponse {
        id,
        status,
        events_processed: processed,
    }))
}

async fn list_jobs(
    State(state): State<Arc<RwLock<ApiState>>>,
    headers: HeaderMap,
) -> Result<Json<Vec<JobRecord>>, StatusCode> {
    let s = state.read().await;
    let _ = check_auth(&s, &headers)?;
    Ok(Json(s.jobs.values().cloned().collect()))
}

async fn get_job(
    State(state): State<Arc<RwLock<ApiState>>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<JobRecord>, StatusCode> {
    let s = state.read().await;
    let _ = check_auth(&s, &headers)?;
    s.jobs.get(&id).cloned().map(Json).ok_or(StatusCode::NOT_FOUND)
}

async fn register_schema(
    State(state): State<Arc<RwLock<ApiState>>>,
    headers: HeaderMap,
    Json(req): Json<RegisterSchemaRequest>,
) -> Result<Json<SchemaRecord>, StatusCode> {
    let mut s = state.write().await;
    let claims = check_auth(&s, &headers)?;
    if claims.role != "admin" && claims.role != "developer" {
        return Err(StatusCode::FORBIDDEN);
    }
    let id = s.next_schema_id;
    s.next_schema_id += 1;
    let rec = SchemaRecord {
        id,
        json: req.json,
    };
    s.schemas.insert(id, rec.clone());
    Ok(Json(rec))
}

async fn list_schemas(
    State(state): State<Arc<RwLock<ApiState>>>,
    headers: HeaderMap,
) -> Result<Json<Vec<SchemaRecord>>, StatusCode> {
    let s = state.read().await;
    let _ = check_auth(&s, &headers)?;
    Ok(Json(s.schemas.values().cloned().collect()))
}

async fn get_schema(
    State(state): State<Arc<RwLock<ApiState>>>,
    headers: HeaderMap,
    Path(id): Path<u32>,
) -> Result<Json<SchemaRecord>, StatusCode> {
    let s = state.read().await;
    let _ = check_auth(&s, &headers)?;
    s.schemas.get(&id).cloned().map(Json).ok_or(StatusCode::NOT_FOUND)
}

async fn create_connector(
    State(state): State<Arc<RwLock<ApiState>>>,
    headers: HeaderMap,
    Json(req): Json<CreateConnectorRequest>,
) -> Result<Json<ConnectorRecord>, StatusCode> {
    let mut s = state.write().await;
    let _ = check_auth(&s, &headers)?;
    let rec = ConnectorRecord {
        name: req.name.clone(),
        kind: req.kind,
        config: req.config,
    };
    s.connectors.insert(req.name, rec.clone());
    Ok(Json(rec))
}

async fn list_connectors(
    State(state): State<Arc<RwLock<ApiState>>>,
    headers: HeaderMap,
) -> Result<Json<Vec<ConnectorRecord>>, StatusCode> {
    let s = state.read().await;
    let _ = check_auth(&s, &headers)?;
    Ok(Json(s.connectors.values().cloned().collect()))
}
