# MVCC 模块设计

**版本**: v2.5.0
**模块**: MVCC (Multi-Version Concurrency Control)

---

## 一、What (是什么)

MVCC (多版本并发控制) 是一种并发控制机制，通过维护数据的多版本，实现读写不阻塞，提高数据库并发性能。

## 二、Why (为什么)

- **读写不阻塞**: 读者不需要获取锁，可以读取历史版本
- **写不阻塞读**: 写入创建新版本，不影响读者
- **快照隔离**: 每个事务看到的是某个时间点的数据快照
- **回滚方便**: 只需丢弃版本，无需修改数据

## 三、How (如何实现)

### 3.1 核心数据结构

```rust
// 版本链节点
struct VersionNode {
    tx_id: u64,           // 事务 ID
    start_ts: u64,         // 开始时间戳
    commit_ts: u64,        // 提交时间戳 (0 表示未提交)
    status: TxStatus,      // 事务状态
    data: Vec<u8>,         // 版本数据
    prev: Option<Box<VersionNode>>,  // 前一个版本
}

// 版本链管理器
struct VersionChainMap {
    chains: HashMap<Key, VersionChain>,
    gc_sender: Sender<()>,
}

// 快照
struct Snapshot {
    tx_id: u64,            // 事务 ID
    start_ts: u64,         // 开始时间戳
    active_txs: Vec<u64>, // 活跃事务列表
    min_commit_ts: u64,    // 最小提交时间戳
}
```

### 3.2 读取流程

```
1. 事务开始，获取 start_ts
2. 读取时，遍历版本链:
   - 跳过未提交版本
   - 跳过 commit_ts > start_ts 的版本
   - 找到第一个满足条件的版本
3. 返回版本数据
```

### 3.3 写入流程

```
1. 事务开始，获取 tx_id 和 start_ts
2. 写入时，创建新版本节点:
   - tx_id = 当前事务 ID
   - start_ts = start_ts
   - commit_ts = 0 (未提交)
3. 提交时，更新 commit_ts:
   - 遍历版本链，标记当前版本
   - 设置 commit_ts = 当前时间戳
4. 返回提交成功
```

### 3.4 GC (垃圾回收)

```
1. 定期检查版本链
2. 找出不再需要的旧版本:
   - 没有活跃事务在 start_ts 之前开始
   - 版本已提交且足够老
3. 断开旧版本的链接
4. 释放内存
```

## 四、接口设计

### 4.1 公开 API

```rust
impl MVCCStorage {
    // 创建 MVCC 存储
    pub fn new(inner: Storage) -> Self;

    // 读取指定快照的数据
    pub fn read(&self, key: &Key, snapshot: &Snapshot) -> Result<Option<Vec<u8>>>;

    // 写入数据 (创建新版本)
    pub fn write(&self, key: &Key, value: Vec<u8>, tx: &Transaction) -> Result<()>;

    // 提交事务
    pub fn commit(&self, tx: &Transaction) -> Result<()>;

    // 回滚事务
    pub fn rollback(&self, tx: &Transaction) -> Result<()>;

    // 获取当前快照
    pub fn snapshot(&self, tx_id: u64) -> Snapshot;
}
```

### 4.2 错误类型

```rust
pub enum MVCCError {
    WriteConflict { key: Key, tx_id: u64 },
    TransactionNotFound { tx_id: u64 },
    VersionNotFound { key: Key },
    StorageError(StorageError),
}
```

## 五、性能考虑

| 操作 | 时间复杂度 | 说明 |
|------|------------|------|
| 读取 | O(version_count) | 最坏 O(n)，通常 O(1) |
| 写入 | O(1) | 链表头部插入 |
| GC | O(version_count) | 增量回收 |

### 优化策略

1. **版本数量限制**: 超过阈值触发 GC
2. **索引优化**: 使用 HashMap 加速版本链查找
3. **批量 GC**: 合并多次 GC 操作
4. **内存池**: 预分配版本节点，减少分配开销

## 六、测试策略

| 测试类型 | 测试点 |
|----------|--------|
| 基本读写 | 单线程读写验证 |
| 并发读写 | 多线程同时读写 |
| 快照一致性 | 验证快照隔离 |
| 写冲突 | 检测写-写冲突 |
| GC 正确性 | 验证旧版本清理 |
| 崩溃恢复 | WAL 恢复验证 |

## 七、相关文档

- [MVCC_DESIGN.md](../../MVCC_DESIGN.md) - 详细设计文档
- [MVCC_DESIGN.md](./WAL_DESIGN.md) - WAL 集成

---

*MVCC 模块设计 v2.5.0*
