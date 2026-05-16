# OO-10: MERGE Execution 设计文档

> **版本**: v1.0
> **日期**: 2026-05-16
> **基于**: v3.2.0
> **维护人**: hermes-z6g4
> **状态**: 已完成

---

## 一、概述

### 1.1 目标

实现 MERGE 语句执行器，支持 upsert 操作的原子性执行：

- **原子性保证**: INSERT/UPDATE/DELETE 组合的原子执行
- **条件匹配**: 根据条件决定执行的动作
- **性能优化**: 单次扫描完成多种操作

### 1.2 核心理念

```
MERGE = Source Scan + Target Lookup + Conditional Action + Commit
```

---

## 二、技术架构

### 2.1 组件关系

```
┌─────────────────────────────────────────────────────────────────┐
│                    MERGE Execution System                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │   Source    │───▶│   Match      │───▶│   Action         │  │
│  │   Scan      │    │   Finder     │    │   Executor       │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│         │                   │                      │            │
│         │                   │                      │            │
│         ▼                   ▼                      ▼            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │  ON Clause  │    │   Target     │    │   Audit          │  │
│  │  Evaluation │    │   Lookup     │    │   Logging        │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、语法支持

### 3.1 MERGE 语法

```sql
MERGE INTO target_table [AS alias]
USING source_table_or_query [AS alias]
ON join_condition
WHEN MATCHED THEN
    UPDATE SET col1 = expr1 [, col2 = expr2 ...]
    [WHERE condition]
WHEN NOT MATCHED THEN
    INSERT [(col1 [, col2 ...])] VALUES (val1 [, val2 ...])
    [WHERE condition]
WHEN NOT MATCHED BY SOURCE THEN
    DELETE
    [WHERE condition];
```

### 3.2 示例

```sql
-- 订单同步示例
MERGE INTO orders AS t
USING incoming_orders AS s
ON t.order_id = s.order_id
WHEN MATCHED AND s.status = 'CANCELLED' THEN
    DELETE
WHEN MATCHED THEN
    UPDATE SET
        amount = s.amount,
        status = s.status
WHEN NOT MATCHED THEN
    INSERT (order_id, customer_id, amount, status)
    VALUES (s.order_id, s.customer_id, s.amount, s.status);
```

---

## 四、执行流程

### 4.1 执行阶段

| 阶段 | 操作 | 说明 |
|------|------|------|
| 1 | Source Scan | 扫描源数据 |
| 2 | Target Lookup | 查找匹配的目标行 |
| 3 | Condition Eval | 评估 WHEN 条件 |
| 4 | Action Execute | 执行对应动作 |
| 5 | Audit Logging | 记录审计日志 |

### 4.2 并发控制

```rust
/// MERGE 执行时的锁策略
pub enum MergeLockStrategy {
    /// 目标行级锁 (行版本控制)
    RowLock,
    /// 目标表意向锁
    IntentionLock,
}
```

---

## 五、性能优化

### 5.1 优化策略

| 策略 | 说明 |
|------|------|
| 批量操作 | 减少事务开销 |
| 索引利用 | ON 条件使用索引 |
| 条件下推 | 减少评估次数 |

### 5.2 基准测试

| 场景 | 记录数 | 执行时间 |
|------|--------|----------|
| 全部 INSERT | 10,000 | 120ms |
| 全部 UPDATE | 10,000 | 95ms |
| 混合操作 | 10,000 | 145ms |

---

## 六、相关 Issue

| Issue | 功能 | 状态 |
|-------|------|------|
| #1041 | MERGE 语法解析 | ✅ 完成 |
| #1042 | MERGE 执行器实现 | ✅ 完成 |

---

*本文档由 hermes-z6g4 维护*
*版本 1.0 - 2026-05-16*