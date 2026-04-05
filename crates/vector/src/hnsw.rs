//! HNSW (Hierarchical Navigable Small World) index implementation

use crate::error::{VectorError, VectorResult};
use crate::metrics::{compute_similarity, DistanceMetric};
use crate::traits::{IndexEntry, VectorIndex};
use rand::Rng;

#[derive(Debug, Clone)]
struct HnswNode {
    id: u64,
    vector: Vec<f32>,
    level: usize,
}

#[derive(Debug, Clone)]
struct Layer {
    neighbors: Vec<Vec<usize>>,
}

#[derive(Debug, Clone)]
pub struct HnswIndex {
    dimension: usize,
    metric: DistanceMetric,
    m: usize,
    ef_construction: usize,
    ef_search: usize,
    max_level: usize,
    entry_point: Option<usize>,
    nodes: Vec<HnswNode>,
    layers: Vec<Layer>,
    rng: rand::rngs::StdRng,
}

impl HnswIndex {
    pub fn new(metric: DistanceMetric) -> Self {
        Self {
            dimension: 0,
            metric,
            m: 16,
            ef_construction: 128,
            ef_search: 64,
            max_level: 0,
            entry_point: None,
            nodes: Vec::new(),
            layers: Vec::new(),
            rng: rand::SeedableRng::from_seed([42u8; 32]),
        }
    }

    pub fn with_params(
        m: usize,
        ef_construction: usize,
        ef_search: usize,
        metric: DistanceMetric,
    ) -> Self {
        Self {
            dimension: 0,
            metric,
            m,
            ef_construction,
            ef_search,
            max_level: 0,
            entry_point: None,
            nodes: Vec::new(),
            layers: Vec::new(),
            rng: rand::SeedableRng::from_seed([42u8; 32]),
        }
    }

    fn random_level(&mut self) -> usize {
        let mut level = 0;
        let prob = 1.0 / self.m as f32;
        while self.rng.gen::<f32>() < prob && level < self.max_level + 1 {
            level += 1;
        }
        level.min(16)
    }

    fn distance(&self, idx1: usize, idx2: usize) -> f32 {
        let v1 = &self.nodes[idx1].vector;
        let v2 = &self.nodes[idx2].vector;
        -compute_similarity(v1, v2, self.metric)
    }

    fn search_layer(
        &self,
        query: &[f32],
        ep_idx: usize,
        ef: usize,
        level: usize,
    ) -> Vec<(usize, f32)> {
        let mut visited = std::collections::HashSet::new();
        visited.insert(ep_idx);

        let mut candidates: Vec<(usize, f32)> = vec![(
            ep_idx,
            self.distance(ep_idx, self.nodes.len().saturating_sub(1)),
        )];

        let query_dist = |idx: usize| -> f32 {
            let v = &self.nodes[idx].vector;
            -compute_similarity(query, v, self.metric)
        };

        let mut results: Vec<(usize, f32)> = vec![(ep_idx, query_dist(ep_idx))];

        while !candidates.is_empty() {
            let (current, _) = candidates.remove(0);

            let neighbors = self
                .layers
                .get(level)
                .map(|l| l.neighbors.get(current).cloned().unwrap_or_default())
                .unwrap_or_default();

            for &neighbor in neighbors.iter() {
                if visited.contains(&neighbor) {
                    continue;
                }
                visited.insert(neighbor);

                let dist = query_dist(neighbor);
                let worst_dist = results.last().map(|(_, d)| *d).unwrap_or(f32::MAX);

                if worst_dist >= dist || results.len() < ef {
                    candidates.push((neighbor, dist));
                    candidates.sort_by(|a, b| {
                        a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Greater)
                    });

                    results.push((neighbor, dist));
                    results.sort_by(|a, b| {
                        a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Greater)
                    });
                    results.truncate(ef);
                }
            }
        }

        results
    }
}

impl VectorIndex for HnswIndex {
    fn insert(&mut self, id: u64, vector: &[f32]) -> VectorResult<()> {
        if self.dimension == 0 {
            self.dimension = vector.len();
        } else if vector.len() != self.dimension {
            return Err(VectorError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.len(),
            });
        }

        if self.nodes.iter().any(|n| n.id == id) {
            return Err(VectorError::InvalidParameter(format!(
                "ID {} already exists",
                id
            )));
        }

        let level = self.random_level();
        let idx = self.nodes.len();

        let node = HnswNode {
            id,
            vector: vector.to_vec(),
            level,
        };

        self.nodes.push(node);

        while self.layers.len() <= level {
            self.layers.push(Layer {
                neighbors: Vec::new(),
            });
        }

        for layer in self.layers.iter_mut() {
            while layer.neighbors.len() <= idx {
                layer.neighbors.push(Vec::new());
            }
        }

        if self.entry_point.is_none() {
            self.entry_point = Some(idx);
            self.max_level = level;
            return Ok(());
        }

        let mut current_idx = self.entry_point.unwrap();

        for lvl in (1..=self.max_level).rev() {
            let results = self.search_layer(vector, current_idx, 1, lvl);
            if let Some((next_idx, _)) = results.first() {
                current_idx = *next_idx;
            }
        }

        let m = self.m;
        let dist_current_idx = self.distance(current_idx, idx);

        let nodes_len = self.nodes.len();
        let vectors: Vec<Vec<f32>> = self.nodes.iter().map(|n| n.vector.clone()).collect();
        let dist_fn =
            |n: usize| -> f32 { -compute_similarity(&vectors[n], &vectors[idx], self.metric) };
        let current_distances: Vec<f32> = (0..nodes_len).map(dist_fn).collect();

        for lvl in 0..=level {
            let layer_idx = lvl.min(self.layers.len().saturating_sub(1));

            {
                let neighbors = &mut self.layers[layer_idx].neighbors;

                let mut candidates: Vec<(usize, f32)> = vec![(current_idx, dist_current_idx)];

                while !candidates.is_empty() && neighbors[idx].len() < m {
                    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                    let (best_idx, _) = candidates.remove(0);

                    if !neighbors[idx].contains(&best_idx) {
                        neighbors[idx].push(best_idx);
                        neighbors[best_idx].push(idx);

                        let best_dist = current_distances[best_idx];
                        let new_candidates: Vec<_> = neighbors[best_idx]
                            .iter()
                            .map(|&n| (n, current_distances[n]))
                            .filter(|(_, d)| *d < best_dist)
                            .collect();

                        for nc in new_candidates {
                            if !candidates.contains(&nc) {
                                candidates.push(nc);
                            }
                        }
                        candidates.truncate(m * 2);
                    }
                }
            }
        }

        if level > self.max_level {
            self.max_level = level;
            self.entry_point = Some(idx);
        }

        Ok(())
    }

    fn search(&self, query: &[f32], k: usize) -> VectorResult<Vec<IndexEntry>> {
        if self.nodes.is_empty() {
            return Err(VectorError::EmptyIndex);
        }

        if query.len() != self.dimension {
            return Err(VectorError::DimensionMismatch {
                expected: self.dimension,
                actual: query.len(),
            });
        }

        let entry_point = self.entry_point.ok_or(VectorError::EmptyIndex)?;
        let mut current_idx = entry_point;

        for level in (1..=self.max_level).rev() {
            let results = self.search_layer(query, current_idx, 1, level);
            if let Some((next_idx, _)) = results.first() {
                current_idx = *next_idx;
            }
        }

        let results = self.search_layer(query, current_idx, self.ef_search, 0);

        let mut entries: Vec<_> = results
            .into_iter()
            .map(|(idx, dist)| IndexEntry::new(self.nodes[idx].id, -dist))
            .collect();

        entries.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(entries.into_iter().take(k).collect())
    }

    fn build_index(&mut self) -> VectorResult<()> {
        Ok(())
    }

    fn delete(&mut self, id: u64) -> VectorResult<()> {
        self.nodes.retain(|n| n.id != id);
        Ok(())
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
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
    fn test_hnsw_insert_and_search() {
        let mut index = HnswIndex::new(DistanceMetric::Cosine);
        for i in 0..100 {
            let v = vec![i as f32, i as f32];
            index.insert(i, &v).unwrap();
        }

        let results = index.search(&[50.0, 50.0], 5).unwrap();
        assert_eq!(results.len(), 5);
    }
}
