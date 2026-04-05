//! Flat index implementation (brute-force O(n) search)

use crate::error::{VectorError, VectorResult};
use crate::metrics::{compute_similarity, DistanceMetric};
use crate::traits::{IndexEntry, VectorIndex};

#[derive(Debug, Clone)]
struct VectorEntry {
    id: u64,
    vector: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct FlatIndex {
    dimension: usize,
    metric: DistanceMetric,
    vectors: Vec<VectorEntry>,
    built: bool,
}

impl FlatIndex {
    pub fn new(metric: DistanceMetric) -> Self {
        Self {
            dimension: 0,
            metric,
            vectors: Vec::new(),
            built: false,
        }
    }

    pub fn with_dimension(dimension: usize, metric: DistanceMetric) -> Self {
        Self {
            dimension,
            metric,
            vectors: Vec::new(),
            built: false,
        }
    }
}

impl VectorIndex for FlatIndex {
    fn insert(&mut self, id: u64, vector: &[f32]) -> VectorResult<()> {
        if self.dimension == 0 {
            self.dimension = vector.len();
        } else if vector.len() != self.dimension {
            return Err(VectorError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.len(),
            });
        }

        self.vectors.push(VectorEntry {
            id,
            vector: vector.to_vec(),
        });
        self.built = false;
        Ok(())
    }

    fn search(&self, query: &[f32], k: usize) -> VectorResult<Vec<IndexEntry>> {
        if self.vectors.is_empty() {
            return Err(VectorError::EmptyIndex);
        }

        if query.len() != self.dimension {
            return Err(VectorError::DimensionMismatch {
                expected: self.dimension,
                actual: query.len(),
            });
        }

        let mut scored: Vec<_> = self
            .vectors
            .iter()
            .map(|entry| {
                let score = compute_similarity(query, &entry.vector, self.metric);
                IndexEntry::new(entry.id, score)
            })
            .collect();

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(scored.into_iter().take(k).collect())
    }

    fn build_index(&mut self) -> VectorResult<()> {
        self.built = true;
        Ok(())
    }

    fn delete(&mut self, id: u64) -> VectorResult<()> {
        let pos = self
            .vectors
            .iter()
            .position(|e| e.id == id)
            .ok_or(VectorError::IdNotFound(id))?;
        self.vectors.remove(pos);
        self.built = false;
        Ok(())
    }

    fn len(&self) -> usize {
        self.vectors.len()
    }

    fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn metric(&self) -> DistanceMetric {
        self.metric
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_index_insert_and_search() {
        let mut index = FlatIndex::new(DistanceMetric::Cosine);
        index.insert(1, &[1.0, 0.0, 0.0]).unwrap();
        index.insert(2, &[0.0, 1.0, 0.0]).unwrap();
        index.insert(3, &[0.0, 0.0, 1.0]).unwrap();
        index.build_index().unwrap();

        let results = index.search(&[1.0, 0.0, 0.0], 2).unwrap();
        assert_eq!(results[0].id, 1);
        assert!((results[0].score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_flat_index_dimension_mismatch() {
        let mut index = FlatIndex::new(DistanceMetric::Euclidean);
        index.insert(1, &[1.0, 0.0]).unwrap();
        let result = index.insert(2, &[1.0, 0.0, 0.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_flat_index_delete() {
        let mut index = FlatIndex::new(DistanceMetric::Cosine);
        index.insert(1, &[1.0, 0.0]).unwrap();
        index.insert(2, &[0.0, 1.0]).unwrap();
        index.delete(1).unwrap();
        assert_eq!(index.len(), 1);
    }
}
