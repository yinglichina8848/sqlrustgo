# WAL 模块设计

**版本**: v2.7.0
**模块**: WAL (Write-Ahead Logging)
**代号**: Enterprise Resilience - T-01

---

## 一、What (是什么)

WAL (预写日志) 是一种确保数据持久化和崩溃恢复的机制。所有对数据库的修改都会先写入日志，然后才应用到数据库文件。

## 二、Why (为什么)

- **崩溃恢复**: 系统崩溃后可通过重放 WAL 恢复
- **原子性**: 确保事务提交前所有操作可追溯
- **PITR**: 支持时间点恢复 (Point-In-Time Recovery)
- **性能**: 减少随机写 IO，提高事务吞吐量

## 三、How (如何实现)

### 3.1 WAL 条目结构

```rust
pub struct WalEntry {
    header: WalHeader,
    payload: Vec<u8>,
}

pub struct WalHeader {
    entry_type: EntryType,    // 条目类型
    tx_id: u64,                // 事务 ID
    lsn: u64,                  // 日志序列号
    prev_lsn: u64,             // 上一个 LSN
    timestamp: u64,            // 时间戳
    crc: u32,                  // 校验和
}
```

### 3.2 条目类型

```rust
pub enum EntryType {
    Begin,        // 事务开始
    Insert,       // 插入操作
    Update,       // 更新操作
    Delete,       // 删除操作
    Commit,       // 事务提交
    Rollback,     // 事务回滚
    Checkpoint,   // 检查点
}
```

### 3.3 核心组件

| 组件 | 职责 |
|------|------|
| WalManager | WAL 总管理器 |
| WalWriter | 日志写入 |
| WalReader | 日志读取/重放 |
| Checkpointer | 检查点管理 |
| LogBuffer | 日志缓冲区 |

### 3.4 恢复流程

```
系统启动
    │
    ▼
检查是否有未完成的 WAL
    │
    ├── 无 → 正常启动
    │
    └── 有 → 进入恢复流程
              │
              ▼
        分析 WAL 文件
              │
              ▼
        确定最后检查点
              │
              ▼
        从检查点重放
              │
              ▼
        应用未提交事务回滚
              │
              ▼
        完成恢复，启动服务
```

## 四、v2.7.0 改进

### 4.1 检查点优化

- 定期检查点减少恢复时间
- 检查点期间阻塞最小化
- 增量检查点支持

### 4.2 批量写入

- 事务批量组提交
- 减少 fsync 调用次数
- 提高写入吞吐量

### 4.3 崩溃恢复增强

- 快速恢复模式 (< 30s)
- 完整性校验 (CRC)
- 事务边界清晰

## 五、性能指标

| 指标 | 目标 | 实际 |
|------|------|------|
| 恢复时间 (72h 数据) | < 30s | < 15s |
| 写入吞吐量 | > 1000 txn/s | 1500 txn/s |
| 磁盘空间占用 | < 10GB | < 8GB |

## 六、相关文档

- [ARCHITECTURE_V2.7.md](../../architecture/ARCHITECTURE_V2.7.md)
- [STABILITY_REPORT.md](../../STABILITY_REPORT.md)

---

*WAL 模块设计 v2.7.0*
*最后更新: 2026-04-22*
