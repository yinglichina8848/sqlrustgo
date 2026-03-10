# SQLRustGo v1.3.0 开发计划

> **版本**: v1.3.0
> **代号**: Architecture Stabilization Release (架构稳定版)
> **状态**: 📋 规划中
> **目标成熟度**: L3 → L4 产品级
> **版本类型**: 🏗️ 架构稳定 + 测试驱动

---

## 一、版本定位

### 1.1 战略定位

**v1.3.0 = Architecture Stabilization Release**

核心目标：
- 不是加很多新功能
- 而是把 v1.x 的核心内核"打牢"
- 为 2.0 的 CBO / Vectorized Execution 做准备

### 1.2 为什么是架构稳定版？

当前系统风险分析：

| 模块 | 覆盖率 | 风险等级 |
|------|--------|----------|
| parser | 75% | ✅ 安全 |
| storage | 58% | ⚠️ 可接受 |
| planner | 43% | ⚠️ 中 |
| optimizer | 18% | 🔴 高 |
| types | 23% | 🔴 高 |
| executor | 0.86% | 🔴 极高 |
| transaction | 0% | 🔴 极高 |
| catalog | 0% | 🔴 极高 |

**关键问题**：
- Executor = 基本没有验证 (最大风险)
- Transaction = 完全无测试
- Catalog = 完全无测试

数据库核心流程：
```
Parser → Planner → Optimizer → Executor → Storage
```

而现在 Executor 几乎没有验证，这是数据库系统最大风险。

### 1.3 版本演进顺序

```
1.0   Parser
↓
1.2   Basic Engine
↓
1.3   Stable Executor  ← 当前位置
↓
1.5   Batch Execution
↓
2.0   Vectorized Engine
↓
3.0   Parallel Engine
```

---

## 二、功能矩阵

### 2.1 测试体系 (P0)

| 功能 | 优先级 | 说明 |
|------|--------|------|
| Executor 单元测试框架 | P0 | 最重要 |
| Transaction 测试 | P0 | MVCC 基础 |
| Catalog 测试 | P0 | 表定义 |

### 2.2 执行器稳定化 (P0)

| 功能 | 优先级 | 说明 |
|------|--------|------|
| Volcano Executor 稳定化 | P0 | 核心执行模型 |
| TableScan | P0 | 必须 |
| Projection | P0 | 必须 |
| Filter | P0 | 必须 |
| HashJoin | P1 | 为 2.0 做准备 |

### 2.3 Planner 清理 (P1)

| 功能 | 优先级 | 说明 |
|------|--------|------|
| LogicalPlan 清理 | P1 | 减少 2.0 重构 |

### 2.4 Optimizer 骨架 (P1)

| 功能 | 优先级 | 说明 |
|------|--------|------|
| Rule Framework | P1 | Cascades 前置 |
| Predicate Pushdown | P1 | 谓词下推 |
| Projection Pushdown | P1 | 投影裁剪 |

### 2.5 存储 (P2)

| 功能 | 优先级 | 说明 |
|------|--------|------|
| 简单统计信息 | P2 | CBO 预留 |

### 2.6 SQL 能力 (P0)

| 功能 | 优先级 | 说明 |
|------|--------|------|
| 基础 SELECT 完整支持 | P0 | demo 能力 |

---

## 三、硬指标目标

### 3.1 覆盖率目标

| 指标 | 目标 |
|------|------|
| 整体覆盖率 | ≥ 65% |
| Executor 覆盖率 | ≥ 60% |
| Planner 覆盖率 | ≥ 60% |
| Optimizer 覆盖率 | ≥ 40% |

### 3.2 SQL 能力目标

必须稳定支持：
```sql
SELECT col FROM table WHERE predicate JOIN table2
```

包含：
- Projection (列投影)
- Filter (条件过滤)
- HashJoin (哈希连接)

### 3.3 执行引擎目标

明确执行模型：**Volcano Model**

统一 trait：
```rust
trait Executor {
    fn open(&mut self);
    fn next(&mut self) -> Option<Tuple>;
    fn close(&mut self);
}
```

### 3.4 Optimizer 目标

不是做 CBO，而是做：
- Rule-based optimizer skeleton
- Predicate Pushdown (谓词下推)
- Projection Pushdown (投影裁剪)

这样 2.0 Cascades 才能接上。

### 3.5 性能目标

| 指标 | 目标 |
|------|------|
| INSERT 100k rows | < 2s (≈ 50k rows/s) |
| SELECT * (100k rows) | < 200ms |
| HashJoin (100k × 100k) | < 2s |

**注意**: 1M rows/s 在 v1.x 阶段不现实，v2.0 向量化执行阶段再追求。

---

## 四、开发路线图 (6 周)

```
v1.3.0-draft → v1.3.0-alpha → v1.3.0-beta → v1.3.0-rc → v1.3.0
```

### Week 1-2: 测试框架

**核心**: Executor Test Framework

**完成**:
- executor test harness
- mock storage
- tuple generator

**目标**: Executor coverage > 40%

### Week 3: 核心算子

**完成算子**:
- TableScan
- Projection
- Filter

**实现**: Iterator executor

### Week 4: Join 实现

**完成**:
- HashJoin

**SQL 支持**:
```sql
SELECT * FROM A JOIN B
```

### Week 5: Optimizer 骨架

**完成**:
- rule framework
- predicate pushdown
- projection pushdown

### Week 6: 测试冲刺

**完成**:
- transaction tests
- catalog tests
- integration tests

**SQL 集成测试**:
- tests/sql/

---

## 五、技术设计

### 5.1 Volcano Executor

```rust
pub trait Executor: Send {
    fn open(&mut self);
    fn next(&mut self) -> Option<Record>;
    fn close(&mut self);
}

pub struct SeqScanExec {
    table_name: String,
    storage: Arc<dyn StorageEngine>,
}

pub struct FilterExec {
    child: Box<dyn Executor>,
    predicate: Expression,
}

pub struct ProjectionExec {
    child: Box<dyn Executor>,
    columns: Vec<String>,
}

pub struct HashJoinExec {
    left: Box<dyn Executor>,
    right: Box<dyn Executor>,
    on: (String, String),
}
```

### 5.2 测试框架

```rust
#[cfg(test)]
mod executor_tests {
    use super::*;

    fn create_mock_storage() -> MockStorage {
        // 创建测试用 mock storage
    }

    #[test]
    fn test_table_scan() {
        let storage = create_mock_storage();
        let exec = SeqScanExec::new("test_table", storage);
        // 验证结果
    }

    #[test]
    fn test_filter() {
        // 测试过滤条件
    }

    #[test]
    fn test_hash_join() {
        // 测试哈希连接
    }
}
```

---

## 六、验收标准

### 6.1 功能验收

| 验收项 | 标准 |
|--------|------|
| Executor 测试 | > 60% 覆盖率 |
| Transaction 测试 | 基础 MVCC 测试通过 |
| Catalog 测试 | 表定义操作测试通过 |
| SQL SELECT | WHERE/JOIN 正常工作 |

### 6.2 架构验收

| 验收项 | 标准 |
|--------|------|
| Volcano Model | 统一执行接口 |
| Optimizer 骨架 | Rule-based 优化生效 |

### 6.3 性能验收

| 指标 | 目标 |
|------|------|
| INSERT 100k | < 2s |
| SELECT 100k | < 200ms |

---

## 七、风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Executor 实现复杂度高 | 高 | 分步实现，从简单算子开始 |
| 测试框架搭建耗时 | 中 | 复用现有 mock 模式 |
| 性能目标未达成 | 中 | 降低预期，聚焦功能正确性 |

---

## 八、关联 Issue

- 父 Issue: #200 (v1.3.0 详细开发任务)
- 前置 Issue: v1.2.0 GA 发布

---

## 九、下一步行动

1. ⏳ 创建 v1.3.0 开发分支
2. ⏳ 基于本文档创建子 Issue
3. ⏳ 开始 Executor 测试框架开发
4. ⏳ v1.2.0 GA 发布后正式启动

---

*制定日期: 2026-03-11*
*基于 v1.2.0 测试覆盖率分析和架构战略讨论*
