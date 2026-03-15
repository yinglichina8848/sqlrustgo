# SQLRustGo v1.6.0 版本计划

> 版本：v1.6.0
> 制定日期：2026-03-15
> 制定人：yinglichina8848
> 目标：Transaction + Optimizer (完整 Mini DBMS)

---

## 一、版本概述

### 1.1 版本目标

| 项目 | 值 |
|------|-----|
| **版本号** | v1.6.0 |
| **目标成熟度** | L3 (Mini DBMS) |
| **核心目标** | 完整 Mini DBMS |
| **预计时间** | v1.5 GA 后 ~2 周 |
| **代号** | Transaction + Optimizer |

### 1.2 前置依赖

- ✅ v1.4.0 SQL Engine 完整化
- ✅ v1.5.0 Storage + Index

### 1.3 版本定位

v1.6 是最终版本，包含原 v1.8 + v1.9 + v1.10 的功能：

| 原规划 | v1.6 任务 |
|--------|-----------|
| v1.8 Transaction | 事务管理器 |
| v1.9 MVCC | MVCC |
| v1.10 Statistics | 统计信息 |
| | Optimizer |

---

## 二、核心能力

### 2.1 v1.6 完成后能力

- **完整单机数据库**
- **Mini PostgreSQL**

### 2.2 技术架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         v1.6.0 架构                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Query Engine (v1.4)                                            │
│       │                                                           │
│       ▼                                                           │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Optimizer (NEW!)                      │    │
│  │     Rule Based + Cost Based (CBO)                       │    │
│  └─────────────────────────────────────────────────────────┘    │
│                           │                                        │
│                           ▼                                        │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Executor Layer                        │    │
│  │   SeqScan / IndexScan / HashJoin / Sort / Aggregate    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                           │                                        │
│                           ▼                                        │
│  Storage Engine (v1.5)                                           │
│       │                                                           │
│       ▼                                                           │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              Transaction Layer (NEW!)                    │    │
│  │     ┌─────────────┐    ┌─────────────┐                 │    │
│  │     │ Transaction │    │    MVCC    │                 │    │
│  │     │  Manager    │    │  Snapshot  │                 │    │
│  │     └─────────────┘    └─────────────┘                 │    │
│  │           │                                        │         │
│  │           ▼                                        │         │
│  │     ┌─────────────────────────────────────────────┐  │         │
│  │     │              Statistics                      │  │         │
│  │     │   (Histogram / NDV / Cost Model)           │  │         │
│  │     └─────────────────────────────────────────────┘  │         │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、开发任务

### 3.1 Transaction Manager

```
┌─────────────────────────────────────────────────────────────────┐
│  Transaction Manager                                             │
├─────────────────────────────────────────────────────────────────┤
│  T-001: TransactionState 枚举 (Active/Committed/Aborted)        │
│  T-002: TransactionId 分配器                                     │
│  T-003: TransactionContext 管理                                   │
│  T-004: BEGIN / COMMIT / ROLLBACK 命令                          │
│  T-005: 事务日志记录                                             │
│  T-006: 回滚机制                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 MVCC

```
┌─────────────────────────────────────────────────────────────────┐
│  MVCC (Multi-Version Concurrency Control)                        │
├─────────────────────────────────────────────────────────────────┤
│  M-001: Tuple Header 扩展 (xmin, xmax, cmin, cmax)             │
│  M-002: Version Chain (版本链)                                  │
│  M-003: Snapshot (可见性快照)                                    │
│  M-004: Visibility Rules (可见性规则)                           │
│  M-005: MVCC 读取 (Snapshot Read)                               │
│  M-006: MVCC 写入 (Write Intent)                               │
│  M-007: GC 基础 (垃圾回收)                                      │
└─────────────────────────────────────────────────────────────────┘
```

### 3.3 Statistics

```
┌─────────────────────────────────────────────────────────────────┐
│  Statistics                                                      │
├─────────────────────────────────────────────────────────────────┤
│  S-001: TableStatistics 结构                                     │
│  S-002: ColumnStatistics 结构                                   │
│  S-003: Row Count 统计                                          │
│  S-004: NDV (Number of Distinct Values)                        │
│  S-005: Histogram (直方图)                                      │
│  S-006: ANALYZE 命令实现                                        │
│  S-007: 统计信息更新策略                                         │
└─────────────────────────────────────────────────────────────────┘
```

### 3.4 Optimizer

```
┌─────────────────────────────────────────────────────────────────┐
│  Optimizer                                                       │
├─────────────────────────────────────────────────────────────────┤
│  O-001: Optimizer trait 定义                                    │
│  O-002: Cost Model 基础                                         │
│  O-003: Access Path 选择 (SeqScan vs IndexScan)                 │
│  O-004: Join Order 优化                                         │
│  O-005: Join Method 选择 (HashJoin vs NestedLoop)              │
│  O-006: Rule Based Optimization (规则优化)                     │
│  O-007: CBO Optimization (代价优化)                             │
│  O-008: Plan Enumeration (计划枚举)                            │
└─────────────────────────────────────────────────────────────────┘
```

---

## 四、任务矩阵

| ID | 任务 | 预估时间 | 优先级 | 依赖 |
|----|------|----------|--------|------|
| **Transaction** |||||
| T-001 | TransactionState | 2h | P0 | - |
| T-002 | TransactionId 分配器 | 2h | P0 | T-001 |
| T-003 | TransactionContext | 4h | P0 | T-001 |
| T-004 | BEGIN/COMMIT/ROLLBACK | 6h | P0 | T-003 |
| T-005 | 事务日志 | 4h | P0 | T-004 |
| T-006 | 回滚机制 | 4h | P0 | T-005 |
| **MVCC** |||||
| M-001 | Tuple Header 扩展 | 4h | P0 | - |
| M-002 | Version Chain | 6h | P0 | M-001 |
| M-003 | Snapshot | 4h | P0 | M-001 |
| M-004 | Visibility Rules | 4h | P0 | M-003 |
| M-005 | MVCC 读取 | 4h | P0 | M-004 |
| M-006 | MVCC 写入 | 4h | P0 | M-005 |
| M-007 | GC 基础 | 4h | P1 | M-006 |
| **Statistics** |||||
| S-001 | TableStatistics | 2h | P0 | - |
| S-002 | ColumnStatistics | 2h | P0 | S-001 |
| S-003 | Row Count | 2h | P0 | S-001 |
| S-004 | NDV 统计 | 4h | P0 | S-002 |
| S-005 | Histogram | 6h | P1 | S-004 |
| S-006 | ANALYZE 命令 | 6h | P0 | S-001 |
| S-007 | 统计更新策略 | 2h | P1 | S-006 |
| **Optimizer** |||||
| O-001 | Optimizer trait | 4h | P0 | - |
| O-002 | Cost Model | 6h | P0 | O-001 |
| O-003 | Access Path 选择 | 6h | P0 | O-002 |
| O-004 | Join Order | 8h | P0 | O-003 |
| O-005 | Join Method 选择 | 4h | P0 | O-004 |
| O-006 | Rule Based Opt | 6h | P1 | O-001 |
| O-007 | CBO Optimization | 8h | P0 | O-002 |
| O-008 | Plan Enumeration | 6h | P1 | O-007 |

**总任务数**: 32 任务  
**总工时**: ~120h

---

## 五、里程碑

```
v1.6.0 开发 (约 2 周)
│
├── Week 1: Transaction + MVCC
│   ├── Day 1-2: Transaction Manager
│   │   └── T-001 ~ T-004
│   │
│   ├── Day 3-4: MVCC
│   │   └── M-001 ~ M-004
│   │
│   └── Day 5: MVCC 读写 + GC
│       └── M-005 ~ M-007
│
├── Week 2: Statistics + Optimizer
│   ├── Day 6-7: Statistics
│   │   └── S-001 ~ S-005
│   │
│   ├── Day 8-10: Optimizer
│   │   └── O-001 ~ O-006
│   │
│   ├── Day 11: 集成测试
│   │   └── 事务 + 优化器测试
│   │
│   └── Day 12-14: 发布
│       └── v1.6.0 GA

v1.6.0-alpha ───────────────────────────────────────────────► Day 4
v1.6.0-beta ─────────────────────────────────────────────────► Day 8
v1.6.0-rc ─────────────────────────────────────────────────► Day 12
v1.6.0 GA ─────────────────────────────────────────────────► Day 14
```

---

## 六、验收标准

### 6.1 功能验收

| 验收项 | 标准 |
|--------|------|
| Transaction | BEGIN/COMMIT/ROLLBACK |
| MVCC | 快照隔离、版本链、可见性规则 |
| Statistics | ANALYZE、NDV、Histogram |
| Optimizer | Access Path 选择、Join Order、CBO |

### 6.2 SQL 兼容性

```sql
-- v1.6 必须支持
BEGIN;
INSERT INTO t VALUES (1, 'test');
UPDATE t SET name = 'new' WHERE id = 1;
DELETE FROM t WHERE id = 2;
COMMIT;

-- 优化器自动选择执行计划
SELECT * FROM t WHERE id = 1;  -- 自动选择 IndexScan 或 SeqScan
SELECT * FROM t1 JOIN t2 ON t1.id = t2.id;  -- 自动选择 HashJoin 或 NestedLoop
```

### 6.3 覆盖率目标

| 模块 | 目标覆盖率 |
|------|------------|
| Transaction | ≥75% |
| MVCC | ≥70% |
| Statistics | ≥70% |
| Optimizer | ≥70% |
| 整体 | ≥80% |

---

## 七、技术细节

### 7.1 MVCC Tuple Header

```
┌─────────────────────────────────────────────┐
│            Tuple Header                      │
│  ┌─────────────────────────────────────┐   │
│  │ xmin: 创建事务ID                     │   │
│  │ xmax: 删除事务ID                     │   │
│  │ cmin: 创建命令ID                     │   │
│  │ cmax: 删除命令ID                     │   │
│  │ tuple_id: 元组ID                     │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

### 7.2 Visibility Rules

```
┌─────────────────────────────────────────────┐
│         Visibility Check                     │
├─────────────────────────────────────────────┤
│  if (txn_id == xmin && status == Active)   │
│     return Visible;                         │
│                                             │
│  if (xmin is Committed &&                  │
│      xmin < snapshot.xmin)                  │
│     return Visible;                         │
│                                             │
│  if (xmax == Invalid ||                     │
│      xmax is Aborted)                      │
│     return Visible;                         │
│                                             │
│  return Invisible;                          │
└─────────────────────────────────────────────┘
```

### 7.3 Cost Model

```
Cost = 
    IO_Cost * pages_read +
    CPU_Cost * rows_processed +
    Network_Cost * (for distributed)
```

---

## 八、风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| MVCC 复杂度 | 高 | 中 | 参考 PostgreSQL MVCC 设计 |
| CBO 实现难度 | 高 | 中 | 优先 Rule Based，再做 CBO |
| 统计信息准确性 | 中 | 中 | 采样 + 增量更新 |

---

## 九、关联 Issue

- 父 Issue: #88 (SQLRustGo 总体开发计划)
- 前置 Issue: v1.5.0 发布完成

---

## 十、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-15 | 初始版本 (Transaction + Optimizer) |

---

*本文档由 yinglichina8848 制定*
