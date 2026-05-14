# Observability Layer Design - v3.1.0

> **版本**: v3.1.0
> **日期**: 2026-05-14
> **Issue**: #793
> **状态**: 设计完成

---

## 一、概述

### 1.1 目标

为 SQLRustGo v3.1.0 实现完整的可观测性层，提供单机版必需的系统表和监控能力。

### 1.2 四个核心系统表

| 系统表 | 功能 | 持久化 |
|--------|------|--------|
| TRANSACTION_HISTORY | 事务历史追踪 | 混合模式 |
| LOCK_WAIT_GRAPH | 锁等待拓扑图 | 纯内存 |
| RECOVERY_HISTORY | 崩溃恢复历史 | 持久化 |
| WAL_STATS | WAL 写入/回放统计 | 纯内存 |

### 1.3 设计原则

- **混合持久化**: 热数据内存，冷数据定期刷盘
- **统一接口**: Observable trait 提供一致的事件收集
- **双重访问**: 支持 SQL 表查询和 SHOW 命令
- **最小侵入**: 复用现有 WAL/Transaction基础设施

---

## 二、架构设计

### 2.1 模块结构

```
crates/observability/
├── src/
│   ├── lib.rs                    # 框架入口
│   ├── observer.rs               # Observable trait + 事件收集器
│   ├── metrics.rs                # 指标收集基础设施
│   ├── tables/
│   │   ├── mod.rs
│   │   ├── transaction_history.rs # TRANSACTION_HISTORY
│   │   ├── lock_wait_graph.rs    # LOCK_WAIT_GRAPH
│   │   ├── recovery_history.rs    # RECOVERY_HISTORY
│   │   └── wal_stats.rs          # WAL_STATS
│   └── commands/
│       ├── mod.rs
│       └── show_commands.rs       # SHOW 命令实现
```

### 2.2 核心组件

#### Observable Trait

```rust
pub trait Observable {
    fn record(&self, event: ObservableEvent);
    fn flush(&self) -> Result<()>;
}
```

#### InMemoryStore<T>

- 内存中的环形缓冲区
- 超过阈值时触发刷盘
- 支持热数据查询

#### PeriodicFlusher

- 后台定时任务
- 将内存数据定期持久化
- 可配置刷盘间隔

---

## 三、数据模型

### 3.1 TRANSACTION_HISTORY

```rust
pub struct TransactionHistoryEntry {
    pub tx_id: u64,
    pub tx_uuid: String,
    pub start_time: u64,
    pub commit_time: Option<u64>,
    pub abort_time: Option<u64>,
    pub isolation_level: String,
    pub status: TransactionStatus,  // Active/Committed/Aborted
    pub read_keys: Vec<Vec<u8>>,
    pub write_keys: Vec<Vec<u8>>,
}
```

### 3.2 LOCK_WAIT_GRAPH

```rust
pub struct LockWaitEdge {
    pub waiter_tx_id: u64,
    pub holder_tx_id: u64,
    pub lock_key: String,
    pub lock_mode: String,
    pub wait_start_time: u64,
}

pub struct LockWaitGraph {
    edges: Vec<LockWaitEdge>,
    timestamp: u64,
}
```

### 3.3 RECOVERY_HISTORY

```rust
pub struct RecoveryHistoryEntry {
    pub recovery_id: u64,
    pub crash_timestamp: u64,
    pub recovery_timestamp: u64,
    pub lsn_recovered: u64,
    pub transactions_replayed: u64,
    pub status: RecoveryStatus,  // Success/Failed
    pub error_message: Option<String>,
}
```

### 3.4 WAL_STATS

```rust
pub struct WalStats {
    pub total_writes: u64,
    pub total_bytes: u64,
    pub flush_count: u64,
    pub replay_count: u64,
    pub replay_time_ms: u64,
    pub last_flush_lsn: u64,
    pub current_lsn: u64,
}
```

---

## 四、SHOW 命令设计

### 4.1 命令列表

```sql
SHOW TRANSACTION HISTORY [LIMIT N]
SHOW LOCK WAITS
SHOW RECOVERY HISTORY [LIMIT N]
SHOW WAL STATS
EXPLAIN ANALYZE TIMELINE
```

### 4.2 输出格式 (类 MySQL)

```sql
mysql> SHOW TRANSACTION HISTORY;
+----+--------------------------------------+-----------+---------------------+---------------------+---------+
| TX | UUID                                 | ISOLATION | START_TIME          | END_TIME            | STATUS  |
+----+--------------------------------------+-----------+---------------------+---------------------+---------+
|  1 | 550e8400-e29b-41d4-a716-446655440000| SI        | 2026-05-14 10:00:00 | 2026-05-14 10:00:05 | COMMIT  |
|  2 | 6ba7b810-9dad-11d1-80b4-00c04fd430c8| RR        | 2026-05-14 10:00:01 | -                   | ACTIVE  |
+----+--------------------------------------+-----------+---------------------+---------------------+---------+
```

### 4.3 Parser 扩展

```rust
// ShowStatement 枚举新增变体
pub enum ShowStatement {
    // ... existing ...
    TransactionHistory { limit: Option<u32> },
    LockWaits,
    RecoveryHistory { limit: Option<u32> },
    WalStats,
}
```

---

## 五、数据流设计

### 5.1 事务事件收集

```
BEGIN Transaction
    ↓
TransactionManager.begin_transaction()
    ↓
Observable::record(BeginTransaction { tx_id, ... })
    ↓
TransactionHistoryStore::append(entry)
    ↓
[If buffer full] → PeriodicFlusher::flush_to_disk()
```

### 5.2 锁等待检测

```
LockManager::acquire_lock()
    ↓
[If lock held by another tx]
    ↓
Observable::record(LockWait { waiter, holder, ... })
    ↓
LockWaitGraph::add_edge(edge)
```

---

## 六、集成设计

### 6.1 与 Transaction 模块集成

```rust
// transaction_manager.rs
impl TransactionManager {
    pub fn begin_transaction(&mut self) -> Result<TxId> {
        // ... existing code ...
        self.observer.record(BeginTransaction { tx_id, ... });
    }

    pub fn commit(&mut self, tx_id: TxId) -> Result<()> {
        // ... existing code ...
        self.observer.record(CommitTransaction { tx_id, ... });
    }
}
```

### 6.2 与 Storage 模块集成

```rust
// wal.rs
impl Wal {
    pub fn append(&mut self, entry: WalEntry) -> Result<u64> {
        // ... existing code ...
        self.observer.record(WalWrite { bytes: entry.size(), ... });
    }
}
```

### 6.3 与 INFORMATION_SCHEMA 集成

```rust
// information-schema/src/lib.rs
impl InformationSchema {
    pub fn get_transaction_history(&self) -> Vec<TransactionHistoryRow> { ... }
    pub fn get_lock_waits(&self) -> Vec<LockWaitRow> { ... }
    pub fn get_recovery_history(&self) -> Vec<RecoveryHistoryRow> { ... }
    pub fn get_wal_stats(&self) -> Vec<WalStatsRow> { ... }
}
```

---

## 七、持久化策略

### 7.1 混合存储

| 数据类型 | 存储位置 | 刷盘策略 |
|----------|----------|----------|
| TRANSACTION_HISTORY | 内存 + 文件 | 每 1000 条或 5 分钟 |
| LOCK_WAIT_GRAPH | 纯内存 | 不持久化，重启清空 |
| RECOVERY_HISTORY | 持久化文件 | 每次恢复后追加 |
| WAL_STATS | 纯内存 | 不持久化，重启清空 |

### 7.2 文件格式

```
data/observability/
├── transaction_history.bin   # 事务历史 (Binary)
├── recovery_history.bin      # 恢复历史 (Binary)
└── .gitkeep                 # 保持目录
```

---

## 八、验收条件

- [ ] `SHOW TRANSACTION HISTORY` 输出事务历史
- [ ] `SHOW LOCK WAITS` 输出当前锁等待
- [ ] `SHOW RECOVERY HISTORY` 输出恢复历史
- [ ] `SHOW WAL STATS` 输出 WAL 统计
- [ ] `EXPLAIN ANALYZE TIMELINE` 输出查询时间线
- [ ] SQL 表查询可用: `SELECT * FROM transaction_history`
- [ ] 内存缓冲区超过阈值自动刷盘
- [ ] 重启后 RECOVERY_HISTORY 持久化数据可恢复
- [ ] Beta Gate 仍通过

---

## 九、测试计划

### 9.1 单元测试

- TransactionHistoryRecorder::append
- LockWaitGraph::add_edge / detect_cycle
- WalStats::increment

### 9.2 集成测试

- observability_integration_test: 完整事件流
- crash_recovery_observability_test: 崩溃后数据恢复

---

## 十、依赖关系

```
observability
├── sqlrustgo-types (Result, Error)
├── sqlrustgo-storage (WAL)
├── sqlrustgo-transaction (TransactionManager)
└── information-schema (SystemTableProvider)
```

---

*设计文档 - 2026-05-14*
