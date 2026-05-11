# CBO 代价模型接入 Planner

> **版本**: v3.1.0  
> **对应 Issue**: #616  
> **状态**: 规划中

---

## 1. 当前状态

### 1.1 已有实现

`crates/optimizer/src/cost.rs` 中 `SimpleCostModel` 已实现基本框架：

```rust
pub struct SimpleCostModel {
    cpu_cost_per_row: f64,
    io_cost_per_page: f64,
    network_cost_per_byte: f64,
}

impl CostModel for SimpleCostModel {
    fn estimate_cost(&self, _plan: &dyn std::any::Any) -> f64 {
        100.0  // 硬编码占位符
    }
}
```

**问题**: 
- `estimate_cost` 返回硬编码 100.0，未接入物理计划
- `optimizer.rs` 未实例化 `SimpleCostModel`
- `path_selector.rs` 未使用代价模型进行计划选择

### 1.2 OO 文档参考

`docs/releases/v3.0.0/oo/cbo/CBO_COST_MODEL.md` (43KB) 提供了完整的代价公式：

```
TotalCost = CPUCost + IOCost + MemoryCost + NetworkCost

SeqScanCost = pages_to_read × seq_io_latency + rows × cpu_cost_per_row
IndexScanCost = BTree_height × page_io + matching_rows × page_io_per_row
HashJoinCost = BuildCost + ProbeCost
```

---

## 2. 目标架构

### 2.1 接入架构

```
Physical Plan
     │
     ▼
┌─────────────────────────┐
│  CostModelRouter         │
│  (path_selector.rs)      │
└────────┬────────────────┘
         │ 计算各路径代价
         ▼
┌─────────────────────────┐
│  SimpleCostModel         │
│  (cost.rs)               │
│                          │
│  - seq_scan_cost()       │
│  - index_scan_cost()     │
│  - join_cost()           │
│  - agg_cost()             │
│  - sort_cost()            │
└────────┬────────────────┘
         │ 选择最低代价计划
         ▼
   Optimal Physical Plan
```

### 2.2 关键文件

| 文件 | 当前状态 | 目标 |
|------|---------|------|
| `crates/optimizer/src/cost.rs` | SimpleCostModel 存在 | 补充完整公式 |
| `crates/optimizer/src/path_selector.rs` | 未使用 CostModel | 接入 CostModel |
| `crates/optimizer/src/optimizer.rs` | NoOpOptimizer | 使用 CostModelRouter |
| `crates/optimizer/src/stats.rs` | TableStats 存在 | 接入统计信息 |

---

## 3. 实现计划

### 3.1 Phase 1: 代价常量配置化

将 `CBO_COST_MODEL.md` 的常量 Rust 化：

```rust
/// CostConstants - 代价模型常数（对应 OO 文档 §2.1）
#[derive(Debug, Clone)]
pub struct CostConstants {
    // CPU 代价
    pub cpu_cost_per_row: f64,           // = 1.0
    pub cpu_cost_index_search: f64,      // = 0.1
    pub cpu_cost_per_page: f64,          // = 0.01
    
    // I/O 代价
    pub seq_io_latency_ms: f64,          // = 0.1  (磁盘顺序读)
    pub random_io_latency_ms: f64,       // = 1.0  (磁盘随机读)
    
    // 内存代价
    pub memory_latency_ns: f64,          // = 100.0
    pub sort_buffer_pages: u64,          // = 1024
    
    // Hash Join
    pub hash_probe_cost_factor: f64,     // = 1.2
    
    // 网络代价
    pub network_latency_ms: f64,         // = 0.5
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
            sort_buffer_pages: 1024,
            hash_probe_cost_factor: 1.2,
            network_latency_ms: 0.5,
        }
    }
}
```

### 3.2 Phase 2: 扫描代价实现

对应 OO 文档 §3：

```rust
impl SimpleCostModel {
    /// 全表扫描代价（对应 OO 文档 §3.1）
    pub fn seq_scan_cost(&self, table_stats: &TableStats, consts: &CostConstants) -> Cost {
        let pages_to_read = table_stats.total_pages;
        let rows_to_process = table_stats.row_count;
        
        // I/O 代价: pages_to_read × seq_io_latency
        let io_cost = pages_to_read as f64 * consts.seq_io_latency_ms;
        
        // CPU 代价: rows × cpu_cost_per_row
        let cpu_cost = rows_to_process as f64 * consts.cpu_cost_per_row;
        
        Cost { io_cost, cpu_cost, memory_cost: 0.0, network_cost: 0.0 }
    }
    
    /// 索引扫描代价（对应 OO 文档 §3.2-3.3）
    pub fn index_scan_cost(
        &self,
        index: &IndexDef,
        predicate: &Predicate,
        table_stats: &TableStats,
        consts: &CostConstants,
    ) -> Cost {
        let selectivity = compute_selectivity(predicate, index, table_stats);
        let matching_rows = (table_stats.row_count as f64 * selectivity).ceil() as u64;
        
        // 索引搜索代价: BTree_height × cpu_cost_index_search
        let index_height = index.bptree.height();
        let index_search_cpu = index_height as f64 * consts.cpu_cost_index_search;
        
        // 索引 I/O 代价
        let index_io = match predicate.range_type() {
            RangeType::Point => 1.0 * consts.random_io_latency_ms,
            RangeType::Range(r) => {
                let leaf_pages = estimate_leaf_pages(index, r);
                (index_height as f64 * consts.random_io_latency_ms) 
                    + (leaf_pages as f64 * consts.seq_io_latency_ms)
            }
            RangeType::Full => index.total_pages() as f64 * consts.seq_io_latency_ms,
        };
        
        // 数据 I/O 代价
        let data_pages = estimate_data_pages(matching_rows, table_stats.row_width);
        let data_io = data_pages as f64 * consts.random_io_latency_ms;
        
        // CPU 代价
        let cpu_cost = matching_rows as f64 * consts.cpu_cost_per_row;
        
        Cost { io_cost: index_io + data_io, cpu_cost, memory_cost: 0.0, network_cost: 0.0 }
    }
}
```

### 3.3 Phase 3: Join 代价实现

对应 OO 文档 §4：

```rust
    /// Hash Join 代价（对应 OO 文档 §4.1）
    pub fn hash_join_cost(
        &self,
        build_rows: u64,
        probe_rows: u64,
        consts: &CostConstants,
    ) -> Cost {
        // Build 阶段: build_rows × cpu_cost_per_row
        let build_cpu = build_rows as f64 * consts.cpu_cost_per_row;
        
        // Probe 阶段: probe_rows × cpu_cost_per_row × hash_probe_cost_factor
        let probe_cpu = probe_rows as f64 * consts.cpu_cost_per_row * consts.hash_probe_cost_factor;
        
        // Hash table 内存代价
        let hash_table_pages = build_rows * std::mem::size_of::<u64>() as u64 / consts.page_size_bytes;
        let memory_cost = hash_table_pages as f64 * consts.memory_latency_ns / 1_000_000.0;
        
        Cost { io_cost: 0.0, cpu_cost: build_cpu + probe_cpu, memory_cost, network_cost: 0.0 }
    }
    
    /// Sort-Merge Join 代价（对应 OO 文档 §4.3）
    pub fn sort_merge_join_cost(
        &self,
        left_rows: u64,
        right_rows: u64,
        left_sorted: bool,
        right_sorted: bool,
        consts: &CostConstants,
    ) -> Cost {
        // 排序代价（如果未排序）
        let left_sort_cost = if !left_sorted {
            left_rows as f64 * consts.cpu_cost_per_row * (left_rows as f64).log2()
        } else { 0.0 };
        let right_sort_cost = if !right_sorted {
            right_rows as f64 * consts.cpu_cost_per_row * (right_rows as f64).log2()
        } else { 0.0 };
        
        // Merge 阶段: (left + right) × comparison_cost
        let merge_cost = (left_rows + right_rows) as f64 * consts.cpu_cost_per_row;
        
        Cost { 
            io_cost: 0.0, 
            cpu_cost: left_sort_cost + right_sort_cost + merge_cost, 
            memory_cost: 0.0, 
            network_cost: 0.0 
        }
    }
```

### 3.4 Phase 4: 集成到 path_selector

```rust
/// PathSelector - 使用 CostModel 选择最优计划
pub struct PathSelector {
    cost_model: SimpleCostModel,
    consts: CostConstants,
}

impl PathSelector {
    /// 为给定的 LogicalPlan 选择最优 PhysicalPlan
    pub fn select_physical_plan(&self, logical: &LogicalPlan) -> PhysicalPlan {
        let candidates = self.generate_candidates(logical);
        
        candidates
            .into_iter()
            .map(|p| {
                let cost = self.cost_of_physical_plan(&p);
                (p, cost)
            })
            .min_by(|a, b| a.1.total().partial_cmp(&b.1.total()).unwrap())
            .map(|(p, _)| p)
            .unwrap_or_else(|| self.default_plan(logical))
    }
    
    fn cost_of_physical_plan(&self, plan: &PhysicalPlan) -> Cost {
        match plan {
            PhysicalPlan::SeqScan(t) => self.cost_model.seq_scan_cost(&t.stats, &self.consts),
            PhysicalPlan::IndexScan(idx) => self.cost_model.index_scan_cost(...),
            PhysicalPlan::HashJoin(h) => self.cost_model.hash_join_cost(...),
            PhysicalPlan::SortMergeJoin(s) => self.cost_model.sort_merge_join_cost(...),
            _ => Cost::zero(),
        }
    }
}
```

---

## 4. 测试计划

### 4.1 单元测试

| 测试 | 描述 |
|------|------|
| `test_seq_scan_cost_calculation` | 验证全表扫描代价公式 |
| `test_index_scan_cost_point` | 验证点查代价 |
| `test_index_scan_cost_range` | 验证范围查代价 |
| `test_hash_join_cost` | 验证 Hash Join 代价 |
| `test_sort_merge_cost` | 验证 Sort-Merge 代价 |
| `test_cost_constants_default` | 验证默认常数 |

### 4.2 集成测试

| 测试 | 描述 |
|------|------|
| `test_cbo_selects_index_over_seqscan` | 选择性高时选索引扫描 |
| `test_cbo_selects_hash_join_over_nl` | 大表 JOIN 时选 Hash Join |
| `test_cbo_respects_memory_limit` | 内存不足时选不同计划 |

---

## 5. 验收条件

- [ ] `SimpleCostModel::estimate_cost` 不再返回硬编码值
- [ ] 全表扫描代价公式与 OO 文档 §3.1 一致
- [ ] 索引扫描代价公式与 OO 文档 §3.2-3.3 一致
- [ ] Hash Join 代价公式与 OO 文档 §4.1 一致
- [ ] `path_selector.rs` 使用 `CostModel` 选择计划
- [ ] TPC-H Q1 查询 CBO 选择索引扫描（如果存在）
- [ ] `cargo clippy --all-features -- -D warnings` 通过
