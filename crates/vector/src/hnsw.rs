//! HNSW (Hierarchical Navigable Small World) index implementation
//!
//! Optimized implementation using binary heaps and efficient data structures.

use crate::error::{VectorError, VectorResult};
use crate::metrics::{compute_similarity, DistanceMetric};
use crate::traits::{IndexEntry, VectorIndex, VectorRecord};
use parking_lot::RwLock;
use rand::Rng;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};

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
struct DistNode {
    dist: f32,
    idx: usize,
}

impl PartialEq for DistNode {
    fn eq(&self, other: &Self) -> bool {
        self.dist == other.dist && self.idx == other.idx
    }
}

impl Eq for DistNode {}

impl Ord for DistNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dist
            .partial_cmp(&other.dist)
            .map(|o| {
                if o == Ordering::Equal {
                    self.idx.cmp(&other.idx)
                } else {
                    o
                }
            })
            .unwrap_or(Ordering::Greater)
    }
}

impl PartialOrd for DistNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
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
    // Generational visited marker for O(1) visited tracking
    visit_marker: AtomicU32,
    visited: RwLock<Vec<u32>>,
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
            visit_marker: AtomicU32::new(0),
            visited: RwLock::new(Vec::new()),
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
            visit_marker: AtomicU32::new(0),
            visited: RwLock::new(Vec::new()),
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

    fn distance_to_query(&self, query: &[f32], idx: usize) -> f32 {
        -compute_similarity(query, &self.nodes[idx].vector, self.metric)
    }

    fn search_layer(
        &self,
        query: &[f32],
        ep_idx: usize,
        ef: usize,
        level: usize,
        visited: &mut [u32],
        marker: u32,
    ) -> Vec<(usize, f32)> {
        visited[ep_idx] = marker;

        let mut candidates: BinaryHeap<DistNode> = BinaryHeap::new();
        candidates.push(DistNode {
            dist: self.distance_to_query(query, ep_idx),
            idx: ep_idx,
        });

        let mut results: BinaryHeap<DistNode> = BinaryHeap::new();

        while let Some(node) = candidates.pop() {
            let worst_result = results.peek().map(|r| r.dist).unwrap_or(f32::MAX);

            if results.len() >= ef && node.dist > worst_result {
                break;
            }

            let node_idx = node.idx;
            results.push(node);

            let neighbors = self
                .layers
                .get(level)
                .map(|l| l.neighbors.get(node_idx).cloned().unwrap_or_default())
                .unwrap_or_default();

            for &neighbor in neighbors.iter() {
                if visited[neighbor] != marker {
                    visited[neighbor] = marker;

                    let neighbor_dist = self.distance_to_query(query, neighbor);

                    if results.len() < ef || neighbor_dist < worst_result {
                        candidates.push(DistNode {
                            dist: neighbor_dist,
                            idx: neighbor,
                        });
                    }
                }
            }
        }

        results
            .into_vec()
            .into_iter()
            .map(|node| (node.idx, node.dist))
            .collect()
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

        let n = self.nodes.len();
        let mut visited = self.visited.write();
        if visited.len() < n {
            visited.resize(n, 0);
        }
        let marker = self.visit_marker.fetch_add(1, AtomicOrdering::Relaxed) + 1;

        for lvl in (1..=self.max_level).rev() {
            let results = self.search_layer(vector, current_idx, 1, lvl, &mut visited, marker);
            if let Some((next_idx, _)) = results.first() {
                current_idx = *next_idx;
            }
        }

        let m = self.m;

        // Use search_layer to find m closest nodes - O(log n) instead of O(n)
        let neighbor_results =
            self.search_layer(vector, current_idx, m.max(1), 0, &mut visited, marker);

        let neighbors = &mut self.layers[0].neighbors;

        for (neighbor_idx, _) in neighbor_results.iter().take(m) {
            if *neighbor_idx != idx && !neighbors[idx].contains(neighbor_idx) {
                neighbors[idx].push(*neighbor_idx);
                neighbors[*neighbor_idx].push(idx);
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

        let n = self.nodes.len();
        let mut visited = self.visited.write();
        if visited.len() < n {
            visited.resize(n, 0);
        }
        let marker = self.visit_marker.fetch_add(1, AtomicOrdering::Relaxed) + 1;

        for level in (1..=self.max_level).rev() {
            let results = self.search_layer(query, current_idx, 1, level, &mut visited, marker);
            if let Some((next_idx, _)) = results.first() {
                current_idx = *next_idx;
            }
        }

        let results =
            self.search_layer(query, current_idx, self.ef_search, 0, &mut visited, marker);

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

    fn get_all(&self) -> Vec<VectorRecord> {
        self.nodes
            .iter()
            .map(|n| VectorRecord {
                id: n.id,
                vector: n.vector.clone(),
            })
            .collect()
    }

    fn iter_vectors(&self) -> Box<dyn Iterator<Item = (u64, &[f32])> + '_> {
        Box::new(self.nodes.iter().map(|n| (n.id, n.vector.as_slice())))
    }
}

impl HnswIndex {
    pub fn build_from_vectors(&mut self, vectors: Vec<(u64, Vec<f32>)>) -> VectorResult<()> {
        if vectors.is_empty() {
            return Ok(());
        }

        let dim = vectors[0].1.len();
        self.dimension = dim;

        for (id, vector) in vectors {
            let node = HnswNode {
                id,
                vector,
                level: 0,
            };
            self.nodes.push(node);
        }

        let n = self.nodes.len();
        let m = self.m.max(1);
        let _ef_construction = self.ef_construction.max(64);

        // Assign random levels to each node based on probability 1/m
        let prob = 1.0 / m as f32;
        for node in self.nodes.iter_mut() {
            let mut lvl = 0;
            while self.rng.gen::<f32>() < prob {
                lvl += 1;
            }
            node.level = lvl;
        }

        // Calculate max level
        self.max_level = self.nodes.iter().map(|n| n.level).max().unwrap_or(0);
        self.max_level = self.max_level.min(16);

        // Initialize layers
        while self.layers.len() <= self.max_level {
            self.layers.push(Layer {
                neighbors: vec![Vec::new(); n],
            });
        }

        // Find entry point (node with highest level)
        self.entry_point = self
            .nodes
            .iter()
            .enumerate()
            .max_by_key(|(_, n)| n.level)
            .map(|(idx, _)| idx);

        // Parallel build layer 0 k-NN
        let nodes = &self.nodes;
        let metric = self.metric;
        let max_candidates = (m * 4).max(64);

        let all_knns: Vec<Vec<usize>> = (0..n)
            .into_par_iter()
            .map(|idx| {
                let query = &nodes[idx].vector;
                let mut results: BinaryHeap<DistNode> = BinaryHeap::new();

                for (j, node) in nodes.iter().enumerate().take(n) {
                    if j == idx {
                        continue;
                    }
                    let d = -compute_similarity(query, &node.vector, metric);
                    results.push(DistNode { dist: d, idx: j });
                    if results.len() > max_candidates {
                        let _ = results.pop();
                    }
                }

                let mut neighbors: Vec<usize> =
                    results.into_vec().into_iter().map(|n| n.idx).collect();

                // Sort by distance and take top m
                neighbors.sort_by(|&a, &b| {
                    let da = -compute_similarity(query, &nodes[a].vector, metric);
                    let db = -compute_similarity(query, &nodes[b].vector, metric);
                    da.partial_cmp(&db).unwrap()
                });

                neighbors.truncate(m);
                neighbors
            })
            .collect();

        // Build bidirectional k-NN graph for layer 0
        for (idx, knns) in all_knns.into_iter().enumerate() {
            for &neighbor in &knns {
                if self.layers[0].neighbors[idx].len() < m
                    && !self.layers[0].neighbors[idx].contains(&neighbor)
                {
                    self.layers[0].neighbors[idx].push(neighbor);
                }
                if self.layers[0].neighbors[neighbor].len() < m
                    && !self.layers[0].neighbors[neighbor].contains(&idx)
                {
                    self.layers[0].neighbors[neighbor].push(idx);
                }
            }
        }

        // Build higher layers - just connect nodes to random neighbors at same level
        for level in 1..=self.max_level {
            let level_nodes: Vec<usize> = self
                .nodes
                .iter()
                .enumerate()
                .filter(|(_, n)| n.level >= level)
                .map(|(idx, _)| idx)
                .collect();

            if level_nodes.len() < 2 {
                continue;
            }

            let m_level = (m / 2 + 1).max(2);

            for &idx in &level_nodes {
                let mut connected = 0;
                for &other in &level_nodes {
                    if other == idx || connected >= m_level {
                        break;
                    }
                    if self.layers[level].neighbors[idx].len() < m_level
                        && !self.layers[level].neighbors[idx].contains(&other)
                    {
                        self.layers[level].neighbors[idx].push(other);
                        connected += 1;
                    }
                }
            }
        }

        // Iterative refinement (NN-descent style)
        self.refine_layer0_graph(n, m)?;

        Ok(())
    }

    fn refine_layer0_graph(&mut self, n: usize, m: usize) -> VectorResult<()> {
        let iterations = 5;
        let nodes = &self.nodes;
        let metric = self.metric;

        for _iter in 0..iterations {
            let mut improved = 0;

            for idx in 0..n {
                let query = &nodes[idx].vector;
                let mut candidates: Vec<usize> = self.layers[0].neighbors[idx].clone();

                for &neighbor in &self.layers[0].neighbors[idx] {
                    for &nn in &self.layers[0].neighbors[neighbor] {
                        if nn != idx && !candidates.contains(&nn) {
                            candidates.push(nn);
                        }
                    }
                }

                let mut best: BinaryHeap<DistNode> = BinaryHeap::new();

                for &cand in &candidates {
                    if cand == idx {
                        continue;
                    }
                    let d = -compute_similarity(query, &nodes[cand].vector, metric);
                    best.push(DistNode { dist: d, idx: cand });
                    if best.len() > m {
                        let _ = best.pop();
                    }
                }

                let new_neighbors: Vec<usize> =
                    best.into_vec().into_iter().map(|n| n.idx).collect();

                let old_set: std::collections::HashSet<_> =
                    self.layers[0].neighbors[idx].iter().cloned().collect();
                let new_set: std::collections::HashSet<_> = new_neighbors.iter().cloned().collect();

                let diff = old_set.symmetric_difference(&new_set).count();
                improved += diff;

                if diff > 0 {
                    self.layers[0].neighbors[idx] = new_neighbors;
                }
            }

            if improved < n / 100 {
                break;
            }
        }

        for i in 0..n {
            for &j in self.layers[0].neighbors[i].clone().iter() {
                if !self.layers[0].neighbors[j].contains(&i)
                    && self.layers[0].neighbors[j].len() < m
                {
                    self.layers[0].neighbors[j].push(i);
                }
            }
        }

        Ok(())
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

    #[test]
    fn test_hnsw_1k_build_and_search() {
        let size = 1_000;
        let dim = 128;

        let vectors: Vec<(u64, Vec<f32>)> = (0..size)
            .map(|i| {
                let v: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
                (i as u64, v)
            })
            .collect();

        let mut index = HnswIndex::with_params(16, 200, 256, DistanceMetric::Cosine);

        let build_start = std::time::Instant::now();
        for (id, v) in vectors.iter() {
            index.insert(*id, v).unwrap();
        }
        let build_time = build_start.elapsed().as_secs_f64();
        println!("1K HNSW build: {:.2}s", build_time);

        let query = vec![0.5f32; dim];
        let search_start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = index.search(&query, 10).unwrap();
        }
        let total_search_time = search_start.elapsed().as_secs_f64();
        let avg_elapsed = total_search_time / 100.0 * 1000.0;

        assert_eq!(index.search(&query, 10).unwrap().len(), 10);
        println!("1K HNSW search: {:.3}ms avg", avg_elapsed);
    }

    #[test]
    #[ignore]
    fn test_hnsw_10k_build_and_search() {
        let size = 10_000;
        let dim = 128;

        let vectors: Vec<(u64, Vec<f32>)> = (0..size)
            .map(|i| {
                let v: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
                (i as u64, v)
            })
            .collect();

        let mut index = HnswIndex::with_params(16, 200, 256, DistanceMetric::Cosine);

        let build_start = std::time::Instant::now();
        for (id, v) in vectors.iter() {
            index.insert(*id, v).unwrap();
        }
        let build_time = build_start.elapsed().as_secs_f64();
        println!("10K HNSW build: {:.2}s", build_time);

        let query = vec![0.5f32; dim];
        let search_start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = index.search(&query, 10).unwrap();
        }
        let total_search_time = search_start.elapsed().as_secs_f64();
        let avg_elapsed = total_search_time / 100.0 * 1000.0;

        assert_eq!(index.search(&query, 10).unwrap().len(), 10);
        println!("10K HNSW search: {:.3}ms avg", avg_elapsed);

        if avg_elapsed < 5.0 {
            println!("10K search PASS ✅");
        } else {
            println!("10K search FAIL ❌");
        }
    }

    #[test]
    #[ignore]
    fn test_hnsw_100k_search_performance() {
        let size = 100_000;
        let dim = 128;

        let vectors: Vec<(u64, Vec<f32>)> = (0..size)
            .map(|i| {
                let v: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
                (i as u64, v)
            })
            .collect();

        let mut index = HnswIndex::with_params(16, 200, 256, DistanceMetric::Cosine);

        let build_start = std::time::Instant::now();
        for (id, v) in vectors.iter() {
            index.insert(*id, v).unwrap();
        }
        let build_time = build_start.elapsed().as_secs_f64();
        println!("100K HNSW build: {:.2}s", build_time);

        let query = vec![0.5f32; dim];
        let search_start = std::time::Instant::now();
        for _ in 0..10 {
            let _ = index.search(&query, 10).unwrap();
        }
        let total_search_time = search_start.elapsed().as_secs_f64();
        let avg_elapsed = total_search_time / 10.0 * 1000.0;

        assert_eq!(index.search(&query, 10).unwrap().len(), 10);
        println!("100K HNSW search: {:.2}ms avg (target < 10ms)", avg_elapsed);

        if avg_elapsed < 10.0 {
            println!("PASS ✅");
        } else {
            println!("FAIL ❌ - Need further optimization");
        }
    }

    #[test]
    #[ignore]
    fn test_hnsw_100k_batch_build() {
        let size = 100_000;
        let dim = 128;

        let vectors: Vec<(u64, Vec<f32>)> = (0..size)
            .map(|i| {
                let v: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
                (i as u64, v)
            })
            .collect();

        let mut index = HnswIndex::with_params(16, 200, 256, DistanceMetric::Cosine);

        let build_start = std::time::Instant::now();
        index.build_from_vectors(vectors).unwrap();
        let build_time = build_start.elapsed().as_secs_f64();
        println!("100K HNSW BATCH build: {:.2}s", build_time);

        let query = vec![0.5f32; dim];
        let search_start = std::time::Instant::now();
        for _ in 0..10 {
            let _ = index.search(&query, 10).unwrap();
        }
        let total_search_time = search_start.elapsed().as_secs_f64();
        let avg_elapsed = total_search_time / 10.0 * 1000.0;

        assert_eq!(index.search(&query, 10).unwrap().len(), 10);
        println!("100K HNSW search: {:.2}ms avg (target < 10ms)", avg_elapsed);

        if avg_elapsed < 10.0 {
            println!("PASS ✅");
        } else {
            println!("FAIL ❌ - Need further optimization");
        }
    }

    #[test]
    #[ignore]
    fn test_hnsw_1m_search_performance() {
        let size = 1_000_000;
        let dim = 128;

        let vectors: Vec<(u64, Vec<f32>)> = (0..size)
            .map(|i| {
                let v: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
                (i as u64, v)
            })
            .collect();

        let mut index = HnswIndex::with_params(16, 200, 256, DistanceMetric::Cosine);

        let build_start = std::time::Instant::now();
        for (id, v) in vectors.iter() {
            index.insert(*id, v).unwrap();
        }
        let build_time = build_start.elapsed().as_secs_f64();
        println!("1M HNSW build: {:.2}s", build_time);

        let query = vec![0.5f32; dim];
        let search_start = std::time::Instant::now();
        for _ in 0..10 {
            let _ = index.search(&query, 10).unwrap();
        }
        let total_search_time = search_start.elapsed().as_secs_f64();
        let avg_elapsed = total_search_time / 10.0 * 1000.0;

        assert_eq!(index.search(&query, 10).unwrap().len(), 10);
        println!("1M HNSW search: {:.2}ms avg (target < 100ms)", avg_elapsed);

        if avg_elapsed < 100.0 {
            println!("PASS ✅");
        } else {
            println!("FAIL ❌ - Need further optimization");
        }
    }
}
