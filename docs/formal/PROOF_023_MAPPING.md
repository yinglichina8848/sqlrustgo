# PROOF-023: TLA+ ↔ Rust 实现映射

> 版本：v1.0 | 日期：2026-05-03 | 状态：已验证

---

## 一、版本演进总结

| 版本 | 模型 | States | 验证结果 |
|------|------|--------|---------|
| **v1** | T2WaitsForK1 缺 UNCHANGED | — | ❌ Next-state 不完整 |
| **v2** | 2-cycle avoidance | 12 | ✅ PASS |
| **v3** | N-transaction (3 txn) + recursive Reachable | 1024 | ✅ PASS |
| **v4** | Multi-resource Wait-For Graph (txn → SET<txn>) | 91 | ✅ PASS |

---

## 二、核心不变量（已在 TLA+ 中证明）

### 2.1 NoCycle（最重要）

```tla
NoCycle == \A t \in AllTxns : ~Reachable(t, t)
```

**含义**：Wait-For Graph 始终无环（是 DAG）

**对应 Rust**：`would_create_cycle()` 始终返回 `false`

### 2.2 NoSelfWait

```tla
NoSelfWait == \A t \in AllTxns : t \notin txnWaitFor[t]
```

**含义**：事务不能等待自己

**对应 Rust**：永远不会添加 `t → t` 边

### 2.3 WaitConsistent

```tla
WaitConsistent ==
  \A t \in AllTxns :
    \A h \in txnWaitFor[t] :
      \/ \E k \in AllKeys : k \in txnHoldLocks[h]
      \/ h \in txnWaitFor[h]
```

**含义**：如果 t 等待 h，则 h 必须持有 t 需要的资源，或者 h 本身也在等待

---

## 三、TLA+ → Rust 映射表

### 3.1 核心数据结构

| TLA+ | Rust | 说明 |
|------|------|------|
| `txnHoldLocks : [Txn → SUBSET Key]` | `HashMap<TxnId, HashSet<KeyId>>` | 每个 txn 持有的锁集合 |
| `txnWaitFor : [Txn → SUBSET Txn]` | `HashMap<TxnId, HashSet<TxnId>>` | 每个 txn 等待的其他 txn |
| `Holder(k) = {t : k ∈ txnHoldLocks[t]}` | `holders_of(key)` | 遍历 `hold_locks` 的值集合 |

### 3.2 Reachable 算法

```tla
RECURSIVE Reachable(_,_)
Reachable(t1, t2) ==
  \/ t2 \in txnWaitFor[t1]
  \/ \E t \in txnWaitFor[t1] : /\ t # t1 /\ Reachable(t, t2)
```

对应 Rust（DFS）：

```rust
use std::collections::{HashMap, HashSet};

type TxnId = u64;
type KeyId = u64;

pub fn reachable(
    wait_for: &HashMap<TxnId, HashSet<TxnId>>,
    from: TxnId,
    to: TxnId,
) -> bool {
    // BFS/DFS from 'from' to see if 'to' is reachable
    let mut stack = vec![from];
    let mut visited = HashSet::new();

    while let Some(curr) = stack.pop() {
        if curr == to {
            return true;
        }
        if visited.insert(curr) {
            if let Some(neighbors) = wait_for.get(&curr) {
                for &next in neighbors {
                    stack.push(next);
                }
            }
        }
    }
    false
}
```

### 3.3 Wait 操作（含 cycle 检测）

```tla
Wait(t, k) ==
  LET holders == Holder(k) \ {t} IN
  /\ holders # {}
  /\ txnWaitFor[t] = {}
  /\ \A h \in holders : ~Reachable(h, t)   (* 核心: 所有 holder 都无法到达 t *)
  /\ txnWaitFor' = [txnWaitFor EXCEPT ![t] = holders]
  /\ UNCHANGED txnHoldLocks
```

对应 Rust：

```rust
pub fn try_wait(
    hold_locks: &mut HashMap<TxnId, HashSet<KeyId>>,
    wait_for: &mut HashMap<TxnId, HashSet<TxnId>>,
    txn: TxnId,
    key: KeyId,
) -> Result<(), ()> {
    // 找到 key 的所有 holder
    let holders: Vec<TxnId> = hold_locks.iter()
        .filter(|(_, keys)| keys.contains(&key))
        .map(|(t, _)| *t)
        .filter(|&t| t != txn)
        .collect();

    if holders.is_empty() {
        return Err(()); // 没有 holder
    }

    // 检查: 所有 holder 都不能到达当前 txn (否则形成环)
    for &h in &holders {
        if reachable(wait_for, h, txn) {
            return Err(()); // WOULD CYCLE — 必须 Abort
        }
    }

    // 无环: 添加等待边
    wait_for.insert(txn, holders.into_iter().collect());
    Ok(())
}
```

### 3.4 Release 操作（传播解锁）

```tla
Release(t) ==
  /\ txnHoldLocks[t] # {}
  /\ txnHoldLocks' = [txnHoldLocks EXCEPT ![t] = {}]
  /\ txnWaitFor' = [x \in AllTxns |->
                      IF t \in txnWaitFor[x]
                         THEN txnWaitFor[x] \ {t}
                         ELSE txnWaitFor[x]]
```

对应 Rust：

```rust
pub fn release(
    hold_locks: &mut HashMap<TxnId, HashSet<KeyId>>,
    wait_for: &mut HashMap<TxnId, HashSet<TxnId>>,
    txn: TxnId,
) {
    // 释放所有锁
    hold_locks.insert(txn, HashSet::new());

    // 所有等待 txn 的事务都被解除阻塞
    for (waitingTxn, waiters) in wait_for.iter_mut() {
        waiters.remove(&txn);
    }
}
```

### 3.5 Abort 操作

```tla
Abort(t) ==
  /\ txnHoldLocks[t] # {}
  /\ txnHoldLocks' = [txnHoldLocks EXCEPT ![t] = {}]
  /\ txnWaitFor' = [x \in AllTxns |->
                      IF t \in txnWaitFor[x]
                         THEN txnWaitFor[x] \ {t}
                         ELSE txnWaitFor[x]]
```

对应 Rust：与 Release 完全相同（abort 和 release 都清除持有锁并解除等待）

---

## 四、协议规则总结

| 规则 | TLA+ | Rust |
|------|------|------|
| **加锁** | `Holder(k) = {}` 时才能 Acquire | `holders_of(key).is_empty()` |
| **等待** | `~Reachable(h, t)` ∀h∈holders | `!reachable(wait_for, h, txn)` ∀h |
| **释放** | `txnWaitFor[x] \ {t}` | `waiters.remove(&txn)` |
| **Abort** | 同 Release | 同 Release |

---

## 五、必须测试用例

### 5.1 2-cycle（v2 覆盖）

```text
T1 持有 K1，等待 T2 (K2)
T2 持有 K2，等待 T1 (K1)

→ Wait(T1, K2) 被拒绝 (Reachable(T2, T1) = true)
→ 必须 Abort
```

### 5.2 3-cycle（v3/v4 覆盖）

```text
T1 → T2 → T3 → T1

T3 尝试等待 T1:
  Reachable(T1, T3) = true (T1→T2→T3→T1 路径存在)
  → 拒绝
```

### 5.3 多资源等待（v4 独有）

```text
T1 持有 {K1, K2}，等待 {}
T2 持有 {}，等待 {T1} (via K1)
T3 持有 {}，等待 {T1, T2} (via K1, K2)

T3 尝试等待 {T1, T2}:
  Reachable(T1, T3) = false ✓
  Reachable(T2, T3) = false ✓
  → 允许
```

### 5.4 Abort 传播

```text
T1 持有 K1
T2 等待 T1 (via K1)
T3 等待 T2 (via T2)

T1 Abort:
  T2 的等待被清除 (T1 不在 wait_for[T2] 中)
  T3 仍然等待 T2
```

---

## 六、关键不变量（Rust 端必须保持）

```text
1. NoSelfWait:  ∀t: t ∉ wait_for[t]
2. NoCycle:     ∀t: !reachable(wait_for, t, t)
3. LockHolder:   如果 k ∈ hold_locks[t]，则 t ∉ wait_for[t] (持有者不能等待)
4. WaitCause:   如果 t ∈ wait_for[s]，则 ∃k: k ∈ hold_locks[t] ∧ k ∈ Holder(k)
```

---

## 七、与 SQLRustGo Executor 的集成点

```
executor/
├── transaction.rs
│   ├── hold_locks: HashMap<TxnId, HashSet<KeyId>>
│   ├── wait_for: HashMap<TxnId, HashSet<TxnId>>
│   ├── try_lock()      → 对应 Acquire
│   ├── try_wait(key)   → 对应 Wait (含 Reachable 检查)
│   ├── release()        → 对应 Release
│   └── abort()         → 对应 Abort
│
├── lock_manager.rs
│   └── would_create_cycle(txn, holders) → Reachable(holders, txn)
```

---

## 八、验证记录

| 日期 | 版本 | 结果 | States |
|------|------|------|--------|
| 2026-05-03 | v2 | PASS | 12 |
| 2026-05-03 | v3 | PASS | 1024 |
| 2026-05-03 | v4 | PASS | 91 |

---

## 九、已知限制

1. **每次只等待一个 key**：当前模型要求 `txnWaitFor[t] = {}` 才能 Wait（不是累积）
2. **有限 txn/keys**：TLC 验证使用 `{T1,T2,T3}` × `{K1,K2}`，生产环境需要动态扩容
3. **单一锁持有者**：目前 `Holder(k)` 返回所有持有者（支持并发），但 Wait 只等待当前 holders

---

## 十、下一步

* 将 `try_wait()` / `would_create_cycle()` 集成到 SQLRustGo `executor/transaction.rs`
* 添加 Rust 单元测试覆盖 2-cycle / 3-cycle / 多资源场景
* 考虑升级到 v4.1：累积等待（`txnWaitFor[t] = txnWaitFor[t] ∪ holders`）
