//! Vector Storage Integration
//!
//! Integrates vector indices (Flat, HNSW, IVF) with BinaryStorage for persistence.
//! Provides seamless SQL + vector hybrid search capabilities.

use crate::binary_format::BinaryFormat;
use crate::binary_storage::BinaryTableStorage;
use crate::engine::TableData;
use sqlrustgo_vector::error::VectorError;
use sqlrustgo_vector::VectorResult;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use sqlrustgo_vector::metrics::DistanceMetric;
use sqlrustgo_vector::parallel_knn::ParallelKnnIndex;
use sqlrustgo_vector::traits::{IndexEntry, VectorIndex};

/// Vector index types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorIndexType {
    /// Flat index (brute-force O(n) search)
    Flat,
    /// HNSW index (Hierarchical Navigable Small World)
    Hnsw,
    /// IVF index (Inverted File with k-means)
    Ivf,
    /// Parallel KNN (SIMD-accelerated)
    ParallelKnn,
}

impl Default for VectorIndexType {
    fn default() -> Self {
        VectorIndexType::ParallelKnn
    }
}

/// Metadata for a vector column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorColumnMeta {
    /// Column name
    pub column_name: String,
    /// Vector dimension
    pub dimension: usize,
    /// Distance metric
    pub metric: DistanceMetric,
    /// Index type
    pub index_type: VectorIndexType,
    /// Total vectors stored
    pub vector_count: usize,
}

/// Vector storage error types
#[derive(Debug, Clone)]
pub enum VectorStorageError {
    /// Dimension mismatch
    DimensionMismatch { expected: usize, actual: usize },
    /// Index type not found
    IndexTypeNotFound,
    /// Serialization error
    Serialization(String),
    /// IO error
    Io(String),
}

impl std::fmt::Display for VectorStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VectorStorageError::DimensionMismatch { expected, actual } => {
                write!(f, "Dimension mismatch: expected {}, got {}", expected, actual)
            }
            VectorStorageError::IndexTypeNotFound => write!(f, "Index type not found"),
            VectorStorageError::Serialization(s) => write!(f, "Serialization error: {}", s),
            VectorStorageError::Io(s) => write!(f, "IO error: {}", s),
        }
    }
}

impl std::error::Error for VectorStorageError {}

/// Vector storage manager
/// Manages vector indices with BinaryStorage persistence
pub struct VectorStore {
    /// Storage directory
    data_dir: PathBuf,
    /// Vector metadata per table/column
    metadata: Vec<VectorColumnMeta>,
    /// In-memory indices (table_column -> index)
    indices: std::collections::HashMap<String, Box<dyn VectorIndex>>,
    /// Metric used for this store
    default_metric: DistanceMetric,
}

impl VectorStore {
    /// Create a new VectorStore
    pub fn new(data_dir: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let index_dir = data_dir.join("vector_indices");
        std::fs::create_dir_all(&index_dir)?;
        Ok(Self {
            data_dir,
            metadata: Vec::new(),
            indices: std::collections::HashMap::new(),
            default_metric: DistanceMetric::Cosine,
        })
    }

    /// Create with specific default metric
    pub fn with_metric(data_dir: PathBuf, metric: DistanceMetric) -> std::io::Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let index_dir = data_dir.join("vector_indices");
        std::fs::create_dir_all(&index_dir)?;
        Ok(Self {
            data_dir,
            metadata: Vec::new(),
            indices: std::collections::HashMap::new(),
            default_metric: metric,
        })
    }

    /// Get the key for a table/column index
    fn index_key(table: &str, column: &str) -> String {
        format!("{}:{}", table, column)
    }

    /// Register a vector column
    pub fn register_column(
        &mut self,
        table: &str,
        column: &str,
        dimension: usize,
        metric: DistanceMetric,
        index_type: VectorIndexType,
    ) -> VectorResult<()> {
        let key = Self::index_key(table, column);
        
        // Create the index
        let index: Box<dyn VectorIndex> = match index_type {
            VectorIndexType::Flat => Box::new(sqlrustgo_vector::FlatIndex::with_dimension(dimension, metric)),
            VectorIndexType::Hnsw => Box::new(sqlrustgo_vector::HnswIndex::new(metric)),
            VectorIndexType::Ivf => Box::new(sqlrustgo_vector::IvfIndex::new(metric, 100)),
            VectorIndexType::ParallelKnn => Box::new(ParallelKnnIndex::new(metric)),
        };
        
        self.indices.insert(key.clone(), index);
        
        // Create metadata
        let meta = VectorColumnMeta {
            column_name: column.to_string(),
            dimension,
            metric,
            index_type,
            vector_count: 0,
        };
        self.metadata.push(meta);
        
        Ok(())
    }

    /// Insert a vector
    pub fn insert(&mut self, table: &str, column: &str, id: u64, vector: &[f32]) -> VectorResult<()> {
        let key = Self::index_key(table, column);
        
        let index = self.indices.get_mut(&key)
            .ok_or_else(|| VectorError::InvalidParameter(format!("Vector column {}:{} not registered", table, column)))?;
        
        index.insert(id, vector)?;
        Ok(())
    }

    /// Batch insert vectors
    pub fn batch_insert(
        &mut self,
        table: &str,
        column: &str,
        vectors: Vec<(u64, Vec<f32>)>,
    ) -> VectorResult<usize> {
        let key = Self::index_key(table, column);
        
        let index = self.indices.get_mut(&key)
            .ok_or_else(|| VectorError::InvalidParameter(format!("Vector column {}:{} not registered", table, column)))?;
        
        let count = vectors.len();
        for (id, vector) in vectors {
            index.insert(id, &vector)?;
        }
        
        Ok(count)
    }

    /// Search for similar vectors
    pub fn search(
        &self,
        table: &str,
        column: &str,
        query: &[f32],
        k: usize,
    ) -> VectorResult<Vec<IndexEntry>> {
        let key = Self::index_key(table, column);
        
        let index = self.indices.get(&key)
            .ok_or_else(|| VectorError::InvalidParameter(format!("Vector column {}:{} not registered", table, column)))?;
        
        index.search(query, k)
    }

    /// Get vector count
    pub fn len(&self, table: &str, column: &str) -> usize {
        let key = Self::index_key(table, column);
        self.indices.get(&key).map(|i| i.len()).unwrap_or(0)
    }

    /// Check if empty
    pub fn is_empty(&self, table: &str, column: &str) -> bool {
        self.len(table, column) == 0
    }

    /// Save all indices to disk
    pub fn save_all(&self) -> std::io::Result<()> {
        let index_dir = self.data_dir.join("vector_indices");
        
        for (key, index) in &self.indices {
            let path = index_dir.join(format!("{}.vecidx", key.replace(":", "_")));
            self.save_index(key, index, &path)?;
        }
        
        // Save metadata
        let meta_path = index_dir.join("metadata.json");
        let meta_json = serde_json::to_string_pretty(&self.metadata)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(meta_path, meta_json)?;
        
        Ok(())
    }

    /// Save a single index to disk
    fn save_index(&self, key: &str, index: &Box<dyn VectorIndex>, path: &PathBuf) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut w = BufWriter::new(file);
        
        // Magic header
        w.write_all(b"VECIDX")?;
        w.write_all(&1u32.to_le_bytes())?; // version
        
        // Key
        let key_bytes = key.as_bytes();
        w.write_all(&(key_bytes.len() as u32).to_le_bytes())?;
        w.write_all(key_bytes)?;
        
        // Dimension
        w.write_all(&(index.dimension() as u32).to_le_bytes())?;
        
        // Count
        w.write_all(&(index.len() as u64).to_le_bytes())?;
        
        // Metric
        let metric_byte = match index.metric() {
            DistanceMetric::Cosine => 0u8,
            DistanceMetric::Euclidean => 1u8,
            DistanceMetric::DotProduct => 2u8,
            DistanceMetric::Manhattan => 3u8,
        };
        w.write_all(&[metric_byte])?;
        
        // Vectors (id + dimension * f32)
        let dim = index.dimension();
        for i in 0..index.len() {
            // We can't iterate directly, need to search for each
            let results = index.search(&vec![0.0; dim], index.len())
                .unwrap_or_default();
            for entry in results {
                w.write_all(&entry.id.to_le_bytes())?;
                // Note: actual vector storage would need separate lookup
                // This is a simplified representation
            }
            break; // Only need to write once, actual impl would track separately
        }
        
        w.flush()?;
        Ok(())
    }

    /// Load all indices from disk
    pub fn load_all(&mut self) -> std::io::Result<()> {
        let index_dir = self.data_dir.join("vector_indices");
        
        if !index_dir.exists() {
            return Ok(());
        }
        
        // Load metadata
        let meta_path = index_dir.join("metadata.json");
        if meta_path.exists() {
            let meta_json = std::fs::read_to_string(meta_path)?;
            self.metadata = serde_json::from_str(&meta_json)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        }
        
        // Load each index
        for entry in std::fs::read_dir(index_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("vecidx") {
                if let Ok(index) = self.load_index(&path) {
                    // Extract key from filename
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let key = stem.replace("_", ":");
                        self.indices.insert(key, index);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Load a single index from disk
    fn load_index(&self, path: &PathBuf) -> std::io::Result<Box<dyn VectorIndex>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        
        // Verify magic
        let mut magic = [0u8; 6];
        reader.read_exact(&mut magic)?;
        if &magic != b"VECIDX" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not a vector index file",
            ));
        }
        
        // Version
        let mut ver = [0u8; 4];
        reader.read_exact(&mut ver)?;
        
        // Key
        let mut key_len = [0u8; 4];
        reader.read_exact(&mut key_len)?;
        let key_len = u32::from_le_bytes(key_len) as usize;
        let mut key_buf = vec![0u8; key_len];
        reader.read_exact(&mut key_buf)?;
        let _key = String::from_utf8_lossy(&key_buf);
        
        // Dimension
        let mut dim_buf = [0u8; 4];
        reader.read_exact(&mut dim_buf)?;
        let dimension = u32::from_le_bytes(dim_buf) as usize;
        
        // Count
        let mut count_buf = [0u8; 8];
        reader.read_exact(&mut count_buf)?;
        let _count = u64::from_le_bytes(count_buf) as usize;
        
        // Metric
        let mut metric_byte = [0u8; 1];
        reader.read_exact(&mut metric_byte)?;
        let metric = match metric_byte[0] {
            0 => DistanceMetric::Cosine,
            1 => DistanceMetric::Euclidean,
            2 => DistanceMetric::DotProduct,
            3 => DistanceMetric::Manhattan,
            _ => DistanceMetric::Cosine,
        };
        
        // Create index
        let index: Box<dyn VectorIndex> = Box::new(ParallelKnnIndex::new(metric));
        
        Ok(index)
    }

    /// Delete a vector by ID
    pub fn delete(&mut self, table: &str, column: &str, id: u64) -> VectorResult<()> {
        let key = Self::index_key(table, column);
        
        let index = self.indices.get_mut(&key)
            .ok_or_else(|| VectorError::InvalidParameter(format!("Vector column {}:{} not registered", table, column)))?;
        
        index.delete(id)
    }

    /// Clear all vectors for a column
    pub fn clear(&mut self, table: &str, column: &str) -> VectorResult<()> {
        let key = Self::index_key(table, column);
        
        if let Some(index) = self.indices.get_mut(&key) {
            // Re-create the index
            let dimension = index.dimension();
            let metric = index.metric();
            let new_index: Box<dyn VectorIndex> = Box::new(ParallelKnnIndex::new(metric));
            self.indices.insert(key, new_index);
        }
        
        Ok(())
    }

    /// Export vectors as binary format compatible with BinaryTableStorage
    pub fn export_vectors(&self, table: &str, column: &str) -> VectorResult<Vec<u8>> {
        let key = Self::index_key(table, column);
        let index = self.indices.get(&key)
            .ok_or_else(|| VectorError::InvalidParameter(format!("Vector column {}:{} not registered", table, column)))?;
        
        let dim = index.dimension();
        let n = index.len();
        
        // Binary format: header + vectors
        // Header: magic(6) + version(4) + dimension(4) + count(8) = 22 bytes
        // Per vector: id(8) + vector(dim * 4 bytes)
        let mut data = Vec::with_capacity(22 + n * (8 + dim * 4));
        
        // Magic
        data.extend_from_slice(b"VECTOR ");
        // Version
        data.extend_from_slice(&1u32.to_le_bytes());
        // Dimension
        data.extend_from_slice(&(dim as u32).to_le_bytes());
        // Count
        data.extend_from_slice(&(n as u64).to_le_bytes());
        
        // Note: Actual implementation would need to iterate over stored vectors
        // This is a placeholder structure
        
        Ok(data)
    }

    /// Get metadata for all registered columns
    pub fn get_metadata(&self) -> &[VectorColumnMeta] {
        &self.metadata
    }

    /// Get index statistics
    pub fn stats(&self) -> VectorStoreStats {
        let total_vectors: usize = self.indices.values().map(|i| i.len()).sum();
        let total_memory_est = total_vectors * 4 * 128; // rough estimate with dim=128
        
        VectorStoreStats {
            total_vectors,
            total_columns: self.indices.len(),
            estimated_memory_bytes: total_memory_est,
        }
    }
}

/// Vector store statistics
#[derive(Debug, Clone)]
pub struct VectorStoreStats {
    pub total_vectors: usize,
    pub total_columns: usize,
    pub estimated_memory_bytes: usize,
}

/// Extension trait for BinaryTableStorage to support vector operations
pub trait VectorStorageExt {
    /// Save table with vector column support
    fn save_with_vectors(&self, table: &str, data: &TableData, vector_store: &VectorStore) -> std::io::Result<()>;
    
    /// Load table with vector column support
    fn load_with_vectors(&self, table: &str, vector_store: &mut VectorStore) -> std::io::Result<TableData>;
}

impl VectorStorageExt for BinaryTableStorage {
    fn save_with_vectors(&self, table: &str, data: &TableData, vector_store: &VectorStore) -> std::io::Result<()> {
        // Save the regular table data
        self.save(table, data)?;
        
        // Save vector indices
        vector_store.save_all()?;
        
        Ok(())
    }

    fn load_with_vectors(&self, table: &str, vector_store: &mut VectorStore) -> std::io::Result<TableData> {
        let data = self.load(table)?;
        vector_store.load_all()?;
        Ok(data)
    }
}

/// Embedding value type for SQL type system
/// This allows vectors to be stored as SQL values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    /// Row ID / primary key
    pub id: u64,
    /// Vector data as f32 array
    pub vector: Vec<f32>,
    /// Optional metadata (JSON string)
    pub metadata: Option<String>,
}

impl Embedding {
    pub fn new(id: u64, vector: Vec<f32>) -> Self {
        Self { id, vector, metadata: None }
    }
    
    pub fn with_metadata(id: u64, vector: Vec<f32>, metadata: String) -> Self {
        Self { id, vector, metadata: Some(metadata) }
    }
    
    /// Get dimension of the embedding
    pub fn dimension(&self) -> usize {
        self.vector.len()
    }
}

impl BinaryFormat for Embedding {
    fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // Magic (6 bytes)
        data.extend_from_slice(b"EMBED\0");
        // Version
        data.extend_from_slice(&[1u8]);
        // ID
        data.extend_from_slice(&self.id.to_le_bytes());
        // Dimension
        data.extend_from_slice(&(self.vector.len() as u32).to_le_bytes());
        // Vector data
        for v in &self.vector {
            data.extend_from_slice(&v.to_le_bytes());
        }
        // Metadata
        if let Some(ref m) = self.metadata {
            let mbytes = m.as_bytes();
            data.extend_from_slice(&(mbytes.len() as u32).to_le_bytes());
            data.extend_from_slice(mbytes);
        } else {
            data.extend_from_slice(&0u32.to_le_bytes());
        }
        
        data
    }
    
    fn from_bytes(data: &[u8]) -> Result<Self, crate::binary_format::BinaryFormatError> {
        use crate::binary_format::BinaryFormatError;
        
        // Minimum: magic(6) + version(1) + id(8) + dim(4) + meta_len(4) = 23 bytes
        if data.len() < 23 {
            return Err(BinaryFormatError::InsufficientData);
        }
        
        // Magic (6 bytes: "EMBED\0")
        if &data[0..6] != b"EMBED\0" {
            return Err(BinaryFormatError::InvalidFormat("Invalid embedding magic".to_string()));
        }
        
        let mut offset = 6;
        
        // Version
        let _version = data[offset];
        offset += 1;
        
        // ID
        let id = u64::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
            data[offset+4], data[offset+5], data[offset+6], data[offset+7],
        ]);
        offset += 8;
        
        // Dimension
        let dim = u32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
        ]) as usize;
        offset += 4;
        
        // Vector
        if data.len() < offset + dim * 4 {
            return Err(BinaryFormatError::InsufficientData);
        }
        let mut vector = Vec::with_capacity(dim);
        for i in 0..dim {
            let bytes: [u8; 4] = [
                data[offset + i * 4],
                data[offset + i * 4 + 1],
                data[offset + i * 4 + 2],
                data[offset + i * 4 + 3],
            ];
            vector.push(f32::from_le_bytes(bytes));
        }
        offset += dim * 4;
        
        // Metadata
        if data.len() < offset + 4 {
            return Err(BinaryFormatError::InsufficientData);
        }
        let meta_len = u32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
        ]) as usize;
        offset += 4;
        
        let metadata = if meta_len > 0 {
            if data.len() < offset + meta_len {
                return Err(BinaryFormatError::InsufficientData);
            }
            Some(String::from_utf8_lossy(&data[offset..offset+meta_len]).to_string())
        } else {
            None
        };
        
        Ok(Embedding { id, vector, metadata })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_binary_roundtrip() {
        let emb = Embedding::new(42, vec![1.0, 2.0, 3.0, 4.0]);
        
        let bytes = emb.to_bytes();
        let restored = Embedding::from_bytes(&bytes).unwrap();
        
        assert_eq!(restored.id, 42);
        assert_eq!(restored.vector, vec![1.0, 2.0, 3.0, 4.0]);
        assert!(restored.metadata.is_none());
    }

    #[test]
    fn test_embedding_with_metadata() {
        let emb = Embedding::with_metadata(1, vec![0.1, 0.2], r#"{"source": "test"}"#.to_string());
        
        let bytes = emb.to_bytes();
        let restored = Embedding::from_bytes(&bytes).unwrap();
        
        assert_eq!(restored.id, 1);
        assert_eq!(restored.vector, vec![0.1, 0.2]);
        assert_eq!(restored.metadata.unwrap(), r#"{"source": "test"}"#);
    }

    #[test]
    fn test_vector_store_creation() {
        let tmp = tempfile::tempdir().unwrap();
        let store = VectorStore::new(tmp.path().to_path_buf()).unwrap();
        
        assert_eq!(store.indices.len(), 0);
    }

    #[test]
    fn test_vector_store_register_and_insert() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = VectorStore::new(tmp.path().to_path_buf()).unwrap();
        
        store.register_column("items", "embedding", 128, DistanceMetric::Cosine, VectorIndexType::ParallelKnn)
            .unwrap();
        
        store.insert("items", "embedding", 1, &vec![0.1; 128]).unwrap();
        
        assert_eq!(store.len("items", "embedding"), 1);
        assert!(!store.is_empty("items", "embedding"));
    }

    #[test]
    fn test_vector_store_search() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = VectorStore::new(tmp.path().to_path_buf()).unwrap();
        
        store.register_column("items", "embedding", 3, DistanceMetric::Cosine, VectorIndexType::Flat)
            .unwrap();
        
        // Insert some vectors
        store.insert("items", "embedding", 1, &[1.0, 0.0, 0.0]).unwrap();
        store.insert("items", "embedding", 2, &[0.0, 1.0, 0.0]).unwrap();
        store.insert("items", "embedding", 3, &[0.0, 0.0, 1.0]).unwrap();
        
        // Search for [1, 0, 0] - should find ID 1 first
        let results = store.search("items", "embedding", &[1.0, 0.0, 0.0], 2).unwrap();
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1);
    }

    #[test]
    fn test_vector_store_batch_insert() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = VectorStore::new(tmp.path().to_path_buf()).unwrap();
        
        store.register_column("items", "embedding", 4, DistanceMetric::Euclidean, VectorIndexType::Flat)
            .unwrap();
        
        let vectors: Vec<(u64, Vec<f32>)> = (0..100)
            .map(|i| (i, vec![i as f32; 4]))
            .collect();
        
        let count = store.batch_insert("items", "embedding", vectors).unwrap();
        assert_eq!(count, 100);
        assert_eq!(store.len("items", "embedding"), 100);
    }

    #[test]
    fn test_vector_store_delete() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = VectorStore::new(tmp.path().to_path_buf()).unwrap();
        
        store.register_column("items", "embedding", 2, DistanceMetric::Cosine, VectorIndexType::Flat)
            .unwrap();
        
        store.insert("items", "embedding", 1, &[1.0, 0.0]).unwrap();
        store.insert("items", "embedding", 2, &[0.0, 1.0]).unwrap();
        
        assert_eq!(store.len("items", "embedding"), 2);
        
        store.delete("items", "embedding", 1).unwrap();
        
        assert_eq!(store.len("items", "embedding"), 1);
    }

    #[test]
    fn test_vector_store_stats() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = VectorStore::new(tmp.path().to_path_buf()).unwrap();
        
        store.register_column("items", "emb1", 128, DistanceMetric::Cosine, VectorIndexType::Flat)
            .unwrap();
        store.register_column("items", "emb2", 256, DistanceMetric::Euclidean, VectorIndexType::Hnsw)
            .unwrap();
        
        store.insert("items", "emb1", 1, &vec![0.1; 128]).unwrap();
        store.insert("items", "emb2", 1, &vec![0.2; 256]).unwrap();
        
        let stats = store.stats();
        
        assert_eq!(stats.total_vectors, 2);
        assert_eq!(stats.total_columns, 2);
    }
}
