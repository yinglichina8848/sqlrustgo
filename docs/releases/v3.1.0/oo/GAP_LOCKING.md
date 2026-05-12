# Gap Locking 实现文档

> **版本**: v3.1.0
> **日期**: 2026-05-12
> **Issue**: #607
> **状态**: 规划中

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

### 2.1 现有锁实现

根据 `crates/transaction/src/lock.rs`：

```rust
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
| Gap Lock | ❌ 未实现 | 需要新增 |
| Next-Key Lock | ❌ 未实现 | 需要新增 |

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

### 3.1 当前执行链路

```
SQL: UPDATE t SET val = 100 WHERE id > 10 AND id < 30
  ↓
Parser: 解析 WHERE 条件
  ↓
Planner: 生成 logical plan
  ↓
Optimizer: 选择索引
  ↓
Executor:
  ├→ LockManager::lock(key, Exclusive)
  │   └→ 检查 holders 和 waiters
  │       └→ 如果可授予，添加到 holders
  │
  └→ StorageEngine::update()
      └→ 修改数据
```

### 3.2 Gap Lock 执行链路（规划）

```
SQL: UPDATE t SET val = 100 WHERE id > 10 AND id < 30
  ↓
Executor:
  ├→ 计算索引范围: (10, 30)
  │   ├→ 锁定 Gap (10, 20)
  │   ├→ 锁定 Gap (20, 30)
  │   └→ 锁定 Record 20, 30
  │
  ├→ NextKeyLock::lock_range(start, end, Exclusive)
  │   └→ 对每个 Gap 和 Record 加锁
  │
  └→ StorageEngine::update()
      └→ 修改数据
```

---

## 四、数据结构

### 4.1 当前 LockInfo

```rust
pub struct LockInfo {
    key: Vec<u8>,              // 锁定的 key
    mode: LockMode,             // 锁模式
    holders: HashSet<TxId>,     // 持有者
    waiters: Vec<(TxId, LockMode)>,  // 等待者
}
```

### 4.2 建议的 Gap Lock 结构

```rust
pub enum LockType {
    Gap(Vec<u8>),           // Gap 锁，key 是起始值
    Record(Vec<u8>),        // 记录锁
    NextKey(Vec<u8>),       // Next-Key 锁
}

pub struct GapLockInfo {
    pub lock_type: LockType,
    pub start_key: Vec<u8>,
    pub end_key: Vec<u8>,    // None 表示 +∞
    pub holders: HashSet<TxId>,
    pub waiters: Vec<(TxId, LockMode)>,
}
```

---

## 五、实现计划

### 5.1 Phase 1: 基础 Gap Lock

- [ ] 在 `LockMode` 中添加 `Gap` 变体
- [ ] 实现 `lock_gap(start, end)` 函数
- [ ] 实现 Gap 锁的释放
- [ ] 添加 Gap 锁的测试

### 5.2 Phase 2: Next-Key Lock

- [ ] 实现 `lock_next_key(start, end)` 函数
- [ ] 集成到 UPDATE/DELETE 执行器
- [ ] 处理索引扫描的锁升级
- [ ] 添加 Next-Key Lock 测试

### 5.3 Phase 3: 唯一性约束

- [ ] 实现唯一索引的 Gap Lock
- [ ] 处理 INSERT 的锁逻辑
- [ ] 添加唯一性约束测试

---

## 六、测试计划

### 6.1 单元测试

| 测试 | 描述 |
|------|------|
| `test_gap_lock_basic` | 基本 Gap 锁获取/释放 |
| `test_gap_lock_overlap` | 重叠范围的 Gap 锁 |
| `test_next_key_lock` | Next-Key 锁测试 |
| `test_lock_compatibility` | 锁兼容性测试 |

### 6.2 集成测试

| 测试 | 描述 |
|------|------|
| `test_phantom_read_prevention` | 验证幻读被阻止 |
| `test_write_skew_prevention` | 验证写偏斜被阻止 |
| `test_unique_constraint` | 唯一性约束测试 |

---

## 七、参考

- `crates/transaction/src/lock.rs` - 锁管理器实现
- `crates/transaction/src/lock_manager.rs` - 锁管理器封装
- `crates/transaction/src/deadlock.rs` - 死锁检测