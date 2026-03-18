# T-05 死锁检测实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现死锁检测机制，防止事务无限等待

**Architecture:** 使用 Wait-For Graph + DFS 环路检测 + 超时兜底

**Tech Stack:** Rust, HashMap, 事务模块

---

### Task 1: 创建 DeadlockDetector 结构体

**Files:**
- Create: `crates/transaction/src/deadlock.rs`

**Step 1: 编写测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deadlock_detector_new() {
        let detector = DeadlockDetector::new();
        assert!(detector.waits_for.is_empty());
    }

    #[test]
    fn test_add_edge() {
        let mut detector = DeadlockDetector::new();
        detector.add_edge(TxId::new(1), TxId::new(2));
        assert!(detector.waits_for.get(&TxId::new(1)).unwrap().contains(&TxId::new(2)));
    }
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test -p sqlrustgo-transaction deadlock::tests::test_deadlock_detector_new`
Expected: FAIL

**Step 3: 编写实现**

```rust
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use crate::mvcc::TxId;

pub struct DeadlockDetector {
    waits_for: HashMap<TxId, HashSet<TxId>>,
    lock_wait_timeout: Duration,
}

impl DeadlockDetector {
    pub fn new() -> Self {
        Self {
            waits_for: HashMap::new(),
            lock_wait_timeout: Duration::from_secs(5),
        }
    }

    pub fn add_edge(&mut self, blocked: TxId, holder: TxId) {
        self.waits_for.entry(blocked).or_default().insert(holder);
    }

    pub fn remove_edges_for(&mut self, tx_id: TxId) {
        self.waits_for.remove(&tx_id);
        for holders in self.waits_for.values_mut() {
            holders.remove(&tx_id);
        }
    }
}
```

**Step 4: 运行测试确认通过**

Run: `cargo test -p sqlrustgo-transaction deadlock::tests`
Expected: PASS

**Step 5: 提交**

```bash
git add crates/transaction/src/deadlock.rs
git commit -m "feat(transaction): T-05 添加 DeadlockDetector 结构体"
```

---

### Task 2: 实现 DFS 环路检测

**Files:**
- Modify: `crates/transaction/src/deadlock.rs`

**Step 1: 编写测试**

```rust
#[test]
fn test_detect_cycle() {
    let mut detector = DeadlockDetector::new();
    // T1 -> T2 -> T3 -> T1 (cycle)
    detector.add_edge(TxId::new(1), TxId::new(2));
    detector.add_edge(TxId::new(2), TxId::new(3));
    detector.add_edge(TxId::new(3), TxId::new(1));
    
    let cycle = detector.detect_cycle(TxId::new(1));
    assert!(cycle.is_some());
}

#[test]
fn test_no_cycle() {
    let mut detector = DeadlockDetector::new();
    detector.add_edge(TxId::new(1), TxId::new(2));
    detector.add_edge(TxId::new(2), TxId::new(3));
    
    let cycle = detector.detect_cycle(TxId::new(1));
    assert!(cycle.is_none());
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test -p sqlrustgo-transaction deadlock::tests::test_detect_cycle`
Expected: FAIL

**Step 3: 编写实现**

```rust
impl DeadlockDetector {
    pub fn detect_cycle(&self, start: TxId) -> Option<Vec<TxId>> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        self.dfs(start, &mut visited, &mut path)
    }

    fn dfs(&self, current: TxId, visited: &mut HashSet<TxId>, path: &mut Vec<TxId>) -> Option<Vec<TxId>> {
        if path.contains(&current) {
            // Found cycle, return from current to end
            let idx = path.iter().position(|&x| x == current).unwrap();
            return Some(path[idx..].to_vec());
        }

        if visited.contains(&current) {
            return None;
        }

        visited.insert(current);
        path.push(current);

        if let Some(holders) = self.waits_for.get(&current) {
            for &holder in holders {
                if let Some(cycle) = self.dfs(holder, visited, path) {
                    return Some(cycle);
                }
            }
        }

        path.pop();
        None
    }
}
```

**Step 4: 运行测试确认通过**

Run: `cargo test -p sqlrustgo-transaction deadlock::tests::test_detect_cycle`
Expected: PASS

**Step 5: 提交**

```bash
git add crates/transaction/src/deadlock.rs
git commit -m "feat(transaction): T-05 实现 DFS 环路检测"
```

---

### Task 3: 集成到 LockManager

**Files:**
- Modify: `crates/transaction/src/lock.rs`

**Step 1: 编写测试**

```rust
#[test]
fn test_lock_with_deadlock_detection() {
    use crate::deadlock::DeadlockDetector;
    let mut lock_manager = LockManager::new();
    let detector = Arc::new(RwLock::new(DeadlockDetector::new()));
    
    // T1 holds lock on key1
    lock_manager.acquire_lock(TxId::new(1), vec![1], LockMode::Exclusive).unwrap();
    // T2 tries to acquire lock on key1 (held by T1)
    lock_manager.acquire_lock(TxId::new(2), vec![1], LockMode::Exclusive).unwrap();
    
    let detector = detector.read().unwrap();
    let cycle = detector.detect_cycle(TxId::new(2));
    assert!(cycle.is_some());
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test -p sqlrustgo-transaction lock::tests::test_lock_with_deadlock_detection`
Expected: FAIL

**Step 3: 集成实现**

在 LockManager 中添加:
```rust
pub struct LockManager {
    locks: HashMap<Vec<u8>, LockInfo>,
    tx_locks: HashMap<TxId, HashSet<Vec<u8>>>,
    deadlock_detector: Arc<RwLock<DeadlockDetector>>,
}

impl LockManager {
    pub fn with_deadlock_detector(detector: Arc<RwLock<DeadlockDetector>>) -> Self {
        Self {
            locks: HashMap::new(),
            tx_locks: HashMap::new(),
            deadlock_detector: detector,
        }
    }
    
    pub fn detect_deadlock(&self, blocked_tx: TxId) -> Option<Vec<TxId>> {
        let detector = self.deadlock_detector.read().unwrap();
        detector.detect_cycle(blocked_tx)
    }
}
```

**Step 4: 运行测试确认通过**

Run: `cargo test -p sqlrustgo-transaction lock::tests::test_lock_with_deadlock_detection`
Expected: PASS

**Step 5: 提交**

```bash
git add crates/transaction/src/lock.rs
git commit -m "feat(transaction): T-05 集成死锁检测到 LockManager"
```

---

### Task 4: 添加超时兜底机制

**Files:**
- Modify: `crates/transaction/src/deadlock.rs`

**Step 1: 编写测试**

```rust
#[test]
fn test_timeout() {
    let mut detector = DeadlockDetector::with_timeout(Duration::from_secs(1));
    assert_eq!(detector.lock_wait_timeout, Duration::from_secs(1));
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test -p sqlrustgo-transaction deadlock::tests::test_timeout`
Expected: FAIL

**Step 3: 编写实现**

```rust
impl DeadlockDetector {
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            waits_for: HashMap::new(),
            lock_wait_timeout: timeout,
        }
    }
    
    pub fn get_timeout(&self) -> Duration {
        self.lock_wait_timeout
    }
}
```

**Step 4: 运行测试确认通过**

Run: `cargo test -p sqlrustgo-transaction deadlock::tests::test_timeout`
Expected: PASS

**Step 5: 提交**

```bash
git add crates/transaction/src/deadlock.rs
git commit -m "feat(transaction): T-05 添加超时兜底机制"
```

---

### Task 5: 更新 lib.rs 和最终测试

**Files:**
- Modify: `crates/transaction/src/lib.rs`

**Step 1: 添加导出**

```rust
pub mod deadlock;

pub use deadlock::DeadlockDetector;
```

**Step 2: 运行完整测试**

Run: `cargo test -p sqlrustgo-transaction`
Expected: ALL PASS

**Step 3: 运行 clippy**

Run: `cargo clippy -p sqlrustgo-transaction`
Expected: NO WARNINGS

**Step 4: 提交**

```bash
git add crates/transaction/src/lib.rs
git commit -m "feat(transaction): T-05 死锁检测完成

- DeadlockDetector 结构体
- Wait-For Graph 维护
- DFS 环路检测
- 超时兜底机制
- 集成到 LockManager"
```

---

### Task 6: 创建 PR

```bash
gh pr create --base develop/v1.6.0 --title "feat(transaction): T-05 死锁检测实现" --body "..."
```

---

## 验收标准

- [ ] DeadlockDetector 结构体正常工作
- [ ] DFS 环路检测正确
- [ ] 集成到 LockManager
- [ ] 超时机制可用
- [ ] 所有测试通过
- [ ] Clippy 无警告
- [ ] PR 创建成功
