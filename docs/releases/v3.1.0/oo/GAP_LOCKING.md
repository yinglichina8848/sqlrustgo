# Gap Locking 实现文档

> **版本**: v3.1.0
> **日期**: 2026-05-14
> **Issue**: #607, #784
> **状态**: ✅ 已完成

---

## 一、Gap Locking 概述

### 1.1 什么是 Gap Lock

Gap Lock 是一种锁机制，用于锁定索引记录之间的"间隙"（Gap），防止其他事务在间隙中插入新记录，从而解决幻读（Phantom Read）问题。

```
索引:    [10] [20] [30] [40]
Gap:    (10,20) (20,30) (30,40) (40,+∞)
```

### 1.2 Gap Lock vs Next-Key Lock

| 锁类型 | 锁定范围 | 用途 |
|--------|---------|------|
| Gap Lock | 索引间隙 | 防止插入 |
| Record Lock | 索引记录本身 | 防止修改/删除 |
| Next-Key Lock | Gap Lock + Record Lock | 同时防止 |

### 1.3 解决的问题

1. **幻读（Phantom Read）**：同一事务两次 SELECT 结果不同
2. **写偏斜（Write Skew）**：两个事务各自读取并更新重叠的数据
3. **唯一性约束**：确保唯一索引的唯一性

---

## 二、当前实现状态

### 2.1 锁实现 (crates/transaction/src/lock.rs)

```rust
pub enum LockTarget {
    Record(Vec<u8>),                    // 行锁
    Gap { start: Option<Vec<u8>>, end: Option<Vec<u8>> },  // 间隙锁
    NextKey(Vec<u8>),                   // Next-Key 锁
}

pub enum LockMode {
    Shared,      // 读锁
    Exclusive,  // 写锁
}
```

| 锁类型 | 状态 | 说明 |
|--------|------|------|
| Shared Lock | ✅ 已实现 | 读锁，多个事务可同时持有 |
| Exclusive Lock | ✅ 已实现 | 写锁，排他持有 |
| Record Lock | ✅ 已实现 | 行级锁 |
| Gap Lock | ✅ 已实现 | 间隙锁 |
| Next-Key Lock | ✅ 已实现 | Next-Key 锁 |
| FOR UPDATE | ✅ 已实现 | PR #775 |

### 2.2 现有锁管理器

```rust
pub struct LockManager {
    locks: HashMap<Vec<u8>, LockInfo>,      // key -> 锁信息
    tx_locks: HashMap<TxId, HashSet<Vec<u8>>>,  // tx -> 持有的锁
    deadlock_detector: DeadlockDetector,
}
```

---

## 三、执行链路

### 3.1 FOR UPDATE 执行链路 (src/execution_engine.rs)

```
SQL: SELECT * FROM t WHERE id = 5 FOR UPDATE
  ↓
Parser: 解析 FOR UPDATE
  ↓
execute_select_for_update():
  ├→ 获取当前事务 tx_id
  ├→ extract_lock_target_from_where(where_clause):
  │   ├→ "=" → LockTarget::NextKey(key)
  │   ├→ ">" → LockTarget::Gap { start: key, end: None }
  │   ├→ "<" → LockTarget::Gap { start: None, end: key }
  │   ├→ ">=" → LockTarget::Gap { start: key, end: None }
  │   └→ "<=" → LockTarget::Gap { start: None, end: key }
  │
  ├→ transaction_manager.acquire_lock_with_target(tx_id, target, Exclusive)
  │   └→ LockGrantMode::Granted / Waiting / Error
  │
  └→ execute_select()  // 执行普通 SELECT
```

### 3.2 Gap Lock 获取范围

| WHERE 条件 | LockTarget | 说明 |
|------------|-----------|------|
| `id = 5` | `NextKey(5)` | 等值查询 → 锁定 key 5 |
| `id > 5` | `Gap { start: 5, end: None }` | 大于 → 锁定 (5, +∞) |
| `id < 5` | `Gap { start: None, end: 5 }` | 小于 → 锁定 (-∞, 5) |
| `id >= 5` | `Gap { start: 5, end: None }` | 大于等于 → 锁定 [5, +∞) |
| `id <= 5` | `Gap { start: None, end: 5 }` | 小于等于 → 锁定 (-∞, 5] |

### 3.3 锁获取结果处理

```rust
match transaction_manager.acquire_lock_with_target(tx_id, target, LockMode::Exclusive) {
    Ok(LockGrantMode::Granted) => {
        // 锁获取成功，继续执行 SELECT
    }
    Ok(LockGrantMode::Waiting) => {
        // 等待其他事务释放锁
        return Err("SELECT FOR UPDATE blocked by concurrent transaction");
    }
    Err(e) => {
        return Err(format!("Failed to acquire FOR UPDATE lock: {}", e));
    }
}
```

---

## 四、数据结构

### 4.1 LockTarget (实际实现)

```rust
pub enum LockTarget {
    Record(Vec<u8>),                    // 行锁
    Gap { start: Option<Vec<u8>>, end: Option<Vec<u8>> },  // 间隙锁
    NextKey(Vec<u8>),                   // Next-Key 锁
}
```

### 4.2 Gap 锁重叠检测

```rust
impl LockTarget {
    pub fn overlaps(&self, other: &LockTarget) -> bool {
        match (self, other) {
            // Record vs Record: exact key match
            (Record(k1), Record(k2)) => k1 == k2,
            // Record vs Gap: key within gap range
            (Record(key), Gap { start, end }) => key_in_gap(key, start, end),
            // Gap vs Gap: overlapping intervals
            (Gap { start: s1, end: e1 }, Gap { start: s2, end: e2 }) => gaps_overlap(s1, e1, s2, e2),
            // ... 更多组合
        }
    }
}
```

---

## 五、测试验证

### 5.1 锁测试 (crates/transaction/tests/lock_*.rs)

| 测试 | 状态 | 说明 |
|------|------|------|
| `test_acquire_range_lock_gap` | ✅ | Gap 锁获取 |
| `test_acquire_range_lock_nextkey` | ✅ | Next-Key 锁获取 |
| `test_gap_lock_for_range_gt` | ✅ | 大于条件 Gap 锁 |
| `test_gap_lock_for_range_lt` | ✅ | 小于条件 Gap 锁 |
| `test_gap_lock_for_range_between` | ✅ | 范围 Gap 锁 |

### 5.2 SSI 压力测试

| 测试 | 状态 | 说明 |
|------|------|------|
| `test_serialization_graph_cycle_detection` | ✅ | 死锁检测 |
| `test_write_skew_detection` | ✅ | 写偏斜检测 |
| `test_concurrent_no_false_positives` | ✅ | 无误报 |

---

## 六、参考

- `crates/transaction/src/lock.rs` - LockTarget 和锁逻辑实现
- `src/execution_engine.rs` - execute_select_for_update 方法
- `crates/transaction/tests/lock_tests.rs` - 锁单元测试
- `crates/transaction/tests/ssi_integration.rs` - SSI 集成测试
- PR #775 - SELECT FOR UPDATE 锁获取实现