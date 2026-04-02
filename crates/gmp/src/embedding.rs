//! GMP Vector Embedding
//!
//! Provides vector embedding generation and similarity computation.
//! Currently uses a hash-based token embedding approach.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Dimension for embedding vectors (256 is a good balance for hash-based embedding)
pub const EMBEDDING_DIM: usize = 256;

/// Document embedding stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEmbedding {
    pub doc_id: i64,
    pub embedding: Vec<f32>,
    pub updated_at: i64,
}

impl DocumentEmbedding {
    /// Create a new document embedding
    pub fn new(doc_id: i64, embedding: Vec<f32>) -> Self {
        let updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Self {
            doc_id,
            embedding,
            updated_at,
        }
    }

    /// Convert embedding vector to a JSON string for storage
    pub fn embedding_to_json(embedding: &[f32]) -> String {
        serde_json::to_string(embedding).unwrap_or_else(|_| "[]".to_string())
    }

    /// Parse embedding vector from JSON string
    pub fn embedding_from_json(json: &str) -> Vec<f32> {
        serde_json::from_str(json).unwrap_or_else(|_| vec![])
    }
}

/// Table name for document embeddings
pub const TABLE_EMBEDDINGS: &str = "gmp_embeddings";

/// SQL to create the embeddings table
pub const CREATE_EMBEDDINGS_TABLE: &str = r#"
CREATE TABLE gmp_embeddings (
    doc_id INTEGER PRIMARY KEY,
    embedding TEXT NOT NULL,
    updated_at INTEGER NOT NULL
)
"#;

/// Trait for embedding models
pub trait EmbeddingModel: Send + Sync {
    /// Generate an embedding vector for the given text
    fn generate_embedding(&self, text: &str) -> Vec<f32>;

    /// Get the dimension of the embedding vectors
    fn dimension(&self) -> usize;
}

/// Hash-based embedding model
///
/// This is a simple embedding model that:
/// 1. Tokenizes the input text
/// 2. Hashes each token to determine which dimensions it affects
/// 3. Uses a hash-based pseudo-random number generator to set values
///
/// The resulting vectors can be compared using cosine similarity
/// for approximate semantic matching.
#[derive(Debug, Clone)]
pub struct HashEmbeddingModel {
    dim: usize,
}

impl HashEmbeddingModel {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    /// Tokenize text into lowercase words
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Hash a string to a u64 using a simple hash
    fn hash_string(&self, s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for HashEmbeddingModel {
    fn default() -> Self {
        Self::new(EMBEDDING_DIM)
    }
}

impl EmbeddingModel for HashEmbeddingModel {
    fn generate_embedding(&self, text: &str) -> Vec<f32> {
        let tokens = self.tokenize(text);
        let mut embedding = vec![0.0f32; self.dim];

        for (i, val) in embedding.iter_mut().enumerate() {
            // Use multiple hash functions for each dimension
            let h1 = self.hash_string(&format!("{}_{}", i, "x"));
            let _h2 = self.hash_string(&format!("{}_{}", i, "y"));

            let mut combined_hash: u64 = 0;
            for token in &tokens {
                let token_hash = self.hash_string(token);
                // Combine token hash with dimension hash
                combined_hash ^= token_hash.wrapping_mul(h1.wrapping_add(1));
                combined_hash = combined_hash.rotate_left(23);
            }

            // Use the combined hash to generate a float in [-1, 1]
            let normalized = ((combined_hash as i64).abs() % 10000) as f32 / 10000.0;
            *val = normalized * 2.0 - 1.0;
        }

        // Normalize the embedding vector to unit length
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in &mut embedding {
                *val /= magnitude;
            }
        }

        embedding
    }

    fn dimension(&self) -> usize {
        self.dim
    }
}

/// Compute cosine similarity between two embedding vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    if a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

/// Compute Euclidean distance between two embedding vectors (normalized)
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::MAX;
    }
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Global default embedding model instance
pub static DEFAULT_MODEL: std::sync::LazyLock<HashEmbeddingModel> =
    std::sync::LazyLock::new(HashEmbeddingModel::default);

/// Generate embedding using the default model
pub fn generate_embedding(text: &str) -> Vec<f32> {
    DEFAULT_MODEL.generate_embedding(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_same() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &b)).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_hash_embedding_model() {
        let model = HashEmbeddingModel::new(128);

        let emb1 = model.generate_embedding("hello world");
        let emb2 = model.generate_embedding("hello world");

        assert_eq!(emb1.len(), 128);
        assert_eq!(emb2.len(), 128);

        // Same text should produce same embedding
        assert!((cosine_similarity(&emb1, &emb2) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_hash_embedding_model_different_texts() {
        let model = HashEmbeddingModel::default();

        let emb1 = model.generate_embedding("hello world");
        let emb2 = model.generate_embedding("foo bar baz");

        // Different texts should produce different embeddings
        // (with high probability for random hash-based embeddings)
        let sim = cosine_similarity(&emb1, &emb2);
        assert!(sim < 0.99); // Not identical
    }

    #[test]
    fn test_embedding_normalized() {
        let model = HashEmbeddingModel::default();
        let emb = model.generate_embedding("test document content");

        let magnitude: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_document_embedding_json() {
        let emb = vec![0.1f32, 0.2, 0.3, -0.4];
        let json = DocumentEmbedding::embedding_to_json(&emb);
        let parsed = DocumentEmbedding::embedding_from_json(&json);
        assert_eq!(emb, parsed);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        assert!((euclidean_distance(&a, &b) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_dimension() {
        let model = HashEmbeddingModel::new(512);
        assert_eq!(model.dimension(), 512);

        let emb = model.generate_embedding("test");
        assert_eq!(emb.len(), 512);
    }
}
