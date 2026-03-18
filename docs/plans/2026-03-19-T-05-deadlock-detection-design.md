# T-05 死锁检测设计

## 目标

实现死锁检测机制，防止事务无限等待。

## 方案

**主动检测 + 超时兜底 (A + B 组合)**

## 设计

### 1. 数据结构

```rust
struct DeadlockDetector {
    waits_for: HashMap<TxId, HashSet<TxId>>,
    lock_wait_timeout: Duration,
}
```

- `waits_for`: 等待图，key=被阻塞事务，value=该事务正在等待的事务集合
- `lock_wait_timeout`: 锁等待超时时间（默认 5s）

### 2. 触发时机

锁冲突导致事务阻塞时，立即触发死锁检测。

### 3. 检测算法

使用 DFS 检测等待图中的环路：

```
fn detect_cycle(&self, start: TxId) -> Option<Vec<TxId>> {
    let mut visited = HashSet::new();
    let mut path = Vec::new();
    self.dfs(start, &mut visited, &mut path)
}
```

### 4. Victim 选择

直接选择当前被阻塞的事务回滚（最简实现）。

### 5. 超时兜底

每把锁等待附带 timeout，超时后自动 abort 事务。

### 6. 集成

在 `LockManager` 中集成 `DeadlockDetector`，锁冲突时调用检测。

## 复杂度

- 检测: O(V + E)，V=事务数，E=等待边数
- 空间: O(E)

## 演进路径

- Phase 1: 最小可用版本（当前）
- Phase 2: 优化 victim 选择策略
