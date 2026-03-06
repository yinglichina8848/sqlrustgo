//! Cost Model Module

use super::*;

/// SimpleCostModel - basic cost estimation implementation
#[derive(Debug, Clone, Default)]
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
            "nested_loop" => {
                (left_rows * right_rows) as f64 * self.cpu_cost_per_row
            }
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
            pages as f64 * self.io_cost_per_page * 2.0
        }
    }
}

impl CostModel for SimpleCostModel {
    fn estimate_cost(&self, plan: &dyn std::any::Any) -> f64 {
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
}
