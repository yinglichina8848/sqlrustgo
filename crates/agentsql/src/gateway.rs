use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::AgentSqlError;
use crate::schema::SchemaService;
use crate::stats::StatsService;

#[derive(Clone)]
pub struct AppState {
    pub schema_service: Arc<SchemaService>,
    pub stats_service: Arc<StatsService>,
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

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/query", post(handle_query))
        .route("/schema", get(get_schema))
        .route("/schema/:table", get(get_table_schema))
        .route("/stats", get(get_stats))
        .route("/stats/:table", get(get_table_stats))
        .route("/stats/queries", get(get_query_stats))
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

    let state = AppState {
        schema_service,
        stats_service,
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
