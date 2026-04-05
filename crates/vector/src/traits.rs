//! Vector index trait definitions

use crate::metrics::DistanceMetric;
use crate::VectorResult;

/// Vector index entry with ID and similarity score
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub id: u64,
    pub score: f32,
}

impl IndexEntry {
    pub fn new(id: u64, score: f32) -> Self {
        Self { id, score }
    }
}

/// Core trait for vector indexes
pub trait VectorIndex: Send + Sync {
    fn insert(&mut self, id: u64, vector: &[f32]) -> VectorResult<()>;

    fn search(&self, query: &[f32], k: usize) -> VectorResult<Vec<IndexEntry>>;

    fn build_index(&mut self) -> VectorResult<()>;

    fn delete(&mut self, id: u64) -> VectorResult<()>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn dimension(&self) -> usize;

    fn metric(&self) -> DistanceMetric;
}

pub trait VectorIndexBuilder: Default {
    fn with_metric(self, metric: DistanceMetric) -> Self;
    fn with_dimension(self, dim: usize) -> Self;
    fn build(self) -> Box<dyn VectorIndex>;
}
