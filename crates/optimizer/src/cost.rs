//! Cost Model Module

use super::*;

/// SimpleCostModel - basic cost estimation implementation
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct SimpleCostModel {
    /// CPU cost per row
    pub cpu_cost_per_row: f64,
    /// I/O cost per page
    pub io_cost_per_page: f64,
    /// Network cost per byte
    pub network_cost_per_byte: f64,
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

    /// Estimate cost using statistics - integrates with StatisticsProvider
    pub fn estimate_with_stats(&self, table_stats: &super::stats::TableStats) -> f64 {
        let row_count = table_stats.row_count();
        let page_count = table_stats.page_count();
        self.seq_scan_cost(row_count, page_count)
    }

    /// Estimate index scan cost using column statistics
    pub fn estimate_index_scan_with_stats(
        &self,
        table_stats: &super::stats::TableStats,
        column_name: &str,
    ) -> f64 {
        let row_count = table_stats.row_count();
        let page_count = table_stats.page_count();

        let index_selectivity = table_stats
            .column_stats(column_name)
            .map(|cs| cs.eq_selectivity())
            .unwrap_or(0.1);

        let estimated_rows = (row_count as f64 * index_selectivity) as u64;
        let index_pages = ((estimated_rows as f64) / 100.0) as u64;

        self.index_scan_cost(estimated_rows, index_pages.max(1), page_count)
    }

    /// Estimate join cost using table statistics
    pub fn estimate_join_with_stats(
        &self,
        left_stats: &super::stats::TableStats,
        right_stats: &super::stats::TableStats,
        join_method: &str,
    ) -> f64 {
        let left_rows = left_stats.row_count();
        let right_rows = right_stats.row_count();
        self.join_cost(left_rows, right_rows, join_method)
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
    fn test_join_cost_sort_merge() {
        let model = SimpleCostModel::default_model();
        let cost = model.join_cost(1000, 2000, "sort_merge");
        assert!(cost > 0.0);
    }

    #[test]
    fn test_join_cost_default() {
        let model = SimpleCostModel::default_model();
        let cost = model.join_cost(1000, 2000, "unknown_method");
        assert!(cost > 0.0);
    }

    #[test]
    fn test_sort_cost_external() {
        let model = SimpleCostModel::default_model();
        let cost = model.sort_cost(2_000_000, 100);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_simple_cost_model_custom() {
        let model = SimpleCostModel::new(2.0, 20.0, 0.002);
        assert_eq!(model.cpu_cost_per_row, 2.0);
        assert_eq!(model.io_cost_per_page, 20.0);
    }

    #[test]
    fn test_agg_cost_no_group_by() {
        let model = SimpleCostModel::default_model();
        let cost = model.agg_cost(1000, 0);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_estimate_with_stats() {
        use super::stats::{ColumnStats, TableStats};

        let model = SimpleCostModel::default_model();
        let table_stats = TableStats::new("users")
            .with_row_count(10000)
            .with_size_bytes(409600);

        let cost = model.estimate_with_stats(&table_stats);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_estimate_index_scan_with_stats() {
        use super::stats::{ColumnStats, TableStats};

        let model = SimpleCostModel::default_model();
        let table_stats = TableStats::new("users")
            .with_row_count(10000)
            .with_size_bytes(409600)
            .add_column_stats(ColumnStats::new("id").with_distinct_count(10000));

        let cost = model.estimate_index_scan_with_stats(&table_stats, "id");
        assert!(cost > 0.0);
    }

    #[test]
    fn test_estimate_join_with_stats() {
        use super::stats::TableStats;

        let model = SimpleCostModel::default_model();
        let left_stats = TableStats::new("users")
            .with_row_count(1000)
            .with_size_bytes(40960);
        let right_stats = TableStats::new("orders")
            .with_row_count(5000)
            .with_size_bytes(204800);

        let cost = model.estimate_join_with_stats(&left_stats, &right_stats, "hash_join");
        assert!(cost > 0.0);
    }
}

pub struct CboOptimizer {
    cost_model: SimpleCostModel,
    stats_provider: Option<Box<dyn super::stats::StatisticsProvider>>,
}

impl CboOptimizer {
    pub fn new() -> Self {
        Self {
            cost_model: SimpleCostModel::default_model(),
            stats_provider: None,
        }
    }

    pub fn with_stats_provider(
        mut self,
        provider: Box<dyn super::stats::StatisticsProvider>,
    ) -> Self {
        self.stats_provider = Some(provider);
        self
    }

    pub fn estimate_scan_cost(&self, table: &str) -> f64 {
        if let Some(ref provider) = self.stats_provider {
            if let Some(stats) = provider.table_stats(table) {
                return self.cost_model.estimate_with_stats(&stats);
            }
        }
        self.cost_model.seq_scan_cost(1000, 10)
    }

    pub fn estimate_index_scan_cost(&self, table: &str, column: &str) -> f64 {
        if let Some(ref provider) = self.stats_provider {
            if let Some(stats) = provider.table_stats(table) {
                return self
                    .cost_model
                    .estimate_index_scan_with_stats(&stats, column);
            }
        }
        self.cost_model.index_scan_cost(100, 1, 10)
    }

    pub fn select_access_method(
        &self,
        table: &str,
        column: &str,
        selectivity_threshold: f64,
    ) -> &str {
        let seq_scan_cost = self.estimate_scan_cost(table);
        let index_scan_cost = self.estimate_index_scan_cost(table, column);

        let selectivity = if let Some(ref provider) = self.stats_provider {
            provider.selectivity(table, column)
        } else {
            0.1
        };

        if selectivity < selectivity_threshold && index_scan_cost < seq_scan_cost {
            "index_scan"
        } else {
            "seq_scan"
        }
    }
}

impl Default for CboOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// 谓词类型 - 用于成本估算决策
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicateType {
    /// 等值谓词 (e.g., WHERE col = 100)
    Eq,
    /// 范围谓词 (e.g., WHERE col > 100)
    Range,
    /// IN 谓词 (e.g., WHERE col IN (1, 2, 3))
    In,
    /// LIKE 谓词 (e.g., WHERE col LIKE '%abc%')
    Like,
}

impl PredicateType {
    /// 从字符串判断谓词类型
    pub fn from_op(op: &str) -> Self {
        match op {
            "=" | "==" | "!=" | "<>" | "<=>" => PredicateType::Eq,
            ">" | "<" | ">=" | "<=" => PredicateType::Range,
            "LIKE" | "NOT LIKE" => PredicateType::Like,
            "IN" | "NOT IN" => PredicateType::In,
            _ => PredicateType::Eq, // Default to Eq for unknown operators
        }
    }
}

/// 访问方法选择
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMethod {
    /// 顺序扫描
    SeqScan,
    /// 索引扫描 (B+Tree)
    IndexScan,
    /// Hash 索引查找
    HashIndex,
}

impl std::fmt::Display for AccessMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessMethod::SeqScan => write!(f, "seq_scan"),
            AccessMethod::IndexScan => write!(f, "index_scan"),
            AccessMethod::HashIndex => write!(f, "hash_index"),
        }
    }
}

impl CboOptimizer {
    /// Estimate cost for range scan using index
    pub fn estimate_range_scan_cost(&self, table: &str, _column: &str, selectivity: f64) -> f64 {
        if let Some(ref provider) = self.stats_provider {
            if let Some(stats) = provider.table_stats(table) {
                let row_count = stats.row_count();
                let page_count = stats.page_count();
                let index_pages = ((row_count as f64 * selectivity) / 100.0).max(1.0) as u64;

                return self.cost_model.index_scan_cost(
                    (row_count as f64 * selectivity) as u64,
                    index_pages,
                    page_count,
                );
            }
        }
        // Default estimate
        self.cost_model.index_scan_cost(100, 1, 10)
    }

    /// Select best access method considering all factors
    pub fn select_best_access_method(
        &self,
        table: &str,
        column: &str,
        predicate_type: PredicateType,
    ) -> AccessMethod {
        let selectivity = self
            .stats_provider
            .as_ref()
            .map(|p| p.selectivity(table, column))
            .unwrap_or(0.1);

        match predicate_type {
            PredicateType::Eq => {
                // For equality, use index if selectivity is low (high cardinality)
                let threshold = 0.05; // 5%
                if selectivity < threshold {
                    AccessMethod::IndexScan
                } else {
                    AccessMethod::SeqScan
                }
            }
            PredicateType::Range => {
                // For range, compare seq_scan vs index_scan costs
                let seq_cost = self.estimate_scan_cost(table);
                let range_cost = self.estimate_range_scan_cost(table, column, selectivity);
                if range_cost < seq_cost {
                    AccessMethod::IndexScan
                } else {
                    AccessMethod::SeqScan
                }
            }
            PredicateType::In => {
                // IN is similar to multiple EQ, usually benefits from index
                AccessMethod::IndexScan
            }
            PredicateType::Like => {
                // LIKE with leading wildcard can't use index
                AccessMethod::SeqScan
            }
        }
    }

    /// Legacy method for backward compatibility
    pub fn select_access_method_simple(
        &self,
        table: &str,
        column: &str,
        selectivity_threshold: f64,
    ) -> &str {
        let access = self.select_best_access_method(table, column, PredicateType::Eq);
        match access {
            AccessMethod::IndexScan if selectivity_threshold < 0.05 => "index_scan",
            _ => "seq_scan",
        }
    }
}
