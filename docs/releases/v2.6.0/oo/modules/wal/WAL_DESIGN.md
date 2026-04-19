# WAL 模块设计

**版本**: v2.6.0
**模块**: WAL (Write-Ahead Logging)

---

## 一、What (是什么)

WAL (预写日志) 是一种确保数据持久化和崩溃恢复的机制。

## 二、Why (为什么)

- **崩溃恢复**: 系统崩溃后可通过重放 WAL 恢复
- **原子性**: 确保事务提交前所有操作可追溯
- **PITR**: 支持时间点恢复

## 三、How (如何实现)

### 3.1 WAL 条目

```rust
pub struct WalEntry {
    header: WalHeader,
    payload: Vec<u8>,
}

pub struct WalHeader {
    entry_type: EntryType,
    tx_id: u64,
    lsn: u64,
    prev_lsn: u64,
    timestamp: u64,
    crc: u32,
}
```

### 3.2 条目类型

```rust
pub enum EntryType {
    Begin,
    Insert,
    Update,
    Delete,
    Commit,
    Rollback,
    Checkpoint,
}
```

## 四、性能优化

v2.6.0 WAL 优化:
- 批量写入
- 异步刷盘
- 检查点优化

## 五、相关文档

- [ARCHITECTURE_V2.6.md](../../architecture/ARCHITECTURE_V2.6.md)

---

*WAL 模块设计 v2.6.0*
