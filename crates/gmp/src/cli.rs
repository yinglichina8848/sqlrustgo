//! SQLRustGo GMP CLI - subprocess interface for GMP operations
//!
//! Protocol: JSON lines stdin/stdout
//! Commands: ping, init, vector_search, hybrid_search, import_doc,
//!           upsert_node (by label), upsert_edge (by label),
//!           graph_neighbors, list_nodes, list_edge_types

use serde::{Deserialize, Serialize};
use sqlrustgo_gmp::{GmpExecutor, SearchResult};
use sqlrustgo_graph::store::{GraphStore, InMemoryGraphStore};
use sqlrustgo_graph::{EdgeId, NodeId, PropertyMap, PropertyValue};
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::SqlResult;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

// ============================================================================
// Request / Response
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(tag = "cmd")]
pub enum Request {
    Ping,
    Init,
    VectorSearch {
        query: String,
        top_k: usize,
    },
    HybridSearch {
        query: String,
        top_k: usize,
    },
    ImportDoc {
        title: String,
        doc_type: String,
        content: String,
        keywords: Vec<String>,
    },
    /// Upsert node by label (string name). Returns internal NodeId as string.
    UpsertNode {
        node_type: String,
        name: String,
        code: Option<String>,
        properties: Option<serde_json::Value>,
    },
    /// Upsert edge between two nodes by their string names.
    UpsertEdge {
        from_name: String,
        from_type: String,
        to_name: String,
        to_type: String,
        rel_type: String,
        weight: Option<f64>,
    },
    /// Get direct neighbors of a node by its string name/type
    GraphNeighbors {
        node_type: String,
        node_name: String,
        rel_type: Option<String>,
    },
    /// List all nodes of a given type (or all if None)
    ListNodes {
        node_type: Option<String>,
    },
    ListEdgeTypes,
    /// Execute raw SQL (limited)
    SqlExec {
        sql: String,
    },
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub time_ms: u64,
}

impl Response {
    fn ok(data: serde_json::Value, ms: u64) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            time_ms: ms,
        }
    }
    fn err(msg: String, ms: u64) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg),
            time_ms: ms,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GraphNeighborsResult {
    pub center: NodeView,
    pub neighbors: Vec<NeighborView>,
}

#[derive(Debug, Serialize)]
pub struct NodeView {
    pub id: String,
    pub node_type: String,
    pub name: String,
    pub props: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct NeighborView {
    pub id: String,
    pub node_type: String,
    pub name: String,
    pub rel_type: String,
    pub props: serde_json::Value,
}

// ============================================================================
// State
// ============================================================================

pub struct GmpCliState {
    storage: Arc<RwLock<MemoryStorage>>,
    graph: Arc<RwLock<InMemoryGraphStore>>,
    gmp: GmpExecutor,
    /// Maps "node_type:name" -> NodeId (u64) for name-based access
    name_to_id: Arc<RwLock<HashMap<String, NodeId>>>,
}

impl GmpCliState {
    fn new() -> Self {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let graph = Arc::new(RwLock::new(InMemoryGraphStore::new()));
        let gmp = GmpExecutor::new(storage.clone());
        Self {
            storage,
            graph,
            gmp,
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn init(&self) -> SqlResult<()> {
        self.gmp.init()
    }

    fn vector_search(&self, q: &str, k: usize) -> SqlResult<Vec<SearchResult>> {
        self.gmp.search(q, k)
    }

    fn hybrid_search(&self, q: &str, k: usize) -> SqlResult<Vec<SearchResult>> {
        self.gmp.hybrid_search(q, k)
    }

    fn import_doc(
        &self,
        title: &str,
        doc_type: &str,
        content: &str,
        keywords: &[String],
    ) -> SqlResult<i64> {
        let kw: Vec<&str> = keywords.iter().map(|s| s.as_str()).collect();
        self.gmp.import_document(title, doc_type, content, &kw)
    }

    fn upsert_node(
        &self,
        node_type: &str,
        name: &str,
        code: Option<&str>,
        props: Option<serde_json::Value>,
    ) -> Result<String, String> {
        let key = format!("{}:{}", node_type, name);
        let label = node_type.to_string();

        // Check if already exists
        let nid = {
            let n2id = self.name_to_id.read().map_err(|e| e.to_string())?;
            n2id.get(&key).copied()
        };

        let _node_id = match nid {
            Some(id) => id,
            None => {
                // Create new node
                let mut graph = self.graph.write().map_err(|e| e.to_string())?;
                let mut pm = PropertyMap::new();
                pm.insert("name".to_string(), PropertyValue::String(name.to_string()));
                if let Some(c) = code {
                    pm.insert("code".to_string(), PropertyValue::String(c.to_string()));
                }
                if let Some(p) = props {
                    if let serde_json::Value::Object(map) = p {
                        for (k, v) in map {
                            let pv = match v {
                                serde_json::Value::String(s) => PropertyValue::String(s),
                                serde_json::Value::Number(n) => {
                                    PropertyValue::Float(n.as_f64().unwrap_or(0.0))
                                }
                                serde_json::Value::Bool(b) => {
                                    PropertyValue::Int(if b { 1 } else { 0 })
                                }
                                _ => PropertyValue::String(v.to_string()),
                            };
                            pm.insert(k, pv);
                        }
                    }
                }
                let id = graph.create_node(&label, pm);
                // Register in name map
                let mut n2id = self.name_to_id.write().map_err(|e| e.to_string())?;
                n2id.insert(key.clone(), id);
                id
            }
        };

        Ok(format!("{}:{}", node_type, name))
    }

    fn upsert_edge(
        &self,
        from_name: &str,
        from_type: &str,
        to_name: &str,
        to_type: &str,
        rel_type: &str,
        weight: Option<f64>,
    ) -> Result<(), String> {
        let from_key = format!("{}:{}", from_type, from_name);
        let to_key = format!("{}:{}", to_type, to_name);

        let (from_id, to_id) = {
            let n2id = self.name_to_id.read().map_err(|e| e.to_string())?;
            let f = *n2id
                .get(&from_key)
                .ok_or_else(|| format!("Node not found: {}", from_key))?;
            let t = *n2id
                .get(&to_key)
                .ok_or_else(|| format!("Node not found: {}", to_key))?;
            (f, t)
        };

        let mut graph = self.graph.write().map_err(|e| e.to_string())?;
        let mut pm = PropertyMap::new();
        if let Some(w) = weight {
            pm.insert("weight".to_string(), PropertyValue::Float(w));
        }
        graph
            .create_edge(from_id, to_id, rel_type, pm)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn graph_neighbors(
        &self,
        node_type: &str,
        node_name: &str,
        rel_type: Option<&str>,
    ) -> Result<GraphNeighborsResult, String> {
        let key = format!("{}:{}", node_type, node_name);
        let nid = {
            let n2id = self.name_to_id.read().map_err(|e| e.to_string())?;
            *n2id
                .get(&key)
                .ok_or_else(|| format!("Node not found: {}", key))?
        };

        let graph = self.graph.read().map_err(|e| e.to_string())?;

        // Get center node
        let center_node = graph.get_node(nid).ok_or("Node not found in graph")?;
        let center = NodeView {
            id: format!("{}:{}", node_type, node_name),
            node_type: node_type.to_string(),
            name: node_name.to_string(),
            props: property_map_to_json(&center_node.properties),
        };

        // Get neighbors
        let neighbor_ids: Vec<NodeId> = match rel_type {
            Some(rt) => graph.neighbors_by_edge_label(nid, rt),
            None => graph.outgoing_neighbors(nid),
        };

        let mut neighbors = Vec::new();
        for nb_id in neighbor_ids {
            if let Some(nb_node) = graph.get_node(nb_id) {
                // Find edge label(s) between center and neighbor
                let edge_label = if let Some(rt) = rel_type {
                    rt.to_string()
                } else {
                    // Try to find edge label from adjacency
                    let out = graph.outgoing_neighbors(nid);
                    let in_ = graph.incoming_neighbors(nid);
                    if out.contains(&nb_id) {
                        "OUT".to_string()
                    } else if in_.contains(&nb_id) {
                        "IN".to_string()
                    } else {
                        "RELATED".to_string()
                    }
                };

                // Get name from node properties
                let nb_name = nb_node
                    .properties
                    .get("name")
                    .and_then(|v| v.as_string())
                    .cloned()
                    .unwrap_or_else(|| nb_id.0.to_string());

                let nb_label_str = graph
                    .label_registry()
                    .get_label(nb_node.label)
                    .map(|s| s.as_str())
                    .unwrap_or("?");
                neighbors.push(NeighborView {
                    id: format!("node:{}", nb_id.0),
                    node_type: nb_label_str.to_string(),
                    name: nb_name,
                    rel_type: edge_label,
                    props: property_map_to_json(&nb_node.properties),
                });
            }
        }

        Ok(GraphNeighborsResult { center, neighbors })
    }

    fn list_nodes(&self, node_type: Option<&str>) -> Result<Vec<NodeView>, String> {
        let graph = self.graph.read().map_err(|e| e.to_string())?;
        let n2id = self.name_to_id.read().map_err(|e| e.to_string())?;

        // Use node_count and manual iteration to collect all nodes
        let count = graph.node_count();
        let mut results = Vec::new();

        // Iterate through all IDs we know about
        let label_registry = graph.label_registry();
        for (key, &nid) in n2id.iter() {
            if let Some(node) = graph.get_node(nid) {
                let label_str = label_registry
                    .get_label(node.label)
                    .map(|s| s.as_str())
                    .unwrap_or("?");
                if node_type.map_or(true, |t| label_str == t) {
                    let name = node
                        .properties
                        .get("name")
                        .and_then(|v| v.as_string())
                        .cloned()
                        .unwrap_or_else(|| nid.0.to_string());
                    results.push(NodeView {
                        id: key.clone(),
                        node_type: label_str.to_string(),
                        name,
                        props: property_map_to_json(&node.properties),
                    });
                }
            }
        }

        // Also add nodes that may not be in our name map
        for i in 0..count {
            let nid = NodeId(i as u64);
            if !n2id.values().any(|&v| v == nid) {
                if let Some(node) = graph.get_node(nid) {
                    let label_str = label_registry
                        .get_label(node.label)
                        .map(|s| s.as_str())
                        .unwrap_or("?");
                    if node_type.map_or(true, |t| label_str == t) {
                        let name = node
                            .properties
                            .get("name")
                            .and_then(|v| v.as_string())
                            .cloned()
                            .unwrap_or_else(|| nid.0.to_string());
                        results.push(NodeView {
                            id: format!("{}:{}", label_str, name),
                            node_type: label_str.to_string(),
                            name,
                            props: property_map_to_json(&node.properties),
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    fn list_edge_types(&self) -> Result<Vec<String>, String> {
        let graph = self.graph.read().map_err(|e| e.to_string())?;
        // Count edges to iterate through them
        let edge_count = graph.edge_count();
        let mut labels: std::collections::HashSet<String> = std::collections::HashSet::new();
        for i in 0..edge_count {
            if let Some(edge) = graph.get_edge(EdgeId(i as u64)) {
                // Get label string from label registry
                let label_str = graph
                    .label_registry()
                    .get_label(edge.label)
                    .map(|s| s.as_str())
                    .unwrap_or("?");
                labels.insert(label_str.to_string());
            }
        }
        let mut result: Vec<String> = labels.into_iter().collect();
        result.sort();
        Ok(result)
    }
}

fn property_map_to_json(pm: &PropertyMap) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (k, v) in pm.iter() {
        let val = match v {
            PropertyValue::String(s) => serde_json::Value::String(s.clone()),
            PropertyValue::Int(i) => serde_json::json!(*i),
            PropertyValue::Float(f) => serde_json::json!(*f),
            PropertyValue::Bool(b) => serde_json::json!(*b),
            PropertyValue::Null => serde_json::Value::Null,
            PropertyValue::Uuid(u) => serde_json::Value::String(u.clone()),
        };
        map.insert(k.clone(), val);
    }
    serde_json::Value::Object(map)
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    let stdin = std::io::stdin();
    let mut input = String::new();
    let state = Arc::new(GmpCliState::new());

    loop {
        input.clear();
        match stdin.read_line(&mut input) {
            Ok(0) | Err(_) => break,
            _ => {}
        }
        let line = input.trim();
        if line.is_empty() {
            continue;
        }

        let start = Instant::now();
        let resp = handle_request(line, &state);
        let ms = start.elapsed().as_millis() as u64;

        let out = match resp {
            Ok(r) => serde_json::to_string(&r)
                .unwrap_or_else(|_| r#"{"success":false,"error":"json error"}"#.to_string()),
            Err(e) => serde_json::to_string(&Response::err(e, ms)).unwrap_or_default(),
        };
        println!("{}", out);
    }
}

fn handle_request(line: &str, state: &GmpCliState) -> Result<Response, String> {
    let req: Request = serde_json::from_str(line).map_err(|e| format!("parse error: {}", e))?;
    let ms = 0u64;

    match req {
        Request::Ping => Ok(Response::ok(serde_json::json!({"status": "pong"}), ms)),

        Request::Init => {
            state.init().map_err(|e| e.to_string())?;
            Ok(Response::ok(serde_json::json!({"initialized": true}), ms))
        }

        Request::VectorSearch { query, top_k } => {
            let results = state
                .vector_search(&query, top_k)
                .map_err(|e| e.to_string())?;
            let data: Vec<_> = results
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "doc_id": r.doc_id,
                        "title": r.title,
                        "doc_type": r.doc_type,
                        "similarity": r.similarity
                    })
                })
                .collect();
            Ok(Response::ok(
                serde_json::to_value(data).unwrap_or_default(),
                ms,
            ))
        }

        Request::HybridSearch { query, top_k } => {
            let results = state
                .hybrid_search(&query, top_k)
                .map_err(|e| e.to_string())?;
            let data: Vec<_> = results
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "doc_id": r.doc_id,
                        "title": r.title,
                        "doc_type": r.doc_type,
                        "similarity": r.similarity
                    })
                })
                .collect();
            Ok(Response::ok(
                serde_json::to_value(data).unwrap_or_default(),
                ms,
            ))
        }

        Request::ImportDoc {
            title,
            doc_type,
            content,
            keywords,
        } => {
            let doc_id = state
                .import_doc(&title, &doc_type, &content, &keywords)
                .map_err(|e| e.to_string())?;
            Ok(Response::ok(serde_json::json!({"doc_id": doc_id}), ms))
        }

        Request::UpsertNode {
            node_type,
            name,
            code,
            properties,
        } => {
            let node_id = state.upsert_node(&node_type, &name, code.as_deref(), properties)?;
            Ok(Response::ok(serde_json::json!({"node_id": node_id}), ms))
        }

        Request::UpsertEdge {
            from_name,
            from_type,
            to_name,
            to_type,
            rel_type,
            weight,
        } => {
            state.upsert_edge(
                &from_name, &from_type, &to_name, &to_type, &rel_type, weight,
            )?;
            Ok(Response::ok(serde_json::json!({"edge_created": true}), ms))
        }

        Request::GraphNeighbors {
            node_type,
            node_name,
            rel_type,
        } => {
            let result = state.graph_neighbors(&node_type, &node_name, rel_type.as_deref())?;
            Ok(Response::ok(
                serde_json::to_value(result).unwrap_or_default(),
                ms,
            ))
        }

        Request::ListNodes { node_type } => {
            let nodes = state.list_nodes(node_type.as_deref())?;
            Ok(Response::ok(
                serde_json::to_value(nodes).unwrap_or_default(),
                ms,
            ))
        }

        Request::ListEdgeTypes => {
            let types = state.list_edge_types()?;
            Ok(Response::ok(
                serde_json::to_value(types).unwrap_or_default(),
                ms,
            ))
        }

        Request::SqlExec { .. } => Ok(Response::err(
            "SQL exec not supported in this build".to_string(),
            ms,
        )),
    }
}
