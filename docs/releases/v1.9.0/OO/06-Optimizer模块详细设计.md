# SQLRustGo Optimizer 模块详细设计文档

> **版本**: v1.9.0
> **日期**: 2026-03-26
> **模块**: sqlrustgo-optimizer

---

## 1. 模块概述

Optimizer 模块负责查询优化的核心功能，包括规则优化和代价优化。

### 1.1 模块职责

- 查询重写 (Query Rewriting)
- 规则优化 (Rule-based Optimization)
- 代价估算 (Cost Estimation)
- 物理计划选择 (Physical Plan Selection)

### 1.2 模块结构

```
crates/optimizer/
├── src/
│   ├── lib.rs               # 模块入口
│   ├── rules.rs             # 优化规则
│   ├── cost.rs              # 成本模型
│   ├── stats.rs             # 统计信息
│   ├── plan.rs              # 优化计划
│   ├── projection_pushdown.rs # 列裁剪
│   └── network_cost.rs      # 网络代价
└── Cargo.toml
```

---

## 2. 核心类设计

### 2.1 优化器架构

```uml
@startuml

class Optimizer {
  -rules: Vec<OptimizeRule>
  -cost_model: CostModel
  -stats_provider: StatisticsProvider
  --
  +optimize(plan): LogicalPlan
}

class RuleOptimizer {
  -rules: Vec<OptimizeRule>
  -max_iterations: usize
  --
  +apply_rules(plan): LogicalPlan
}

class CostOptimizer {
  -cost_model: CostModel
  -search_space: SearchSpace
  --
  +find_best_plan(plan): PhysicalPlan
}

class Memo {
  -groups: HashMap<GroupId, Group>
  --
  +insert(plan): GroupId
  +find_best(group): PhysicalPlan
}

class Group {
  -id: GroupId
  -logical_exprs: Vec<LogicalExpr>
  -physical_exprs: Vec<PhysicalExpr>
  -best_cost: Cost
  -best_plan: PhysicalPlan
}

Optimizer --> RuleOptimizer
Optimizer --> CostOptimizer
Optimizer --> Memo
Memo --> Group

@enduml
```

### 2.2 优化规则

```uml
@startuml

abstract class OptimizeRule {
  -name: String
  --
  +apply(plan): Result<bool>
  +match(plan): bool
}

class PredicatePushdownRule {
  -name: String = "PredicatePushdown"
  --
  +apply(plan): Result<bool>
}

class ColumnPruningRule {
  -name: String = "ColumnPruning"
  --
  +apply(plan): Result<bool>
}

class JoinReorderingRule {
  -name: String = "JoinReordering"
  --
  +apply(plan): Result<bool>
}

class ConstantFoldingRule {
  -name: String = "ConstantFolding"
  --
  +apply(plan): Result<bool>
}

class SortEliminationRule {
  -name: String = "SortElimination"
  --
  +apply(plan): Result<bool>
}

class MergeProjectionRule {
  -name: String = "MergeProjection"
  --
  +apply(plan): Result<bool>
}

class FilterScanRule {
  -name: String = "FilterScan"
  --
  +apply(plan): Result<bool>
}

OptimizeRule <|-- PredicatePushdownRule
OptimizeRule <|-- ColumnPruningRule
OptimizeRule <|-- JoinReorderingRule
OptimizeRule <|-- ConstantFoldingRule
OptimizeRule <|-- SortEliminationRule
OptimizeRule <|-- MergeProjectionRule
OptimizeRule <|-- FilterScanRule

@enduml
```

---

## 3. 成本模型设计

### 3.1 成本参数

```rust
pub struct CostModel {
    // I/O 成本
    pub seq_scan_cost: f64,      // 顺序扫描成本
    pub index_scan_cost: f64,     // 索引扫描成本
    pub page_read_cost: f64,     // 页面读取成本
    
    // CPU 成本
    pub row_eval_cost: f64,      // 行评估成本
    pub hash_build_cost: f64,   // Hash 构建成本
    pub sort_cost_per_row: f64, // 排序每行成本
    
    // 内存成本
    pub hash_join_memory: f64,  // Hash Join 内存系数
    pub sort_memory: f64,        // 排序内存系数
    
    // 网络成本 (分布式)
    pub network_byte_cost: f64, // 每字节网络成本
}

impl Default for CostModel {
    fn default() -> Self {
        Self {
            seq_scan_cost: 1.0,
            index_scan_cost: 0.5,
            page_read_cost: 0.8,
            row_eval_cost: 0.1,
            hash_build_cost: 0.5,
            sort_cost_per_row: 0.2,
            hash_join_memory: 0.01,
            sort_memory: 0.01,
            network_byte_cost: 0.0001,
        }
    }
}
```

### 3.2 成本计算

```uml
@startuml

class CostCalculator {
  -cost_model: CostModel
  --
  +calc_scan_cost(table, filter): Cost
  +calc_join_cost(left, right, method): Cost
  +calc_agg_cost(input, groups): Cost
  +calc_sort_cost(input, keys): Cost
}

class TableStats {
  -row_count: u64
  -page_count: u64
  -avg_row_size: f64
  -histogram: Histogram
  -indexes: Vec<IndexInfo>
}

CostCalculator --> TableStats

@enduml
```

---

## 4. 统计信息设计

### 4.1 统计信息结构

```rust
pub struct Statistics {
    pub row_count: u64,
    pub total_size_bytes: u64,
    pub column_stats: HashMap<String, ColumnStats>,
    pub index_stats: Vec<IndexStats>,
}

pub struct ColumnStats {
    pub null_count: u64,
    pub distinct_count: u64,
    pub min_value: Value,
    pub max_value: Value,
    pub avg_value: f64,
    pub histogram: Vec<HistogramBucket>,
}

pub struct HistogramBucket {
    pub lower_bound: Value,
    pub upper_bound: Value,
    pub count: u64,
}
```

---

## 5. 优化规则实现

### 5.1 谓词下推

```rust
impl OptimizeRule for PredicatePushdownRule {
    fn apply(&self, plan: &mut LogicalPlan) -> Result<bool, SqlError> {
        let mut changed = false;
        
        match plan {
            LogicalPlan::Join { left, right, condition, join_type } => {
                // 分离 Join 条件到左右子节点
                let (left_preds, right_preds, join_conds) = 
                    split_condition(condition, left.schema(), right.schema());
                
                if !left_preds.is_empty() {
                    *left = LogicalPlan::Filter {
                        input: std::mem::replace(left, LogicalPlan::Empty),
                        predicate: combine_conjunctions(left_preds),
                    };
                    changed = true;
                }
                
                // ... 同理处理 right
            }
            _ => {}
        }
        
        Ok(changed)
    }
}
```

### 5.2 列裁剪

```rust
impl OptimizeRule for ColumnPruningRule {
    fn apply(&self, plan: &mut LogicalPlan) -> Result<bool, SqlError> {
        let required_cols = collect_required_columns(plan);
        
        match plan {
            LogicalPlan::Projection { input, exprs, .. } => {
                let pruned: Vec<_> = exprs.iter()
                    .filter(|e| required_cols.contains(&e.name()))
                    .cloned()
                    .collect();
                
                if pruned.len() != exprs.len() {
                    *exprs = pruned;
                    return Ok(true);
                }
            }
            // ... 其他节点类型
        }
        
        Ok(false)
    }
}
```

---

## 6. 与代码对应检查

### 6.1 模块文件对应

| 设计内容 | 代码文件 | 状态 |
|----------|----------|------|
| 优化规则 | `rules.rs` | ✅ 对应 |
| 成本模型 | `cost.rs` | ✅ 对应 |
| 统计信息 | `stats.rs` | ✅ 对应 |
| 列裁剪 | `projection_pushdown.rs` | ✅ 对应 |
| 网络代价 | `network_cost.rs` | ✅ 对应 |

### 6.2 功能覆盖检查

| 功能 | 代码实现 | 状态 |
|------|----------|------|
| 谓词下推 | ✅ | ✅ |
| 列裁剪 | ✅ | ✅ |
| Join 重排 | ✅ | ✅ |
| 常量折叠 | ✅ | ✅ |
| 成本估算 | ✅ | ✅ |
| 统计信息 | ✅ | ✅ |

---

## 7. 测试设计

### 7.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_predicate_pushdown() {
        let rule = PredicatePushdownRule::new();
        let plan = LogicalPlan::Filter {
            input: Box::new(LogicalPlan::Join::...),
            predicate: Expression::...,
        };
        
        let result = rule.apply(&mut plan.clone());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_column_pruning() {
        let rule = ColumnPruningRule::new();
        let plan = LogicalPlan::Projection {
            input: Box::new(LogicalPlan::Scan::...),
            exprs: vec![col("a"), col("b"), col("c")],
        };
        
        // 只使用 a, b
        let result = rule.apply(&mut plan.clone());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_cost_calculation() {
        let cost_model = CostModel::default();
        let stats = TableStats::new(1000, 100, 50.0);
        
        let cost = cost_model.calc_scan_cost(&stats, false);
        assert!(cost.value > 0.0);
    }
}
```

---

**文档版本历史**

| 版本 | 日期 | 作者 | 变更 |
|------|------|------|------|
| 1.0 | 2026-03-26 | OpenCode | 初始版本 |

**文档状态**: ✅ 已完成
