---
entity_type: tool
confidence: 90
domains: [sqlrustgo, storage, database]
last_updated: 2026-04-17
---

# Storage 模块

> 存储引擎 - 数据持久化和检索

## 概述

Storage 模块负责数据的持久化、索引管理和崩溃恢复。

## 源码位置

```
crates/storage/
├── src/
│   ├── lib.rs
│   ├── engine.rs        # StorageEngine trait
│   ├── file_storage.rs  # 文件存储实现
│   ├── memory_storage.rs # 内存存储
│   ├── bplus_tree/      # B+Tree 索引
│   ├── wal.rs           # Write-Ahead Log
│   ├── buffer_pool.rs   # 缓冲池
│   └── ...
└── Cargo.toml
```

## 存储类型

### MemoryStorage

纯内存存储，适用于测试和临时数据。

```rust
pub struct MemoryStorage {
    tables: HashMap<TableName, Vec<Record>>,
    indexes: HashMap<IndexName, BTreeMap<Key, Vec<Rid>>>,
}
```

### FileStorage

持久化文件存储，支持 B+Tree 索引。

```rust
pub struct FileStorage {
    data_dir: PathBuf,
    tables: HashMap<TableName, BTreeMap<Rid, Record>>,
    indexes: HashMap<IndexName, BTreeMap<Key, Vec<Rid>>>,
    wal_writer: Option<WALWriter>,
}
```

### ColumnarStorage

列式存储，分析型查询优化。

### VectorStorage

向量存储，支持 KNN 搜索。

### GraphStore

图存储，边和节点管理。

## 索引支持

| 索引类型 | 状态 | 说明 |
|----------|------|------|
| B+Tree | ✅ | 主索引 |
| Hash Index | ✅ | 内存/文件 |
| Vector Index | ✅ | IVF, HNSW |
| Composite Index | ⚠️ | 部分支持 |

## WAL 状态

| 功能 | 状态 | 说明 |
|------|------|------|
| WAL 写入 | ✅ | 可选启用 |
| WAL 恢复 | ✅ | 崩溃恢复 |
| **默认启用** | ❌ | Issue #1497 |

```rust
// 当前状态
let storage = FileStorage::new(data_dir)?;
// wal_writer: None - 未启用

// 正确方式
let storage = WalStorage::new(inner, wal_path)?;
```

## 事务支持

| 特性 | 状态 |
|------|------|
| MVCC | ✅ |
| 锁管理 | ✅ |
| 隔离级别 | SNAPSHOT |

## 测试

```bash
cargo test --package sqlrustgo-storage
cargo test --test storage_integration_test
```

## 已知问题

1. **WAL 未默认启用** - Issue #1497
2. **索引未默认使用** - `should_use_index() = false`
3. **Composite Index** - 部分实现

## 相关文件

- [Executor 模块](./Executor.md) - 上游消费者
- [Transaction 模块](./Transaction.md) - 事务协调

---

*最后更新: 2026-04-17*
