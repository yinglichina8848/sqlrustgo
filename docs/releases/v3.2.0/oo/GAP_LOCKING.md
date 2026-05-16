# OO-12: Gap Locking 设计文档

> **版本**: v1.0
> **日期**: 2026-05-16
> **基于**: v3.2.0
> **维护人**: hermes-z6g4
> **状态**: 已完成

---

## 一、概述

### 1.1 目标

实现 Gap Locking (间隙锁) 机制，防止幻读问题：

- **幻读防止**: 锁定索引间隙
- **可串行化隔离**: 支持 SERIALIZABLE 隔离级别
- **死锁预防**: 锁排序与超时机制

### 1.2 核心理念

```
Gap Locking = 记录锁 + 间隙锁 + Next-Key Locking
```

---

## 二、技术架构

### 2.1 组件关系

```
┌─────────────────────────────────────────────────────────────────┐
│                    Gap Locking System                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │   Lock      │───▶│   Gap        │───▶│   Next-Key       │  │
│  │   Manager   │    │   Generator  │    │   Locking        │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│         │                   │                      │            │
│         │                   │                      │            │
│         ▼                   ▼                      ▼            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │   Lock       │    │   Index     │    │   Transaction   │  │
│  │   Table      │    │   Structure │    │   Wait/Die      │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、锁类型

### 3.1 锁层次

| 锁类型 | 锁定范围 | 兼容性 |
|--------|----------|--------|
| Record Lock | 单条索引记录 | 与同记录互斥 |
| Gap Lock | 索引间隙 | 与插入互斥 |
| Next-Key Lock | 记录 + 前间隙 | 防止幻读 |

### 3.2 锁模式

```rust
/// Gap Lock 模式
#[derive(Debug, Clone, Copy)]
pub enum GapLockMode {
    /// 共享间隙锁 - 允许读取，不允许插入
    Shared,
    /// 排他间隙锁 - 不允许读取和插入
    Exclusive,
}

/// Next-Key Lock = Record Lock + Gap Lock
pub struct NextKeyLock {
    pub record: Key,
    pub gap_end: Key,  // 间隙结束键 (不包含)
}
```

---

## 四、锁定算法

### 4.1 锁定过程

```rust
/// 获取 Next-Key Lock
pub fn lock_next_key(
    tx: &mut Transaction,
    index: IndexId,
    key: &Key,
) -> Result<LockHandle> {
    // 1. 获取记录锁
    let record_lock = self.lock_record(tx, index, key)?;
    // 2. 获取间隙锁 (到下一个键之前)
    let gap_end = self.find_next_key(index, key)?;
    let gap_lock = self.lock_gap(tx, index, key, gap_end)?;
    // 3. 返回组合锁
    Ok(LockHandle::next_key(record_lock, gap_lock))
}
```

### 4.2 索引结构

```
B+ Tree: [1, 5, 10, 15, 20, 25]

锁定 key = 15 时:
├── Record Lock: 锁定 key=15 的记录
├── Gap Lock: 锁定 (10, 15) 和 (15, 20) 间隙
└── Next-Key Lock: 锁定 (-∞, 15] 和 (15, 20)
```

---

## 五、隔离级别集成

### 5.1 隔离级别与锁

| 隔离级别 | Gap Locking | 说明 |
|----------|-------------|------|
| READ UNCOMMITTED | 不使用 | 脏读允许 |
| READ COMMITTED | 只使用 Record Lock | 可能幻读 |
| REPEATABLE READ | 使用 Next-Key Lock | 防止幻读 (MySQL 默认) |
| SERIALIZABLE | 全部锁定 | 串行执行 |

### 5.2 触发条件

```rust
/// 判断是否需要 Gap Locking
pub fn requires_gap_lock(isolation: IsolationLevel, op: &Operator) -> bool {
    match isolation {
        IsolationLevel::RepeatableRead => true,
        IsolationLevel::Serializable => true,
        IsolationLevel::ReadCommitted => {
            // 等值条件不使用间隙锁
            !matches!(op, Operator::Eq)
        }
        _ => false,
    }
}
```

---

## 六、死锁处理

### 6.1 Wait-Die 机制

```rust
/// 老事务等待新事务 - 死亡
pub fn wait_or_die(tx: &Transaction, other: &Transaction, lock: &Lock) -> Result<()> {
    if tx.start_ts < other.start_ts {
        // 老事务等待
        self.wait_for(lock, other)?;
    } else {
        // 新事务死亡
        Err(TransactionError::Deadlock)
    }
}
```

### 6.2 超时机制

```rust
/// 锁等待超时配置
pub struct LockWaitConfig {
    /// 等待超时 (毫秒)
    pub wait_timeout: u64,
    /// 死锁检测间隔
    pub deadlock_detection_interval: u64,
}

impl Default for LockWaitConfig {
    fn default() -> Self {
        Self {
            wait_timeout: 5000,  // 5 秒
            deadlock_detection_interval: 1000,
        }
    }
}
```

---

## 七、相关 Issue

| Issue | 功能 | 状态 |
|-------|------|------|
| #1031 | Gap Lock 基础实现 | ✅ 完成 |
| #1032 | Next-Key Locking | ✅ 完成 |
| #1033 | 死锁检测 | ✅ 完成 |

---

*本文档由 hermes-z6g4 维护*
*版本 1.0 - 2026-05-16*