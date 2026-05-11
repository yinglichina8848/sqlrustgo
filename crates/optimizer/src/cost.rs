//! Cost Model Module
//!
//! Implements the cost model from `docs/releases/v3.0.0/oo/cbo/CBO_COST_MODEL.md`
//! TotalCost = CPUCost + IOCost + MemoryCost + NetworkCost

use super::*;

// ============================================================================
// Cost Structures
// ============================================================================

/// Cost breakdown structure (对应 OO 文档 §1 代价要素分解)
#[derive(Debug, Clone, Copy, Default)]
pub struct Cost {
    /// I/O cost (ms)
    pub io_cost: f64,
    /// CPU cost (relative units)
    pub cpu_cost: f64,
    /// Memory cost (ms equivalent)
    pub memory_cost: f64,
    /// Network cost (ms)
    pub network_cost: f64,
}

impl Cost {
    /// Total cost = CPU + I/O + Memory + Network
    pub fn total(&self) -> f64 {
        self.io_cost + self.cpu_cost + self.memory_cost + self.network_cost
    }

    /// Create zero cost
    pub fn zero() -> Self {
        Self {
            io_cost: 0.0,
            cpu_cost: 0.0,
            memory_cost: 0.0,
            network_cost: 0.0,
        }
    }

    /// Add two costs
    pub fn add(&self, other: &Cost) -> Cost {
        Cost {
            io_cost: self.io_cost + other.io_cost,
            cpu_cost: self.cpu_cost + other.cpu_cost,
            memory_cost: self.memory_cost + other.memory_cost,
            network_cost: self.network_cost + other.network_cost,
        }
    }

    /// Compare costs by total
    pub fn less_than(&self, other: &Cost) -> bool {
        self.total() < other.total()
    }
}

impl std::ops::Add for Cost {
    type Output = Cost;
    fn add(self, other: Cost) -> Cost {
        Cost {
            io_cost: self.io_cost + other.io_cost,
            cpu_cost: self.cpu_cost + other.cpu_cost,
            memory_cost: self.memory_cost + other.memory_cost,
            network_cost: self.network_cost + other.network_cost,
        }
    }
}

impl std::fmt::Display for Cost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cost(total={:.2}, cpu={:.2}, io={:.2}, mem={:.2}, net={:.2})",
            self.total(),
            self.cpu_cost,
            self.io_cost,
            self.memory_cost,
            self.network_cost
        )
    }
}

// ============================================================================
// Cost Constants (对应 OO 文档 §2.1 系统级常数)
// ============================================================================

/// Cost constants - matches `CBO_COST_MODEL.md §2.1`
#[derive(Debug, Clone)]
pub struct CostConstants {
    /// CPU cost per row (relative units), default = 1.0
    pub cpu_cost_per_row: f64,
    /// CPU cost per index search, default = 0.1
    pub cpu_cost_index_search: f64,
    /// CPU cost per page, default = 0.01
    pub cpu_cost_per_page: f64,
    /// Disk sequential I/O latency (ms), default = 0.1
    pub seq_io_latency_ms: f64,
    /// Disk random I/O latency (ms), default = 1.0
    pub random_io_latency_ms: f64,
    /// Memory access latency (ns), default = 100.0
    pub memory_latency_ns: f64,
    /// Page size (bytes), default = 16384 (16KB)
    pub page_size_bytes: u64,
    /// Sort buffer size (pages), default = 1024
    pub sort_buffer_pages: u64,
    /// Hash join probe cost factor, default = 1.2
    pub hash_probe_cost_factor: f64,
    /// Network latency (ms), default = 0.5
    pub network_latency_ms: f64,
    /// Page size as f64 for calculations
    pub page_size_f64: f64,
}

impl Default for CostConstants {
    fn default() -> Self {
        Self {
            cpu_cost_per_row: 1.0,
            cpu_cost_index_search: 0.1,
            cpu_cost_per_page: 0.01,
            seq_io_latency_ms: 0.1,
            random_io_latency_ms: 1.0,
            memory_latency_ns: 100.0,
            page_size_bytes: 16384,
            sort_buffer_pages: 1024,
            hash_probe_cost_factor: 1.2,
            network_latency_ms: 0.5,
            page_size_f64: 16384.0,
        }
    }
}

impl CostConstants {
    /// Create with custom I/O latencies (for benchmarking different storage)
    pub fn with_io_latency(seq_ms: f64, random_ms: f64) -> Self {
        Self {
            seq_io_latency_ms: seq_ms,
            random_io_latency_ms: random_ms,
            ..Self::default()
        }
    }
}

// ============================================================================
// Cost Model Implementation
// ============================================================================

/// SimpleCostModel - basic cost estimation implementation
///
/// Implements cost formulas from `CBO_COST_MODEL.md`:
/// - SeqScan: pages × seq_io + rows × cpu_per_row
/// - IndexScan: BTree_height × index_io + matching_rows × data_io
/// - HashJoin: build_cpu + probe_cpu × hash_probe_factor
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct SimpleCostModel {
    /// Cost constants
    consts: CostConstants,
}

impl SimpleCostModel {
    /// Create with default constants
    pub fn new() -> Self {
        Self {
            consts: CostConstants::default(),
        }
    }

    /// Create with custom constants
    pub fn with_constants(consts: CostConstants) -> Self {
        Self { consts }
    }

    /// Default cost model with standard values (backward compatible)
    pub fn default_model() -> Self {
        Self::new()
    }

    // -------------------------------------------------------------------------
    // Scan Costs (对应 OO 文档 §3)
    // -------------------------------------------------------------------------

    /// Sequential scan cost (对应 OO 文档 §3.1)
    ///
    /// Formula: pages × seq_io_latency + rows × cpu_cost_per_row
    pub fn seq_scan_cost(&self, row_count: u64, page_count: u64) -> Cost {
        let io_cost = page_count as f64 * self.consts.seq_io_latency_ms;
        let cpu_cost = row_count as f64 * self.consts.cpu_cost_per_row;
        Cost {
            io_cost,
            cpu_cost,
            memory_cost: 0.0,
            network_cost: 0.0,
        }
    }

    /// Index scan cost (对应 OO 文档 §3.2-3.3)
    ///
    /// Formula: BTree_height × index_io + matching_rows × page_io
    pub fn index_scan_cost(
        &self,
        btree_height: u32,
        index_pages: u64,
        matching_rows: u64,
        data_pages: u64,
    ) -> Cost {
        // Index search CPU: BTree_height × cpu_cost_index_search
        let index_search_cpu = btree_height as f64 * self.consts.cpu_cost_index_search;

        // Index I/O: index_pages × random_io (B+Tree pages are randomly accessed)
        let index_io = index_pages as f64 * self.consts.random_io_latency_ms;

        // Data I/O: data_pages × seq_io (data pages are sequentially read)
        let data_io = data_pages as f64 * self.consts.seq_io_latency_ms;

        // CPU for row processing
        let cpu_cost = matching_rows as f64 * self.consts.cpu_cost_per_row + index_search_cpu;

        Cost {
            io_cost: index_io + data_io,
            cpu_cost,
            memory_cost: 0.0,
            network_cost: 0.0,
        }
    }

    /// Point query index scan (single key lookup)
    pub fn index_point_cost(&self, btree_height: u32, data_pages: u64) -> Cost {
        // Point query: 1 index page access + data page access
        let index_io = self.consts.random_io_latency_ms;
        let data_io = data_pages as f64 * self.consts.seq_io_latency_ms;
        let cpu_cost = btree_height as f64 * self.consts.cpu_cost_index_search;

        Cost {
            io_cost: index_io + data_io,
            cpu_cost,
            memory_cost: 0.0,
            network_cost: 0.0,
        }
    }

    /// Range query index scan
    pub fn index_range_cost(
        &self,
        btree_height: u32,
        index_pages: u64,
        matching_rows: u64,
        data_pages: u64,
    ) -> Cost {
        // Range: height × random_io + leaf_pages × seq_io + data × seq_io
        let index_io = btree_height as f64 * self.consts.random_io_latency_ms
            + index_pages as f64 * self.consts.seq_io_latency_ms;
        let data_io = data_pages as f64 * self.consts.seq_io_latency_ms;
        let cpu_cost = matching_rows as f64 * self.consts.cpu_cost_per_row;

        Cost {
            io_cost: index_io + data_io,
            cpu_cost,
            memory_cost: 0.0,
            network_cost: 0.0,
        }
    }

    // -------------------------------------------------------------------------
    // Join Costs (对应 OO 文档 §4)
    // -------------------------------------------------------------------------

    /// Hash join cost (对应 OO 文档 §4.1)
    ///
    /// Formula: BuildCost + ProbeCost
    /// BuildCost = build_rows × cpu_cost_per_row
    /// ProbeCost = probe_rows × cpu_cost_per_row × hash_probe_factor
    pub fn hash_join_cost(&self, build_rows: u64, probe_rows: u64) -> Cost {
        let build_cpu = build_rows as f64 * self.consts.cpu_cost_per_row;
        let probe_cpu =
            probe_rows as f64 * self.consts.cpu_cost_per_row * self.consts.hash_probe_cost_factor;

        // Memory cost for hash table
        let hash_table_bytes = build_rows * 8; // 8 bytes per entry (u64)
        let hash_table_pages = (hash_table_bytes as f64 / self.consts.page_size_f64).ceil();
        let memory_cost = hash_table_pages * self.consts.memory_latency_ns / 1_000_000.0;

        Cost {
            io_cost: 0.0,
            cpu_cost: build_cpu + probe_cpu,
            memory_cost,
            network_cost: 0.0,
        }
    }

    /// Nested loop join cost (对应 OO 文档 §4.2)
    ///
    /// NLJ: outer_rows × inner_rows × inner_probe_cost
    /// BNL: outer_blocks × inner_blocks × block_io
    pub fn nested_loop_join_cost(&self, outer_rows: u64, inner_rows: u64) -> Cost {
        let cpu_cost = outer_rows as f64 * inner_rows as f64 * self.consts.cpu_cost_per_row;
        Cost {
            io_cost: 0.0,
            cpu_cost,
            memory_cost: 0.0,
            network_cost: 0.0,
        }
    }

    /// Block nested loop join cost
    pub fn block_nested_loop_cost(&self, outer_blocks: u64, inner_blocks: u64) -> Cost {
        let io_cost = outer_blocks as f64 * inner_blocks as f64 * self.consts.seq_io_latency_ms;
        let cpu_cost = outer_blocks as f64 * inner_blocks as f64 * self.consts.cpu_cost_per_row;
        Cost {
            io_cost,
            cpu_cost,
            memory_cost: 0.0,
            network_cost: 0.0,
        }
    }

    /// Sort-merge join cost (对应 OO 文档 §4.3)
    ///
    /// SortCost = rows × log2(rows) × cpu_cost_per_row
    /// MergeCost = (left + right) × cpu_cost_per_row
    pub fn sort_merge_join_cost(
        &self,
        left_rows: u64,
        right_rows: u64,
        left_sorted: bool,
        right_sorted: bool,
    ) -> Cost {
        let left_sort_cpu = if !left_sorted && left_rows > 0 {
            left_rows as f64 * (left_rows as f64).log2() * self.consts.cpu_cost_per_row
        } else {
            0.0
        };

        let right_sort_cpu = if !right_sorted && right_rows > 0 {
            right_rows as f64 * (right_rows as f64).log2() * self.consts.cpu_cost_per_row
        } else {
            0.0
        };

        let merge_cpu = (left_rows + right_rows) as f64 * self.consts.cpu_cost_per_row;

        Cost {
            io_cost: 0.0,
            cpu_cost: left_sort_cpu + right_sort_cpu + merge_cpu,
            memory_cost: 0.0,
            network_cost: 0.0,
        }
    }

    // -------------------------------------------------------------------------
    // Aggregation Costs (对应 OO 文档 §5)
    // -------------------------------------------------------------------------

    /// Hash aggregation cost
    ///
    /// BuildPhase: rows × hash_function_cost
    /// Memory: num_groups × group_size
    pub fn hash_agg_cost(&self, row_count: u64, num_groups: u64) -> Cost {
        let hash_cpu = row_count as f64 * self.consts.cpu_cost_per_row;
        // Group comparison cost
        let group_cpu =
            num_groups as f64 * (num_groups as f64).log2() * self.consts.cpu_cost_per_row;

        // Memory cost for hash table
        let hash_table_bytes = num_groups * 64; // estimate 64 bytes per group
        let hash_table_pages = (hash_table_bytes as f64 / self.consts.page_size_f64).ceil();
        let memory_cost = hash_table_pages * self.consts.memory_latency_ns / 1_000_000.0;

        Cost {
            io_cost: 0.0,
            cpu_cost: hash_cpu + group_cpu,
            memory_cost,
            network_cost: 0.0,
        }
    }

    /// Sort-based aggregation cost
    pub fn sort_agg_cost(&self, row_count: u64) -> Cost {
        let sort_cpu = row_count as f64 * (row_count as f64).log2() * self.consts.cpu_cost_per_row;
        Cost {
            io_cost: 0.0,
            cpu_cost: sort_cpu,
            memory_cost: 0.0,
            network_cost: 0.0,
        }
    }

    // -------------------------------------------------------------------------
    // Sort Costs (对应 OO 文档 §5)
    // -------------------------------------------------------------------------

    /// Sort cost (detail) - returns full Cost breakdown
    ///
    /// In-memory: rows × log2(rows) × cpu_cost
    /// External: pages × io_cost × 2 (read + write)
    pub fn sort_cost_detail(&self, row_count: u64, avg_row_size: u32) -> Cost {
        let memory_sort_threshold =
            self.consts.sort_buffer_pages * self.consts.page_size_bytes;
        let required_bytes = row_count * avg_row_size as u64;

        if required_bytes <= memory_sort_threshold {
            // In-memory sort
            let cpu_cost = if row_count > 0 {
                row_count as f64 * (row_count as f64).log2() * self.consts.cpu_cost_per_row
            } else {
                0.0
            };
            Cost {
                io_cost: 0.0,
                cpu_cost,
                memory_cost: 0.0,
                network_cost: 0.0,
            }
        } else {
            // External sort (I/O bound)
            let pages = (required_bytes as f64 / self.consts.page_size_f64).ceil();
            let io_cost = pages * self.consts.seq_io_latency_ms * 2.0; // read + write
            let cpu_cost =
                row_count as f64 * (row_count as f64).log2() * self.consts.cpu_cost_per_row;
            Cost {
                io_cost,
                cpu_cost,
                memory_cost: 0.0,
                network_cost: 0.0,
            }
        }
    }

    // -------------------------------------------------------------------------
    // Legacy f64 API (backward compatibility with unified_cost.rs)
    // -------------------------------------------------------------------------

    /// Sequential scan cost (returns f64, legacy API)
    pub fn seq_scan_cost_f64(&self, row_count: u64, page_count: u64) -> f64 {
        self.seq_scan_cost(row_count, page_count).total()
    }

    /// Index scan cost (returns f64, legacy API)
    pub fn index_scan_cost_f64(
        &self,
        btree_height: u32,
        index_pages: u64,
        matching_rows: u64,
        data_pages: u64,
    ) -> f64 {
        self.index_scan_cost(btree_height, index_pages, matching_rows, data_pages)
            .total()
    }

    /// Aggregation cost as f64 (legacy API for unified_cost.rs compatibility)
    pub fn agg_cost(&self, row_count: u64, _group_by_cols: u32) -> f64 {
        let num_groups = row_count / 100;
        self.hash_agg_cost(row_count, num_groups.max(1)).total()
    }

    /// Join cost as f64 (legacy API)
    pub fn join_cost(&self, left_rows: u64, right_rows: u64, join_method: &str) -> f64 {
        match join_method {
            "nested_loop" => self.nested_loop_join_cost(left_rows, right_rows).total(),
            "hash_join" => self.hash_join_cost(left_rows, right_rows).total(),
            "sort_merge" => self
                .sort_merge_join_cost(left_rows, right_rows, false, false)
                .total(),
            _ => self.nested_loop_join_cost(left_rows, right_rows).total(),
        }
    }

    /// Sort cost as f64 (legacy API - calls sort_cost_detail internally)
    pub fn sort_cost(&self, row_count: u64, avg_row_size: u32) -> f64 {
        self.sort_cost_detail(row_count, avg_row_size).total()
    }
}

impl CostModel for SimpleCostModel {
    fn estimate_cost(&self, _plan: &dyn std::any::Any) -> f64 {
        // Fallback: use default scan cost
        self.seq_scan_cost(1000, 10).total()
    }
}

// ============================================================================
// CboOptimizer - Statistics-aware optimizer
// ============================================================================

/// CboOptimizer - Cost-Based Optimizer with statistics support
///
/// Integrates SimpleCostModel with statistics provider for accurate estimates.
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
            cost_model: SimpleCostModel::new(),
            default_row_count: 1000,
            default_page_count: 10,
        }
    }

    /// Create with custom defaults
    pub fn with_defaults(row_count: u64, page_count: u64) -> Self {
        Self {
            cost_model: SimpleCostModel::new(),
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
    pub fn estimate_scan_cost(&self, row_count: u64, page_count: u64) -> Cost {
        self.cost_model.seq_scan_cost(row_count, page_count)
    }

    /// Estimate cost for an index scan
    pub fn estimate_index_scan_cost(
        &self,
        btree_height: u32,
        index_pages: u64,
        matching_rows: u64,
        data_pages: u64,
    ) -> Cost {
        self.cost_model
            .index_scan_cost(btree_height, index_pages, matching_rows, data_pages)
    }

    /// Estimate cost for a join operation
    pub fn estimate_join_cost(&self, left_rows: u64, right_rows: u64, method: &str) -> Cost {
        match method {
            "hash_join" => self.cost_model.hash_join_cost(left_rows, right_rows),
            "nested_loop" => self.cost_model.nested_loop_join_cost(left_rows, right_rows),
            "sort_merge" => self
                .cost_model
                .sort_merge_join_cost(left_rows, right_rows, false, false),
            _ => self.cost_model.nested_loop_join_cost(left_rows, right_rows),
        }
    }
}

impl Default for CboOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl CostModel for CboOptimizer {
    fn estimate_cost(&self, _plan: &dyn std::any::Any) -> f64 {
        self.cost_model
            .seq_scan_cost(self.default_row_count, self.default_page_count)
            .total()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_total() {
        let c = Cost {
            io_cost: 10.0,
            cpu_cost: 5.0,
            memory_cost: 2.0,
            network_cost: 1.0,
        };
        assert_eq!(c.total(), 18.0);
    }

    #[test]
    fn test_cost_add() {
        let a = Cost {
            io_cost: 10.0,
            cpu_cost: 5.0,
            memory_cost: 0.0,
            network_cost: 0.0,
        };
        let b = Cost {
            io_cost: 2.0,
            cpu_cost: 3.0,
            memory_cost: 0.0,
            network_cost: 0.0,
        };
        let sum = a.add(&b);
        assert_eq!(sum.io_cost, 12.0);
        assert_eq!(sum.cpu_cost, 8.0);
    }

    #[test]
    fn test_cost_display() {
        let c = Cost {
            io_cost: 10.0,
            cpu_cost: 5.0,
            memory_cost: 2.0,
            network_cost: 1.0,
        };
        let s = format!("{}", c);
        assert!(s.contains("18.00") || s.contains("18.0"));
    }

    #[test]
    fn test_cost_constants_default() {
        let c = CostConstants::default();
        assert_eq!(c.cpu_cost_per_row, 1.0);
        assert_eq!(c.seq_io_latency_ms, 0.1);
        assert_eq!(c.random_io_latency_ms, 1.0);
    }

    #[test]
    fn test_simple_cost_model_default() {
        let model = SimpleCostModel::default_model();
        assert_eq!(model.consts.cpu_cost_per_row, 1.0);
    }

    #[test]
    fn test_seq_scan_cost() {
        let model = SimpleCostModel::new();
        // 100 rows, 10 pages
        // cost = 10 * 0.1 + 100 * 1.0 = 1.0 + 100.0 = 101.0
        let cost = model.seq_scan_cost(100, 10);
        assert!((cost.total() - 101.0).abs() < 0.001);
        assert_eq!(cost.io_cost, 1.0);
        assert_eq!(cost.cpu_cost, 100.0);
    }

    #[test]
    fn test_index_scan_cost() {
        let model = SimpleCostModel::new();
        // height=3, index_pages=5, matching_rows=50, data_pages=10
        // index_search_cpu = 3 * 0.1 = 0.3
        // index_io = 5 * 1.0 = 5.0
        // data_io = 10 * 0.1 = 1.0
        // cpu = 50 * 1.0 = 50.0
        // total = 0.3 + 5.0 + 1.0 + 50.0 = 56.3
        let cost = model.index_scan_cost(3, 5, 50, 10);
        assert!((cost.total() - 56.3).abs() < 0.001);
    }

    #[test]
    fn test_hash_join_cost() {
        let model = SimpleCostModel::new();
        // build=1000, probe=2000
        // build_cpu = 1000 * 1.0 = 1000
        // probe_cpu = 2000 * 1.0 * 1.2 = 2400
        let cost = model.hash_join_cost(1000, 2000);
        assert!(cost.cpu_cost > 3300.0); // 1000 + 2400
    }

    #[test]
    fn test_nested_loop_cost() {
        let model = SimpleCostModel::new();
        let cost = model.nested_loop_join_cost(100, 200);
        // 100 * 200 * 1.0 = 20000
        assert_eq!(cost.cpu_cost, 20000.0);
    }

    #[test]
    fn test_sort_merge_cost_unsorted() {
        let model = SimpleCostModel::new();
        // left=100, right=200, both unsorted
        // left_sort = 100 * log2(100) * 1.0 = 100 * 6.64 = 664
        // right_sort = 200 * log2(200) * 1.0 = 200 * 7.64 = 1528
        // merge = 300 * 1.0 = 300
        let cost = model.sort_merge_join_cost(100, 200, false, false);
        assert!(cost.cpu_cost > 2000.0); // 664 + 1528 + 300
    }

    #[test]
    fn test_sort_merge_cost_sorted() {
        let model = SimpleCostModel::new();
        // Both sorted = no sort cost
        let cost = model.sort_merge_join_cost(100, 200, true, true);
        assert_eq!(cost.cpu_cost, 300.0); // just merge
    }

    #[test]
    fn test_sort_cost_in_memory() {
        let model = SimpleCostModel::new();
        // 1000 rows, 100 bytes each = 100KB < sort_buffer (16MB)
        let cost = model.sort_cost_detail(1000, 100);
        assert!(cost.io_cost == 0.0); // in-memory
        assert!(cost.cpu_cost > 0.0);
    }

    #[test]
    fn test_sort_cost_external() {
        let model = SimpleCostModel::new();
        // 2M rows, 100 bytes each = 200MB > 16MB buffer
        let cost = model.sort_cost_detail(2_000_000, 100);
        assert!(cost.io_cost > 0.0); // external sort
    }

    #[test]
    fn test_hash_agg_cost() {
        let model = SimpleCostModel::new();
        let cost = model.hash_agg_cost(10000, 100);
        assert!(cost.cpu_cost > 0.0);
    }

    #[test]
    fn test_cost_model_estimate_cost() {
        let model = SimpleCostModel::new();
        let cost = model.estimate_cost(&());
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cbo_optimizer_new() {
        let cbo = CboOptimizer::new();
        assert_eq!(cbo.default_row_count(), 1000);
        assert_eq!(cbo.default_page_count(), 10);
    }

    #[test]
    fn test_cbo_estimate_scan_cost() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_scan_cost(1000, 10);
        assert!(cost.total() > 0.0);
    }

    #[test]
    fn test_cbo_cost_model_trait() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_cost(&());
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cost_less_than() {
        let cheap = Cost {
            io_cost: 1.0,
            cpu_cost: 1.0,
            memory_cost: 0.0,
            network_cost: 0.0,
        };
        let expensive = Cost {
            io_cost: 10.0,
            cpu_cost: 10.0,
            memory_cost: 0.0,
            network_cost: 0.0,
        };
        assert!(cheap.less_than(&expensive));
        assert!(!expensive.less_than(&cheap));
    }

    #[test]
    fn test_cost_constants_with_io_latency() {
        let c = CostConstants::with_io_latency(0.05, 0.5);
        assert_eq!(c.seq_io_latency_ms, 0.05);
        assert_eq!(c.random_io_latency_ms, 0.5);
    }

    #[test]
    fn test_legacy_f64_apis() {
        let model = SimpleCostModel::new();
        // Legacy f64 APIs used by unified_cost.rs
        assert!(model.agg_cost(1000, 3) > 0.0);
        assert!(model.join_cost(100, 200, "hash_join") > 0.0);
        assert!(model.sort_cost(1000, 100) > 0.0);
        assert!(model.seq_scan_cost_f64(100, 10) > 0.0);
    }
}
