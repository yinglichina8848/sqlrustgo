//! SQLite persistence layer for GMP CLI
//!
//! Implements the `StorageBackend` trait using rusqlite as the persistence adapter.
//! This is Stage 1 of the SQLRustGo migration path (SQLite → SQLRustGo kernel).
//!
//! # Design
//!
//! - All GMP data persisted in a single SQLite file
//! - `StorageBackend` trait allows future backend swap (SQLite → SQLRustGo)
//! - Embedding vectors stored as BLOB (zero-copy via bytemuck)
//! - Graph uses adjacency-friendly schema with indexes for O(1) lookup
//!
//! # Schema
//!
//! - `gmp_documents`: document metadata + content
//! - `gmp_embeddings`: doc_id → 3584-dim f32 BLOB
//! - `graph_nodes`: node_id, name, type, properties (JSON)
//! - `graph_edges`: src, dst, edge_type, properties (JSON)

use bytemuck::cast_slice;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

/// Default runtime home directory, overridden by $SQLRUSTGO_HOME env var
pub fn default_home() -> PathBuf {
    std::env::var("SQLRUSTGO_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".sqlrustgo")
        })
}

/// GMP database path (SQLite file)
pub fn gmp_db_path() -> PathBuf {
    let home = default_home();
    std::fs::create_dir_all(&home).ok();
    home.join("gmp.db")
}

/// Graph database path (separate SQLite file)
pub fn graph_db_path() -> PathBuf {
    let home = default_home();
    std::fs::create_dir_all(&home).ok();
    home.join("graph.db")
}

// ============================================================================
// Record types
// ============================================================================

/// Embedding record stored in `gmp_embeddings`
#[derive(Debug, Clone)]
pub struct EmbeddingRecord {
    pub doc_id: i64,
    pub embedding: Vec<f32>, // 3584-dim qwen2.5
    pub updated_at: i64,
}

/// Document record stored in `gmp_documents`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRecord {
    pub id: i64,
    pub title: String,
    pub doc_type: String,
    pub content: String,
    pub keywords: Vec<String>,
    pub properties: serde_json::Value, // department, category, chapter, devices
    pub version: i32,
    pub created_at: i64,
    pub updated_at: i64,
    pub effective_date: Option<i32>,
    pub status: String,
}

/// Graph node record from `graph_nodes`
#[derive(Debug, Clone)]
pub struct NodeRecord {
    pub id: i64,
    pub name: String,
    pub node_type: String,
    pub properties: serde_json::Value,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Graph edge record from `graph_edges`
#[derive(Debug, Clone)]
pub struct EdgeRecord {
    pub id: i64,
    pub src: i64,
    pub dst: i64,
    pub edge_type: String,
    pub properties: serde_json::Value,
    pub created_at: i64,
}

/// Persistent edge representation using string names (for upsert by name)
#[derive(Debug, Clone)]
pub struct EdgeUpsertRecord {
    pub from_name: String,
    pub from_type: String,
    pub to_name: String,
    pub to_type: String,
    pub rel_type: String,
    pub weight: Option<f64>,
    pub created_at: i64,
}

// ============================================================================
// StorageBackend trait (Stage 1 → Stage 3 migration contract)
// ============================================================================

/// Unified storage backend trait.
///
/// Stage 1: SQLiteBackend
/// Stage 2: SQLRustGoBackend (future)
/// Stage 3: Unified kernel
///
/// CLI and business logic depend ONLY on this trait.
/// Switching backends requires zero CLI changes.
pub trait StorageBackend: Send + Sync {
    /// Load all document records
    fn load_documents(&self) -> Result<Vec<DocumentRecord>, String>;

    /// Load all embedding records
    fn load_embeddings(&self) -> Result<Vec<EmbeddingRecord>, String>;

    /// Save a single document (insert or replace by doc_id)
    fn save_document(&self, doc: &DocumentRecord) -> Result<i64, String>;

    /// Save a single embedding (insert or replace by doc_id)
    fn save_embedding(&self, emb: &EmbeddingRecord) -> Result<(), String>;

    /// Load all graph nodes
    fn load_nodes(&self) -> Result<Vec<NodeRecord>, String>;

    /// Save a graph node (insert or update by name+type)
    fn save_node(&self, node: &NodeRecord) -> Result<i64, String>;

    /// Load all graph edges
    fn load_edges(&self) -> Result<Vec<EdgeRecord>, String>;

    /// Save an edge (insert or replace)
    fn save_edge(&self, edge: &EdgeRecord) -> Result<i64, String>;

    /// Upsert edge by node names (resolve name → id, create nodes if missing)
    fn upsert_edge_by_names(&self, edge: &EdgeUpsertRecord) -> Result<(), String>;

    /// Get a node id by its type+name, or None
    fn get_node_id(&self, node_type: &str, name: &str) -> Result<Option<i64>, String>;

    /// Get a node record by type+name
    fn get_node(&self, node_type: &str, name: &str) -> Result<Option<NodeRecord>, String>;

    /// Initialize / create all tables and indexes
    fn init(&self) -> Result<(), String>;
}

// ============================================================================
// SQLite backend implementation
// ============================================================================

/// SQLite implementation of StorageBackend.
/// All data stored in $SQLRUSTGO_HOME/gmp.db (docs+embeddings) and
/// $SQLRUSTGO_HOME/graph.db (graph).
pub struct SqliteBackend {
    /// Mutex needed because rusqlite::Connection is not Send+Sync
    gmp_conn: Mutex<Connection>,
    graph_conn: Mutex<Connection>,
}

impl SqliteBackend {
    /// Open or create the GMP and graph SQLite databases
    pub fn open() -> Result<Self, String> {
        let gmp_path = gmp_db_path();
        let graph_path = graph_db_path();

        let gmp_conn = Connection::open(&gmp_path)
            .map_err(|e| format!("failed to open gmp.db: {}", e))?;
        let graph_conn = Connection::open(&graph_path)
            .map_err(|e| format!("failed to open graph.db: {}", e))?;

        Ok(Self {
            gmp_conn: Mutex::new(gmp_conn),
            graph_conn: Mutex::new(graph_conn),
        })
    }

    /// Get a connection suitable for use as `&dyn StorageBackend`
    pub fn as_trait(&self) -> &dyn StorageBackend {
        self
    }
}

impl StorageBackend for SqliteBackend {
    fn init(&self) -> Result<(), String> {
        // GMP schema: documents + embeddings
        self.gmp_conn.lock().unwrap().execute_batch(
            r#"
                CREATE TABLE IF NOT EXISTS gmp_documents (
                    id            INTEGER PRIMARY KEY AUTOINCREMENT,
                    title         TEXT NOT NULL,
                    doc_type      TEXT NOT NULL,
                    content       TEXT NOT NULL,
                    keywords      TEXT NOT NULL,
                    properties    TEXT NOT NULL DEFAULT '{}',
                    version       INTEGER DEFAULT 1,
                    created_at    INTEGER NOT NULL,
                    updated_at    INTEGER NOT NULL,
                    effective_date INTEGER,
                    status        TEXT DEFAULT 'ACTIVE'
                );

                CREATE TABLE IF NOT EXISTS gmp_embeddings (
                    doc_id      INTEGER PRIMARY KEY,
                    embedding   BLOB NOT NULL,
                    updated_at  INTEGER NOT NULL
                );
            "#,
        ).map_err(|e| format!("gmp init error: {}", e))?;

        // Graph schema
        self.graph_conn.lock().unwrap().execute_batch(
            r#"
                CREATE TABLE IF NOT EXISTS graph_nodes (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    name        TEXT NOT NULL,
                    node_type   TEXT NOT NULL,
                    properties  TEXT NOT NULL DEFAULT '{}',
                    created_at  INTEGER NOT NULL,
                    updated_at  INTEGER NOT NULL,
                    UNIQUE(node_type, name)
                );

                CREATE TABLE IF NOT EXISTS graph_edges (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    src         INTEGER NOT NULL,
                    dst         INTEGER NOT NULL,
                    edge_type   TEXT NOT NULL,
                    properties  TEXT NOT NULL DEFAULT '{}',
                    created_at  INTEGER NOT NULL,
                    FOREIGN KEY (src) REFERENCES graph_nodes(id) ON DELETE CASCADE,
                    FOREIGN KEY (dst) REFERENCES graph_nodes(id) ON DELETE CASCADE,
                    UNIQUE(src, dst, edge_type)
                );

                CREATE INDEX IF NOT EXISTS idx_node_name ON graph_nodes(name);
                CREATE INDEX IF NOT EXISTS idx_node_type ON graph_nodes(node_type);
                CREATE INDEX IF NOT EXISTS idx_edge_src  ON graph_edges(src);
                CREATE INDEX IF NOT EXISTS idx_edge_dst  ON graph_edges(dst);
            "#,
        ).map_err(|e| format!("graph init error: {}", e))?;

        Ok(())
    }

    fn load_documents(&self) -> Result<Vec<DocumentRecord>, String> {
        let conn = self.gmp_conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, title, doc_type, content, keywords, properties, version,
                        created_at, updated_at, effective_date, status FROM gmp_documents",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let keywords_str: String = row.get(4)?;
                let keywords: Vec<String> = serde_json::from_str(&keywords_str).unwrap_or_default();
                let props_str: String = row.get(5)?;
                let properties: serde_json::Value =
                    serde_json::from_str(&props_str).unwrap_or(serde_json::json!({}));
                Ok(DocumentRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    doc_type: row.get(2)?,
                    content: row.get(3)?,
                    keywords,
                    properties,
                    version: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                    effective_date: row.get(9)?,
                    status: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(rows)
    }

    fn load_embeddings(&self) -> Result<Vec<EmbeddingRecord>, String> {
        let conn = self.gmp_conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT doc_id, embedding, updated_at FROM gmp_embeddings")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let blob: Vec<u8> = row.get(1)?;
                let embedding: Vec<f32> = if blob.len() % 4 == 0 {
                    cast_slice(&blob).to_vec()
                } else {
                    vec![]
                };
                Ok(EmbeddingRecord {
                    doc_id: row.get(0)?,
                    embedding,
                    updated_at: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(rows)
    }

    fn save_document(&self, doc: &DocumentRecord) -> Result<i64, String> {
        let keywords_json = serde_json::to_string(&doc.keywords).unwrap_or_else(|_| "[]".into());
        let properties_json = serde_json::to_string(&doc.properties).unwrap_or_else(|_| "{}".into());
        self.gmp_conn.lock().unwrap()
            .execute(
                r#"INSERT INTO gmp_documents
                   (title, doc_type, content, keywords, properties, version, created_at, updated_at, effective_date, status)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                   ON CONFLICT(id) DO UPDATE SET
                   title=excluded.title, doc_type=excluded.doc_type,
                   content=excluded.content, keywords=excluded.keywords,
                   properties=excluded.properties,
                   updated_at=excluded.updated_at, status=excluded.status"#,
                params![
                    doc.title, doc.doc_type, doc.content, keywords_json, properties_json,
                    doc.version, doc.created_at, doc.updated_at,
                    doc.effective_date, doc.status,
                ],
            )
            .map_err(|e| e.to_string())?;

        let doc_id = self.gmp_conn.lock().unwrap()
            .query_row(
                "SELECT id FROM gmp_documents WHERE title=?1 AND doc_type=?2 LIMIT 1",
                params![doc.title, doc.doc_type],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?;

        Ok(doc_id)
    }

    fn save_embedding(&self, emb: &EmbeddingRecord) -> Result<(), String> {
        let blob = cast_slice(&emb.embedding).to_vec();
        self.gmp_conn.lock().unwrap()
            .execute(
                "INSERT OR REPLACE INTO gmp_embeddings (doc_id, embedding, updated_at) VALUES (?1, ?2, ?3)",
                params![emb.doc_id, blob, emb.updated_at],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn load_nodes(&self) -> Result<Vec<NodeRecord>, String> {
        let conn = self.graph_conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, node_type, properties, created_at, updated_at FROM graph_nodes",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let props_str: String = row.get(3)?;
                let properties = serde_json::from_str(&props_str)
                    .unwrap_or(serde_json::Value::Object(Default::default()));
                Ok(NodeRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    node_type: row.get(2)?,
                    properties,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(rows)
    }

    fn save_node(&self, node: &NodeRecord) -> Result<i64, String> {
        let props_json =
            serde_json::to_string(&node.properties).unwrap_or_else(|_| "{}".into());
        self.graph_conn.lock().unwrap()
            .execute(
                "INSERT INTO graph_nodes (name, node_type, properties, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(node_type, name) DO UPDATE SET
                 properties=excluded.properties, updated_at=excluded.updated_at",
                params![
                    node.name, node.node_type, props_json,
                    node.created_at, node.updated_at,
                ],
            )
            .map_err(|e| e.to_string())?;

        let node_id = self.graph_conn.lock().unwrap()
            .query_row(
                "SELECT id FROM graph_nodes WHERE node_type=?1 AND name=?2 LIMIT 1",
                params![node.node_type, node.name],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?;

        Ok(node_id)
    }

    fn load_edges(&self) -> Result<Vec<EdgeRecord>, String> {
        let conn = self.graph_conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, src, dst, edge_type, properties, created_at FROM graph_edges")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let props_str: String = row.get(3)?;
                let properties = serde_json::from_str(&props_str)
                    .unwrap_or(serde_json::Value::Object(Default::default()));
                Ok(EdgeRecord {
                    id: row.get(0)?,
                    src: row.get(1)?,
                    dst: row.get(2)?,
                    edge_type: row.get(3)?,
                    properties,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(rows)
    }

    fn save_edge(&self, edge: &EdgeRecord) -> Result<i64, String> {
        let props_json =
            serde_json::to_string(&edge.properties).unwrap_or_else(|_| "{}".into());
        self.graph_conn.lock().unwrap()
            .execute(
                "INSERT OR REPLACE INTO graph_edges (src, dst, edge_type, properties, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    edge.src, edge.dst, edge.edge_type, props_json, edge.created_at,
                ],
            )
            .map_err(|e| e.to_string())?;

        let edge_id = self.graph_conn.lock().unwrap()
            .query_row(
                "SELECT id FROM graph_edges WHERE src=?1 AND dst=?2 AND edge_type=?3 LIMIT 1",
                params![edge.src, edge.dst, edge.edge_type],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?;

        Ok(edge_id)
    }

    fn upsert_edge_by_names(&self, edge: &EdgeUpsertRecord) -> Result<(), String> {
        // Resolve or create from_node
        let from_id = match self.get_node_id_internal(&edge.from_type, &edge.from_name)? {
            Some(id) => id,
            None => {
                let now = edge.created_at;
                let node = NodeRecord {
                    id: 0,
                    name: edge.from_name.clone(),
                    node_type: edge.from_type.clone(),
                    properties: serde_json::json!({}),
                    created_at: now,
                    updated_at: now,
                };
                self.save_node(&node)?
            }
        };

        // Resolve or create to_node
        let to_id = match self.get_node_id_internal(&edge.to_type, &edge.to_name)? {
            Some(id) => id,
            None => {
                let now = edge.created_at;
                let node = NodeRecord {
                    id: 0,
                    name: edge.to_name.clone(),
                    node_type: edge.to_type.clone(),
                    properties: serde_json::json!({}),
                    created_at: now,
                    updated_at: now,
                };
                self.save_node(&node)?
            }
        };

        let mut props = serde_json::Map::new();
        if let Some(w) = edge.weight {
            props.insert("weight".to_string(), serde_json::json!(w));
        }
        let props_val = serde_json::Value::Object(props);

        let edge_rec = EdgeRecord {
            id: 0,
            src: from_id,
            dst: to_id,
            edge_type: edge.rel_type.clone(),
            properties: props_val,
            created_at: edge.created_at,
        };
        self.save_edge(&edge_rec)?;
        Ok(())
    }

    fn get_node_id(&self, node_type: &str, name: &str) -> Result<Option<i64>, String> {
        self.get_node_id_internal(node_type, name)
    }

    fn get_node(&self, node_type: &str, name: &str) -> Result<Option<NodeRecord>, String> {
        let result = self.graph_conn.lock().unwrap()
            .query_row(
                "SELECT id, name, node_type, properties, created_at, updated_at
                 FROM graph_nodes WHERE node_type=?1 AND name=?2 LIMIT 1",
                params![node_type, name],
                |row| {
                    let props_str: String = row.get(3)?;
                    let properties = serde_json::from_str(&props_str)
                        .unwrap_or(serde_json::Value::Object(Default::default()));
                    Ok(NodeRecord {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        node_type: row.get(2)?,
                        properties,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                },
            );

        match result {
            Ok(node) => Ok(Some(node)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }
}

// Private helper impl (Mutex forces us to duplicate logic)
impl SqliteBackend {
    fn get_node_id_internal(&self, node_type: &str, name: &str) -> Result<Option<i64>, String> {
        let result = self.graph_conn.lock().unwrap()
            .query_row(
                "SELECT id FROM graph_nodes WHERE node_type=?1 AND name=?2 LIMIT 1",
                params![node_type, name],
                |row| row.get::<_, i64>(0),
            );

        match result {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }
}
