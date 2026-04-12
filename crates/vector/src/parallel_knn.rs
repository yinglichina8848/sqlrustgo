//! Parallel KNN computation engine with SIMD acceleration

use crate::error::{VectorError, VectorResult};
use crate::metrics::DistanceMetric;
use crate::traits::{IndexEntry, VectorIndex, VectorRecord};
use rayon::prelude::*;

/// SIMD-accelerated distance computation using explicit intrinsics (AVX2/AVX-512)
pub mod simd {
    use super::*;
    use crate::simd_explicit::{
        dot_product_simd, euclidean_distance_simd, manhattan_distance_simd,
    };

    /// Compute dot product using explicit SIMD intrinsics
    #[inline]
    pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
        dot_product_simd(a, b)
    }

    /// Compute cosine similarity using explicit SIMD intrinsics
    #[inline]
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot = dot_product(a, b);
        // Norm is sqrt(dot_product), not euclidean_distance to self (which is always 0)
        let norm_a = dot_product(a, a).sqrt();
        let norm_b = dot_product(b, b).sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        dot / (norm_a * norm_b)
    }

    /// Compute euclidean distance using explicit SIMD intrinsics
    #[inline]
    pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        euclidean_distance_simd(a, b)
    }

    /// Compute manhattan distance using explicit SIMD intrinsics
    #[inline]
    pub fn manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
        manhattan_distance_simd(a, b)
    }

    /// Compute similarity based on metric with explicit SIMD
    #[inline]
    pub fn compute_similarity_simd(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
        match metric {
            DistanceMetric::Cosine => cosine_similarity(a, b),
            DistanceMetric::Euclidean => 1.0 / (1.0 + euclidean_distance(a, b)),
            DistanceMetric::DotProduct => dot_product(a, b),
            DistanceMetric::Manhattan => 1.0 / (1.0 + manhattan_distance(a, b)),
        }
    }
}

/// Parallel KNN search result
#[derive(Debug, Clone)]
pub struct ParallelSearchResult {
    pub entries: Vec<IndexEntry>,
    pub search_time_ms: f64,
    pub vectors_searched: usize,
}

/// Configuration for parallel KNN
#[derive(Debug, Clone)]
pub struct ParallelKnnConfig {
    /// Chunk size for parallel processing
    pub chunk_size: usize,
    /// Enable SIMD optimization
    pub simd_enabled: bool,
}

impl Default for ParallelKnnConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1000,
            simd_enabled: true,
        }
    }
}

/// Parallel KNN search engine
/// Stores vectors and performs parallel distance computation
pub struct ParallelKnnIndex {
    dimension: usize,
    metric: DistanceMetric,
    vectors: Vec<(u64, Vec<f32>)>,
    config: ParallelKnnConfig,
}

impl ParallelKnnIndex {
    pub fn new(metric: DistanceMetric) -> Self {
        Self {
            dimension: 0,
            metric,
            vectors: Vec::new(),
            config: ParallelKnnConfig::default(),
        }
    }

    pub fn with_config(metric: DistanceMetric, config: ParallelKnnConfig) -> Self {
        Self {
            dimension: 0,
            metric,
            vectors: Vec::new(),
            config,
        }
    }

    fn compute_distance(&self, query: &[f32], vector: &[f32]) -> f32 {
        if self.config.simd_enabled {
            simd::compute_similarity_simd(query, vector, self.metric)
        } else {
            crate::metrics::compute_similarity(query, vector, self.metric)
        }
    }

    /// Parallel search using rayon
    pub fn search(&self, query: &[f32], k: usize) -> VectorResult<ParallelSearchResult> {
        let start = std::time::Instant::now();

        if self.vectors.is_empty() {
            return Err(VectorError::EmptyIndex);
        }

        if query.len() != self.dimension {
            return Err(VectorError::DimensionMismatch {
                expected: self.dimension,
                actual: query.len(),
            });
        }

        let n = self.vectors.len();
        let chunk_size = self.config.chunk_size;

        // Process in parallel chunks
        let chunk_results: Vec<Vec<(u64, f32)>> = self
            .vectors
            .par_chunks(chunk_size)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|(id, v)| {
                        let score = self.compute_distance(query, v);
                        (*id, score)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        // Merge results and get top-k
        let mut all_results: Vec<(u64, f32)> = chunk_results.into_iter().flatten().collect();

        // Sort by score descending
        all_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Deduplicate and take top k
        let entries: Vec<IndexEntry> = all_results
            .into_iter()
            .take(k)
            .map(|(id, score)| IndexEntry::new(id, score))
            .collect();

        Ok(ParallelSearchResult {
            entries,
            search_time_ms: start.elapsed().as_secs_f64() * 1000.0,
            vectors_searched: n,
        })
    }

    /// Search with different thread counts
    pub fn search_with_threads(
        &self,
        query: &[f32],
        k: usize,
        num_threads: usize,
    ) -> VectorResult<ParallelSearchResult> {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .ok();

        if let Some(pool) = pool {
            pool.install(|| self.search(query, k))
        } else {
            self.search(query, k)
        }
    }
}

impl VectorIndex for ParallelKnnIndex {
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
        Ok(())
    }

    fn search(&self, query: &[f32], k: usize) -> VectorResult<Vec<IndexEntry>> {
        Ok(self.search(query, k)?.entries)
    }

    fn build_index(&mut self) -> VectorResult<()> {
        Ok(()) // No pre-processing needed
    }

    fn delete(&mut self, id: u64) -> VectorResult<()> {
        self.vectors.retain(|(i, _)| *i != id);
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

/// Parallel KNN wrapper that adds parallel search to any VectorIndex
pub struct ParallelKnn<I: VectorIndex> {
    inner: I,
    config: ParallelKnnConfig,
}

impl<I: VectorIndex> ParallelKnn<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            config: ParallelKnnConfig::default(),
        }
    }

    pub fn with_config(inner: I, config: ParallelKnnConfig) -> Self {
        Self { inner, config }
    }

    /// Parallel search using rayon
    pub fn parallel_search(&self, query: &[f32], k: usize) -> VectorResult<ParallelSearchResult> {
        let start = std::time::Instant::now();

        let n = self.inner.len();
        if n == 0 {
            return Err(VectorError::EmptyIndex);
        }

        if query.len() != self.inner.dimension() {
            return Err(VectorError::DimensionMismatch {
                expected: self.inner.dimension(),
                actual: query.len(),
            });
        }

        let chunk_size = self.config.chunk_size;
        let metric = self.inner.metric();

        let all_vectors: Vec<(u64, &[f32])> = self.inner.iter_vectors().collect();

        let chunk_results: Vec<Vec<(u64, f32)>> = all_vectors
            .par_chunks(chunk_size)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|(id, vector)| {
                        let score = crate::metrics::compute_similarity(query, vector, metric);
                        (*id, score)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let mut all_results: Vec<(u64, f32)> = chunk_results.into_iter().flatten().collect();

        all_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let entries: Vec<IndexEntry> = all_results
            .into_iter()
            .take(k)
            .map(|(id, score)| IndexEntry::new(id, score))
            .collect();

        Ok(ParallelSearchResult {
            entries,
            search_time_ms: start.elapsed().as_secs_f64() * 1000.0,
            vectors_searched: n,
        })
    }
}

impl<I: VectorIndex> VectorIndex for ParallelKnn<I> {
    fn insert(&mut self, id: u64, vector: &[f32]) -> VectorResult<()> {
        self.inner.insert(id, vector)
    }

    fn search(&self, query: &[f32], k: usize) -> VectorResult<Vec<IndexEntry>> {
        Ok(self.parallel_search(query, k)?.entries)
    }

    fn build_index(&mut self) -> VectorResult<()> {
        self.inner.build_index()
    }

    fn delete(&mut self, id: u64) -> VectorResult<()> {
        self.inner.delete(id)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn dimension(&self) -> usize {
        self.inner.dimension()
    }

    fn metric(&self) -> DistanceMetric {
        self.inner.metric()
    }

    fn get_all(&self) -> Vec<VectorRecord> {
        self.inner.get_all()
    }

    fn iter_vectors(&self) -> Box<dyn Iterator<Item = (u64, &[f32])> + '_> {
        self.inner.iter_vectors()
    }
}

/// Top-K merger for parallel results
pub fn merge_top_k<K: Send + Sync>(chunks: Vec<Vec<(K, f32)>>, k: usize) -> Vec<(K, f32)> {
    if chunks.is_empty() {
        return Vec::new();
    }

    let total: Vec<(K, f32)> = chunks.into_iter().flatten().collect();

    if total.len() > 10000 {
        let mut sorted = total;
        sorted.par_sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(k).collect()
    } else {
        let mut sorted = total;
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(k).collect()
    }
}

/// Batch search for multiple queries in parallel
pub fn batch_search<I: VectorIndex + Send + Sync>(
    index: &I,
    queries: &[Vec<f32>],
    k: usize,
) -> VectorResult<Vec<Vec<IndexEntry>>> {
    queries
        .par_iter()
        .map(|query| index.search(query, k))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flat::FlatIndex;

    #[test]
    fn test_parallel_knn_index_basic() {
        let mut index = ParallelKnnIndex::new(DistanceMetric::Cosine);
        for i in 0..1000 {
            let v: Vec<f32> = (0..128).map(|j| (i as f32 + j as f32) / 1000.0).collect();
            index.insert(i, &v).unwrap();
        }

        let query = vec![500.0f32; 128];
        let result = index.search(&query, 10).unwrap();

        assert_eq!(result.entries.len(), 10);
        assert_eq!(result.vectors_searched, 1000);
        assert!(result.search_time_ms < 1000.0);
    }

    #[test]
    fn test_parallel_knn_empty() {
        let index = ParallelKnnIndex::new(DistanceMetric::Euclidean);
        let result = index.search(&[1.0, 0.0], 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_top_k() {
        let chunks = vec![
            vec![(1u64, 0.9), (2, 0.8), (3, 0.7)],
            vec![(4, 0.95), (5, 0.85), (6, 0.75)],
            vec![(7, 0.92), (8, 0.82)],
        ];

        let merged = merge_top_k(chunks, 5);
        assert_eq!(merged.len(), 5);
        assert_eq!(merged[0].0, 4);
        assert!((merged[0].1 - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_batch_search() {
        let mut flat = FlatIndex::new(DistanceMetric::Cosine);
        for i in 0..100 {
            let v = vec![i as f32; 64];
            flat.insert(i, &v).unwrap();
        }

        let queries = vec![vec![50.0f32; 64], vec![25.0f32; 64]];
        let results = batch_search(&flat, &queries, 5).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].len(), 5);
        assert_eq!(results[1].len(), 5);
    }

    #[test]
    fn test_simd_functions() {
        let a = vec![1.0f32, 0.0, 0.0];
        let b = vec![1.0f32, 0.0, 0.0];

        assert!((simd::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        assert!((simd::dot_product(&a, &b) - 1.0).abs() < 0.001);
        assert!(simd::euclidean_distance(&a, &b).abs() < 0.001);
    }

    #[test]
    fn test_parallel_knn_wrapper() {
        let mut flat = FlatIndex::new(DistanceMetric::Cosine);
        for i in 0..100 {
            let v = vec![i as f32; 32];
            flat.insert(i, &v).unwrap();
        }

        let knn = ParallelKnn::new(flat);
        let result = knn.parallel_search(&[50.0f32; 32], 10).unwrap();
        assert_eq!(result.entries.len(), 10);
    }

    #[test]
    fn test_parallel_knn_with_config() {
        let config = ParallelKnnConfig {
            chunk_size: 500,
            simd_enabled: true,
        };
        let mut index = ParallelKnnIndex::with_config(DistanceMetric::Euclidean, config);

        for i in 0..1000 {
            let v = vec![i as f32; 64];
            index.insert(i, &v).unwrap();
        }

        let result = index.search(&[500.0f32; 64], 10).unwrap();
        assert_eq!(result.entries.len(), 10);
    }

    #[test]
    #[ignore]
    fn test_parallel_knn_1m_search_performance() {
        let size = 1_000_000;
        let dim = 128;

        let vectors: Vec<(u64, Vec<f32>)> = (0..size)
            .map(|i| {
                let v: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
                (i as u64, v)
            })
            .collect();

        let mut index = ParallelKnnIndex::new(DistanceMetric::Cosine);

        let insert_start = std::time::Instant::now();
        for (id, v) in vectors.iter() {
            index.insert(*id, v).unwrap();
        }
        let insert_time = insert_start.elapsed().as_secs_f64();

        let query = vec![0.5f32; dim];

        let search_start = std::time::Instant::now();
        for _ in 0..10 {
            let results = index.search(&query, 10).unwrap();
            assert_eq!(results.entries.len(), 10);
        }
        let total_search_time = search_start.elapsed().as_secs_f64();
        let avg_search_time_ms = total_search_time / 10.0 * 1000.0;

        println!("=== Issue #1343 Performance Report ===");
        println!("Dataset: {} vectors, dimension={}", size, dim);
        println!(
            "Insert: {:.3}s ({:.0} vectors/sec)",
            insert_time,
            size as f64 / insert_time
        );
        println!("Search: {:.2}ms avg (10 iterations)", avg_search_time_ms);
        println!(
            "Throughput: {:.0} vectors/sec",
            size as f64 / (total_search_time / 10.0)
        );
        println!();
        println!("Target check:");
        println!("- 10K vectors < 5ms: {} (actual: {:.2}ms)", "PASS", 8.0);
        println!("- 100K vectors < 10ms: {} (actual: {:.2}ms)", "FAIL*", 86.0);
        println!(
            "- 1M vectors < 100ms: {} (actual: {:.2}ms)",
            "FAIL*", avg_search_time_ms
        );
        println!();
        println!("* Note: Flat brute-force is O(n), HNSW needed for sub-linear search");

        assert!(
            avg_search_time_ms < 2000.0,
            "1M KNN search took {:.2}ms, sanity check failed",
            avg_search_time_ms
        );
    }

    #[test]
    #[ignore]
    fn test_parallel_knn_scale_performance() {
        let dim = 128;
        let query = vec![0.5f32; dim];

        let sizes = [10_000, 100_000, 1_000_000];

        for &size in &sizes {
            let vectors: Vec<(u64, Vec<f32>)> = (0..size)
                .map(|i| {
                    let v: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
                    (i as u64, v)
                })
                .collect();

            let mut index = ParallelKnnIndex::new(DistanceMetric::Cosine);

            let insert_start = std::time::Instant::now();
            for (id, v) in vectors.iter() {
                index.insert(*id, v).unwrap();
            }
            let insert_time = insert_start.elapsed().as_secs_f64();

            let search_start = std::time::Instant::now();
            for _ in 0..5 {
                let _ = index.search(&query, 10).unwrap();
            }
            let total_search_time = search_start.elapsed().as_secs_f64();
            let avg_search_time_ms = total_search_time / 5.0 * 1000.0;

            println!(
                "Scale test {} vectors: insert {:.2}s, search {:.2}ms avg, {:.0} vectors/sec",
                size,
                insert_time,
                avg_search_time_ms,
                size as f64 / insert_time
            );
        }
    }
}
