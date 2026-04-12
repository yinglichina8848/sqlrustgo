//! Vector Cost Model Module
//!
//! Provides cost estimation for vector operations (ANN search, similarity search, KNN).

use crate::unified_plan::{UnifiedPlan, VectorScanType};
use std::collections::HashMap;
use std::hash::Hash;

/// Vector index types for cost estimation
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub enum VectorIndexType {
    /// IVF (Inverted File) index
    Ivf,
    /// HNSW (Hierarchical Navigable Small World) index
    #[default]
    Hnsw,
    /// Simple brute force (no index)
    BruteForce,
}

/// Cost factors for vector operations
#[derive(Debug, Clone)]
pub struct VectorCostFactors {
    /// CPU cost per vector comparison
    pub cpu_cost_per_vector_cmp: f64,
    /// Memory access cost per vector
    pub memory_cost_per_vector: f64,
    /// I/O cost per page for vector index
    pub io_cost_per_page: f64,
    /// Network cost per byte for distributed vector search
    pub network_cost_per_byte: f64,
    /// Index type specific cost multipliers
    pub index_type_multipliers: HashMap<VectorIndexType, f64>,
}

impl Default for VectorCostFactors {
    fn default() -> Self {
        let mut index_type_multipliers = HashMap::new();
        index_type_multipliers.insert(VectorIndexType::Ivf, 0.1);
        index_type_multipliers.insert(VectorIndexType::Hnsw, 0.05);
        index_type_multipliers.insert(VectorIndexType::BruteForce, 1.0);

        Self {
            cpu_cost_per_vector_cmp: 0.001,
            memory_cost_per_vector: 0.0001,
            io_cost_per_page: 10.0,
            network_cost_per_byte: 0.001,
            index_type_multipliers,
        }
    }
}

impl VectorCostFactors {
    /// Get cost multiplier for index type
    pub fn get_index_multiplier(&self, index_type: &VectorIndexType) -> f64 {
        self.index_type_multipliers
            .get(index_type)
            .copied()
            .unwrap_or(1.0)
    }
}

/// VectorCostModel - cost estimation for vector operations
#[derive(Debug, Clone)]
pub struct VectorCostModel {
    factors: VectorCostFactors,
}

impl VectorCostModel {
    /// Create a new VectorCostModel with given factors
    pub fn new(factors: VectorCostFactors) -> Self {
        Self { factors }
    }

    /// Create a VectorCostModel with default factors
    pub fn default_model() -> Self {
        Self {
            factors: VectorCostFactors::default(),
        }
    }

    /// Estimate cost for KNN search
    ///
    /// Parameters:
    /// - k: number of nearest neighbors
    /// - n: total vectors in index
    /// - dimension: vector dimension
    /// - index_type: type of index used
    pub fn knn_cost(&self, _k: usize, n: u64, dimension: u32, index_type: &VectorIndexType) -> f64 {
        let base_comparisons = n as f64 * self.factors.cpu_cost_per_vector_cmp * dimension as f64;
        let index_multiplier = self.factors.get_index_multiplier(index_type);
        base_comparisons * index_multiplier
    }

    /// Estimate cost for ANN search
    ///
    /// Parameters:
    /// - n: total vectors in index
    /// - ef: search parameter (higher = more accurate but slower)
    /// - dimension: vector dimension
    /// - index_type: type of index used
    pub fn ann_cost(&self, n: u64, ef: usize, dimension: u32, index_type: &VectorIndexType) -> f64 {
        // ANN search scans a fraction of the index
        let scan_fraction = (ef as f64 / 100.0).min(1.0);
        let scanned_vectors = n as f64 * scan_fraction;
        let base_cost = scanned_vectors * self.factors.cpu_cost_per_vector_cmp * dimension as f64;
        let index_multiplier = self.factors.get_index_multiplier(index_type);
        base_cost * index_multiplier
    }

    /// Estimate cost for similarity search
    ///
    /// Parameters:
    /// - n: total vectors
    /// - dimension: vector dimension
    /// - threshold: similarity threshold
    /// - index_type: type of index used
    pub fn similarity_cost(
        &self,
        n: u64,
        dimension: u32,
        threshold: f32,
        index_type: &VectorIndexType,
    ) -> f64 {
        // Similarity search typically scans more vectors than ANN
        let scan_fraction = (1.0 - threshold as f64).max(0.1);
        let scanned_vectors = n as f64 * scan_fraction;
        let base_cost = scanned_vectors * self.factors.cpu_cost_per_vector_cmp * dimension as f64;
        let index_multiplier = self.factors.get_index_multiplier(index_type);
        base_cost * index_multiplier
    }

    /// Estimate cost for range search in vector space
    ///
    /// Parameters:
    /// - n: total vectors
    /// - dimension: vector dimension
    /// - radius: search radius
    /// - index_type: type of index used
    pub fn range_cost(
        &self,
        n: u64,
        dimension: u32,
        radius: f32,
        index_type: &VectorIndexType,
    ) -> f64 {
        // Range search scans vectors within radius
        let scan_fraction = (radius as f64 * 0.1).min(1.0);
        let scanned_vectors = n as f64 * scan_fraction;
        let base_cost = scanned_vectors * self.factors.cpu_cost_per_vector_cmp * dimension as f64;
        let index_multiplier = self.factors.get_index_multiplier(index_type);
        base_cost * index_multiplier
    }

    /// Estimate cost for a UnifiedPlan node
    pub fn estimate_plan_cost(&self, plan: &UnifiedPlan, dimension: u32) -> f64 {
        match plan {
            UnifiedPlan::VectorScan {
                scan_type, limit, ..
            } => {
                let n = 10000; // Assume 10k vectors - would come from stats
                let k = limit.unwrap_or(100);
                match scan_type {
                    VectorScanType::Knn { k: knn_k } => {
                        self.knn_cost(*knn_k, n, dimension, &VectorIndexType::Hnsw)
                    }
                    VectorScanType::Ann { threshold: _ } => {
                        self.ann_cost(n, k * 2, dimension, &VectorIndexType::Hnsw)
                    }
                    VectorScanType::Similarity { threshold } => {
                        self.similarity_cost(n, dimension, *threshold, &VectorIndexType::Hnsw)
                    }
                    VectorScanType::Range { radius } => {
                        self.range_cost(n, dimension, *radius, &VectorIndexType::Hnsw)
                    }
                }
            }
            UnifiedPlan::HybridVectorScan {
                sql_filter: Some(_),
                scan_type,
                limit,
                ..
            } => {
                // Hybrid has extra cost due to SQL pre-filtering
                let base_cost = self.estimate_plan_cost(
                    &UnifiedPlan::VectorScan {
                        vector_index: String::new(),
                        query_vector: vec![],
                        scan_type: scan_type.clone(),
                        limit: *limit,
                    },
                    dimension,
                );
                base_cost * 1.2 // 20% overhead for hybrid
            }
            _ => 0.0,
        }
    }

    /// Compare vector scan cost vs SQL table scan cost
    /// Returns true if vector scan is cheaper
    pub fn vector_scan_cheaper_than_sql(
        &self,
        vector_cardinality: u64,
        sql_cardinality: u64,
        dimension: u32,
        index_type: &VectorIndexType,
    ) -> bool {
        let vector_cost = self.ann_cost(vector_cardinality, 100, dimension, index_type);
        let sql_cost = sql_cardinality as f64 * 0.01; // Simplified SQL scan cost
        vector_cost < sql_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_cost_factors_default() {
        let factors = VectorCostFactors::default();
        assert_eq!(factors.cpu_cost_per_vector_cmp, 0.001);
        assert!(
            factors.get_index_multiplier(&VectorIndexType::Hnsw)
                < factors.get_index_multiplier(&VectorIndexType::BruteForce)
        );
    }

    #[test]
    fn test_knn_cost() {
        let model = VectorCostModel::default_model();
        let cost = model.knn_cost(10, 10000, 128, &VectorIndexType::Hnsw);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_ann_cost() {
        let model = VectorCostModel::default_model();
        let cost = model.ann_cost(10000, 100, 128, &VectorIndexType::Hnsw);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_similarity_cost() {
        let model = VectorCostModel::default_model();
        let cost = model.similarity_cost(10000, 128, 0.8, &VectorIndexType::Hnsw);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_range_cost() {
        let model = VectorCostModel::default_model();
        let cost = model.range_cost(10000, 128, 0.5, &VectorIndexType::Hnsw);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_vector_scan_cheaper_than_sql() {
        let model = VectorCostModel::default_model();
        // For high selectivity, vector should be cheaper
        let cheaper = model.vector_scan_cheaper_than_sql(10, 10000, 128, &VectorIndexType::Hnsw);
        assert!(cheaper);
    }

    #[test]
    fn test_estimate_plan_cost() {
        let model = VectorCostModel::default_model();
        let plan = UnifiedPlan::VectorScan {
            vector_index: "embeddings_idx".to_string(),
            query_vector: vec![0.1; 128],
            scan_type: VectorScanType::Knn { k: 10 },
            limit: Some(10),
        };
        let cost = model.estimate_plan_cost(&plan, 128);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_vector_index_type_default() {
        let index_type = VectorIndexType::default();
        assert!(matches!(index_type, VectorIndexType::Hnsw));
    }
}
