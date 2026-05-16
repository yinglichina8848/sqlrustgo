# OO-11: CBO Integration 设计文档

> **版本**: v1.0
> **日期**: 2026-05-16
> **基于**: v3.2.0
> **维护人**: hermes-z6g4
> **状态**: 已完成

---

## 一、概述

### 1.1 目标

实现基于成本的优化器 (CBO) 集成，优化查询计划选择：

- **成本模型**: 基于统计信息的成本估算
- **计划比较**: 多计划候选评估
- **自适应优化**: 运行时统计信息收集

### 1.2 核心理念

```
CBO = 统计信息 + 成本模型 + 计划枚举 + 计划选择
```

---

## 二、技术架构

### 2.1 组件关系

```
┌─────────────────────────────────────────────────────────────────┐
│                    CBO Integration System                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │  Statistics  │───▶│  Cost Model │───▶│  Plan Enumerator  │  │
│  │  Collector   │    │              │    │                  │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│         │                   │                      │            │
│         │                   │                      │            │
│         ▼                   ▼                      ▼            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │   Column     │    │    CPU +    │    │   Best Plan     │  │
│  │   Stats      │    │    IO Cost  │    │   Selection     │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、统计信息

### 3.1 统计数据类型

| 统计类型 | 说明 |
|----------|------|
| Row Count | 表行数 |
| Column NDV | 列基数 (不同值数量) |
| Histogram | 值分布直方图 |
| Null Count | NULL 值数量 |

### 3.2 统计收集

```rust
/// 统计信息收集器
pub struct StatisticsCollector {
    /// 表统计
    pub table_stats: HashMap<TableId, TableStats>,
    /// 列统计
    pub column_stats: HashMap<ColumnId, ColumnStats>,
}

/// ANALYZE TABLE 命令
pub fn analyze_table(conn: &Connection, table: &str) -> Result<()> {
    // 收集统计信息
    let stats = collect_statistics(conn, table)?;
    // 存储到系统表
    store_statistics(stats)?;
}
```

---

## 四、成本模型

### 4.1 成本因素

| 成本因素 | 说明 | 权重 |
|----------|------|------|
| CPU Cost | 计算 CPU 周期 | 1.0 |
| IO Cost | 磁盘 I/O 操作 | 10.0 |
| Memory Cost | 内存使用 | 0.5 |
| Network Cost | 网络传输 | 100.0 |

### 4.2 成本估算公式

```
TotalCost = CPUCost * 1.0 + IOCost * 10.0 + MemoryCost * 0.5

CPUCost = Rows * CPUPerRow
IOCost = PagesRead * IOPS * SeekTime
```

---

## 五、计划枚举

### 5.1 物理计划候选

| 计划类型 | 适用场景 |
|----------|----------|
| Seq Scan | 小表、无索引 |
| Index Scan | 范围查询、点查 |
| Hash Join | 等值连接、大表 |
| Nested Loop | 小表驱动、大表被驱动 |
| Sort Merge | 排序后连接 |

### 5.2 计划选择算法

```rust
/// 贪心选择算法
pub fn greedy_select(candidates: Vec<Plan>) -> Plan {
    candidates
        .into_iter()
        .min_by_key(|p| p.estimated_cost())
        .unwrap()
}

/// 动态规划算法 (最优子结构)
pub fn dp_select(levels: Vec<HashSet<Plan>>) -> Plan {
    let mut dp: HashMap<LogicalProperty, Plan> = HashMap::new();
    for level in levels {
        for plan in level {
            let cost = plan.estimated_cost() + dp.get(&plan.input()).unwrap_or(&Cost::ZERO);
            dp.insert(plan.output(), plan);
        }
    }
    dp.into_values().min_by_key(|p| p.total_cost()).unwrap()
}
```

---

## 六、集成点

### 6.1 优化器集成

```rust
/// CBO 优化器
pub struct CBOOptimizer {
    statistics: StatisticsManager,
    cost_model: CostModel,
    plan_enumerator: PlanEnumerator,
}

impl Optimizer for CBOOptimizer {
    fn optimize(&self, logical: LogicalPlan) -> Result<PhysicalPlan> {
        // 1. 收集统计信息
        let stats = self.statistics.get_stats(&logical)?;
        // 2. 枚举候选计划
        let candidates = self.plan_enumerator.enumerate(&logical);
        // 3. 评估成本
        let best = candidates
            .iter()
            .min_by_key(|p| self.cost_model.estimate(p, &stats))
            .unwrap();
        Ok(best.clone())
    }
}
```

---

## 七、相关 Issue

| Issue | 功能 | 状态 |
|-------|------|------|
| #1051 | 统计信息收集 | ✅ 完成 |
| #1052 | 成本模型实现 | ✅ 完成 |
| #1053 | 计划枚举器 | ✅ 完成 |

---

*本文档由 hermes-z6g4 维护*
*版本 1.0 - 2026-05-16*