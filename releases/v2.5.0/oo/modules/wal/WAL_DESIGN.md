# WAL 模块设计

**版本**: v2.5.0
**模块**: WAL (Write-Ahead Logging)

---

## 一、What (是什么)

WAL (预写日志) 是一种确保数据持久化的机制。在数据写入磁盘前，先将操作记录写入日志，确保崩溃后可以通过重放日志恢复数据。

## 二、Why (为什么)

- **崩溃恢复**: 系统崩溃后可以通过重放 WAL 恢复数据
- **原子性**: 确保事务提交前所有操作可追溯
- **性能**: 批量写入日志比随机写数据文件快
- **PITR**: 支持时间点恢复

## 三、How (如何实现)

### 3.1 核心数据结构

```rust
// WAL 条目
struct WalEntry {
    header: WalHeader,
    payload: Vec<u8>,
}

struct WalHeader {
    entry_type: EntryType,
    tx_id: u64,
    lsn: u64,           // Log Sequence Number
    prev_lsn: u64,
    timestamp: u64,
    crc: u32,
}

enum EntryType {
    Begin,
    Insert,
    Update,
    Delete,
    Commit,
    Rollback,
    Checkpoint,
}
```

### 3.2 写入流程

```
1. 事务开始，写入 BEGIN
2. 每次修改，写入:
   - INSERT: 数据内容
   - UPDATE: 旧值 + 新值
   - DELETE: 旧值
3. 事务提交，写入 COMMIT
4. 定期写 CHECKPOINT
```

### 3.3 恢复流程

```
1. 系统启动时检查 WAL
2. 分析日志确定恢复点:
   - 如果有 CHECKPOINT，从 CHECKPOINT 开始
   - 否则从头开始
3. 重放日志条目:
   - BEGIN + COMMIT: 提交事务
   - BEGIN + 无 COMMIT: 回滚事务
   - DML 操作: 应用到数据库
4. 完成恢复
```

### 3.4 PITR (时间点恢复)

```rust
impl WalManager {
    // 恢复到指定时间戳
    pub fn recover_to_timestamp(&self, timestamp: u64) -> Result<()>;

    // 恢复到指定 LSN
    pub fn recover_to_lsn(&self, lsn: u64) -> Result<()>;

    // 列出可用时间点
    pub fn list_recovery_points(&self) -> Vec<RecoveryPoint>;
}
```

## 四、接口设计

### 4.1 公开 API

```rust
impl WalManager {
    // 创建 WAL 管理器
    pub fn new(config: WalConfig) -> Result<Self>;

    // 追加日志条目
    pub fn append(&self, entry: WalEntry) -> Result<u64>;

    // 刷新日志到磁盘
    pub fn flush(&self) -> Result<()>;

    // 创建检查点
    pub fn checkpoint(&self) -> Result<()>;

    // 恢复到检查点
    pub fn recover_from_checkpoint(&self, checkpoint: Checkpoint) -> Result<()>;

    // 恢复到时间点
    pub fn recover_to_timestamp(&self, ts: u64) -> Result<()>;

    // 获取 WAL 状态
    pub fn status(&self) -> WalStatus;
}
```

### 4.2 配置

```rust
struct WalConfig {
    dir: PathBuf,                    // WAL 目录
    file_size: usize,               // 单个文件大小 (默认 256MB)
    sync_mode: SyncMode,            // 同步模式
    pitr_enabled: bool,             // 是否启用 PITR
    archive_enabled: bool,           // 是否启用归档
    archive_retention_days: u32,     // 归档保留天数
}

enum SyncMode {
    Fsync,      // 每次写入都 fsync
    Fdatasync,  // 使用 fdatasync
    None,       // 不同步 (风险高)
}
```

## 五、性能考虑

| 操作 | 时间复杂度 | 说明 |
|------|------------|------|
| append | O(1) | 顺序追加 |
| flush | O(write_size) | 批量刷盘 |
| checkpoint | O(log_size) | 需要遍历日志 |
| recovery | O(log_size) | 取决于日志长度 |

### 优化策略

1. **批量写入**: 合并多次写操作
2. **异步刷盘**: 后台线程定期刷盘
3. **日志压缩**: 定期清理已提交的日志
4. **检查点优化**: 增量检查点

## 六、相关文档

- [MVCC_DESIGN.md](../mvcc/MVCC_DESIGN.md) - MVCC 集成
- *(已归档 - PITR 恢复文档不存在)*

---

*WAL 模块设计 v2.5.0*
