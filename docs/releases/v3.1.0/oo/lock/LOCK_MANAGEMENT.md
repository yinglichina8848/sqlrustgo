# 锁管理器与死锁检测 (v3.1.0)

> **基于 GitNexus 分析** | 67,755 符号, 102,165 关系
> Record Lock / Gap Lock / Next-Key Lock / Deadlock Detection

## 1. 锁管理器架构

### 1.1 核心数据结构

```rust
pub enum LockTarget {
    Record(Vec<u8>),
    Gap { start: Option<Vec<u8>>, end: Option<Vec<u8>> },
    NextKey(Vec<u8>),
}

pub enum LockMode {
    Shared,
    Exclusive,
}

pub struct LockInfo {
    pub key: Vec<u8>,
    pub mode: LockMode,
    pub holders: HashSet<TxId>,
    pub waiters: Vec<(TxId, LockMode)>,
}

pub struct RangeLockInfo {
    pub target: LockTarget,
    pub mode: LockMode,
    pub holders: HashSet<TxId>,
    pub waiters: Vec<(TxId, LockMode)>,
}

pub struct LockManager {
    locks: HashMap<Vec<u8>, LockInfo>,
    range_locks: BTreeMap<Vec<u8>, RangeLockInfo>,
    tx_locks: HashMap<TxId, HashSet<Vec<u8>>>,
    tx_range_locks: HashMap<TxId, HashSet<Vec<u8>>>,
    deadlock_detector: DeadlockDetector,
}
```

### 1.2 锁类型

| 锁类型 | 说明 | 兼容性 |
|--------|------|--------|
| Shared (S) | 读锁 | S-S 兼容, S-X 不兼容 |
| Exclusive (X) | 写锁 | X-任何 不兼容 |
| Gap Lock | 间隙锁 | Gap-Gap 兼容 (只阻塞插入) |
| Next-Key Lock | Record + Gap | 等价于 Record Lock + Gap Lock |

### 1.3 关键文件

| 文件 | 行数 | 作用 |
|------|------|------|
| [lock.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/transaction/src/lock.rs) | ~1100 | 锁管理器 |
| [deadlock.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/transaction/src/deadlock.rs) | ~200 | 死锁检测器 |

## 2. 锁获取链路

### 2.1 锁获取时序图

```
Transaction T1: SELECT * FROM orders WHERE id = 1 FOR UPDATE
    │
    ▼
┌──────────────────────────────────────────────────┐
│ 1. LockManager.acquire_lock(key=1, mode=X, tx=1) │
│    ├── 检查 locks[1] 是否存在                     │
│    ├── 存在: 检查兼容性                           │
│    │   ├── holders 只有 S 锁且无 waiters → 升级   │
│    │   └── holders 有 X 锁 → 加入 waiters        │
│    └── 不存在: 创建 LockInfo, 持有锁              │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│ 2. 死锁检测                                       │
│    ├── DeadlockDetector.try_wait_edge()           │
│    ├── Pre-check: would_create_cycle()?           │
│    │   └── DFS 从 to_set 搜索 from               │
│    ├── 有环 → LockError::Deadlock                │
│    └── 无环 → 添加等待边, 返回 Ok                │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│ 3. 锁授予                                         │
│    ├── 兼容 → 立即授予                            │
│    └── 不兼容 → 等待 (加入 waiters)               │
└──────────────────────────────────────────────────┘
```

### 2.2 锁授予规则

```
┌─────────────────────────────────────────────────────────────┐
│                    锁授予规则                                 │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Shared 锁授予条件:                                         │
│  ─────────────────────────────────────────────────────────  │
│  1. holders 为空 → 授予                                     │
│  2. holders 只有 Shared 且无 waiters → 授予                 │
│  3. 否则 → 加入 waiters                                    │
│                                                              │
│  Exclusive 锁授予条件:                                      │
│  ─────────────────────────────────────────────────────────  │
│  1. holders 和 waiters 都为空 → 授予                        │
│  2. 否则 → 加入 waiters                                    │
│                                                              │
│  Gap 锁特殊规则:                                            │
│  ─────────────────────────────────────────────────────────  │
│  Gap 锁之间不冲突 (只阻塞 INSERT)                          │
│  Gap 锁与同一 key 的 Record 锁不冲突                       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 3. Next-Key Lock 与 Gap Lock

### 3.1 Next-Key Lock 活动图

```
SELECT * FROM orders WHERE id = 5 FOR UPDATE
    │
    ▼
┌──────────────────────────────────────────────────┐
│ 1. InnoDB 兼容: 对 id=5 加 Next-Key Lock        │
│    ├── Record Lock: 锁定 id=5 的行              │
│    └── Gap Lock: 锁定 (3, 5) 间隙               │
│        (假设前一行 id=3)                          │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│ 2. 范围锁冲突检测                                │
│    ├── acquire_lock_with_target(NextKey(5))       │
│    ├── 遍历 range_locks 检查 Gap 重叠            │
│    ├── LockTarget::overlaps() 检查               │
│    └── 冲突 → 加入 waiters 或 Deadlock          │
└──────────────────────────────────────────────────┘
```

### 3.2 Gap Lock 冲突矩阵

```
           │  Gap(S)  │  Gap(X)  │  Record(S) │  Record(X) │  NextKey(S) │  NextKey(X)
───────────┼──────────┼──────────┼────────────┼────────────┼─────────────┼─────────────
Gap(S)     │    ✅    │    ✅    │     ✅     │     ✅     │     ✅      │     ✅
Gap(X)     │    ✅    │    ✅    │     ✅     │     ✅     │     ✅      │     ✅
Record(S)  │    ✅    │    ✅    │     ✅     │     ❌     │     ✅      │     ❌
Record(X)  │    ✅    │    ✅    │     ❌     │     ❌     │     ❌      │     ❌
NextKey(S) │    ✅    │    ✅    │     ✅     │     ❌     │     ✅      │     ❌
NextKey(X) │    ✅    │    ✅    │     ❌     │     ❌     │     ❌      │     ❌

注: Gap 锁之间始终兼容 (只阻塞 INSERT)
```

## 4. 死锁检测

### 4.1 Wait-For Graph

```rust
struct Inner {
    waits_for: HashMap<TxId, HashSet<TxId>>,
}

pub struct DeadlockDetector {
    inner: Mutex<Inner>,
    lock_wait_timeout: Duration,  // 5s
}
```

### 4.2 死锁检测时序图

```
T1 等待 T2, T2 等待 T1 → 死锁!

    ┌──────────────────────────────────────────┐
    │ DeadlockDetector.try_wait_edge(T1, {T2}) │
    │                                          │
    │ 1. Lock Mutex                            │
    │ 2. would_create_cycle(T1, {T2})?         │
    │    ├── DFS 从 T2 开始搜索 T1             │
    │    ├── T2 → waits_for[T2] = {T1}        │
    │    └── 找到 T1 → 有环!                   │
    │ 3. Return Err(LockError::Deadlock)       │
    │    (边未添加, T1 事务应回滚)             │
    └──────────────────────────────────────────┘
```

### 4.3 PROOF-023 原子性保证

```
try_wait_edge() TOCTOU-safe:

    ┌──────────────────────────────────────────┐
    │  Mutex Lock                              │
    │  ├── 1. Filter self-dependency           │
    │  ├── 2. Pre-check: would_create_cycle?   │
    │  │   └── DFS in Mutex → 无 TOCTOU 窗口  │
    │  ├── 3a. 有环 → Err(Deadlock)            │
    │  │        (边未添加)                      │
    │  └── 3b. 无环 → add_edge()               │
    │           (仍在 Mutex 内)                 │
    │  Mutex Unlock                            │
    └──────────────────────────────────────────┘
```

## 5. 算法复杂度与性能分析

### 5.1 操作复杂度

| 操作 | 复杂度 | 说明 |
|------|--------|------|
| acquire_lock | O(H + D) | H=holders数, D=DFS图遍历 |
| acquire_lock_with_target | O(R + D) | R=range_locks遍历, D=死锁检测 |
| release_lock | O(W) | W=waiters数 |
| LockTarget::overlaps | O(K) | K=key字节长度 |
| upgrade_lock | O(1) | 仅检查holders数量 |
| would_create_cycle | O(V + E) | DFS全图扫描 |
| try_wait_edge | O(V + E) | 原子 pre-check + add |
| remove_edges_for | O(V + E) | 删除出边+遍历入边 |

### 5.2 ⚠️ 已知问题

| 问题 | 严重性 | 影响 | 修复建议 |
|------|--------|------|---------|
| **范围锁线性扫描** | 🟡 中等 | O(R) 遍历所有 range_locks | 利用 BTreeMap 范围查询 |
| **DFS 全图扫描** | 🟡 中等 | 每次加锁都做 DFS | 缓存可达性信息 |
| **无锁等待超时** | 🟡 中等 | 事务可能无限等待 | 实现 LockTimeout |
| **Gap 与 Record 独立** | 🟡 中等 | 可能幻读 | 统一 Next-Key 语义 |
| **debug_assertions 开销** | 🟢 轻微 | O(V*(V+E)) | 仅 debug 构建影响 |

### 5.3 性能优化建议

```
优化1: BTreeMap 范围查询
  当前: acquire_lock_with_target 遍历所有 range_locks
  建议: 使用 BTreeMap::range() 只检查重叠区间
  预期: O(log R + K) 替代 O(R)

优化2: 增量死锁检测
  当前: 每次加锁都从零开始 DFS
  建议: 维护增量可达性矩阵或使用时间戳算法
  预期: O(1) 或 O(V) 替代 O(V+E)

优化3: 锁等待超时
  当前: LockError::LockTimeout 已定义但未使用
  建议: 实现 wait_with_timeout()
  预期: 防止事务无限等待
```

## 6. 与其他模块的依赖

```
LockManager
  ├── 依赖: transaction::deadlock::DeadlockDetector
  ├── 依赖: transaction::mvcc::TxId
  ├── 被依赖: TransactionManager (事务管理)
  ├── 被依赖: WalTransactionalExecutor (WAL 事务)
  └── 被依赖: MVCCStorage (并发控制)
```

## 变更记录

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-05-15 | v2.0 | 基于 GitNexus 重新分析，补充 Next-Key Lock、死锁检测 PROOF-023 |
