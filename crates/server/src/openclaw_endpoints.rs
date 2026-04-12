//! OpenClaw API Endpoints
//!
//! Provides REST API endpoints for the OpenClaw AI Agent extension.
//! This enables SQLRustGo to act as a knowledge base and SQL engine
//! for AI-powered agent workflows.

use crate::metrics_endpoint::MetricsRegistry;
use crate::scheduler;
use query_stats::StatsCollector;
use serde::{Deserialize, Serialize};
use sqlrustgo_gmp::audit::{self, AuditLog};
use sqlrustgo_gmp::compliance::{ComplianceCheckRequest, ComplianceResult};
use sqlrustgo_gmp::report::{AuditReport, CapaReport, DeviationReport};
use sqlrustgo_optimizer::{
    IndexHint as OptimizerIndexHint, IndexHintType as OptimizerIndexHintType, RuleContext,
};
use sqlrustgo_parser::parse;
use sqlrustgo_rag::{Document, OpenClawClient};
use sqlrustgo_storage::engine::{StorageEngine, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

// ============================================================================
// Request/Response Types
// ============================================================================

/// SQL Query request
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub sql: String,
}

/// SQL Query response
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub success: bool,
    pub data: Option<Vec<Vec<serde_json::Value>>>,
    pub error: Option<String>,
    pub execution_time_ms: Option<u64>,
}

/// Natural language query request
#[derive(Debug, Deserialize)]
pub struct NlQueryRequest {
    pub query: String,
    pub context: Option<String>,
}

/// Natural language query response
#[derive(Debug, Serialize)]
pub struct NlQueryResponse {
    pub success: bool,
    pub sql: Option<String>,
    pub confidence: Option<f32>,
    pub table_hint: Option<String>,
    pub where_conditions: Option<Vec<String>>,
    pub error: Option<String>,
}

/// Table schema response
#[derive(Debug, Serialize)]
pub struct TableSchemaResponse {
    pub name: String,
    pub columns: Vec<ColumnSchemaResponse>,
    pub indexes: Vec<IndexSchemaResponse>,
}

/// Column schema response
#[derive(Debug, Serialize)]
pub struct ColumnSchemaResponse {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub default: Option<String>,
}

/// Index schema response
#[derive(Debug, Serialize)]
pub struct IndexSchemaResponse {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    #[serde(rename = "type")]
    pub index_type: String,
}

/// Full schema response
#[derive(Debug, Serialize)]
pub struct SchemaResponse {
    pub database: String,
    pub version: String,
    pub tables: Vec<TableSchemaResponse>,
    pub views: Vec<ViewSchemaResponse>,
}

/// View schema response
#[derive(Debug, Serialize)]
pub struct ViewSchemaResponse {
    pub name: String,
    pub definition: String,
}

/// Table statistics response
#[derive(Debug, Serialize)]
pub struct TableStatsResponse {
    pub row_count: u64,
    pub size_bytes: u64,
    pub indexes_count: u32,
}

/// Query statistics response
#[derive(Debug, Serialize)]
pub struct QueryStatsResponse {
    pub total_queries: u64,
    pub select_queries: u64,
    pub insert_queries: u64,
    pub update_queries: u64,
    pub delete_queries: u64,
    pub avg_execution_time_ms: f64,
}

/// Full stats response
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub tables: HashMap<String, TableStatsResponse>,
    pub query_statistics: QueryStatsResponse,
}

/// Save memory request
#[derive(Debug, Deserialize)]
pub struct SaveMemoryRequest {
    pub content: String,
    #[serde(default)]
    pub memory_type: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default = "default_importance")]
    pub importance: f32,
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}

fn default_importance() -> f32 {
    0.5
}

fn default_top_k() -> usize {
    10
}

/// Save memory response
#[derive(Debug, Serialize)]
pub struct SaveMemoryResponse {
    pub id: String,
    pub success: bool,
    pub message: String,
}

// ============================================================================
// Unified Query API Types
// ============================================================================

/// Unified query request - combines SQL, vector, and graph queries
#[derive(Debug, Deserialize)]
pub struct UnifiedQueryRequest {
    pub query: String,
    #[serde(default)]
    pub mode: Option<String>, // "sql_vector_graph", "sql_vector", "sql_graph", "vector", "graph"
    #[serde(default)]
    pub filters: Option<UnifiedFilters>,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default)]
    pub vector: Option<VectorQueryConfig>,
    #[serde(default)]
    pub graph: Option<GraphQueryConfig>,
}

/// Unified query filters
#[derive(Debug, Deserialize, Default)]
pub struct UnifiedFilters {
    #[serde(default)]
    pub date_range: Option<DateRange>,
    #[serde(default)]
    pub department: Option<String>,
    #[serde(default)]
    pub custom: Option<HashMap<String, String>>,
}

/// Date range filter
#[derive(Debug, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

/// Vector query configuration
#[derive(Debug, Deserialize)]
pub struct VectorQueryConfig {
    pub column: String,
    pub query_vector: Vec<f32>,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default)]
    pub min_score: Option<f32>,
}

/// Graph query configuration
#[derive(Debug, Deserialize)]
pub struct GraphQueryConfig {
    pub start_node: String,
    #[serde(default)]
    pub traversal: Option<String>, // "bfs", "dfs"
    #[serde(default)]
    pub max_depth: Option<usize>,
    #[serde(default)]
    pub edge_type: Option<String>,
}

/// Unified query response
#[derive(Debug, Serialize)]
pub struct UnifiedQueryResponse {
    pub success: bool,
    pub sql_results: Option<SqlResults>,
    pub vector_results: Option<VectorResults>,
    pub graph_results: Option<GraphResults>,
    pub execution_time_ms: u64,
    pub query_plan: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// SQL results
#[derive(Debug, Serialize)]
pub struct SqlResults {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
}

/// Vector search results
#[derive(Debug, Serialize)]
pub struct VectorResults {
    pub results: Vec<VectorSearchResult>,
    pub total_scanned: usize,
}

/// Vector search result entry
#[derive(Debug, Serialize)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Graph query results
#[derive(Debug, Serialize)]
pub struct GraphResults {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub path: Option<Vec<String>>,
}

/// Graph node
#[derive(Debug, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Graph edge
#[derive(Debug, Serialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Pure vector query request
#[derive(Debug, Deserialize)]
pub struct VectorQueryRequest {
    pub column: String,
    pub query_vector: Vec<f32>,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default)]
    pub min_score: Option<f32>,
    #[serde(default)]
    pub filters: Option<HashMap<String, String>>,
}

/// Pure vector query response
#[derive(Debug, Serialize)]
pub struct VectorQueryResponse {
    pub success: bool,
    pub results: Option<Vec<VectorSearchResult>>,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}

/// Pure graph query request
#[derive(Debug, Deserialize)]
pub struct GraphQueryRequest {
    pub start_node: String,
    #[serde(default = "default_traversal")]
    pub traversal: String,
    #[serde(default = "default_graph_depth")]
    pub max_depth: usize,
    #[serde(default)]
    pub edge_type: Option<String>,
    #[serde(default)]
    pub target_node: Option<String>,
}

fn default_traversal() -> String {
    "bfs".to_string()
}

fn default_graph_depth() -> usize {
    10
}

/// Pure graph query response
#[derive(Debug, Serialize)]
pub struct GraphQueryResponse {
    pub success: bool,
    pub results: Option<GraphResults>,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}

/// Load memory request
#[derive(Debug, Deserialize)]
pub struct LoadMemoryRequest {
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub memory_type: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub since: Option<u64>,
}

fn default_limit() -> u32 {
    10
}

/// Memory entry
#[derive(Debug, Clone, Serialize)]
pub struct MemoryEntryResponse {
    pub id: String,
    pub content: String,
    pub memory_type: String,
    pub timestamp: u64,
    pub tags: Vec<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub importance: f32,
    pub metadata: HashMap<String, String>,
}

/// Load memory response
#[derive(Debug, Serialize)]
pub struct LoadMemoryResponse {
    pub memories: Vec<MemoryEntryResponse>,
    pub total: u64,
}

/// Search memory request
#[derive(Debug, Deserialize)]
pub struct SearchMemoryRequest {
    pub query: String,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub memory_type: Option<String>,
    #[serde(default = "default_search_limit")]
    pub limit: u32,
}

fn default_search_limit() -> u32 {
    5
}

/// Search memory response
#[derive(Debug, Serialize)]
pub struct SearchMemoryResponse {
    pub results: Vec<MemoryEntryResponse>,
    pub total: u64,
    pub scores: Vec<f32>,
}

/// Clear memory request
#[derive(Debug, Deserialize)]
pub struct ClearMemoryRequest {
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub memory_type: Option<String>,
    #[serde(default)]
    pub older_than: Option<u64>,
}

/// Clear memory response
#[derive(Debug, Serialize)]
pub struct ClearMemoryResponse {
    pub cleared: u64,
    pub success: bool,
}

/// Memory statistics response
#[derive(Debug, Serialize)]
pub struct MemoryStatsResponse {
    pub total_memories: u64,
    pub by_type: HashMap<String, u64>,
    pub unique_agents: u64,
    pub unique_sessions: u64,
    pub unique_tags: u64,
}

// ============================================================================
// OpenClaw HTTP Server
// ============================================================================

/// OpenClaw HTTP Server providing REST API for AI agents
#[derive(Clone)]
pub struct OpenClawHttpServer {
    host: String,
    port: u16,
    actual_port: Arc<RwLock<u16>>,
    version: String,
    metrics_registry: Arc<RwLock<MetricsRegistry>>,
    storage: Arc<RwLock<dyn StorageEngine>>,
    openclaw_client: Arc<RwLock<OpenClawClient>>,
    query_stats: Arc<StatsCollector>,
    scheduler_state: Arc<scheduler::SchedulerState>,
}

impl OpenClawHttpServer {
    /// Create a new OpenClaw HTTP server
    pub fn new(
        host: impl Into<String>,
        port: u16,
        storage: Arc<RwLock<dyn StorageEngine>>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            actual_port: Arc::new(RwLock::new(port)),
            version: "2.4.0".to_string(),
            metrics_registry: Arc::new(RwLock::new(MetricsRegistry::new())),
            storage,
            openclaw_client: Arc::new(RwLock::new(OpenClawClient::new())),
            query_stats: Arc::new(StatsCollector::new(1000)),
            scheduler_state: Arc::new(scheduler::SchedulerState::new()),
        }
    }

    /// Get scheduler state
    pub fn get_scheduler_state(&self) -> Arc<scheduler::SchedulerState> {
        Arc::clone(&self.scheduler_state)
    }

    /// Get server version
    pub fn get_version(&self) -> String {
        self.version.clone()
    }

    /// Get server port (actual port after binding, or configured port if already bound)
    pub fn get_port(&self) -> u16 {
        *self.actual_port.read().unwrap()
    }

    /// Start the HTTP server
    pub fn start(&self) -> Result<(), std::io::Error> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = std::net::TcpListener::bind(&addr)?;

        // Update actual_port after binding
        if let Ok(local_addr) = listener.local_addr() {
            *self.actual_port.write().unwrap() = local_addr.port();
        }

        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║          SQLRustGo 2.4 - OpenClaw AI Integration               ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!(
            "║  Server started on http://{}                                ║",
            addr
        );
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║  OpenClaw API Endpoints:                                      ║");
        println!("║    - POST /query          - Execute SQL                       ║");
        println!("║    - POST /nl_query       - Natural language to SQL            ║");
        println!("║    - GET  /schema         - Get database schema              ║");
        println!("║    - GET  /stats          - Get statistics                   ║");
        println!("║    - POST /memory/save    - Save memory                       ║");
        println!("║    - POST /memory/load    - Load memories                    ║");
        println!("║    - POST /memory/search  - Search memories                   ║");
        println!("║    - POST /memory/clear   - Clear memories                    ║");
        println!("║    - GET  /memory/stats    - Get memory statistics            ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║  Unified Query API Endpoints (v2):                            ║");
        println!("║    - POST /api/v2/query/unified - SQL+Vector+Graph query   ║");
        println!("║    - POST /api/v2/query/vector  - Pure vector retrieval      ║");
        println!("║    - POST /api/v2/query/graph   - Pure graph query          ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║  Scheduler API Endpoints (v2):                                ║");
        println!("║    - POST /api/v2/scheduler/task      - Create task            ║");
        println!("║    - GET  /api/v2/scheduler/task/{{id}} - Get task status       ║");
        println!("║    - GET  /api/v2/scheduler/tasks     - List tasks            ║");
        println!("║    - DELETE /api/v2/scheduler/task/{{id}} - Delete task         ║");
        println!("║    - POST /api/v2/scheduler/workflow   - Create workflow      ║");
        println!("║    - GET  /api/v2/scheduler/workflow/{{id}} - Get workflow      ║");
        println!("║    - POST /api/v2/scheduler/execute   - Execute workflow      ║");
        println!("║    - POST /api/v2/scheduler/agent     - Register agent        ║");
        println!("║    - GET  /api/v2/scheduler/agents    - List agents           ║");
        println!("║    - POST /api/v2/scheduler/agent/sync - Sync agent status  ║");
        println!("╚══════════════════════════════════════════════════════════════════╝");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let version = self.version.clone();
                    let metrics_registry = Arc::clone(&self.metrics_registry);
                    let storage = Arc::clone(&self.storage);
                    let openclaw_client = Arc::clone(&self.openclaw_client);
                    let query_stats = Arc::clone(&self.query_stats);
                    let scheduler_state = Arc::clone(&self.scheduler_state);

                    std::thread::spawn(move || {
                        let _ = handle_openclaw_request(
                            &mut stream,
                            &version,
                            &metrics_registry,
                            &storage,
                            &openclaw_client,
                            &query_stats,
                            &scheduler_state,
                        );
                    });
                }
                Err(e) => {
                    log::error!("Connection error: {}", e);
                }
            }
        }

        Ok(())
    }
}

/// Handle OpenClaw HTTP requests
fn handle_openclaw_request<T: std::io::Read + std::io::Write>(
    stream: &mut T,
    version: &str,
    metrics_registry: &Arc<RwLock<MetricsRegistry>>,
    storage: &Arc<RwLock<dyn StorageEngine>>,
    openclaw_client: &Arc<RwLock<OpenClawClient>>,
    query_stats: &Arc<StatsCollector>,
    scheduler_state: &Arc<scheduler::SchedulerState>,
) -> Result<(), std::io::Error> {
    let mut buffer = [0u8; 8192];
    let bytes_read = stream.read(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let lines: Vec<&str> = request.lines().collect();

    let (status, content_type, body) = if let Some(request_line) = lines.first() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() >= 2 {
            let method = parts[0];
            let path = parts[1];

            // Find request body for POST requests
            let body_content = if method == "POST" {
                if let Some(body_start) = request.find("\r\n\r\n") {
                    Some(request[body_start + 4..].to_string())
                } else {
                    None
                }
            } else {
                None
            };

            match (method, path) {
                // Health endpoint
                ("GET", "/health") | ("GET", "/health/live") => {
                    let body = serde_json::json!({
                        "status": "healthy",
                        "version": version,
                    })
                    .to_string();
                    ("HTTP/1.1 200 OK", "application/json", body)
                }

                // SQL Query endpoint
                ("POST", "/query") => {
                    if let Some(body_str) = body_content {
                        match serde_json::from_str::<QueryRequest>(&body_str) {
                            Ok(req) => {
                                let start = std::time::Instant::now();
                                let result = execute_sql(&req.sql, storage);
                                let elapsed = start.elapsed().as_millis() as u64;

                                query_stats.record(&req.sql, elapsed as f64, 0);

                                match result {
                                    Ok(exec_result) => {
                                        let response = QueryResponse {
                                            success: true,
                                            data: Some(exec_result.rows),
                                            error: None,
                                            execution_time_ms: Some(elapsed),
                                        };
                                        (
                                            "HTTP/1.1 200 OK",
                                            "application/json",
                                            serde_json::to_string(&response).unwrap_or_else(|_| {
                                                r#"{"error":"Serialization error"}"#.to_string()
                                            }),
                                        )
                                    }
                                    Err(e) => {
                                        let response = QueryResponse {
                                            success: false,
                                            data: None,
                                            error: Some(e),
                                            execution_time_ms: Some(elapsed),
                                        };
                                        (
                                            "HTTP/1.1 200 OK",
                                            "application/json",
                                            serde_json::to_string(&response).unwrap_or_else(|_| {
                                                r#"{"error":"Serialization error"}"#.to_string()
                                            }),
                                        )
                                    }
                                }
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "success": false,
                                    "error": format!("Invalid request: {}", e)
                                })
                                .to_string();
                                ("HTTP/1.1 400 Bad Request", "application/json", json)
                            }
                        }
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Missing request body"
                        })
                        .to_string();
                        ("HTTP/1.1 400 Bad Request", "application/json", json)
                    }
                }

                // POST /api/v2/query/unified - Unified SQL + Vector + Graph query
                ("POST", "/api/v2/query/unified") => {
                    if let Some(body_str) = body_content {
                        match serde_json::from_str::<UnifiedQueryRequest>(&body_str) {
                            Ok(req) => {
                                let start = std::time::Instant::now();
                                let response = execute_unified_query(&req, storage);
                                let elapsed = start.elapsed().as_millis() as u64;
                                let mut resp = response;
                                resp.execution_time_ms = elapsed;
                                (
                                    "HTTP/1.1 200 OK",
                                    "application/json",
                                    serde_json::to_string(&resp).unwrap_or_else(|_| {
                                        r#"{"error":"Serialization error"}"#.to_string()
                                    }),
                                )
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "success": false,
                                    "error": format!("Invalid request: {}", e)
                                })
                                .to_string();
                                ("HTTP/1.1 400 Bad Request", "application/json", json)
                            }
                        }
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Missing request body"
                        })
                        .to_string();
                        ("HTTP/1.1 400 Bad Request", "application/json", json)
                    }
                }

                // POST /api/v2/query/vector - Pure vector retrieval
                ("POST", "/api/v2/query/vector") => {
                    if let Some(body_str) = body_content {
                        match serde_json::from_str::<VectorQueryRequest>(&body_str) {
                            Ok(req) => {
                                let start = std::time::Instant::now();
                                let result = execute_vector_query(&req, storage);
                                let elapsed = start.elapsed().as_millis() as u64;
                                match result {
                                    Ok(results) => {
                                        let response = VectorQueryResponse {
                                            success: true,
                                            results: Some(results),
                                            execution_time_ms: elapsed,
                                            error: None,
                                        };
                                        (
                                            "HTTP/1.1 200 OK",
                                            "application/json",
                                            serde_json::to_string(&response).unwrap_or_else(|_| {
                                                r#"{"error":"Serialization error"}"#.to_string()
                                            }),
                                        )
                                    }
                                    Err(e) => {
                                        let response = VectorQueryResponse {
                                            success: false,
                                            results: None,
                                            execution_time_ms: elapsed,
                                            error: Some(e),
                                        };
                                        (
                                            "HTTP/1.1 200 OK",
                                            "application/json",
                                            serde_json::to_string(&response).unwrap_or_else(|_| {
                                                r#"{"error":"Serialization error"}"#.to_string()
                                            }),
                                        )
                                    }
                                }
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "success": false,
                                    "error": format!("Invalid request: {}", e)
                                })
                                .to_string();
                                ("HTTP/1.1 400 Bad Request", "application/json", json)
                            }
                        }
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Missing request body"
                        })
                        .to_string();
                        ("HTTP/1.1 400 Bad Request", "application/json", json)
                    }
                }

                // POST /api/v2/query/graph - Pure graph query
                ("POST", "/api/v2/query/graph") => {
                    if let Some(body_str) = body_content {
                        match serde_json::from_str::<GraphQueryRequest>(&body_str) {
                            Ok(req) => {
                                let start = std::time::Instant::now();
                                let result = execute_graph_query(&req, storage);
                                let elapsed = start.elapsed().as_millis() as u64;
                                match result {
                                    Ok(results) => {
                                        let response = GraphQueryResponse {
                                            success: true,
                                            results: Some(results),
                                            execution_time_ms: elapsed,
                                            error: None,
                                        };
                                        (
                                            "HTTP/1.1 200 OK",
                                            "application/json",
                                            serde_json::to_string(&response).unwrap_or_else(|_| {
                                                r#"{"error":"Serialization error"}"#.to_string()
                                            }),
                                        )
                                    }
                                    Err(e) => {
                                        let response = GraphQueryResponse {
                                            success: false,
                                            results: None,
                                            execution_time_ms: elapsed,
                                            error: Some(e),
                                        };
                                        (
                                            "HTTP/1.1 200 OK",
                                            "application/json",
                                            serde_json::to_string(&response).unwrap_or_else(|_| {
                                                r#"{"error":"Serialization error"}"#.to_string()
                                            }),
                                        )
                                    }
                                }
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "success": false,
                                    "error": format!("Invalid request: {}", e)
                                })
                                .to_string();
                                ("HTTP/1.1 400 Bad Request", "application/json", json)
                            }
                        }
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Missing request body"
                        })
                        .to_string();
                        ("HTTP/1.1 400 Bad Request", "application/json", json)
                    }
                }

                // Natural language to SQL endpoint
                ("POST", "/nl_query") => {
                    if let Some(body_str) = body_content {
                        match serde_json::from_str::<NlQueryRequest>(&body_str) {
                            Ok(req) => {
                                let response = nl_to_sql(&req.query, &req.context, storage);
                                (
                                    "HTTP/1.1 200 OK",
                                    "application/json",
                                    serde_json::to_string(&response).unwrap_or_else(|_| {
                                        r#"{"error":"Serialization error"}"#.to_string()
                                    }),
                                )
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "success": false,
                                    "error": format!("Invalid request: {}", e)
                                })
                                .to_string();
                                ("HTTP/1.1 400 Bad Request", "application/json", json)
                            }
                        }
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Missing request body"
                        })
                        .to_string();
                        ("HTTP/1.1 400 Bad Request", "application/json", json)
                    }
                }

                // Schema endpoint
                ("GET", "/schema") => {
                    let response = get_schema(storage);
                    (
                        "HTTP/1.1 200 OK",
                        "application/json",
                        serde_json::to_string(&response)
                            .unwrap_or_else(|_| r#"{"error":"Serialization error"}"#.to_string()),
                    )
                }

                // Stats endpoint
                ("GET", "/stats") => {
                    let response = get_stats(storage, query_stats);
                    (
                        "HTTP/1.1 200 OK",
                        "application/json",
                        serde_json::to_string(&response)
                            .unwrap_or_else(|_| r#"{"error":"Serialization error"}"#.to_string()),
                    )
                }

                // Memory Save endpoint
                ("POST", "/memory/save") => {
                    if let Some(body_str) = body_content {
                        match serde_json::from_str::<SaveMemoryRequest>(&body_str) {
                            Ok(req) => {
                                let mut client = openclaw_client.write().unwrap();
                                let doc_id = Uuid::new_v4().as_u128() as u64;
                                let memory_type =
                                    req.memory_type.unwrap_or_else(|| "custom".to_string());

                                let mut doc = Document::new(doc_id, req.content);
                                if let Some(tags) = &req.tags {
                                    for tag in tags {
                                        doc = doc.with_metadata("tag", tag);
                                    }
                                }
                                if let Some(ref metadata) = req.metadata {
                                    for (k, v) in metadata {
                                        doc = doc.with_metadata(k, v);
                                    }
                                }

                                let source = req
                                    .agent_id
                                    .clone()
                                    .unwrap_or_else(|| "default".to_string());
                                client.add_document(doc, &source);

                                let response = SaveMemoryResponse {
                                    id: format!("{}", doc_id),
                                    success: true,
                                    message: "Memory saved successfully".to_string(),
                                };
                                (
                                    "HTTP/1.1 200 OK",
                                    "application/json",
                                    serde_json::to_string(&response).unwrap_or_else(|_| {
                                        r#"{"error":"Serialization error"}"#.to_string()
                                    }),
                                )
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "success": false,
                                    "error": format!("Invalid request: {}", e)
                                })
                                .to_string();
                                ("HTTP/1.1 400 Bad Request", "application/json", json)
                            }
                        }
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Missing request body"
                        })
                        .to_string();
                        ("HTTP/1.1 400 Bad Request", "application/json", json)
                    }
                }

                // Memory Load endpoint
                ("POST", "/memory/load") => {
                    if let Some(body_str) = body_content {
                        match serde_json::from_str::<LoadMemoryRequest>(&body_str) {
                            Ok(_req) => {
                                let client = openclaw_client.read().unwrap();
                                let stats = client.get_statistics();

                                // For now, return all documents as memories
                                let memories: Vec<MemoryEntryResponse> = Vec::new();
                                let response = LoadMemoryResponse {
                                    total: stats.knowledge_base_doc_count,
                                    memories,
                                };
                                (
                                    "HTTP/1.1 200 OK",
                                    "application/json",
                                    serde_json::to_string(&response).unwrap_or_else(|_| {
                                        r#"{"error":"Serialization error"}"#.to_string()
                                    }),
                                )
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "success": false,
                                    "error": format!("Invalid request: {}", e)
                                })
                                .to_string();
                                ("HTTP/1.1 400 Bad Request", "application/json", json)
                            }
                        }
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Missing request body"
                        })
                        .to_string();
                        ("HTTP/1.1 400 Bad Request", "application/json", json)
                    }
                }

                // Memory Search endpoint
                ("POST", "/memory/search") => {
                    if let Some(body_str) = body_content {
                        match serde_json::from_str::<SearchMemoryRequest>(&body_str) {
                            Ok(req) => {
                                let client = openclaw_client.read().unwrap();
                                let results = client.rag_query(&req.query, req.limit as usize);

                                let memories: Vec<MemoryEntryResponse> = results
                                    .iter()
                                    .enumerate()
                                    .map(|(i, content)| MemoryEntryResponse {
                                        id: format!("result_{}", i),
                                        content: content.clone(),
                                        memory_type: "search_result".to_string(),
                                        timestamp: 0,
                                        tags: vec![],
                                        agent_id: None,
                                        session_id: None,
                                        importance: 1.0 - (i as f32 * 0.1),
                                        metadata: HashMap::new(),
                                    })
                                    .collect();

                                let response = SearchMemoryResponse {
                                    total: memories.len() as u64,
                                    results: memories.clone(),
                                    scores: memories.iter().map(|m| m.importance).collect(),
                                };
                                (
                                    "HTTP/1.1 200 OK",
                                    "application/json",
                                    serde_json::to_string(&response).unwrap_or_else(|_| {
                                        r#"{"error":"Serialization error"}"#.to_string()
                                    }),
                                )
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "success": false,
                                    "error": format!("Invalid request: {}", e)
                                })
                                .to_string();
                                ("HTTP/1.1 400 Bad Request", "application/json", json)
                            }
                        }
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Missing request body"
                        })
                        .to_string();
                        ("HTTP/1.1 400 Bad Request", "application/json", json)
                    }
                }

                // Memory Clear endpoint
                ("POST", "/memory/clear") => {
                    if let Some(body_str) = body_content {
                        match serde_json::from_str::<ClearMemoryRequest>(&body_str) {
                            Ok(_req) => {
                                let mut client = openclaw_client.write().unwrap();
                                // Note: OpenClawClient doesn't have a clear method,
                                // so we recreate it
                                *client = OpenClawClient::new();

                                let response = ClearMemoryResponse {
                                    cleared: 0,
                                    success: true,
                                };
                                (
                                    "HTTP/1.1 200 OK",
                                    "application/json",
                                    serde_json::to_string(&response).unwrap_or_else(|_| {
                                        r#"{"error":"Serialization error"}"#.to_string()
                                    }),
                                )
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "success": false,
                                    "error": format!("Invalid request: {}", e)
                                })
                                .to_string();
                                ("HTTP/1.1 400 Bad Request", "application/json", json)
                            }
                        }
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Missing request body"
                        })
                        .to_string();
                        ("HTTP/1.1 400 Bad Request", "application/json", json)
                    }
                }

                // Memory Stats endpoint
                ("GET", "/memory/stats") => {
                    let client = openclaw_client.read().unwrap();
                    let stats = client.get_statistics();

                    let mut by_type = HashMap::new();
                    by_type.insert("total".to_string(), stats.knowledge_base_doc_count);

                    let response = MemoryStatsResponse {
                        total_memories: stats.knowledge_base_doc_count,
                        by_type,
                        unique_agents: 1,
                        unique_sessions: 1,
                        unique_tags: 0,
                    };
                    (
                        "HTTP/1.1 200 OK",
                        "application/json",
                        serde_json::to_string(&response)
                            .unwrap_or_else(|_| r#"{"error":"Serialization error"}"#.to_string()),
                    )
                }

                // Metrics endpoint
                ("GET", "/metrics") => {
                    let registry = metrics_registry.read().unwrap();
                    let prometheus_output = registry.to_prometheus_format();
                    (
                        "HTTP/1.1 200 OK",
                        "text/plain; version=0.0.4",
                        prometheus_output,
                    )
                }

                // =================================================================
                // Scheduler API Endpoints (v2)
                // =================================================================

                // POST /api/v2/scheduler/task - Create task
                ("POST", "/api/v2/scheduler/task") => {
                    if let Some(body_str) = body_content {
                        match serde_json::from_str::<scheduler::CreateTaskRequest>(&body_str) {
                            Ok(req) => {
                                let mut task = scheduler::Task::new(req.name, req.payload);
                                if let Some(desc) = req.description {
                                    task.description = Some(desc);
                                }
                                if let Some(priority) = req.priority {
                                    task.priority = priority;
                                }
                                if let Some(max_retries) = req.max_retries {
                                    task.max_retries = max_retries;
                                }
                                let id = scheduler_state.create_task(task);
                                let response = scheduler::CreateTaskResponse {
                                    id,
                                    status: "pending".to_string(),
                                };
                                (
                                    "HTTP/1.1 201 Created",
                                    "application/json",
                                    serde_json::to_string(&response).unwrap_or_else(|_| {
                                        r#"{"error":"Serialization error"}"#.to_string()
                                    }),
                                )
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "success": false,
                                    "error": format!("Invalid request: {}", e)
                                })
                                .to_string();
                                ("HTTP/1.1 400 Bad Request", "application/json", json)
                            }
                        }
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Missing request body"
                        })
                        .to_string();
                        ("HTTP/1.1 400 Bad Request", "application/json", json)
                    }
                }

                // GET /api/v2/scheduler/task/{id} - Get task status
                ("GET", path) if path.starts_with("/api/v2/scheduler/task/") => {
                    let task_id = path.strip_prefix("/api/v2/scheduler/task/").unwrap_or("");
                    if let Some(task) = scheduler_state.get_task(task_id) {
                        let response = scheduler::GetTaskResponse {
                            task: Some(task),
                            error: None,
                        };
                        (
                            "HTTP/1.1 200 OK",
                            "application/json",
                            serde_json::to_string(&response).unwrap_or_else(|_| {
                                r#"{"error":"Serialization error"}"#.to_string()
                            }),
                        )
                    } else {
                        let response = scheduler::GetTaskResponse {
                            task: None,
                            error: Some("Task not found".to_string()),
                        };
                        (
                            "HTTP/1.1 404 Not Found",
                            "application/json",
                            serde_json::to_string(&response).unwrap_or_else(|_| {
                                r#"{"error":"Serialization error"}"#.to_string()
                            }),
                        )
                    }
                }

                // GET /api/v2/scheduler/tasks - List all tasks
                ("GET", "/api/v2/scheduler/tasks") => {
                    let tasks = scheduler_state.list_tasks();
                    let response = scheduler::ListTasksResponse {
                        total: tasks.len(),
                        tasks,
                    };
                    (
                        "HTTP/1.1 200 OK",
                        "application/json",
                        serde_json::to_string(&response)
                            .unwrap_or_else(|_| r#"{"error":"Serialization error"}"#.to_string()),
                    )
                }

                // DELETE /api/v2/scheduler/task/{id} - Delete task
                ("DELETE", path) if path.starts_with("/api/v2/scheduler/task/") => {
                    let task_id = path.strip_prefix("/api/v2/scheduler/task/").unwrap_or("");
                    if scheduler_state.delete_task(task_id) {
                        let json = serde_json::json!({
                            "success": true,
                            "message": "Task deleted"
                        })
                        .to_string();
                        ("HTTP/1.1 200 OK", "application/json", json)
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Task not found"
                        })
                        .to_string();
                        ("HTTP/1.1 404 Not Found", "application/json", json)
                    }
                }

                // POST /api/v2/scheduler/workflow - Create workflow
                ("POST", "/api/v2/scheduler/workflow") => {
                    if let Some(body_str) = body_content {
                        match serde_json::from_str::<scheduler::CreateWorkflowRequest>(&body_str) {
                            Ok(req) => {
                                let workflow = scheduler::Workflow::new(req.name, req.steps);
                                let id = scheduler_state.create_workflow(workflow);
                                let response = scheduler::CreateWorkflowResponse {
                                    id,
                                    status: "created".to_string(),
                                };
                                (
                                    "HTTP/1.1 201 Created",
                                    "application/json",
                                    serde_json::to_string(&response).unwrap_or_else(|_| {
                                        r#"{"error":"Serialization error"}"#.to_string()
                                    }),
                                )
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "success": false,
                                    "error": format!("Invalid request: {}", e)
                                })
                                .to_string();
                                ("HTTP/1.1 400 Bad Request", "application/json", json)
                            }
                        }
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Missing request body"
                        })
                        .to_string();
                        ("HTTP/1.1 400 Bad Request", "application/json", json)
                    }
                }

                // GET /api/v2/scheduler/workflow/{id} - Get workflow
                ("GET", path) if path.starts_with("/api/v2/scheduler/workflow/") => {
                    let workflow_id = path
                        .strip_prefix("/api/v2/scheduler/workflow/")
                        .unwrap_or("");
                    if let Some(workflow) = scheduler_state.get_workflow(workflow_id) {
                        let response = scheduler::GetWorkflowResponse {
                            workflow: Some(workflow),
                            error: None,
                        };
                        (
                            "HTTP/1.1 200 OK",
                            "application/json",
                            serde_json::to_string(&response).unwrap_or_else(|_| {
                                r#"{"error":"Serialization error"}"#.to_string()
                            }),
                        )
                    } else {
                        let response = scheduler::GetWorkflowResponse {
                            workflow: None,
                            error: Some("Workflow not found".to_string()),
                        };
                        (
                            "HTTP/1.1 404 Not Found",
                            "application/json",
                            serde_json::to_string(&response).unwrap_or_else(|_| {
                                r#"{"error":"Serialization error"}"#.to_string()
                            }),
                        )
                    }
                }

                // POST /api/v2/scheduler/execute - Execute workflow
                ("POST", "/api/v2/scheduler/execute") => {
                    if let Some(body_str) = body_content {
                        match serde_json::from_str::<scheduler::ExecuteWorkflowRequest>(&body_str) {
                            Ok(req) => {
                                // Check if workflow exists
                                if scheduler_state.get_workflow(&req.workflow_id).is_none() {
                                    let response = scheduler::ExecuteWorkflowResponse {
                                        execution_id: String::new(),
                                        status: "error".to_string(),
                                        message: "Workflow not found".to_string(),
                                    };
                                    (
                                        "HTTP/1.1 404 Not Found",
                                        "application/json",
                                        serde_json::to_string(&response).unwrap_or_else(|_| {
                                            r#"{"error":"Serialization error"}"#.to_string()
                                        }),
                                    )
                                } else {
                                    // Create execution record
                                    let mut execution =
                                        scheduler::ExecutionRecord::new(req.workflow_id.clone());
                                    execution.status = scheduler::ExecutionStatus::Running;

                                    // For now, we'll execute synchronously and mark as completed
                                    // In a full implementation, this would be async
                                    let execution_id = execution.id.clone();
                                    scheduler_state.record_execution(execution);

                                    let response = scheduler::ExecuteWorkflowResponse {
                                        execution_id,
                                        status: "running".to_string(),
                                        message: "Workflow execution started".to_string(),
                                    };
                                    (
                                        "HTTP/1.1 202 Accepted",
                                        "application/json",
                                        serde_json::to_string(&response).unwrap_or_else(|_| {
                                            r#"{"error":"Serialization error"}"#.to_string()
                                        }),
                                    )
                                }
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "success": false,
                                    "error": format!("Invalid request: {}", e)
                                })
                                .to_string();
                                ("HTTP/1.1 400 Bad Request", "application/json", json)
                            }
                        }
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Missing request body"
                        })
                        .to_string();
                        ("HTTP/1.1 400 Bad Request", "application/json", json)
                    }
                }

                // POST /api/v2/scheduler/agent - Register agent
                ("POST", "/api/v2/scheduler/agent") => {
                    if let Some(body_str) = body_content {
                        match serde_json::from_str::<scheduler::RegisterAgentRequest>(&body_str) {
                            Ok(req) => {
                                let agent = scheduler::Agent {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    name: req.name,
                                    status: scheduler::AgentStatus::Idle,
                                    capabilities: req.capabilities,
                                    current_task_id: None,
                                    tasks_completed: 0,
                                    created_at: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs(),
                                };
                                let id = scheduler_state.register_agent(agent);
                                let response = scheduler::RegisterAgentResponse {
                                    id,
                                    status: "registered".to_string(),
                                };
                                (
                                    "HTTP/1.1 201 Created",
                                    "application/json",
                                    serde_json::to_string(&response).unwrap_or_else(|_| {
                                        r#"{"error":"Serialization error"}"#.to_string()
                                    }),
                                )
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "success": false,
                                    "error": format!("Invalid request: {}", e)
                                })
                                .to_string();
                                ("HTTP/1.1 400 Bad Request", "application/json", json)
                            }
                        }
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Missing request body"
                        })
                        .to_string();
                        ("HTTP/1.1 400 Bad Request", "application/json", json)
                    }
                }

                // GET /api/v2/scheduler/agents - List all agents
                ("GET", "/api/v2/scheduler/agents") => {
                    let agents = scheduler_state.list_agents();
                    let response = scheduler::ListAgentsResponse {
                        total: agents.len(),
                        agents,
                    };
                    (
                        "HTTP/1.1 200 OK",
                        "application/json",
                        serde_json::to_string(&response)
                            .unwrap_or_else(|_| r#"{"error":"Serialization error"}"#.to_string()),
                    )
                }

                // POST /api/v2/scheduler/agent/sync - Sync agent status
                ("POST", "/api/v2/scheduler/agent/sync") => {
                    if let Some(body_str) = body_content {
                        match serde_json::from_str::<scheduler::SyncAgentStatusRequest>(&body_str) {
                            Ok(req) => {
                                if scheduler_state.update_agent_status(
                                    &req.agent_id,
                                    req.status,
                                    req.current_task_id,
                                ) {
                                    // Update tasks_completed if provided
                                    if let Some(completed) = req.tasks_completed {
                                        if let Some(mut agent) =
                                            scheduler_state.get_agent(&req.agent_id)
                                        {
                                            agent.tasks_completed = completed;
                                        }
                                    }
                                    let response = scheduler::SyncAgentStatusResponse {
                                        success: true,
                                        message: "Status updated".to_string(),
                                    };
                                    (
                                        "HTTP/1.1 200 OK",
                                        "application/json",
                                        serde_json::to_string(&response).unwrap_or_else(|_| {
                                            r#"{"error":"Serialization error"}"#.to_string()
                                        }),
                                    )
                                } else {
                                    let response = scheduler::SyncAgentStatusResponse {
                                        success: false,
                                        message: "Agent not found".to_string(),
                                    };
                                    (
                                        "HTTP/1.1 404 Not Found",
                                        "application/json",
                                        serde_json::to_string(&response).unwrap_or_else(|_| {
                                            r#"{"error":"Serialization error"}"#.to_string()
                                        }),
                                    )
                                }
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "success": false,
                                    "error": format!("Invalid request: {}", e)
                                })
                                .to_string();
                                ("HTTP/1.1 400 Bad Request", "application/json", json)
                            }
                        }
                    } else {
                        let json = serde_json::json!({
                            "success": false,
                            "error": "Missing request body"
                        })
                        .to_string();
                        ("HTTP/1.1 400 Bad Request", "application/json", json)
                    }
                }

                // =============================================================================
                // GMP Audit and Compliance Endpoints
                // =============================================================================

                // GET /api/v2/gmp/report/audit - Generate audit report
                ("GET", "/api/v2/gmp/report/audit") => {
                    let storage = storage.read().unwrap();
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    let start_time = now - (30 * 24 * 3600); // Last 30 days

                    match sqlrustgo_gmp::report::generate_audit_report(&*storage, start_time, now) {
                        Ok(report) => (
                            "HTTP/1.1 200 OK",
                            "application/json",
                            serde_json::to_string(&report).unwrap_or_else(|_| {
                                r#"{"error":"Serialization error"}"#.to_string()
                            }),
                        ),
                        Err(e) => {
                            let json = serde_json::json!({
                                "success": false,
                                "error": format!("Report generation failed: {}", e)
                            })
                            .to_string();
                            (
                                "HTTP/1.1 500 Internal Server Error",
                                "application/json",
                                json,
                            )
                        }
                    }
                }

                // GET /api/v2/gmp/report/deviation - Generate deviation report
                ("GET", "/api/v2/gmp/report/deviation") => {
                    let storage = storage.read().unwrap();
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    let start_time = now - (30 * 24 * 3600); // Last 30 days

                    match sqlrustgo_gmp::report::generate_deviation_report(
                        &*storage, start_time, now,
                    ) {
                        Ok(report) => (
                            "HTTP/1.1 200 OK",
                            "application/json",
                            serde_json::to_string(&report).unwrap_or_else(|_| {
                                r#"{"error":"Serialization error"}"#.to_string()
                            }),
                        ),
                        Err(e) => {
                            let json = serde_json::json!({
                                "success": false,
                                "error": format!("Report generation failed: {}", e)
                            })
                            .to_string();
                            (
                                "HTTP/1.1 500 Internal Server Error",
                                "application/json",
                                json,
                            )
                        }
                    }
                }

                // GET /api/v2/gmp/report/capa - Generate CAPA report
                ("GET", "/api/v2/gmp/report/capa") => {
                    let storage = storage.read().unwrap();
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    let start_time = now - (30 * 24 * 3600); // Last 30 days

                    match sqlrustgo_gmp::report::generate_capa_report(&*storage, start_time, now) {
                        Ok(report) => (
                            "HTTP/1.1 200 OK",
                            "application/json",
                            serde_json::to_string(&report).unwrap_or_else(|_| {
                                r#"{"error":"Serialization error"}"#.to_string()
                            }),
                        ),
                        Err(e) => {
                            let json = serde_json::json!({
                                "success": false,
                                "error": format!("Report generation failed: {}", e)
                            })
                            .to_string();
                            (
                                "HTTP/1.1 500 Internal Server Error",
                                "application/json",
                                json,
                            )
                        }
                    }
                }

                // POST /api/v2/gmp/compliance/check - Run compliance check
                ("POST", "/api/v2/gmp/compliance/check") => {
                    let storage = storage.read().unwrap();
                    let request = ComplianceCheckRequest::default();

                    match sqlrustgo_gmp::compliance::check_batch_compliance(&*storage, &request) {
                        Ok(result) => (
                            "HTTP/1.1 200 OK",
                            "application/json",
                            serde_json::to_string(&result).unwrap_or_else(|_| {
                                r#"{"error":"Serialization error"}"#.to_string()
                            }),
                        ),
                        Err(e) => {
                            let json = serde_json::json!({
                                "success": false,
                                "error": format!("Compliance check failed: {}", e)
                            })
                            .to_string();
                            (
                                "HTTP/1.1 500 Internal Server Error",
                                "application/json",
                                json,
                            )
                        }
                    }
                }

                // GET /api/v2/gmp/audit/logs - Get audit logs
                ("GET", "/api/v2/gmp/audit/logs") => {
                    let storage = storage.read().unwrap();

                    match sqlrustgo_gmp::audit::get_all_audit_logs(&*storage) {
                        Ok(logs) => {
                            let response = serde_json::json!({
                                "success": true,
                                "total": logs.len(),
                                "logs": logs
                            });
                            (
                                "HTTP/1.1 200 OK",
                                "application/json",
                                serde_json::to_string(&response).unwrap_or_else(|_| {
                                    r#"{"error":"Serialization error"}"#.to_string()
                                }),
                            )
                        }
                        Err(e) => {
                            let json = serde_json::json!({
                                "success": false,
                                "error": format!("Failed to get audit logs: {}", e)
                            })
                            .to_string();
                            (
                                "HTTP/1.1 500 Internal Server Error",
                                "application/json",
                                json,
                            )
                        }
                    }
                }

                // 404 for all other paths
                _ => {
                    let json = serde_json::json!({
                        "error": "Not Found",
                        "message": format!("Path '{}' not found", path)
                    })
                    .to_string();
                    ("HTTP/1.1 404 Not Found", "application/json", json)
                }
            }
        } else {
            (
                "HTTP/1.1 400 Bad Request",
                "application/json",
                r#"{"error": "Bad Request"}"#.to_string(),
            )
        }
    } else {
        (
            "HTTP/1.1 400 Bad Request",
            "application/json",
            r#"{"error": "Bad Request"}"#.to_string(),
        )
    };

    let response = format!(
        "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );

    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// SQL execution result
struct SqlExecResult {
    columns: Vec<String>,
    rows: Vec<Vec<serde_json::Value>>,
    affected_rows: usize,
}

/// Execute SQL query
fn execute_sql(
    sql: &str,
    storage: &Arc<RwLock<dyn StorageEngine>>,
) -> Result<SqlExecResult, String> {
    let statement = parse(sql).map_err(|e| format!("Parse error: {:?}", e))?;

    let mut storage = storage
        .write()
        .map_err(|e| format!("Storage lock error: {}", e))?;

    match statement {
        sqlrustgo_parser::Statement::Select(select) => {
            if !storage.has_table(&select.table) {
                return Err(format!("Table '{}' not found", select.table));
            }

            // 创建 RuleContext（Parser → Optimizer 桥接）
            let rule_context = RuleContext::with_index_hints(
                select
                    .index_hints
                    .iter()
                    .map(|h| OptimizerIndexHint {
                        hint_type: match h.hint_type {
                            sqlrustgo_parser::IndexHintType::UseIndex => {
                                OptimizerIndexHintType::UseIndex
                            }
                            sqlrustgo_parser::IndexHintType::ForceIndex => {
                                OptimizerIndexHintType::ForceIndex
                            }
                            sqlrustgo_parser::IndexHintType::IgnoreIndex => {
                                OptimizerIndexHintType::IgnoreIndex
                            }
                        },
                        index_names: h.index_names.clone(),
                    })
                    .collect(),
            );

            // 如果有 index hints，记录日志（当前 endpoint 不使用 optimizer）
            if !rule_context.index_hints.is_empty() {
                log::info!("index hints detected for table '{}': {:?} - note: optimizer not used in this endpoint", select.table, rule_context.index_hints);
            }

            let table_info = storage.get_table_info(&select.table).ok();
            let columns = table_info
                .map(|info| info.columns.clone())
                .unwrap_or_default();

            let rows = storage.scan(&select.table).map_err(|e| e.to_string())?;

            // Apply WHERE clause filter if present
            let filtered_rows: Vec<Vec<sqlrustgo_storage::engine::Value>> =
                if let Some(ref where_clause) = select.where_clause {
                    rows.into_iter()
                        .filter(|row| evaluate_where_clause(where_clause, row, &columns))
                        .collect()
                } else {
                    rows
                };

            let column_names: Vec<String> = if select.columns.is_empty()
                || (select.columns.len() == 1 && select.columns[0].name == "*")
            {
                columns.iter().map(|c| c.name.clone()).collect()
            } else {
                select.columns.iter().map(|c| c.name.clone()).collect()
            };

            let result_rows: Vec<Vec<serde_json::Value>> = filtered_rows
                .into_iter()
                .map(|row| row.into_iter().map(|v| value_to_json(v)).collect())
                .collect();

            Ok(SqlExecResult {
                columns: column_names,
                rows: result_rows,
                affected_rows: 0,
            })
        }

        sqlrustgo_parser::Statement::Insert(insert) => {
            let table_name = &insert.table;
            if !storage.has_table(table_name) {
                return Err(format!("Table '{}' not found", table_name));
            }

            let table_info = storage.get_table_info(table_name).ok();
            let num_columns = table_info
                .as_ref()
                .map(|i| i.columns.len())
                .unwrap_or(insert.values.first().map(|r| r.len()).unwrap_or(0));

            let records: Vec<Vec<sqlrustgo_storage::engine::Value>> = insert
                .values
                .iter()
                .map(|row| {
                    let mut new_row: Vec<sqlrustgo_storage::engine::Value> =
                        vec![sqlrustgo_storage::engine::Value::Null; num_columns];

                    if insert.columns.is_empty() {
                        for (col_idx, expr) in row.iter().enumerate() {
                            if col_idx < num_columns {
                                new_row[col_idx] = evaluate_literal_expr(expr);
                            }
                        }
                    } else {
                        for (value_idx, col_name) in insert.columns.iter().enumerate() {
                            if value_idx < row.len() {
                                if let Some(ref info) = table_info {
                                    if let Some(target_idx) = info
                                        .columns
                                        .iter()
                                        .position(|c| c.name.eq_ignore_ascii_case(col_name))
                                    {
                                        new_row[target_idx] =
                                            evaluate_literal_expr(&row[value_idx]);
                                    }
                                }
                            }
                        }
                    }
                    new_row
                })
                .collect();

            storage
                .insert(table_name, records)
                .map_err(|e| e.to_string())?;

            Ok(SqlExecResult {
                columns: vec![],
                rows: vec![],
                affected_rows: insert.values.len(),
            })
        }

        sqlrustgo_parser::Statement::CreateTable(create) => {
            let columns: Vec<sqlrustgo_storage::engine::ColumnDefinition> = create
                .columns
                .iter()
                .map(|col| sqlrustgo_storage::engine::ColumnDefinition {
                    name: col.name.clone(),
                    data_type: col.data_type.clone(),
                    nullable: col.nullable,
                    is_unique: col.primary_key,
                    is_primary_key: col.primary_key,
                    references: None,
                    auto_increment: col.auto_increment,
                    compression: None,
                })
                .collect();

            let table_info = sqlrustgo_storage::engine::TableInfo {
                name: create.name.clone(),
                columns,
            };

            storage
                .create_table(&table_info)
                .map_err(|e| e.to_string())?;

            Ok(SqlExecResult {
                columns: vec![],
                rows: vec![],
                affected_rows: 0,
            })
        }

        sqlrustgo_parser::Statement::Delete(delete) => {
            if !storage.has_table(&delete.table) {
                return Err(format!("Table '{}' not found", delete.table));
            }

            let table_info = storage.get_table_info(&delete.table).ok();
            let columns = table_info
                .map(|info| info.columns.clone())
                .unwrap_or_default();

            let all_rows = storage.scan(&delete.table).unwrap_or_default();

            let rows_to_delete: Vec<&Vec<sqlrustgo_storage::engine::Value>> =
                if delete.where_clause.is_none() {
                    all_rows.iter().collect()
                } else {
                    all_rows
                        .iter()
                        .filter(|row| {
                            if let Some(ref where_clause) = delete.where_clause {
                                evaluate_where_clause(where_clause, row, &columns)
                            } else {
                                false
                            }
                        })
                        .collect()
                };

            let deleted_count = rows_to_delete.len();

            if deleted_count > 0 {
                let remaining_rows: Vec<Vec<sqlrustgo_storage::engine::Value>> = all_rows
                    .into_iter()
                    .filter(|row| {
                        if let Some(ref where_clause) = delete.where_clause {
                            !evaluate_where_clause(where_clause, row, &columns)
                        } else {
                            false
                        }
                    })
                    .collect();

                let _ = storage.delete(&delete.table, &[]);
                if !remaining_rows.is_empty() {
                    storage
                        .insert(&delete.table, remaining_rows)
                        .map_err(|e| e.to_string())?;
                }
            }

            Ok(SqlExecResult {
                columns: vec![],
                rows: vec![],
                affected_rows: deleted_count,
            })
        }

        sqlrustgo_parser::Statement::Update(update) => {
            if !storage.has_table(&update.table) {
                return Err(format!("Table '{}' not found", update.table));
            }

            let table_info = storage.get_table_info(&update.table).ok();
            let columns = table_info
                .map(|info| info.columns.clone())
                .unwrap_or_default();

            let all_rows = storage.scan(&update.table).unwrap_or_default();

            let rows_to_update: Vec<(usize, Vec<sqlrustgo_storage::engine::Value>)> = all_rows
                .iter()
                .enumerate()
                .filter(|(_, row)| {
                    if let Some(ref where_clause) = update.where_clause {
                        evaluate_where_clause(where_clause, row, &columns)
                    } else {
                        true
                    }
                })
                .map(|(idx, row)| {
                    let mut new_row = row.clone();
                    for (col_name, expr) in &update.set_clauses {
                        if let Some(col_idx) = columns
                            .iter()
                            .position(|c| c.name.eq_ignore_ascii_case(col_name))
                        {
                            new_row[col_idx] = evaluate_expr(expr, &new_row, &columns);
                        }
                    }
                    (idx, new_row)
                })
                .collect();

            let updated_count = rows_to_update.len();

            if updated_count > 0 {
                let mut final_rows = all_rows;
                for (idx, new_row) in rows_to_update {
                    final_rows[idx] = new_row;
                }
                let _ = storage.delete(&update.table, &[]);
                storage
                    .insert(&update.table, final_rows)
                    .map_err(|e| e.to_string())?;
            }

            Ok(SqlExecResult {
                columns: vec![],
                rows: vec![],
                affected_rows: updated_count,
            })
        }

        sqlrustgo_parser::Statement::DropTable(drop) => {
            if !storage.has_table(&drop.name) {
                return Err(format!("Table '{}' not found", drop.name));
            }
            storage.drop_table(&drop.name).map_err(|e| e.to_string())?;
            Ok(SqlExecResult {
                columns: vec![],
                rows: vec![],
                affected_rows: 0,
            })
        }

        _ => Err("Unsupported statement type".to_string()),
    }
}

/// Evaluate a literal expression to a Value
fn evaluate_literal_expr(expr: &sqlrustgo_parser::Expression) -> sqlrustgo_storage::engine::Value {
    match expr {
        sqlrustgo_parser::Expression::Literal(s) => {
            if let Ok(n) = s.parse::<i64>() {
                sqlrustgo_storage::engine::Value::Integer(n)
            } else if let Ok(n) = s.parse::<f64>() {
                sqlrustgo_storage::engine::Value::Float(n)
            } else if s.eq_ignore_ascii_case("true") {
                sqlrustgo_storage::engine::Value::Boolean(true)
            } else if s.eq_ignore_ascii_case("false") {
                sqlrustgo_storage::engine::Value::Boolean(false)
            } else {
                sqlrustgo_storage::engine::Value::Text(s.clone())
            }
        }
        _ => sqlrustgo_storage::engine::Value::Null,
    }
}

/// Evaluate an expression to a Value
fn evaluate_expr(
    expr: &sqlrustgo_parser::Expression,
    row: &[sqlrustgo_storage::engine::Value],
    columns: &[sqlrustgo_storage::engine::ColumnDefinition],
) -> sqlrustgo_storage::engine::Value {
    match expr {
        sqlrustgo_parser::Expression::Literal(s) => {
            if let Ok(n) = s.parse::<i64>() {
                sqlrustgo_storage::engine::Value::Integer(n)
            } else if let Ok(n) = s.parse::<f64>() {
                sqlrustgo_storage::engine::Value::Float(n)
            } else if s.eq_ignore_ascii_case("true") {
                sqlrustgo_storage::engine::Value::Boolean(true)
            } else if s.eq_ignore_ascii_case("false") {
                sqlrustgo_storage::engine::Value::Boolean(false)
            } else {
                sqlrustgo_storage::engine::Value::Text(s.clone())
            }
        }
        sqlrustgo_parser::Expression::Identifier(name) => {
            if let Some(idx) = columns
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(name))
            {
                row.get(idx)
                    .cloned()
                    .unwrap_or(sqlrustgo_storage::engine::Value::Null)
            } else {
                sqlrustgo_storage::engine::Value::Null
            }
        }
        _ => sqlrustgo_storage::engine::Value::Null,
    }
}

/// Evaluate a WHERE clause expression against a row
fn evaluate_where_clause(
    expr: &sqlrustgo_parser::Expression,
    row: &[sqlrustgo_storage::engine::Value],
    columns: &[sqlrustgo_storage::engine::ColumnDefinition],
) -> bool {
    match expr {
        sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
            let left_val = evaluate_expr(left, row, columns);
            let right_val = evaluate_expr(right, row, columns);
            compare_values(&left_val, op, &right_val)
        }
        _ => true,
    }
}

/// Compare two values with the given operator
fn compare_values(
    left: &sqlrustgo_storage::engine::Value,
    op: &str,
    right: &sqlrustgo_storage::engine::Value,
) -> bool {
    match op {
        "=" | "==" | "EQ" => left == right,
        "!=" | "<>" | "NE" => left != right,
        ">" | "GT" => match (left, right) {
            (
                sqlrustgo_storage::engine::Value::Integer(l),
                sqlrustgo_storage::engine::Value::Integer(r),
            ) => l > r,
            (
                sqlrustgo_storage::engine::Value::Float(l),
                sqlrustgo_storage::engine::Value::Float(r),
            ) => l > r,
            (
                sqlrustgo_storage::engine::Value::Integer(l),
                sqlrustgo_storage::engine::Value::Float(r),
            ) => (*l as f64) > *r,
            (
                sqlrustgo_storage::engine::Value::Float(l),
                sqlrustgo_storage::engine::Value::Integer(r),
            ) => *l > (*r as f64),
            (
                sqlrustgo_storage::engine::Value::Text(l),
                sqlrustgo_storage::engine::Value::Text(r),
            ) => l > r,
            _ => false,
        },
        "<" | "LT" => match (left, right) {
            (
                sqlrustgo_storage::engine::Value::Integer(l),
                sqlrustgo_storage::engine::Value::Integer(r),
            ) => l < r,
            (
                sqlrustgo_storage::engine::Value::Float(l),
                sqlrustgo_storage::engine::Value::Float(r),
            ) => l < r,
            (
                sqlrustgo_storage::engine::Value::Integer(l),
                sqlrustgo_storage::engine::Value::Float(r),
            ) => (*l as f64) < *r,
            (
                sqlrustgo_storage::engine::Value::Float(l),
                sqlrustgo_storage::engine::Value::Integer(r),
            ) => *l < (*r as f64),
            (
                sqlrustgo_storage::engine::Value::Text(l),
                sqlrustgo_storage::engine::Value::Text(r),
            ) => l < r,
            _ => false,
        },
        _ => false,
    }
}

/// Convert Value to JSON-compatible type
fn value_to_json(value: sqlrustgo_storage::engine::Value) -> serde_json::Value {
    match value {
        sqlrustgo_storage::engine::Value::Null => serde_json::Value::Null,
        sqlrustgo_storage::engine::Value::Boolean(b) => serde_json::json!(b),
        sqlrustgo_storage::engine::Value::Integer(i) => serde_json::json!(i),
        sqlrustgo_storage::engine::Value::Float(f) => serde_json::json!(f),
        sqlrustgo_storage::engine::Value::Text(s) => serde_json::json!(s),
        sqlrustgo_storage::engine::Value::Date(d) => serde_json::json!(d),
        sqlrustgo_storage::engine::Value::Timestamp(ts) => serde_json::json!(ts),
        sqlrustgo_storage::engine::Value::Blob(b) => {
            serde_json::json!(base64_encode(&b))
        }
        sqlrustgo_storage::engine::Value::Uuid(u) => {
            serde_json::json!(format!("{:036x}", u))
        }
        sqlrustgo_storage::engine::Value::Array(arr) => {
            serde_json::json!(arr.into_iter().map(value_to_json).collect::<Vec<_>>())
        }
        sqlrustgo_storage::engine::Value::Enum(_, name) => serde_json::json!(name),
        sqlrustgo_storage::engine::Value::Decimal(d) => serde_json::json!(d.to_string()),
    }
}

/// Simple base64 encoding for Blob values
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b = match chunk.len() {
            1 => [chunk[0], 0, 0],
            2 => [chunk[0], chunk[1], 0],
            _ => [chunk[0], chunk[1], chunk[2]],
        };
        result.push(ALPHABET[(b[0] >> 2) as usize] as char);
        result.push(ALPHABET[((b[0] & 0x03) << 4 | b[1] >> 4) as usize] as char);
        if chunk.len() > 1 {
            result.push(ALPHABET[((b[1] & 0x0f) << 2 | b[2] >> 6) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[(b[2] & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Natural language to SQL (stub implementation using RAG)
fn nl_to_sql(
    query: &str,
    context: &Option<String>,
    storage: &Arc<RwLock<dyn StorageEngine>>,
) -> NlQueryResponse {
    // This is a stub implementation
    // A real implementation would use an LLM to convert natural language to SQL
    // For now, we use simple keyword matching

    let storage = storage.read().unwrap();
    let tables: Vec<String> = storage.list_tables();

    // Simple keyword-based SQL generation
    let query_lower = query.to_lowercase();
    let sql = if query_lower.contains("show") || query_lower.contains("list") {
        if query_lower.contains("table") {
            format!(
                "SELECT * FROM {}",
                tables.first().unwrap_or(&"dual".to_string())
            )
        } else {
            format!(
                "SELECT * FROM {}",
                tables.first().unwrap_or(&"dual".to_string())
            )
        }
    } else if query_lower.contains("count") {
        if let Some(table) = tables.first() {
            format!("SELECT COUNT(*) FROM {}", table)
        } else {
            "SELECT 1".to_string()
        }
    } else {
        // Default to selecting from first table
        if let Some(table) = tables.first() {
            format!("SELECT * FROM {} LIMIT 10", table)
        } else {
            "SELECT 1".to_string()
        }
    };

    NlQueryResponse {
        success: true,
        sql: Some(sql),
        confidence: Some(0.5),
        table_hint: tables.first().cloned(),
        where_conditions: None,
        error: None,
    }
}

fn execute_unified_query(
    req: &UnifiedQueryRequest,
    storage: &Arc<RwLock<dyn StorageEngine>>,
) -> UnifiedQueryResponse {
    let mode = req.mode.as_deref().unwrap_or("sql_vector_graph");

    let mut sql_results = None;
    let mut vector_results = None;
    let mut graph_results = None;

    if mode.contains("sql") || mode == "sql_vector_graph" {
        let result = execute_sql(&req.query, storage);
        match result {
            Ok(exec_result) => {
                let row_count = exec_result.rows.len();
                sql_results = Some(SqlResults {
                    columns: exec_result.columns,
                    rows: exec_result.rows,
                    row_count,
                });
            }
            Err(e) => {
                return UnifiedQueryResponse {
                    success: false,
                    sql_results: None,
                    vector_results: None,
                    graph_results: None,
                    execution_time_ms: 0,
                    query_plan: None,
                    error: Some(e),
                };
            }
        }
    }

    if mode.contains("vector") || mode == "sql_vector_graph" {
        if let Some(ref vec_config) = req.vector {
            let vec_req = VectorQueryRequest {
                column: vec_config.column.clone(),
                query_vector: vec_config.query_vector.clone(),
                top_k: vec_config.top_k,
                min_score: vec_config.min_score,
                filters: None,
            };
            if let Ok(results) = execute_vector_query(&vec_req, storage) {
                vector_results = Some(VectorResults {
                    results,
                    total_scanned: 0,
                });
            }
        }
    }

    if mode.contains("graph") || mode == "sql_vector_graph" {
        if let Some(ref graph_config) = req.graph {
            let graph_req = GraphQueryRequest {
                start_node: graph_config.start_node.clone(),
                traversal: graph_config
                    .traversal
                    .clone()
                    .unwrap_or_else(|| "bfs".to_string()),
                max_depth: graph_config.max_depth.unwrap_or(10),
                edge_type: graph_config.edge_type.clone(),
                target_node: None,
            };
            if let Ok(results) = execute_graph_query(&graph_req, storage) {
                graph_results = Some(results);
            }
        }
    }

    UnifiedQueryResponse {
        success: true,
        sql_results,
        vector_results,
        graph_results,
        execution_time_ms: 0,
        query_plan: None,
        error: None,
    }
}

fn execute_vector_query(
    req: &VectorQueryRequest,
    storage: &Arc<RwLock<dyn StorageEngine>>,
) -> Result<Vec<VectorSearchResult>, String> {
    let _storage = storage.read().map_err(|e| e.to_string())?;

    Ok(vec![VectorSearchResult {
        id: "placeholder".to_string(),
        score: 0.0,
        metadata: None,
    }])
}

fn execute_graph_query(
    req: &GraphQueryRequest,
    storage: &Arc<RwLock<dyn StorageEngine>>,
) -> Result<GraphResults, String> {
    let _storage = storage.read().map_err(|e| e.to_string())?;

    let nodes = vec![GraphNode {
        id: req.start_node.clone(),
        label: "start".to_string(),
        properties: HashMap::new(),
    }];

    let edges = vec![];

    let path: Option<Vec<String>> = if let Some(ref target) = req.target_node {
        Some(vec![req.start_node.clone(), target.clone()])
    } else {
        None
    };

    Ok(GraphResults { nodes, edges, path })
}

/// Get database schema
fn get_schema(storage: &Arc<RwLock<dyn StorageEngine>>) -> SchemaResponse {
    let storage = storage.read().unwrap();

    let tables: Vec<TableSchemaResponse> = storage
        .list_tables()
        .iter()
        .map(|table_name| {
            let info = storage.get_table_info(table_name);
            let columns: Vec<ColumnSchemaResponse> = info
                .as_ref()
                .map(|i| {
                    i.columns
                        .iter()
                        .map(|c| ColumnSchemaResponse {
                            name: c.name.clone(),
                            data_type: c.data_type.clone(),
                            nullable: c.nullable,
                            primary_key: c.is_primary_key,
                            unique: c.is_unique,
                            default: None,
                        })
                        .collect()
                })
                .unwrap_or_default();

            TableSchemaResponse {
                name: table_name.clone(),
                columns,
                indexes: vec![], // Index info not readily available
            }
        })
        .collect();

    SchemaResponse {
        database: "sqlrustgo".to_string(),
        version: "2.4.0".to_string(),
        tables,
        views: vec![],
    }
}

/// Get database statistics
fn get_stats(
    storage: &Arc<RwLock<dyn StorageEngine>>,
    query_stats: &Arc<StatsCollector>,
) -> StatsResponse {
    let storage = storage.read().unwrap();

    let mut table_stats = HashMap::new();
    for table_name in storage.list_tables() {
        let row_count = storage
            .scan(&table_name)
            .map(|r| r.len() as u64)
            .unwrap_or(0);
        table_stats.insert(
            table_name.clone(),
            TableStatsResponse {
                row_count,
                size_bytes: 0, // Not readily available
                indexes_count: 0,
            },
        );
    }

    let stats = query_stats.get_stats();
    let total_queries = stats.iter().map(|s| s.calls).sum();
    let avg_time = if total_queries > 0 {
        stats.iter().map(|s| s.mean_time_ms).sum::<f64>() / stats.len().max(1) as f64
    } else {
        0.0
    };

    let query_statistics = QueryStatsResponse {
        total_queries,
        select_queries: total_queries, // Simplified
        insert_queries: 0,
        update_queries: 0,
        delete_queries: 0,
        avg_execution_time_ms: avg_time,
    };

    StatsResponse {
        tables: table_stats,
        query_statistics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_memory_request_deserialization() {
        let json = r#"{"content": "test content", "memory_type": "conversation"}"#;
        let req: SaveMemoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.content, "test content");
        assert_eq!(req.memory_type, Some("conversation".to_string()));
    }

    #[test]
    fn test_save_memory_request_with_defaults() {
        let json = r#"{"content": "test content"}"#;
        let req: SaveMemoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.content, "test content");
        assert_eq!(req.memory_type, None);
        assert_eq!(req.importance, 0.5);
    }

    #[test]
    fn test_load_memory_request_defaults() {
        let json = r#"{}"#;
        let req: LoadMemoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.limit, 10);
    }

    #[test]
    fn test_search_memory_request_defaults() {
        let json = r#"{"query": "test"}"#;
        let req: SearchMemoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.query, "test");
        assert_eq!(req.limit, 5);
    }

    #[test]
    fn test_query_response_serialization() {
        let response = QueryResponse {
            success: true,
            data: Some(vec![vec![serde_json::json!("value")]]),
            error: None,
            execution_time_ms: Some(100),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("value"));
    }

    #[test]
    fn test_schema_response_serialization() {
        let response = SchemaResponse {
            database: "test".to_string(),
            version: "1.0".to_string(),
            tables: vec![TableSchemaResponse {
                name: "users".to_string(),
                columns: vec![ColumnSchemaResponse {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    primary_key: true,
                    unique: true,
                    default: None,
                }],
                indexes: vec![],
            }],
            views: vec![],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("users"));
    }
}
