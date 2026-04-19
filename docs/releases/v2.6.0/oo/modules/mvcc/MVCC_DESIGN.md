# MVCC 模块设计

**版本**: v2.6.0
**模块**: MVCC (Multi-Version Concurrency Control)

---

## 一、What (是什么)

MVCC (多版本并发控制) 是一种并发控制机制，v2.6.0 实现 Serializable Snapshot Isolation (SSI)，提供最强的事务隔离级别。

## 二、Why (为什么)

- **读写不阻塞**: 读者不需要获取锁
- **快照隔离**: 每个事务看到某个时间点的数据快照
- **SSI 支持**: v2.6.0 实现可串行化快照隔离

## 三、How (如何实现)

### 3.1 SSI 实现

```rust
// SSI 事务状态
pub struct SSITransaction {
    tx_id: u64,
    start_ts: u64,
    commit_ts: u64,
    status: TxStatus,
    read_set: Vec<ReadRecord>,
    write_set: Vec<WriteRecord>,
    depend_on: Vec<u64>,  // 依赖的事务
}

// 冲突检测
impl SSITransaction {
    pub fn check_conflict(&self, other: &SSITransaction) -> bool {
        // 检查读写冲突
        for read in &self.read_set {
            for write in &other.write_set {
                if read.key == write.key && self.overlaps(read.ts, write.ts) {
                    return true;
                }
            }
        }
        false
    }
}
```

### 3.2 版本链

```rust
pub struct VersionNode {
    tx_id: u64,
    start_ts: u64,
    commit_ts: u64,
    status: TxStatus,
    data: Vec<u8>,
    prev: Option<Box<VersionNode>>,
}
```

## 四、相关文档

- [ARCHITECTURE_V2.6.md](../architecture/ARCHITECTURE_V2.6.md)

---

*MVCC 模块设计 v2.6.0*
