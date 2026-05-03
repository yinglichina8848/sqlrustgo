//! Cost Model Module

use super::*;

/// SimpleCostModel - basic cost estimation implementation
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct SimpleCostModel {
    /// CPU cost per row
    cpu_cost_per_row: f64,
    /// I/O cost per page
    io_cost_per_page: f64,
    /// Network cost per byte
    network_cost_per_byte: f64,
}

impl SimpleCostModel {
    pub fn new(cpu_cost_per_row: f64, io_cost_per_page: f64, network_cost_per_byte: f64) -> Self {
        Self {
            cpu_cost_per_row,
            io_cost_per_page,
            network_cost_per_byte,
        }
    }

    /// Default cost model with standard values
    pub fn default_model() -> Self {
        Self::new(1.0, 10.0, 0.001)
    }

    /// Estimate cost for sequential scan
    pub fn seq_scan_cost(&self, row_count: u64, page_count: u64) -> f64 {
        (row_count as f64 * self.cpu_cost_per_row) + (page_count as f64 * self.io_cost_per_page)
    }

    /// Estimate cost for index scan
    pub fn index_scan_cost(&self, row_count: u64, index_pages: u64, data_pages: u64) -> f64 {
        let index_cost = index_pages as f64 * self.io_cost_per_page;
        let data_cost = data_pages as f64 * self.io_cost_per_page;
        let cpu_cost = row_count as f64 * self.cpu_cost_per_row;
        index_cost + data_cost + cpu_cost
    }

    /// Estimate cost for join operation
    pub fn join_cost(&self, left_rows: u64, right_rows: u64, join_method: &str) -> f64 {
        match join_method {
            "nested_loop" => (left_rows * right_rows) as f64 * self.cpu_cost_per_row,
            "hash_join" => {
                let build_cost = left_rows as f64 * self.cpu_cost_per_row;
                let probe_cost = right_rows as f64 * self.cpu_cost_per_row;
                build_cost + probe_cost
            }
            "sort_merge" => {
                let sort_cost = (left_rows + right_rows) as f64 * self.cpu_cost_per_row;
                let merge_cost = (left_rows + right_rows) as f64 * self.cpu_cost_per_row;
                sort_cost + merge_cost
            }
            _ => (left_rows + right_rows) as f64 * self.cpu_cost_per_row,
        }
    }

    /// Estimate cost for aggregation
    pub fn agg_cost(&self, row_count: u64, group_by_cols: u32) -> f64 {
        let base_cost = row_count as f64 * self.cpu_cost_per_row;
        let sort_cost = if group_by_cols > 0 {
            row_count as f64 * self.cpu_cost_per_row * (group_by_cols as f64).log2()
        } else {
            0.0
        };
        base_cost + sort_cost
    }

    /// Estimate cost for sorting
    pub fn sort_cost(&self, row_count: u64, avg_row_size: u32) -> f64 {
        let memory_sort_threshold = 1_000_000; // Assume 1M rows fit in memory
        if row_count < memory_sort_threshold {
            row_count as f64 * self.cpu_cost_per_row * (row_count as f64).log2()
        } else {
            // External sort - I/O bound
            let pages = (row_count as f64 * avg_row_size as f64) / 4096.0;
            pages * self.io_cost_per_page * 2.0
        }
    }
}

impl CostModel for SimpleCostModel {
    fn estimate_cost(&self, _plan: &dyn std::any::Any) -> f64 {
        // Simplified - just return a default cost for now
        100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_cost_model_default() {
        let model = SimpleCostModel::default_model();
        assert_eq!(model.cpu_cost_per_row, 1.0);
        assert_eq!(model.io_cost_per_page, 10.0);
    }

    #[test]
    fn test_seq_scan_cost() {
        let model = SimpleCostModel::default_model();
        let cost = model.seq_scan_cost(1000, 10);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_index_scan_cost() {
        let model = SimpleCostModel::default_model();
        let cost = model.index_scan_cost(100, 2, 5);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_join_cost_nested_loop() {
        let model = SimpleCostModel::default_model();
        let cost = model.join_cost(1000, 2000, "nested_loop");
        assert!(cost > 0.0);
    }

    #[test]
    fn test_join_cost_hash_join() {
        let model = SimpleCostModel::default_model();
        let cost = model.join_cost(1000, 2000, "hash_join");
        assert!(cost > 0.0);
    }

    #[test]
    fn test_agg_cost() {
        let model = SimpleCostModel::default_model();
        let cost = model.agg_cost(10000, 2);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_sort_cost_in_memory() {
        let model = SimpleCostModel::default_model();
        let cost = model.sort_cost(1000, 100);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cost_model_trait() {
        fn _check_cost_model<T: CostModel>(_model: &T) {}
        _check_cost_model(&SimpleCostModel::default_model());
    }

    #[test]
    fn test_simple_cost_model_new() {
        let model = SimpleCostModel::new(2.0, 5.0, 0.01);
        assert_eq!(model.cpu_cost_per_row, 2.0);
        assert_eq!(model.io_cost_per_page, 5.0);
        assert_eq!(model.network_cost_per_byte, 0.01);
    }

    #[test]
    fn test_seq_scan_cost_zero_rows() {
        let model = SimpleCostModel::default_model();
        let cost = model.seq_scan_cost(0, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_index_scan_cost_zero_rows() {
        let model = SimpleCostModel::default_model();
        let cost = model.index_scan_cost(0, 0, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_join_cost_sort_merge() {
        let model = SimpleCostModel::default_model();
        let cost = model.join_cost(100, 200, "sort_merge");
        assert!(cost > 0.0);
    }

    #[test]
    fn test_join_cost_unknown_method() {
        let model = SimpleCostModel::default_model();
        let cost = model.join_cost(100, 200, "unknown");
        assert!(cost > 0.0);
    }

    #[test]
    fn test_agg_cost_zero_groups() {
        let model = SimpleCostModel::default_model();
        let cost = model.agg_cost(1000, 0);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_sort_cost_external() {
        let model = SimpleCostModel::default_model();
        let cost = model.sort_cost(2_000_000, 100);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cost_model_estimate_cost() {
        let model = SimpleCostModel::default_model();
        let cost = model.estimate_cost(&());
        assert_eq!(cost, 100.0);
    }
}

/// CboOptimizer - Cost-Based Optimizer with statistics support
///
/// This is a wrapper around SimpleCostModel that integrates with the statistics
/// provider for more accurate cost estimations.
#[derive(Debug, Clone)]
pub struct CboOptimizer {
    /// Base cost model
    cost_model: SimpleCostModel,
    /// Default row count estimate when no statistics available
    default_row_count: u64,
    /// Default page count estimate when no statistics available
    default_page_count: u64,
}

impl CboOptimizer {
    /// Create a new CboOptimizer with default settings
    pub fn new() -> Self {
        Self {
            cost_model: SimpleCostModel::default_model(),
            default_row_count: 1000,
            default_page_count: 10,
        }
    }

    /// Create a new CboOptimizer with custom defaults
    pub fn with_defaults(row_count: u64, page_count: u64) -> Self {
        Self {
            cost_model: SimpleCostModel::default_model(),
            default_row_count: row_count,
            default_page_count: page_count,
        }
    }

    /// Get the underlying cost model
    pub fn cost_model(&self) -> &SimpleCostModel {
        &self.cost_model
    }

    /// Get default row count estimate
    pub fn default_row_count(&self) -> u64 {
        self.default_row_count
    }

    /// Get default page count estimate
    pub fn default_page_count(&self) -> u64 {
        self.default_page_count
    }

    /// Estimate cost for a table scan using statistics
    pub fn estimate_scan_cost(&self, row_count: u64, page_count: u64) -> f64 {
        self.cost_model.seq_scan_cost(row_count, page_count)
    }

    /// Estimate cost for an index scan
    pub fn estimate_index_scan_cost(
        &self,
        row_count: u64,
        index_pages: u64,
        data_pages: u64,
    ) -> f64 {
        self.cost_model
            .index_scan_cost(row_count, index_pages, data_pages)
    }

    /// Estimate cost for a join operation
    pub fn estimate_join_cost(&self, left_rows: u64, right_rows: u64, method: &str) -> f64 {
        self.cost_model.join_cost(left_rows, right_rows, method)
    }
}

impl Default for CboOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl CostModel for CboOptimizer {
    fn estimate_cost(&self, _plan: &dyn std::any::Any) -> f64 {
        // Simplified - return default cost based on row count
        self.cost_model
            .seq_scan_cost(self.default_row_count, self.default_page_count)
    }
}

#[cfg(test)]
mod cbo_tests {
    use super::*;

    #[test]
    fn test_cbo_optimizer_new() {
        let cbo = CboOptimizer::new();
        assert_eq!(cbo.default_row_count(), 1000);
        assert_eq!(cbo.default_page_count(), 10);
    }

    #[test]
    fn test_cbo_optimizer_with_defaults() {
        let cbo = CboOptimizer::with_defaults(5000, 50);
        assert_eq!(cbo.default_row_count(), 5000);
        assert_eq!(cbo.default_page_count(), 50);
    }

    #[test]
    fn test_cbo_estimate_scan_cost() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_scan_cost(1000, 10);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cbo_estimate_index_scan_cost() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_index_scan_cost(100, 2, 5);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cbo_estimate_join_cost() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_join_cost(100, 200, "hash_join");
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cbo_cost_model_trait() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_cost(&());
        assert!(cost > 0.0);
    }
}
