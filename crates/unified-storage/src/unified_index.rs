//! Unified Index - Manage SQL, Vector, and Graph indexes from a single interface

use serde::{Deserialize, Serialize};

/// Unified index types across SQL, Vector, and Graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnifiedIndexType {
    /// B+Tree index for range queries
    BTree,
    /// Hash index for point lookups
    Hash,
    /// Composite B+Tree for multi-column
    CompositeBTree,
    /// Flat vector index (brute-force)
    VectorFlat,
    /// HNSW vector index
    VectorHnsw,
    /// IVF vector index
    VectorIvf,
    /// Graph index for traversal
    Graph,
}

impl UnifiedIndexType {
    /// Check if this is a vector index type
    pub fn is_vector(&self) -> bool {
        matches!(
            self,
            UnifiedIndexType::VectorFlat
                | UnifiedIndexType::VectorHnsw
                | UnifiedIndexType::VectorIvf
        )
    }

    /// Check if this is a SQL index type
    pub fn is_sql(&self) -> bool {
        matches!(
            self,
            UnifiedIndexType::BTree | UnifiedIndexType::Hash | UnifiedIndexType::CompositeBTree
        )
    }

    /// Check if this is a graph index type
    pub fn is_graph(&self) -> bool {
        matches!(self, UnifiedIndexType::Graph)
    }
}

/// Unified index ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnifiedIndexId(pub u32);

impl UnifiedIndexId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Metadata for a unified index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedIndexMeta {
    /// Index ID
    pub id: UnifiedIndexId,
    /// Index name
    pub name: String,
    /// Index type
    pub index_type: UnifiedIndexType,
    /// Column names this index applies to
    pub column_names: Vec<String>,
    /// Index configuration (JSON for flexibility)
    pub config: Option<String>,
}

impl UnifiedIndexMeta {
    /// Create new index metadata
    pub fn new(
        id: UnifiedIndexId,
        name: &str,
        index_type: UnifiedIndexType,
        columns: Vec<String>,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            index_type,
            column_names: columns,
            config: None,
        }
    }

    /// Create with configuration
    pub fn with_config(mut self, config: &str) -> Self {
        self.config = Some(config.to_string());
        self
    }
}

/// Unified Index manager for a document table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedIndex {
    /// Table name this index belongs to
    table_name: String,
    /// All indexes
    indexes: Vec<UnifiedIndexMeta>,
}

impl UnifiedIndex {
    /// Create a new unified index manager for a table
    pub fn new(table_name: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            indexes: Vec::new(),
        }
    }

    /// Add a B+Tree index
    pub fn add_btree_index(&mut self, columns: Vec<String>) -> UnifiedIndexId {
        let id = UnifiedIndexId(self.indexes.len() as u32);
        let name = format!("btree_{}_{}", self.table_name, columns.join("_"));
        self.indexes.push(UnifiedIndexMeta::new(
            id,
            &name,
            UnifiedIndexType::BTree,
            columns,
        ));
        id
    }

    /// Add a Hash index
    pub fn add_hash_index(&mut self, columns: Vec<String>) -> UnifiedIndexId {
        let id = UnifiedIndexId(self.indexes.len() as u32);
        let name = format!("hash_{}_{}", self.table_name, columns.join("_"));
        self.indexes.push(UnifiedIndexMeta::new(
            id,
            &name,
            UnifiedIndexType::Hash,
            columns,
        ));
        id
    }

    /// Add a composite B+Tree index
    pub fn add_composite_index(&mut self, columns: Vec<String>) -> UnifiedIndexId {
        let id = UnifiedIndexId(self.indexes.len() as u32);
        let name = format!("composite_{}_{}", self.table_name, columns.join("_"));
        self.indexes.push(UnifiedIndexMeta::new(
            id,
            &name,
            UnifiedIndexType::CompositeBTree,
            columns,
        ));
        id
    }

    /// Add a flat vector index
    pub fn add_flat_vector_index(&mut self, column: &str, dimension: usize) -> UnifiedIndexId {
        let id = UnifiedIndexId(self.indexes.len() as u32);
        let name = format!("vec_flat_{}_{}", self.table_name, column);
        let config = serde_json::json!({
            "dimension": dimension,
            "metric": "cosine"
        })
        .to_string();
        self.indexes.push(
            UnifiedIndexMeta::new(
                id,
                &name,
                UnifiedIndexType::VectorFlat,
                vec![column.to_string()],
            )
            .with_config(&config),
        );
        id
    }

    /// Add an HNSW vector index
    pub fn add_hnsw_vector_index(
        &mut self,
        column: &str,
        dimension: usize,
        m: usize,
        ef_construction: usize,
    ) -> UnifiedIndexId {
        let id = UnifiedIndexId(self.indexes.len() as u32);
        let name = format!("vec_hnsw_{}_{}", self.table_name, column);
        let config = serde_json::json!({
            "dimension": dimension,
            "metric": "cosine",
            "m": m,
            "ef_construction": ef_construction
        })
        .to_string();
        self.indexes.push(
            UnifiedIndexMeta::new(
                id,
                &name,
                UnifiedIndexType::VectorHnsw,
                vec![column.to_string()],
            )
            .with_config(&config),
        );
        id
    }

    /// Add an IVF vector index
    pub fn add_ivf_vector_index(
        &mut self,
        column: &str,
        dimension: usize,
        nlist: usize,
    ) -> UnifiedIndexId {
        let id = UnifiedIndexId(self.indexes.len() as u32);
        let name = format!("vec_ivf_{}_{}", self.table_name, column);
        let config = serde_json::json!({
            "dimension": dimension,
            "metric": "cosine",
            "nlist": nlist
        })
        .to_string();
        self.indexes.push(
            UnifiedIndexMeta::new(
                id,
                &name,
                UnifiedIndexType::VectorIvf,
                vec![column.to_string()],
            )
            .with_config(&config),
        );
        id
    }

    /// Add a graph index
    pub fn add_graph_index(&mut self, column: &str) -> UnifiedIndexId {
        let id = UnifiedIndexId(self.indexes.len() as u32);
        let name = format!("graph_{}_{}", self.table_name, column);
        self.indexes.push(UnifiedIndexMeta::new(
            id,
            &name,
            UnifiedIndexType::Graph,
            vec![column.to_string()],
        ));
        id
    }

    /// Get table name
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// Get all indexes
    pub fn indexes(&self) -> &[UnifiedIndexMeta] {
        &self.indexes
    }

    /// Get index count
    pub fn index_count(&self) -> usize {
        self.indexes.len()
    }

    /// Get SQL indexes only
    pub fn sql_indexes(&self) -> impl Iterator<Item = &UnifiedIndexMeta> {
        self.indexes.iter().filter(|i| i.index_type.is_sql())
    }

    /// Get vector indexes only
    pub fn vector_indexes(&self) -> impl Iterator<Item = &UnifiedIndexMeta> {
        self.indexes.iter().filter(|i| i.index_type.is_vector())
    }

    /// Get graph indexes only
    pub fn graph_indexes(&self) -> impl Iterator<Item = &UnifiedIndexMeta> {
        self.indexes.iter().filter(|i| i.index_type.is_graph())
    }

    /// Find index by name
    pub fn find_by_name(&self, name: &str) -> Option<&UnifiedIndexMeta> {
        self.indexes.iter().find(|i| i.name == name)
    }

    /// Find index by ID
    pub fn find_by_id(&self, id: UnifiedIndexId) -> Option<&UnifiedIndexMeta> {
        self.indexes.iter().find(|i| i.id == id)
    }

    /// Check if table has any vector indexes
    pub fn has_vector_index(&self) -> bool {
        self.indexes.iter().any(|i| i.index_type.is_vector())
    }

    /// Check if table has any graph indexes
    pub fn has_graph_index(&self) -> bool {
        self.indexes.iter().any(|i| i.index_type.is_graph())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_index_creation() {
        let mut index = UnifiedIndex::new("products");

        let btree_id = index.add_btree_index(vec!["category".to_string()]);
        let vector_id = index.add_hnsw_vector_index("embedding", 384, 16, 200);
        let graph_id = index.add_graph_index("node_ref");

        assert_eq!(index.index_count(), 3);
        assert_eq!(index.sql_indexes().count(), 1);
        assert_eq!(index.vector_indexes().count(), 1);
        assert_eq!(index.graph_indexes().count(), 1);

        assert!(index.has_vector_index());
        assert!(index.has_graph_index());

        assert_eq!(
            index.find_by_id(btree_id).unwrap().name,
            "btree_products_category"
        );
        assert_eq!(
            index.find_by_id(vector_id).unwrap().name,
            "vec_hnsw_products_embedding"
        );
    }

    #[test]
    fn test_index_types() {
        assert!(UnifiedIndexType::BTree.is_sql());
        assert!(UnifiedIndexType::VectorHnsw.is_vector());
        assert!(UnifiedIndexType::Graph.is_graph());
        assert!(!UnifiedIndexType::Graph.is_vector());
    }

    #[test]
    fn test_unified_index_id() {
        let id = UnifiedIndexId::new(42);
        assert_eq!(id.0, 42);

        let id2 = id;
        assert_eq!(id2.0, 42);
    }

    #[test]
    fn test_unified_index_meta() {
        let meta = UnifiedIndexMeta::new(
            UnifiedIndexId::new(1),
            "idx_name",
            UnifiedIndexType::BTree,
            vec!["col1".to_string(), "col2".to_string()],
        );

        assert_eq!(meta.name, "idx_name");
        assert_eq!(meta.column_names.len(), 2);
        assert!(meta.config.is_none());

        let meta_with_config = meta.with_config(r#"{"level": 10}"#);
        assert!(meta_with_config.config.is_some());
    }

    #[test]
    fn test_unified_index_find_methods() {
        let mut index = UnifiedIndex::new("test");

        let btree_id = index.add_btree_index(vec!["col1".to_string()]);
        let hash_id = index.add_hash_index(vec!["col2".to_string()]);

        assert!(index.find_by_name("btree_test_col1").is_some());
        assert!(index.find_by_name("nonexistent").is_none());

        // Find by id
        assert!(index.find_by_id(btree_id).is_some());
        assert!(index.find_by_id(hash_id).is_some());
        let invalid_id = UnifiedIndexId::new(9999);
        assert!(index.find_by_id(invalid_id).is_none());
    }

    #[test]
    fn test_unified_index_type_display() {
        assert_eq!(format!("{:?}", UnifiedIndexType::BTree), "BTree");
        assert_eq!(format!("{:?}", UnifiedIndexType::VectorHnsw), "VectorHnsw");
        assert_eq!(format!("{:?}", UnifiedIndexType::Graph), "Graph");
    }

    #[test]
    fn test_unified_index_meta_debug() {
        let meta = UnifiedIndexMeta::new(
            UnifiedIndexId::new(1),
            "test_idx",
            UnifiedIndexType::Hash,
            vec!["id".to_string()],
        );
        let debug = format!("{:?}", meta);
        assert!(debug.contains("test_idx"));
        assert!(debug.contains("Hash"));
    }
}
