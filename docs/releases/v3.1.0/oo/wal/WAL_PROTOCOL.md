# WAL 协议详解 (v3.1.0)

> **基于 GitNexus 分析** | 67,755 符号, 102,165 关系
> Write-Ahead Logging: 持久化保证、恢复、归档

## 1. WAL 架构

### 1.1 核心数据结构

```rust
pub struct WalEntry {
    pub tx_id: u64,
    pub entry_type: WalEntryType,
    pub table_id: u64,
    pub key: Option<Vec<u8>>,
    pub data: Option<Vec<u8>>,
    pub lsn: u64,
    pub timestamp: u64,
}

pub enum WalEntryType {
    Begin = 1, Insert = 2, Update = 3, Delete = 4,
    Commit = 5, Rollback = 6, Checkpoint = 7, Prepare = 8,
}

pub struct WalWriter {
    writer: BufWriter<File>,
    lsn: u64,
    batch_mode: bool,
    records_since_flush: usize,
    flush_threshold: usize,
}

pub struct GroupCommitWriter {
    inner: WalWriter,
    pending_commits: Vec<u64>,
    max_batch_size: usize,
}

pub struct WalArchiveManager {
    wal_dir: PathBuf,
    archive_dir: PathBuf,
    archive_id: u64,
    enable_compression: bool,
    max_archive_age_secs: u64,
    max_archive_size_bytes: u64,
}
```

### 1.2 WAL 特性

| 特性 | 值 | 说明 |
|------|-----|------|
| LSN | u64 单调递增 | 日志序列号 |
| 批量写入 | batch_mode | BufWriter 缓冲 |
| flush_threshold | 100 | 每 100 条 fsync |
| Group Commit | ✅ | 批量提交减少 fsync |
| 归档压缩 | miniz_oxide deflate | 7 天/100MB 自动归档 |
| 2PC 支持 | Prepare 条目 | 分布式事务 |

### 1.3 关键文件

| 文件 | 行数 | 作用 |
|------|------|------|
| [wal.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/storage/src/wal.rs) | 2143 | WAL 管理器核心 |
| [wal_storage.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/storage/src/wal_storage.rs) | - | WAL 存储适配器 |
| [pitr_recovery.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/storage/src/pitr_recovery.rs) | - | PITR 时间点恢复 |
| [recovery/replay_verifier.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/storage/src/recovery/replay_verifier.rs) | - | 恢复验证 |

## 2. WAL 写入链路

### 2.1 写入时序图

```
Transaction: INSERT INTO orders VALUES (1, 'test')
    │
    ▼
┌──────────────────────────────────────────────────┐
│ 1. WalStorage.insert()                           │
│    ├── WalManager.log_begin(tx_id)               │
│    │   └── append(Begin { tx_id, lsn, ts })     │
│    └── WalManager.log_insert(tx_id, table, data) │
│        └── append(Insert { tx_id, key, data })   │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│ 2. WalWriter.append(entry)                       │
│    ├── 编码: [len(4B)][type(1B)][tx_id(8B)]...  │
│    ├── writer.write_all(&encoded)                │
│    ├── records_since_flush++                     │
│    └── if !batch_mode: flush() + fsync()        │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│ 3. COMMIT                                        │
│    ├── WalManager.log_commit(tx_id)              │
│    └── GroupCommitWriter.flush_pending()         │
│        └── 批量 fsync (减少 IO 次数)            │
└──────────────────────────────────────────────────┘
```

### 2.2 WAL 条目编码格式

```
┌──────────┬──────────┬──────────┬──────────┬──────────┬──────────┐
│  Length   │  Type    │  TxId    │  TableId │  Key     │  Data    │
│  (4B)     │  (1B)    │  (8B)    │  (8B)    │  (var)   │  (var)   │
└──────────┴──────────┴──────────┴──────────┴──────────┴──────────┘

Length: 总条目长度 (4 bytes, big-endian)
Type: 1=Begin, 2=Insert, 3=Update, 4=Delete, 5=Commit, 6=Rollback, 7=Checkpoint, 8=Prepare
```

### 2.3 Group Commit 活动

```
    ┌──────────────────────────────────┐
    │      多个事务同时 COMMIT          │
    └──────────────┬───────────────────┘
                   │
                   ▼
    ┌──────────────────────────────────┐
    │  pending_commits.push(tx_id)     │
    │  等待更多事务加入批量            │
    └──────────────┬───────────────────┘
                   │
            ┌──────┴──────┐
            │ batch 满 或 │
            │ 超时?       │
            └──┬──────┬───┘
            YES│      │NO
               ▼      │
    ┌──────────────────┐
    │  单次 fsync      │
    │  提交所有 pending │
    │  事务            │
    └──────────────────┘
```

## 3. 崩溃恢复链路

### 3.1 恢复时序图

```
Server Restart
    │
    ▼
┌──────────────────────────────────────────────────┐
│ 1. WalManager.recover()                          │
│    ├── 读取 WAL 文件                              │
│    ├── 逐条解析 WalEntry                          │
│    └── 返回 Vec<WalEntry>                         │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│ 2. Recovery.scan_incomplete_transactions()       │
│    ├── 构建 tx_map: HashMap<tx_id, TxState>      │
│    ├── Begin → 记录事务开始                       │
│    ├── Prepare → has_prepare = true               │
│    ├── Commit → has_commit = true                 │
│    └── Rollback → has_rollback = true             │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│ 3. 不完整事务识别                                 │
│    ├── 有 Prepare 无 Commit/Rollback              │
│    │   → 已 Prepare 未决定 (2PC 恢复)            │
│    └── 无 Prepare/Commit/Rollback                 │
│        → 已开始未完成 (需要回滚)                  │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│ 4. WalStorage.recover()                          │
│    ├── 已提交事务: 重放 Insert/Update/Delete      │
│    └── 未提交事务: 忽略 (或回滚)                  │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│ 5. 恢复验证                                      │
│    ├── manifest_valid: RecoveryManifest 校验      │
│    ├── wal_chain_valid: WalChainState 校验        │
│    └── page_checksums_valid: PageChecksumStore    │
└──────────────────────────────────────────────────┘
```

### 3.2 崩溃注入测试点

```
CrashPoint 枚举:
├── BeforeWalWrite    → WAL 写入前崩溃 (数据丢失, 可恢复)
├── AfterWalWrite     → WAL 写入后、提交前 (重放恢复)
├── BeforeCommit      → 提交标记前 (事务回滚)
├── AfterCommit       → 提交标记后 (正常恢复)
├── BeforeCheckpoint  → 检查点前 (从 WAL 恢复)
└── AfterCheckpoint   → 检查点后 (从检查点恢复)
```

### 3.3 恢复状态图

```
            ┌──────────────┐
            │ SERVER START │
            └──────┬───────┘
                   │
                   ▼
            ┌──────────────┐
            │  READ WAL    │
            └──────┬───────┘
                   │
                   ▼
            ┌──────────────┐
            │ SCAN TX MAP  │
            └──────┬───────┘
                   │
        ┌──────────┼──────────┐
        │          │          │
        ▼          ▼          ▼
  ┌──────────┐ ┌────────┐ ┌──────────┐
  │COMMITTED │ │PREPARED│ │ABORTED/  │
  │          │ │(2PC)   │ │INCOMPLETE│
  └────┬─────┘ └───┬────┘ └────┬─────┘
       │           │           │
       ▼           ▼           ▼
  ┌──────────┐ ┌────────┐ ┌──────────┐
  │REPLAY    │ │ASK     │ │ROLLBACK  │
  │entries   │ │COORD   │ │/IGNORE   │
  └────┬─────┘ └───┬────┘ └────┬─────┘
       │           │           │
       └───────────┼───────────┘
                   │
                   ▼
            ┌──────────────┐
            │  VERIFY      │
            │  CHECKSUMS   │
            └──────┬───────┘
                   │
                   ▼
            ┌──────────────┐
            │  READY       │
            └──────────────┘
```

## 4. 算法复杂度与性能分析

### 4.1 操作复杂度

| 操作 | 复杂度 | 说明 |
|------|--------|------|
| append (写入) | O(1) 均摊 | BufWriter 缓冲 + 长度前缀编码 |
| recover (恢复) | O(N) | N = 总条目数，顺序读取 |
| PITR 恢复 | O(N) | 全量读取 + 时间戳过滤 |
| 归档压缩 | O(S) | S = WAL 文件大小 |
| Group Commit | O(B) | B = batch_size |

### 4.2 吞吐量基准

| 模式 | 目标 | 实际 (debug) | 实际 (release) |
|------|------|-------------|----------------|
| 顺序写入 | ≥ 50 MB/s | ≥ 5 MB/s | ≥ 50 MB/s |
| Group Commit | ≥ 100K tx/s | - | - |
| 恢复 | ≥ 10 MB/s | - | - |

### 4.3 ⚠️ 已知问题

| 问题 | 严重性 | 影响 | 修复建议 |
|------|--------|------|---------|
| **非 batch 每次 fsync** | 🟡 中等 | 吞吐量受限 | 默认启用 batch_mode |
| **PITR 全量扫描** | 🟡 中等 | 大 WAL 恢复慢 | 增加时间戳索引 |
| **归档压缩恢复未实现** | 🟡 中等 | 压缩归档不可读 | 实现 deflate 解压恢复 |
| **LSN 非持久化** | 🟡 中等 | 重启后 LSN 从 0 开始 | 持久化 LSN 到元数据 |

### 4.4 性能优化建议

```
优化1: 默认启用 batch_mode
  当前: batch_mode=false, 每次 append 都 flush
  建议: batch_mode=true, flush_threshold=100
  预期: 吞吐量提升 10-50x

优化2: PITR 时间戳索引
  当前: recover_to_timestamp() 全量读取所有条目
  建议: 每 N 条记录一个时间戳索引点
  预期: PITR 恢复时间从 O(N) 降到 O(N/B + B)

优化3: LSN 持久化
  当前: WalWriter 每次创建从 0 开始
  建议: 在 WAL 文件头写入起始 LSN
  预期: 重启后 LSN 连续，支持增量备份
```

## 5. 与其他模块的依赖

```
WalManager
  ├── 依赖: miniz_oxide (归档压缩)
  ├── 依赖: sqlrustgo_observability (统计指标)
  ├── 被依赖: WalStorage (存储适配)
  ├── 被依赖: WalTransactionalExecutor (事务执行)
  ├── 被依赖: ClusteredIndex (WAL 集成)
  ├── 被依赖: PitrRecovery (时间点恢复)
  └── 被依赖: ColumnarStorage (列存 WAL)
```

## 变更记录

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-05-15 | v2.0 | 基于 GitNexus 重新分析，补充 Group Commit、PITR、恢复链路 |
