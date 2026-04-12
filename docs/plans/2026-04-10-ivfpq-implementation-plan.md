# IVFPQ Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement Product Quantization for IVF index to achieve 1M vector KNN < 100ms

**Architecture:** Add PQ encoding to existing IVF - vectors are clustered via k-means, then each cluster's vectors are encoded with PQ. Search uses ADC (Asymmetric Distance Computation) to compute distances without decoding.

**Tech Stack:** Rust, Rayon for parallelism, existing IVF infrastructure

---

## Task 1: Create ProductQuantizer module

**Files:**
- Create: `crates/vector/src/pq.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pq_train_and_encode() {
        let dimension = 128;
        let m_pq = 16;
        let k_sub = 256;
        
        let mut pq = ProductQuantizer::new(dimension, m_pq, k_sub);
        
        // Create 1000 random vectors for training
        let vectors: Vec<Vec<f32>> = (0..1000)
            .map(|_| (0..dimension).map(|_| rand::random::<f32>()).collect())
            .collect();
        
        pq.train(&vectors).unwrap();
        
        // Encode a vector
        let test_vector: Vec<f32> = (0..dimension).map(|_| rand::random::<f32>()).collect();
        let code = pq.encode(&test_vector);
        
        assert_eq!(code.len(), m_pq); // 16 bytes
    }

    #[test]
    fn test_pq_adc_distance() {
        let dimension = 8;
        let m_pq = 4;  // 4 sub-vectors, each 2 dims
        let k_sub = 4;
        
        let mut pq = ProductQuantizer::new(dimension, m_pq, k_sub);
        
        let vectors: Vec<Vec<f32>> = vec![
            vec![0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
            vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
            vec![0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
        ];
        pq.train(&vectors).unwrap();
        
        let query = vec![0.1, 0.9, 0.1, 0.9, 0.1, 0.9, 0.1, 0.9];
        let code = pq.encode(&vectors[0]);
        
        let dist = pq.adc_distance(&query, &code);
        assert!(dist >= 0.0);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-vector pq -- --nocapture`
Expected: FAIL - module not found

**Step 3: Write minimal ProductQuantizer implementation**

```rust
use crate::error::{VectorError, VectorResult};

/// Product Quantization for vector compression
pub struct ProductQuantizer {
    pub dimension: usize,
    pub m_pq: usize,
    pub k_sub: usize,
    pub sub_dim: usize,
    pub centroids: Vec<Vec<Vec<f32>>>,  // [sub_idx][centroid_idx][sub_dim]
}

impl ProductQuantizer {
    pub fn new(dimension: usize, m_pq: usize, k_sub: usize) -> Self {
        let sub_dim = dimension / m_pq;
        Self {
            dimension,
            m_pq,
            k_sub,
            sub_dim,
            centroids: Vec::new(),
        }
    }

    pub fn train(&mut self, vectors: &[Vec<f32>]) -> VectorResult<()> {
        if vectors.is_empty() {
            return Err(VectorError::EmptyIndex);
        }
        
        let sub_dim = self.dimension / self.m_pq;
        
        // Initialize centroids for each sub-space using k-means++
        self.centroids = Vec::with_capacity(self.m_pq);
        
        for sub_idx in 0..self.m_pq {
            let mut sub_centroids = Vec::with_capacity(self.k_sub);
            
            // Extract sub-vectors
            let sub_vectors: Vec<Vec<f32>> = vectors
                .iter()
                .map(|v| {
                    v[sub_idx * sub_dim..(sub_idx + 1) * sub_dim].to_vec()
                })
                .collect();
            
            // Simple k-means: start with random centroid
            let mut rng = rand::thread_rng();
            let first_idx = rand::seq::index::sample(&mut rng, vectors.len(), 1)[0];
            sub_centroids.push(sub_vectors[first_idx].clone());
            
            // k-means++ initialization
            while sub_centroids.len() < self.k_sub {
                let distances: Vec<f32> = sub_vectors
                    .iter()
                    .map(|sv| {
                        sub_centroids
                            .iter()
                            .map(|c| euclidean(sv, c))
                            .fold(f32::MAX, f32::min)
                    })
                    .collect();
                
                let total: f32 = distances.iter().sum();
                let mut threshold = rng.gen::<f32>() * total;
                
                for (i, d) in distances.iter().enumerate() {
                    threshold -= d;
                    if threshold <= 0.0 {
                        sub_centroids.push(sub_vectors[i].clone());
                        break;
                    }
                }
            }
            
            self.centroids.push(sub_centroids);
        }
        
        // Iterate k-means
        for _iter in 0..50 {
            // Assign each vector to nearest centroid
            let mut assignments: Vec<Vec<usize>> = vec![Vec::new(); self.k_sub];
            
            for (vec_idx, v) in vectors.iter().enumerate() {
                let mut min_dist = f32::MAX;
                let mut min_centroid = 0;
                
                for sub_idx in 0..self.m_pq {
                    let sub_vec = &v[sub_idx * sub_dim..(sub_idx + 1) * sub_dim];
                    for (c_idx, centroid) in self.centroids[sub_idx].iter().enumerate() {
                        let dist = euclidean(sub_vec, centroid);
                        if dist < min_dist {
                            min_dist = dist;
                            min_centroid = c_idx;
                        }
                    }
                }
                assignments[min_centroid].push(vec_idx);
            }
            
            // Update centroids
            let mut converged = true;
            for sub_idx in 0..self.m_pq {
                for (c_idx, members) in assignments.iter().enumerate() {
                    if members.is_empty() {
                        continue;
                    }
                    
                    let mut new_centroid = vec![0.0f32; sub_dim];
                    for &member_idx in members {
                        let sub_vec = &vectors[member_idx][sub_idx * sub_dim..(sub_idx + 1) * sub_dim];
                        for (i, &val) in sub_vec.iter().enumerate() {
                            new_centroid[i] += val;
                        }
                    }
                    for val in new_centroid.iter_mut() {
                        *val /= members.len() as f32;
                    }
                    
                    if euclidean(&new_centroid, &self.centroids[sub_idx][c_idx]) > 1e-4 {
                        converged = false;
                    }
                    self.centroids[sub_idx][c_idx] = new_centroid;
                }
            }
            
            if converged {
                break;
            }
        }
        
        Ok(())
    }

    pub fn encode(&self, vector: &[f32]) -> Vec<u8> {
        let sub_dim = self.dimension / self.m_pq;
        let mut code = Vec::with_capacity(self.m_pq);
        
        for sub_idx in 0..self.m_pq {
            let sub_vec = &vector[sub_idx * sub_dim..(sub_idx + 1) * sub_dim];
            
            let mut min_dist = f32::MAX;
            let mut min_idx = 0u8;
            
            for (c_idx, centroid) in self.centroids[sub_idx].iter().enumerate() {
                let dist = euclidean(sub_vec, centroid);
                if dist < min_dist {
                    min_dist = dist;
                    min_idx = c_idx as u8;
                }
            }
            code.push(min_idx);
        }
        
        code
    }

    pub fn decode(&self, code: &[u8]) -> Vec<f32> {
        let sub_dim = self.dimension / self.m_pq;
        let mut vector = Vec::with_capacity(self.dimension);
        
        for sub_idx in 0..self.m_pq {
            let c_idx = code[sub_idx] as usize;
            vector.extend_from_slice(&self.centroids[sub_idx][c_idx]);
        }
        
        vector
    }

    pub fn adc_distance(&self, query: &[f32], code: &[u8]) -> f32 {
        let sub_dim = self.dimension / self.m_pq;
        let mut total = 0.0f32;
        
        for sub_idx in 0..self.m_pq {
            let sub_query = &query[sub_idx * sub_dim..(sub_idx + 1) * sub_dim];
            let c_idx = code[sub_idx] as usize;
            let dist = euclidean(sub_query, &self.centroids[sub_idx][c_idx]);
            total += dist * dist;
        }
        
        total.sqrt()
    }
}

#[inline]
fn euclidean(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-vector pq -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/vector/src/pq.rs
git commit -m "feat(vector): add ProductQuantizer with PQ16 encoding

- Implements train, encode, decode, adc_distance
- k-means++ initialization for centroids
- 128 dim / 16 sub_vecs = 8 dim per sub-vector
- m_pq=16, k_sub=256"

---

## Task 2: Create IVFPQ index module

**Files:**
- Create: `crates/vector/src/ivfpq.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ivfpq_basic() {
        let mut index = IvfpqIndex::new(DistanceMetric::Euclidean, 2, 4);
        
        // Insert vectors
        for i in 0..100 {
            let v = vec![i as f32, (100 - i) as f32];
            index.insert(i, &v).unwrap();
        }
        
        index.build_index().unwrap();
        
        let results = index.search(&[50.0, 50.0], 5).unwrap();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_ivfpq_10k_performance() {
        let size = 10_000;
        let dim = 128;
        
        let mut index = IvfpqIndex::new(DistanceMetric::Cosine, 256, 16);
        
        let vectors: Vec<(u64, Vec<f32>)> = (0..size)
            .map(|i| {
                let v: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
                (i as u64, v)
            })
            .collect();
        
        for (id, v) in &vectors {
            index.insert(*id, v).unwrap();
        }
        
        let build_start = std::time::Instant::now();
        index.build_index().unwrap();
        let build_time = build_start.elapsed();
        
        let query = vec![0.5f32; dim];
        
        let search_start = std::time::Instant::now();
        let results = index.search(&query, 10).unwrap();
        let search_time = search_start.elapsed();
        
        println!("IVFPQ {} vectors: build {}ms, search {}ms", 
                 size, build_time.as_millis(), search_time.as_millis());
        
        assert!(results.len() <= 10);
        assert!(search_time.as_millis() < 10); // < 10ms target
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-vector ivfpq -- --nocapture`
Expected: FAIL - module not found

**Step 3: Write IVFPQ implementation**

```rust
use crate::error::{VectorError, VectorResult};
use crate::metrics::{compute_similarity, DistanceMetric};
use crate::pq::ProductQuantizer;
use crate::traits::{IndexEntry, VectorIndex, VectorRecord};
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct Cluster {
    center: Vec<f32>,
    vector_ids: Vec<u64>,
    codes: Vec<Vec<u8>>,
}

pub struct IvfpqIndex {
    dimension: usize,
    metric: DistanceMetric,
    nlist: usize,
    m_pq: usize,
    k_sub: usize,
    vectors: Vec<(u64, Vec<f32>)>,
    clusters: Vec<Cluster>,
    pq: ProductQuantizer,
    built: bool,
}

impl IvfpqIndex {
    pub fn new(metric: DistanceMetric, nlist: usize, m_pq: usize) -> Self {
        Self {
            dimension: 0,
            metric,
            nlist,
            m_pq,
            k_sub: 256,
            vectors: Vec::new(),
            clusters: Vec::new(),
            pq: ProductQuantizer::new(128, m_pq, 256), // Placeholder
            built: false,
        }
    }

    pub fn with_params(nlist: usize, m_pq: usize, k_sub: usize, metric: DistanceMetric) -> Self {
        Self {
            dimension: 0,
            metric,
            nlist,
            m_pq,
            k_sub,
            vectors: Vec::new(),
            clusters: Vec::new(),
            pq: ProductQuantizer::new(128, m_pq, k_sub), // Placeholder
            built: false,
        }
    }

    fn kmeans_init(vectors: &[Vec<f32>], k: usize) -> Vec<Vec<f32>> {
        if vectors.is_empty() || k == 0 {
            return Vec::new();
        }
        let k = k.min(vectors.len());
        let mut centers: Vec<Vec<f32>> = Vec::with_capacity(k);
        centers.push(vectors[0].clone());

        for _ in 1..k {
            let mut distances: Vec<f32> = Vec::with_capacity(vectors.len());
            for v in vectors {
                let min_dist = centers
                    .iter()
                    .map(|c| {
                        c.iter()
                            .zip(v.iter())
                            .map(|(x, y)| (x - y).powi(2))
                            .sum::<f32>()
                            .sqrt()
                    })
                    .fold(f32::MAX, f32::min);
                distances.push(min_dist);
            }
            let total: f32 = distances.iter().sum();
            let mut threshold = rand::random::<f32>() * total;
            let mut selected = 0;
            for (i, d) in distances.iter().enumerate() {
                threshold -= d;
                if threshold <= 0.0 {
                    selected = i;
                    break;
                }
            }
            centers.push(vectors[selected].clone());
        }
        centers
    }

    fn kmeans_assign(vectors: &[Vec<f32>], centers: &[Vec<f32>]) -> Vec<usize> {
        vectors
            .iter()
            .map(|v| {
                centers
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        (i, c.iter().zip(v.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f32>().sqrt())
                    })
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .unwrap()
                    .0
            })
            .collect()
    }

    fn build_clusters(&mut self) -> VectorResult<()> {
        if self.vectors.is_empty() {
            return Err(VectorError::EmptyIndex);
        }

        self.dimension = self.vectors[0].1.len();
        let nlist = self.nlist.min(self.vectors.len());
        
        // Re-create PQ with correct dimension
        self.pq = ProductQuantizer::new(self.dimension, self.m_pq, self.k_sub);
        
        let vectors_only: Vec<_> = self.vectors.iter().map(|(_, v)| v.clone()).collect();
        
        // k-means for clustering
        let centers = Self::kmeans_init(&vectors_only, nlist);
        
        self.clusters = centers
            .into_iter()
            .enumerate()
            .map(|(i, vector)| Cluster {
                center: vector,
                vector_ids: Vec::new(),
                codes: Vec::new(),
            })
            .collect();

        let assignments = Self::kmeans_assign(
            &vectors_only,
            &self.clusters.iter().map(|c| c.center.clone()).collect::<Vec<_>>(),
        );

        // Collect vectors per cluster
        let mut cluster_vectors: Vec<Vec<Vec<f32>>> = vec![Vec::new(); nlist];
        let mut cluster_ids: Vec<Vec<u64>> = vec![Vec::new(); nlist];
        
        for ((id, v), &cluster) in self.vectors.iter().zip(assignments.iter()) {
            cluster_vectors[cluster].push(v.clone());
            cluster_ids[cluster].push(*id);
        }

        // Train PQ on each cluster (parallel)
        let pq_trained: Vec<_> = cluster_vectors
            .par_iter()
            .map(|cvecs| {
                if cvecs.is_empty() {
                    return None;
                }
                let mut pq = ProductQuantizer::new(self.dimension, self.m_pq, self.k_sub);
                pq.train(cvecs).ok()?;
                Some(pq)
            })
            .collect();

        // Build final clusters with PQ codes
        for (i, (ids, vectors)) in cluster_ids.into_iter().zip(cluster_vectors.into_iter()).enumerate() {
            if ids.is_empty() {
                continue;
            }
            
            let pq = pq_trained[i].as_ref().unwrap();
            
            let codes: Vec<Vec<u8>> = vectors.iter().map(|v| pq.encode(v)).collect();
            
            self.clusters[i].vector_ids = ids;
            self.clusters[i].codes = codes;
        }

        self.built = true;
        Ok(())
    }
}

impl VectorIndex for IvfpqIndex {
    fn insert(&mut self, id: u64, vector: &[f32]) -> VectorResult<()> {
        if self.dimension == 0 {
            self.dimension = vector.len();
        } else if vector.len() != self.dimension {
            return Err(VectorError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.len(),
            });
        }

        self.vectors.push((id, vector.to_vec()));
        self.built = false;
        Ok(())
    }

    fn search(&self, query: &[f32], k: usize) -> VectorResult<Vec<IndexEntry>> {
        if !self.built {
            return Err(VectorError::IndexNotBuilt);
        }

        if query.len() != self.dimension {
            return Err(VectorError::DimensionMismatch {
                expected: self.dimension,
                actual: query.len(),
            });
        }

        // Find nearest nprobe clusters
        let nprobe = (self.clusters.len() as f32 * 0.1).ceil() as usize;
        
        let mut cluster_scores: Vec<_> = self
            .clusters
            .iter()
            .enumerate()
            .filter(|c| !c.1.vector_ids.is_empty())
            .map(|(i, c)| {
                let score = compute_similarity(query, &c.center, self.metric);
                (i, score)
            })
            .collect();

        cluster_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let selected: Vec<usize> = cluster_scores.iter().take(nprobe).map(|(i, _)| *i).collect();

        // Search within selected clusters using ADC
        let mut candidates: Vec<(u64, f32)> = Vec::new();

        for &cluster_idx in &selected {
            let cluster = &self.clusters[cluster_idx];
            
            // Parallel search within cluster
            let results: Vec<(u64, f32)> = cluster
                .codes
                .par_iter()
                .zip(cluster.vector_ids.par_iter())
                .map(|(code, &id)| {
                    let dist = self.pq.adc_distance(query, code);
                    (id, -dist) // Negative for similarity ordering
                })
                .collect();
            
            candidates.extend(results);
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        let entries: Vec<IndexEntry> = candidates
            .into_iter()
            .take(k)
            .map(|(id, score)| IndexEntry::new(id, -score)) // Un-negate
            .collect();

        Ok(entries)
    }

    fn build_index(&mut self) -> VectorResult<()> {
        self.build_clusters()
    }

    fn delete(&mut self, _id: u64) -> VectorResult<()> {
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

    fn get_all(&self) -> Vec<VectorRecord> {
        self.vectors
            .iter()
            .map(|(id, vector)| VectorRecord {
                id: *id,
                vector: vector.clone(),
            })
            .collect()
    }

    fn iter_vectors(&self) -> Box<dyn Iterator<Item = (u64, &[f32])> + '_> {
        Box::new(
            self.vectors
                .iter()
                .map(|(id, vector)| (*id, vector.as_slice())),
        )
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-vector ivfpq -- --nocapture`
Expected: PASS (or adjust implementation as needed)

**Step 5: Commit**

```bash
git add crates/vector/src/ivfpq.rs
git commit -m "feat(vector): add IvfpqIndex with PQ encoding

- IVF clustering + Product Quantization
- ADC distance computation for fast search
- Parallel search within clusters"
```

---

## Task 3: Integrate into lib.rs

**Files:**
- Modify: `crates/vector/src/lib.rs:25-53`

**Step 1: Add module and export**

Add to `lib.rs`:
```rust
pub mod ivfpq;
pub mod pq;
```

Add to exports:
```rust
pub use ivfpq::IvfpqIndex;
pub use pq::ProductQuantizer;
```

**Step 2: Verify compilation**

Run: `cargo build -p sqlrustgo-vector`
Expected: SUCCESS

**Step 3: Commit**

```bash
git add crates/vector/src/lib.rs
git commit -m "feat(vector): export IvfpqIndex and ProductQuantizer"
```

---

## Task 4: Performance test with 1M vectors

**Files:**
- Modify: `crates/vector/src/ivfpq.rs` (add ignored test)

**Step 1: Add performance test**

```rust
#[test]
#[ignore]
fn test_ivfpq_1m_performance() {
    let size = 1_000_000;
    let dim = 128;
    
    let mut index = IvfpqIndex::new(DistanceMetric::Cosine, 256, 16);
    
    println!("Generating {} vectors...", size);
    let vectors: Vec<(u64, Vec<f32>)> = (0..size)
        .map(|i| {
            let v: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
            (i as u64, v)
        })
        .collect();
    
    println!("Inserting vectors...");
    for (id, v) in vectors {
        index.insert(id, &v).unwrap();
    }
    
    println!("Building index...");
    let build_start = std::time::Instant::now();
    index.build_index().unwrap();
    let build_time = build_start.elapsed();
    
    let query = vec![0.5f32; dim];
    
    println!("Warming up...");
    for _ in 0..3 {
        let _ = index.search(&query, 10).unwrap();
    }
    
    println!("Running search benchmark...");
    let search_start = std::time::Instant::now();
    for _ in 0..10 {
        let results = index.search(&query, 10).unwrap();
        assert_eq!(results.len(), 10);
    }
    let total_search_time = search_start.elapsed();
    let avg_search_time_ms = total_search_time.as_secs_f64() / 10.0 * 1000.0;
    
    println!("\n=== IVFPQ 1M Performance ===");
    println!("Build time: {:.2}s", build_time.as_secs_f64());
    println!("Search time: {:.2}ms avg", avg_search_time_ms);
    println!("Target: < 100ms");
    println!("Status: {}", if avg_search_time_ms < 100.0 { "PASS" } else { "FAIL" });
    
    assert!(
        avg_search_time_ms < 100.0,
        "1M search took {}ms, target is < 100ms",
        avg_search_time_ms
    );
}
```

**Step 2: Run performance test**

Run: `cargo test -p sqlrustgo-vector -- --ignored --nocapture`
Expected: Build completes, search < 100ms

**Step 3: Commit**

```bash
git add crates/vector/src/ivfpq.rs
git commit -m "perf(vector): add 1M IVFPQ performance test"
```

---

## Task 5: Final verification

**Step 1: Run all tests**

Run: `cargo test -p sqlrustgo-vector`
Expected: All pass

**Step 2: Run clippy**

Run: `cargo clippy -p sqlrustgo-vector -- -D warnings`
Expected: Clean

**Step 3: Push**

```bash
git push
```

---

## Summary

| Task | Description | Expected Time |
|------|-------------|---------------|
| 1 | ProductQuantizer module | ~30 min |
| 2 | IvfpqIndex with ADC search | ~45 min |
| 3 | Integration into lib.rs | ~5 min |
| 4 | 1M performance test | ~5 min |
| 5 | Final verification | ~10 min |

**Total estimated time: ~95 minutes**
