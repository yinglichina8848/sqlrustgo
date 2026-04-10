//! IVFPQ: Inverted File with Product Quantization
//!
//! Combines IVF clustering with PQ encoding for fast approximate nearest neighbor search.
//! Architecture: vectors are clustered via k-means, then each cluster's vectors are
//! encoded with PQ. Search uses ADC (Asymmetric Distance Computation).

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
    pq: ProductQuantizer,
}

pub struct IvfpqIndex {
    dimension: usize,
    metric: DistanceMetric,
    nlist: usize,
    m_pq: usize,
    k_sub: usize,
    vectors: Vec<(u64, Vec<f32>)>,
    clusters: Vec<Cluster>,
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
                    .map(|c| euclidean(v, c))
                    .fold(f32::MAX, f32::min);
                distances.push(min_dist);
            }
            let total: f32 = distances.iter().sum();
            if total <= 0.0 {
                let idx = rand::random::<usize>() % vectors.len();
                centers.push(vectors[idx].clone());
                continue;
            }
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
                    .map(|(i, c)| (i, euclidean(v, c)))
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

        let vectors_only: Vec<_> = self.vectors.iter().map(|(_, v)| v.clone()).collect();

        // k-means for clustering
        let centers = Self::kmeans_init(&vectors_only, nlist);

        self.clusters = centers
            .into_iter()
            .enumerate()
            .map(|(_, vector)| Cluster {
                center: vector,
                vector_ids: Vec::new(),
                codes: Vec::new(),
                pq: ProductQuantizer::new(self.dimension, self.m_pq, self.k_sub),
            })
            .collect();

        let assignments = Self::kmeans_assign(
            &vectors_only,
            &self
                .clusters
                .iter()
                .map(|c| c.center.clone())
                .collect::<Vec<_>>(),
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
        for (i, (ids, vectors)) in cluster_ids
            .into_iter()
            .zip(cluster_vectors.into_iter())
            .enumerate()
        {
            if ids.is_empty() {
                continue;
            }

            if let Some(ref pq) = pq_trained[i] {
                let codes: Vec<Vec<u8>> = vectors.iter().map(|v| pq.encode(v)).collect();

                self.clusters[i].vector_ids = ids;
                self.clusters[i].codes = codes;
                self.clusters[i].pq = pq.clone();
            }
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
        let nprobe = ((self.clusters.len() as f32) * 0.1).ceil() as usize;

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
        let selected: Vec<usize> = cluster_scores
            .iter()
            .take(nprobe)
            .map(|(i, _)| *i)
            .collect();

        // Search within selected clusters using ADC
        let mut candidates: Vec<(u64, f32)> = Vec::new();

        for &cluster_idx in &selected {
            let cluster = &self.clusters[cluster_idx];
            let pq = &cluster.pq;

            let results: Vec<(u64, f32)> = cluster
                .codes
                .par_iter()
                .zip(cluster.vector_ids.par_iter())
                .map(|(code, &id)| {
                    let dist = pq.adc_distance(query, code);
                    (id, -dist)
                })
                .collect();

            candidates.extend(results);
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let entries: Vec<IndexEntry> = candidates
            .into_iter()
            .take(k)
            .map(|(id, score)| IndexEntry::new(id, -score))
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

fn euclidean(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ivfpq_new() {
        let index = IvfpqIndex::new(DistanceMetric::Euclidean, 2, 4);
        assert_eq!(index.dimension, 0);
        assert!(!index.built);
    }

    #[test]
    fn test_ivfpq_basic() {
        let mut index = IvfpqIndex::new(DistanceMetric::Euclidean, 2, 4);

        for i in 0..100 {
            let v = vec![i as f32, (100 - i) as f32];
            index.insert(i, &v).unwrap();
        }

        index.build_index().unwrap();

        let results = index.search(&[50.0, 50.0], 5).unwrap();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_ivfpq_dimension_mismatch() {
        let mut index = IvfpqIndex::new(DistanceMetric::Euclidean, 2, 4);
        index.insert(1, &[1.0, 0.0]).unwrap();
        let result = index.insert(2, &[1.0, 0.0, 0.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_ivfpq_search_before_build() {
        let mut index = IvfpqIndex::new(DistanceMetric::Euclidean, 2, 4);
        index.insert(1, &[1.0, 0.0]).unwrap();
        let result = index.search(&[1.0, 0.0], 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_ivfpq_empty() {
        let index = IvfpqIndex::new(DistanceMetric::Euclidean, 2, 4);
        assert!(index.is_empty());
    }

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
        println!(
            "Status: {}",
            if avg_search_time_ms < 100.0 {
                "PASS"
            } else {
                "FAIL"
            }
        );

        assert!(
            avg_search_time_ms < 100.0,
            "1M search took {}ms, target is < 100ms",
            avg_search_time_ms
        );
    }
}
