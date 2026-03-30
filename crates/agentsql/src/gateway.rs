use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

use crate::error::AgentSqlError;
use crate::memory::{
    ClearMemoryRequest, ClearMemoryResponse, LoadMemoryRequest, LoadMemoryResponse, MemoryService,
    SaveMemoryRequest, SaveMemoryResponse,
};
use crate::nl2sql::SqlExplanation;
use crate::schema::SchemaService;
use crate::stats::StatsService;

#[derive(Clone)]
pub struct AppState {
    pub schema_service: Arc<SchemaService>,
    pub stats_service: Arc<StatsService>,
    pub nl2sql_service: Arc<crate::nl2sql::Nl2SqlService>,
    pub memory_service: Arc<RwLock<MemoryService>>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(Deserialize)]
pub struct QueryRequest {
    pub sql: String,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub success: bool,
    pub data: Option<Vec<Vec<serde_json::Value>>>,
    pub error: Option<String>,
    pub execution_time_ms: Option<u64>,
}

pub async fn handle_query(
    State(_state): State<AppState>,
    Json(_req): Json<QueryRequest>,
) -> Json<QueryResponse> {
    Json(QueryResponse {
        success: true,
        data: None,
        error: None,
        execution_time_ms: Some(5),
    })
}

#[derive(Deserialize)]
pub struct NlQueryRequestDto {
    pub query: String,
    pub context: Option<String>,
}

#[derive(Serialize)]
pub struct NlQueryResponseDto {
    pub success: bool,
    pub sql: Option<String>,
    pub confidence: Option<f32>,
    pub table_hint: Option<String>,
    pub where_conditions: Option<Vec<String>>,
    pub error: Option<String>,
}

pub async fn handle_nl_query(
    State(state): State<AppState>,
    Json(req): Json<NlQueryRequestDto>,
) -> Json<NlQueryResponseDto> {
    let result = state.nl2sql_service.natural_language_to_sql(&req.query);
    Json(NlQueryResponseDto {
        success: result.error.is_none(),
        sql: if result.error.is_none() { Some(result.sql) } else { None },
        confidence: Some(result.confidence),
        table_hint: result.table_hint,
        where_conditions: Some(result.where_conditions),
        error: result.error,
    })
}

#[derive(Deserialize)]
pub struct ExplainRequest {
    pub sql: String,
}

pub async fn handle_explain(
    State(state): State<AppState>,
    Json(req): Json<ExplainRequest>,
) -> Json<SqlExplanation> {
    let explanation = state.nl2sql_service.explain_sql(&req.sql);
    Json(explanation)
}

pub async fn handle_save_memory(
    State(state): State<AppState>,
    Json(req): Json<SaveMemoryRequest>,
) -> Json<SaveMemoryResponse> {
    let mut memory_service = state.memory_service.write();
    let response = memory_service.save_memory(req);
    Json(response)
}

pub async fn handle_load_memory(
    State(state): State<AppState>,
    Json(req): Json<LoadMemoryRequest>,
) -> Json<LoadMemoryResponse> {
    let memory_service = state.memory_service.read();
    let response = memory_service.load_memory(req);
    Json(response)
}

pub async fn handle_search_memory(
    State(state): State<AppState>,
    Json(req): Json<crate::memory::SearchMemoryRequest>,
) -> Json<crate::memory::SearchMemoryResponse> {
    let memory_service = state.memory_service.read();
    let response = memory_service.search_memory(req);
    Json(response)
}

pub async fn handle_clear_memory(
    State(state): State<AppState>,
    Json(req): Json<ClearMemoryRequest>,
) -> Json<ClearMemoryResponse> {
    let mut memory_service = state.memory_service.write();
    let response = memory_service.clear_memory(req);
    Json(response)
}

pub async fn handle_memory_stats(
    State(state): State<AppState>,
) -> Json<crate::memory::MemoryStats> {
    let memory_service = state.memory_service.read();
    let stats = memory_service.get_stats();
    Json(stats)
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/query", post(handle_query))
        .route("/nl_query", post(handle_nl_query))
        .route("/explain", post(handle_explain))
        .route("/schema", get(get_schema))
        .route("/schema/:table", get(get_table_schema))
        .route("/stats", get(get_stats))
        .route("/stats/:table", get(get_table_stats))
        .route("/stats/queries", get(get_query_stats))
        .route("/memory/save", post(handle_save_memory))
        .route("/memory/load", post(handle_load_memory))
        .route("/memory/search", post(handle_search_memory))
        .route("/memory/clear", post(handle_clear_memory))
        .route("/memory/stats", get(handle_memory_stats))
        .with_state(state)
}

pub async fn get_schema(State(state): State<AppState>) -> Json<serde_json::Value> {
    let schema = state.schema_service.get_schema();
    Json(schema)
}

pub async fn get_table_schema(
    State(state): State<AppState>,
    axum::extract::Path(table_name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match state.schema_service.get_table_schema(&table_name) {
        Some(schema) => Json(schema),
        None => Json(serde_json::json!({
            "error": "Table not found",
            "table_name": table_name
        })),
    }
}

pub async fn get_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let stats = state.stats_service.get_stats();
    Json(stats)
}

pub async fn get_table_stats(
    State(state): State<AppState>,
    axum::extract::Path(table_name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match state.stats_service.get_table_stats(&table_name) {
        Some(stats) => Json(stats),
        None => Json(serde_json::json!({
            "error": "Table not found",
            "table_name": table_name
        })),
    }
}

pub async fn get_query_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let stats = state.stats_service.get_query_stats();
    Json(stats)
}

pub async fn start_server(port: u16) -> Result<(), AgentSqlError> {
    let schema_service = Arc::new(SchemaService::new());
    let stats_service = Arc::new(StatsService::new());
    let nl2sql_service = Arc::new(crate::nl2sql::Nl2SqlService::new(schema_service.clone()));
    let memory_service = Arc::new(RwLock::new(MemoryService::new()));

    let state = AppState {
        schema_service,
        stats_service,
        nl2sql_service,
        memory_service,
    };

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .map_err(|e| AgentSqlError::HttpError(e.to_string()))?;

    tracing::info!("AgentSQL Gateway listening on port {}", port);

    axum::serve(listener, create_router(state))
        .await
        .map_err(|e| AgentSqlError::HttpError(e.to_string()))?;

    Ok(())
}
