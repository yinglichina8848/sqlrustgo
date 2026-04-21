//! SQLRustGo GMP CLI - subprocess interface for GMP operations
//!
//! Protocol: JSON lines stdin/stdout
//! Commands: ping, init, vector_search, hybrid_search, import_doc,
//!           upsert_node (by label), upsert_edge (by label),
//!           graph_neighbors, list_nodes, list_edge_types

use serde::{Deserialize, Serialize};
use sqlrustgo_gmp::embedding;
use sqlrustgo_gmp::{GmpExecutor, SearchResult};
use sqlrustgo_graph::store::{GraphStore, InMemoryGraphStore};
use sqlrustgo_graph::{EdgeId, NodeId, PropertyMap, PropertyValue};
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::SqlResult;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use sqlrustgo_gmp::persist_sqlite::{
    DocumentRecord, EdgeUpsertRecord, EmbeddingRecord, NodeRecord, SqliteBackend, StorageBackend,
};

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
        /// Optional structured properties (department, category, chapter, code, version)
        properties: Option<serde_json::Value>,
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
    /// Exact SQL search (title/content LIKE match with optional filters)
    SqlSearch {
        query: String,
        department: Option<String>,
        category: Option<String>,
        chapter: Option<String>,
        top_k: usize,
    },
    /// Three-way RRF fusion: vector + graph + exact
    TripleSearch {
        query: String,
        top_k: usize,
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
    /// SQLite persistence backend (Stage 1 → Stage 3 migration contract)
    backend: Arc<SqliteBackend>,
}

impl GmpCliState {
    /// Open SQLite backend and reload all data from disk.
    /// This is the Stage 1 persistence entry point.
    fn new() -> Self {
        let backend = Arc::new(SqliteBackend::open().expect("failed to open SQLite backend"));

        // Initialize tables if they don't exist
        if let Err(e) = backend.as_trait().init() {
            eprintln!("WARN: backend init error: {}", e);
        }

        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let graph = Arc::new(RwLock::new(InMemoryGraphStore::new()));
        let gmp = GmpExecutor::new(storage.clone());
        let name_to_id = Arc::new(RwLock::new(HashMap::new()));

        // Reload persisted data into in-memory structures
        Self::reload_from_backend(&backend, &storage, &graph, &name_to_id, &gmp);

        Self {
            storage,
            graph,
            gmp,
            name_to_id,
            backend,
        }
    }

    /// Reload documents, embeddings, nodes, and edges from SQLite into memory
    fn reload_from_backend(
        backend: &Arc<SqliteBackend>,
        storage: &Arc<RwLock<MemoryStorage>>,
        graph: &Arc<RwLock<InMemoryGraphStore>>,
        name_to_id: &Arc<RwLock<HashMap<String, NodeId>>>,
        gmp: &GmpExecutor,
    ) {
        // Build sqlite_node_id -> (mem_id, name, type) mapping while loading nodes
        let mut sqlite_to_mem: std::collections::HashMap<
            i64,
            (sqlrustgo_graph::NodeId, String, String),
        > = std::collections::HashMap::new();

        // Load graph nodes first (needed to resolve edge references)
        if let Ok(nodes) = backend.as_trait().load_nodes() {
            let mut g = graph.write().unwrap();
            let mut n2id = name_to_id.write().unwrap();
            for node in nodes {
                let label = node.node_type.clone();
                let mut pm = sqlrustgo_graph::PropertyMap::new();
                pm.insert(
                    "name".to_string(),
                    sqlrustgo_graph::PropertyValue::String(node.name.clone()),
                );
                if let Ok(props_map) = serde_json::from_value::<
                    serde_json::Map<String, serde_json::Value>,
                >(node.properties.clone())
                {
                    for (k, v) in props_map {
                        let pv = match v {
                            serde_json::Value::String(s) => {
                                sqlrustgo_graph::PropertyValue::String(s)
                            }
                            serde_json::Value::Number(n) => {
                                sqlrustgo_graph::PropertyValue::Float(n.as_f64().unwrap_or(0.0))
                            }
                            serde_json::Value::Bool(b) => {
                                sqlrustgo_graph::PropertyValue::Int(if b { 1 } else { 0 })
                            }
                            _ => sqlrustgo_graph::PropertyValue::String(v.to_string()),
                        };
                        pm.insert(k, pv);
                    }
                }
                let id = g.create_node(&label, pm);
                let key = format!("{}:{}", node.node_type, node.name);
                n2id.insert(key, id);
                // Track sqlite_id -> (mem_id, name, type)
                sqlite_to_mem.insert(node.id, (id, node.name, node.node_type));
            }
        }

        // Load edges using sqlite_id -> mem_id mapping
        if let Ok(edges) = backend.as_trait().load_edges() {
            let mut g = graph.write().unwrap();
            for edge in edges {
                // Map sqlite IDs to in-memory IDs
                let src_mem_id = match sqlite_to_mem.get(&edge.src) {
                    Some((id, _, _)) => *id,
                    None => continue, // skip edges with unresolved src
                };
                let dst_mem_id = match sqlite_to_mem.get(&edge.dst) {
                    Some((id, _, _)) => *id,
                    None => continue, // skip edges with unresolved dst
                };

                let mut pm = sqlrustgo_graph::PropertyMap::new();
                if let Ok(props_map) = serde_json::from_value::<
                    serde_json::Map<String, serde_json::Value>,
                >(edge.properties.clone())
                {
                    for (k, v) in props_map {
                        let pv = match v {
                            serde_json::Value::Number(n) => {
                                sqlrustgo_graph::PropertyValue::Float(n.as_f64().unwrap_or(0.0))
                            }
                            _ => sqlrustgo_graph::PropertyValue::String(v.to_string()),
                        };
                        pm.insert(k, pv);
                    }
                }
                let _ = g.create_edge(src_mem_id, dst_mem_id, &edge.edge_type, pm);
            }
        }

        // Reload documents + embeddings from SQLite into MemoryStorage
        // This makes vector_search work after process restart
        if let (Ok(docs), Ok(embs)) = (
            backend.as_trait().load_documents(),
            backend.as_trait().load_embeddings(),
        ) {
            // Build a map: doc_id → embedding
            let emb_map: std::collections::HashMap<i64, Vec<f32>> =
                embs.into_iter().map(|e| (e.doc_id, e.embedding)).collect();

            for doc in docs {
                let kw: Vec<&str> = doc.keywords.iter().map(|s| s.as_str()).collect();
                // Check if we have an embedding for this doc
                if let Some(emb) = emb_map.get(&doc.id) {
                    // Import into MemoryStorage and then patch the embedding
                    if let Ok(doc_id) =
                        gmp.import_document(&doc.title, &doc.doc_type, &doc.content, &kw)
                    {
                        let _ = Self::patch_embedding(storage, doc_id, emb.clone());
                    }
                }
            }
        }
    }

    /// Patch an embedding directly in MemoryStorage (for reload from SQLite)
    fn patch_embedding(
        storage: &Arc<RwLock<MemoryStorage>>,
        doc_id: i64,
        embedding: Vec<f32>,
    ) -> SqlResult<()> {
        use sqlrustgo_gmp::vector_search::upsert_embedding;
        let mut s = storage.write().unwrap();
        upsert_embedding(&mut *s, doc_id, &embedding)
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
        properties: Option<&serde_json::Value>,
    ) -> SqlResult<i64> {
        let kw: Vec<&str> = keywords.iter().map(|s| s.as_str()).collect();

        // Write to MemoryStorage (existing path)
        let doc_id = self.gmp.import_document(title, doc_type, content, &kw)?;

        // Also write to SQLite (dual-write for persistence)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let props = properties.cloned().unwrap_or(serde_json::json!({}));

        let doc_rec = DocumentRecord {
            id: doc_id,
            title: title.to_string(),
            doc_type: doc_type.to_string(),
            content: content.to_string(),
            keywords: keywords.to_vec(),
            version: 1,
            created_at: now,
            updated_at: now,
            effective_date: None,
            status: "ACTIVE".to_string(),
            properties: props,
        };
        let _ = self.backend.as_trait().save_document(&doc_rec);

        // Generate embedding and write BLOB to SQLite
        let emb = embedding::generate_embedding(content);
        let emb_rec = EmbeddingRecord {
            doc_id,
            embedding: emb,
            updated_at: now,
        };
        let _ = self.backend.as_trait().save_embedding(&emb_rec);

        Ok(doc_id)
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
                // Build properties JSON for SQLite persistence
                let mut props_map = serde_json::Map::new();
                props_map.insert("name".to_string(), serde_json::json!(name));
                if let Some(c) = code {
                    props_map.insert("code".to_string(), serde_json::json!(c));
                }
                if let Some(p) = props.clone() {
                    if let serde_json::Value::Object(map) = p {
                        for (k, v) in map {
                            props_map.insert(k, v);
                        }
                    }
                }
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);

                // Create new node in-memory
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

                // Also write to SQLite (dual-write)
                let node_rec = NodeRecord {
                    id: id.0 as i64,
                    name: name.to_string(),
                    node_type: node_type.to_string(),
                    properties: serde_json::Value::Object(props_map),
                    created_at: now,
                    updated_at: now,
                };
                let _ = self.backend.as_trait().save_node(&node_rec);

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

        // Also write to SQLite (dual-write: resolve from/to names to ids via backend)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let edge_rec = EdgeUpsertRecord {
            from_name: from_name.to_string(),
            from_type: from_type.to_string(),
            to_name: to_name.to_string(),
            to_type: to_type.to_string(),
            rel_type: rel_type.to_string(),
            weight,
            created_at: now,
        };
        let _ = self.backend.as_trait().upsert_edge_by_names(&edge_rec);

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

        // Get neighbors (both outgoing AND incoming, since GMP graph edges are bidirectional)
        let neighbor_ids: Vec<NodeId> = match rel_type {
            Some(_rt) => {
                let out = graph.neighbors_by_edge_label(nid, rel_type.unwrap());
                let incoming = graph.incoming_neighbors_by_edge_label(nid, rel_type.unwrap());
                out.into_iter().chain(incoming).collect()
            }
            None => {
                let out = graph.outgoing_neighbors(nid);
                let incoming = graph.incoming_neighbors(nid);
                out.into_iter().chain(incoming).collect()
            }
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
            properties,
        } => {
            let doc_id = state
                .import_doc(&title, &doc_type, &content, &keywords, properties.as_ref())
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

        Request::SqlSearch {
            query,
            department,
            category,
            chapter,
            top_k,
        } => {
            use std::collections::HashMap;
            // Load all docs from SQLite backend, score by text match
            let docs = state
                .backend
                .as_trait()
                .load_documents()
                .map_err(|e| e.to_string())?;
            let query_lower = query.to_lowercase();
            let qwords: Vec<&str> = query_lower
                .split_whitespace()
                .filter(|w| w.len() > 1)
                .collect();

            let mut scored: Vec<(i64, String, f64)> = Vec::new();
            for doc in docs {
                // Apply filters
                if let Some(ref dept) = department {
                    let props_dept = doc
                        .properties
                        .get("department")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if !props_dept.contains(dept) {
                        continue;
                    }
                }
                if let Some(ref cat) = category {
                    let props_cat = doc
                        .properties
                        .get("category")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if !props_cat.contains(cat) {
                        continue;
                    }
                }
                if let Some(ref chap) = chapter {
                    let props_chap = doc
                        .properties
                        .get("chapter")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if !props_chap.contains(chap) {
                        continue;
                    }
                }

                // Score: count matching query words in title + content (first 500 chars)
                let title_lower = doc.title.to_lowercase();
                let content_snippet = doc
                    .content
                    .chars()
                    .take(500)
                    .collect::<String>()
                    .to_lowercase();
                let score = qwords
                    .iter()
                    .filter(|w| title_lower.contains(*w) || content_snippet.contains(*w))
                    .count() as f64;
                if score > 0.0 {
                    scored.push((doc.id, doc.title.clone(), score));
                }
            }

            // Sort by score descending, take top_k
            scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
            let top: Vec<_> = scored
                .into_iter()
                .take(top_k)
                .map(|(id, title, score)| {
                    serde_json::json!({
                        "doc_id": id,
                        "title": title,
                        "score": score,
                        "source": "exact"
                    })
                })
                .collect();
            Ok(Response::ok(
                serde_json::to_value(top).unwrap_or_default(),
                ms,
            ))
        }

        Request::TripleSearch { query, top_k } => {
            // Three-way RRF fusion: vector + graph + exact
            let k_rrf = 60;
            let mut doc_scores: HashMap<i64, f64> = HashMap::new();
            let mut doc_meta: HashMap<i64, serde_json::Value> = HashMap::new();

            // ── Path 1: Vector search ──────────────────────────────────────
            if let Ok(results) = state.vector_search(&query, top_k * 2) {
                for (pos, r) in results.into_iter().enumerate() {
                    let rrf = 1.0 / (k_rrf as f64 + pos as f64 + 1.0);
                    *doc_scores.entry(r.doc_id).or_insert(0.0) += rrf * 0.4;
                    doc_meta.entry(r.doc_id).or_insert_with(|| {
                        serde_json::json!({
                            "title": r.title,
                            "doc_type": r.doc_type,
                            "vector_score": r.similarity
                        })
                    });
                }
            }

            // ── Path 2: Exact SQL search ───────────────────────────────────
            if let Ok(docs) = state.backend.as_trait().load_documents() {
                let qwords: Vec<String> = query
                    .to_lowercase()
                    .split_whitespace()
                    .filter(|w| w.len() > 1)
                    .map(|s| s.to_string())
                    .collect();
                let mut exact: Vec<(i64, String, f64)> = Vec::new();
                for doc in docs {
                    let title_l = doc.title.to_lowercase();
                    let content_l = doc
                        .content
                        .chars()
                        .take(500)
                        .collect::<String>()
                        .to_lowercase();
                    let score = qwords
                        .iter()
                        .filter(|w| title_l.contains(w.as_str()) || content_l.contains(w.as_str()))
                        .count() as f64;
                    if score > 0.0 {
                        exact.push((doc.id, doc.title.clone(), score));
                    }
                }
                exact.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
                for (pos, (id, title, _score)) in exact.into_iter().take(top_k * 2).enumerate() {
                    let rrf = 1.0 / (k_rrf as f64 + pos as f64 + 1.0);
                    *doc_scores.entry(id).or_insert(0.0) += rrf * 0.3;
                    doc_meta.entry(id).or_insert_with(|| {
                        serde_json::json!({ "title": title, "doc_type": "SOP", "exact_score": _score })
                    });
                }
            }

            // ── Path 3: Graph search (BFS 2-hop from query keyword nodes) ──
            {
                let graph = state.graph.read().map_err(|e| e.to_string())?;
                let n2id = state.name_to_id.read().map_err(|e| e.to_string())?;
                // Find SOP nodes whose name contains any query keyword
                let qkw = query.to_lowercase();
                for (key, &nid) in n2id.iter() {
                    if key.starts_with("SOP:") && key.to_lowercase().contains(&qkw) {
                        // BFS 2-hop
                        let mut visited: HashMap<u64, usize> = HashMap::new(); // node_id -> min_hop
                        visited.insert(nid.0, 0);
                        let mut queue = vec![nid];
                        for hop in 1..=2 {
                            let mut next = Vec::new();
                            for current in queue.drain(..) {
                                for neighbor in graph.outgoing_neighbors(current) {
                                    let nid0 = neighbor.0;
                                    if !visited.contains_key(&nid0) {
                                        visited.insert(nid0, hop);
                                        next.push(neighbor);
                                    }
                                }
                            }
                            queue = next;
                        }
                        // Score each visited SOP node
                        let start_id = nid.0;
                        for (nid0, hop) in visited {
                            if nid0 == start_id {
                                continue;
                            }
                            let neighbor_key =
                                n2id.iter().find(|(_, &v)| v.0 == nid0).map(|(k, _)| k);
                            if let Some(nk) = neighbor_key {
                                if nk.starts_with("SOP:") {
                                    let rrf = 1.0 / (k_rrf as f64 + (hop as f64 * 10.0));
                                    // Use nid0 directly as doc_id proxy (not exact but for ranking)
                                    let doc_id_i64 = nid0 as i64;
                                    *doc_scores.entry(doc_id_i64).or_insert(0.0) += rrf * 0.3;
                                }
                            }
                        }
                    }
                }
            }

            // ── Sort and return top_k ──────────────────────────────────────
            let mut ranked: Vec<_> = doc_scores.into_iter().collect();
            ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let results: Vec<_> = ranked
                .into_iter()
                .take(top_k)
                .map(|(doc_id, score)| {
                    let mut item = doc_meta.remove(&doc_id).unwrap_or(serde_json::json!({}));
                    item["doc_id"] = serde_json::json!(doc_id);
                    item["rrf_score"] = serde_json::json!(score);
                    item
                })
                .collect();
            Ok(Response::ok(
                serde_json::to_value(results).unwrap_or_default(),
                ms,
            ))
        }

        Request::SqlExec { .. } => Ok(Response::err(
            "SQL exec not supported in this build".to_string(),
            ms,
        )),
    }
}
