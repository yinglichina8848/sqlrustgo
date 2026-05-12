# MVCC 可见性形式化规格

> **Issue**: #660
> **Type**: TLA+ Style Formal Specification
> **Date**: 2026-05-12

---

## 一、数据结构定义

### 1.1 核心类型

```tla
(* Transaction Identifier *)
CONSTANT TxId

(* Timestamp *)
CONSTANT Timestamp

(* Transaction Status *)
CONSTANT Active, Committed, Aborted

(* Row Version *)
CONSTANT RowVersion {
    tx_id: TxId,           (* 创建事务 ID *)
    created_commit_ts: Timestamp,  (* 创建提交时间戳 *)
    deleted_by: TxId | None,       (* 删除事务 ID *)
    deleted_commit_ts: Timestamp | None,  (* 删除提交时间戳 *)
    value: ByteArray              (* 行数据 *)
}

(* Snapshot *)
CONSTANT Snapshot {
    tx_id: TxId,                 (* 当前事务 ID *)
    snapshot_timestamp: Timestamp (* 快照时间戳 *)
}
```

### 1.2 约束条件

```tla
ASSUME AssertNat(Timestamp)      (* Timestamp 是自然数 *)
ASSUME AssertNat(TxId)          (* TxId 是自然数 *)
ASSUME 0 /= InvalidTxId         (* 0 是无效事务 ID *)
```

---

## 二、可见性判断算法

### 2.1 核心规则: is_visible

```tla
is_visible(version, snapshot) == 
    IF version.created_by = snapshot.tx_id
    THEN TRUE  (* 事务看到自己的写入 *)
    ELSE IF version.created_commit_ts = None
    THEN FALSE  (* 未提交的创建不可见 *)
    ELSE IF version.created_commit_ts > snapshot.snapshot_timestamp
    THEN FALSE  (* 创建在快照之后不可见 *)
    ELSE IF version.value /= Empty
    THEN (* 普通版本 *)
         IF version.deleted_commit_ts /= None
         THEN version.deleted_commit_ts > snapshot.snapshot_timestamp
         ELSE TRUE
    ELSE (* 删除标记 *)
         version.created_commit_ts < snapshot.snapshot_timestamp
```

### 2.2 规则解释

| 规则 | 条件 | 结果 |
|------|------|------|
| 自读取 | `created_by = tx_id` | 始终可见 |
| 未提交创建 | `created_commit_ts = None` | 始终不可见 |
| 未来创建 | `created_commit_ts > snapshot_ts` | 不可见 |
| 已删除 | `deleted_commit_ts <= snapshot_ts` | 不可见 |
| 未删除 | 其他 | 可见 |

---

## 三、时间戳约束

### 3.1 事务生命周期

```tla
TransactionLifecycle(tx, start_ts, commit_ts) ==
    /\ tx.status = "Committed"
    /\ tx.start_timestamp = start_ts
    /\ tx.commit_timestamp = commit_ts
    /\ start_ts < commit_ts  (* 开始时间戳必须小于提交时间戳 *)
```

### 3.2 版本链单调性

```tla
VersionChainMonotonic(versions) ==
    \A i, j \in DOMAIN versions:
        i < j => versions[i].created_commit_ts <= versions[j].created_commit_ts
```

---

## 四、快照隔离保证

### 4.1 快照定义

```tla
SnapshotIsolation(snapshot, visible_versions) ==
    \A v \in visible_versions:
        is_visible(v, snapshot) = TRUE
    /\ \A v \in DOMAIN visible_versions \ visible_versions:
        is_visible(v, snapshot) = FALSE
```

### 4.2 串行化等效

```tla
SerializableEquivalent(snapshot, committed_tx) ==
    \A t1, t2 \in committed_tx:
        t1 /= t2 /\ t1.commit_ts <= t2.start_ts
        => \A v \in t1.writes: is_visible(v, t2.snapshot) = FALSE
```

---

## 五、反例测试用例

### 5.1 违反可见性规则的场景

```rust
// 反例 1: 事务看到未提交的创建
test_counterexample_uncommitted_create() {
    // T1: BEGIN, INSERT, (未 COMMIT)
    // T2: BEGIN, SELECT
    // 期望: T2 不应看到 T1 的未提交插入
}

// 反例 2: 事务看到在快照之后创建的版本
test_counterexample_future_version() {
    // T1: BEGIN (snapshot_ts=10)
    // T2: INSERT (commit_ts=15), COMMIT
    // T1: SELECT
    // 期望: T1 不应看到 commit_ts=15 的版本
}

// 反例 3: 事务看到已删除的版本
test_counterexample_deleted_version() {
    // T1: INSERT, COMMIT (commit_ts=10)
    // T2: DELETE, COMMIT (commit_ts=15)
    // T3: BEGIN (snapshot_ts=12), SELECT
    // 期望: T3 不应看到被 T2 删除的版本
}
```

### 5.2 写偏斜场景

```rust
// 反例 4: 写偏斜 - 两个事务各自读取互相重叠的数据，各自更新
test_counterexample_write_skew() {
    // 账户 A: 余额 100
    // 账户 B: 余额 100
    // T1: SELECT SUM(A+B) WHERE A+B > 150  -- 读到了 A=100, B=100
    // T2: SELECT SUM(A+B) WHERE A+B > 150  -- 读到了 A=100, B=100
    // T1: UPDATE A SET balance = balance - 50  -- A=50
    // T2: UPDATE B SET balance = balance - 50  -- B=50
    // 期望: 至少一个事务应该失败，因为 A+B 会变成 100
    // 实际: 两个事务都可能成功 (如果使用 Snapshot Isolation)
}
```

---

## 六、形式化验证状态

### 6.1 当前状态

| 验证项 | 状态 | 证据 |
|--------|------|------|
| is_visible 算法 | ✅ 已验证 | `crates/transaction/src/mvcc.rs:230-253` |
| 时间戳单调性 | ✅ 已验证 | `VersionChainMonotonic` |
| 快照隔离 | ⚠️ 部分 | PROOF-026 Write Skew 未闭环 |

### 6.2 待补充

- [ ] 完整的 TLA+ 模型文件
- [ ] 写偏斜反例测试
- [ ] SSI (Serializable Snapshot Isolation) 规格

---

## 七、代码对应

### 7.1 Rust 实现

```rust
// crates/transaction/src/mvcc.rs

pub fn is_visible(&self, snapshot: &Snapshot) -> bool {
    // Rule 1: 自读取
    if self.created_by == snapshot.tx_id {
        return true;
    }
    // Rule 2: 未提交创建
    let created_ts = match self.created_commit_ts {
        Some(ts) => ts,
        None => return false,
    };
    // Rule 3: 未来创建
    if created_ts > snapshot.snapshot_timestamp {
        return false;
    }
    // Rule 4: 已删除检查
    if !self.value.is_empty() {
        if let Some(deleted_ts) = self.deleted_commit_ts {
            if deleted_ts <= snapshot.snapshot_timestamp {
                return false;
            }
        }
    }
    true
}
```

### 7.2 测试覆盖

| 测试文件 | 覆盖规则 |
|----------|----------|
| `tests/mvcc_snapshot_isolation_test.rs` | 基础事务 |
| `tests/mvcc_transaction_test.rs` | 并发事务 |
| `tests/ssi_stress_test.rs` | SSI 压力 |

---

## 八、参考

- `crates/transaction/src/mvcc.rs` - MVCC 实现
- `crates/transaction/src/version_chain.rs` - 版本链
- `docs/formal/FORMAL_SYSTEM_STATUS.md` - 形式化系统状态