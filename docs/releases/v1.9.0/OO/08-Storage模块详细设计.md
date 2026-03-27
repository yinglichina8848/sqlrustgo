# SQLRustGo Storage 模块详细设计文档

> **版本**: v1.9.0
> **日期**: 2026-03-26
> **模块**: sqlrustgo-storage

---

## 1. 模块概述

Storage 模块负责数据持久化、索引管理和缓存。

### 1.1 模块职责

- 存储引擎实现 (Memory/File)
- 页面管理 (Page Management)
- Buffer Pool 缓存
- B+Tree 索引
- WAL 日志
- 数据压缩

### 1.2 模块结构

```
crates/storage/
├── src/
│   ├── lib.rs               # 模块入口
│   ├── engine.rs            # 存储引擎接口
│   ├── buffer_pool.rs       # Buffer Pool
│   ├── page.rs              # 页面结构
│   ├── heap.rs              # 堆文件
│   ├── file_storage.rs      # 文件存储
│   ├── columnar.rs          # 列式存储
│   ├── binary_format.rs     # 二进制格式
│   ├── backup.rs            # 备份恢复
│   ├── buffer_pool_metrics.rs # Buffer Pool 指标
│   └── bplus_tree/          # B+Tree 索引
│       ├── mod.rs
│       └── index.rs
└── Cargo.toml
```

---

## 2. 核心类设计

### 2.1 存储引擎架构

```uml
@startuml

abstract class StorageEngine {
  --
  +create_table(info): Result<()>
  +drop_table(name): Result<()>
  +insert(table, rows): Result<usize>
  +scan(table): Result<Vec<Row>>
  +update(table, key, values): Result<usize>
  +delete(table, key): Result<usize>
}

class MemoryStorage {
  -tables: HashMap<String, TableData>
  -catalog: CatalogRef
}

class FileStorage {
  -base_path: PathBuf
  -buffer_pool: BufferPool
  -wal: WAL
  -catalog: CatalogRef
}

class ColumnarStorage {
  -columns: HashMap<String, ColumnData>
  -buffer_pool: BufferPool
}

StorageEngine <|-- MemoryStorage
StorageEngine <|-- FileStorage
StorageEngine <|-- ColumnarStorage

@enduml
```

### 2.2 Buffer Pool 设计

```uml
@startuml

class BufferPool {
  -pool: Vec<PageFrame>
  -page_table: HashMap<PageId, FrameId>
  -lru: LruCache<FrameId>
  -config: PoolConfig
  --
  +get_page(PageId): &mut PageFrame
  +allocate_page(): PageId
  +pin(FrameId)
  +unpin(FrameId)
  +evict(): Option<FrameId>
  +flush_dirty(): Result<()>
}

class PageFrame {
  -page_id: PageId
  -data: [u8; PAGE_SIZE]
  -pin_count: u32
  -is_dirty: bool
  -last_access: Timestamp
}

class PageTable {
  -entries: HashMap<PageId, TableEntry>
  --
  +lookup(PageId): Option<FrameId>
  +insert(PageId, FrameId)
  +remove(PageId)
}

BufferPool --> PageFrame
BufferPool --> PageTable

@enduml
```

### 2.3 B+Tree 索引

```uml
@startuml

class BPlusTree {
  -root: NodePtr
  -degree: usize
  -storage: &mut Storage
  --
  +insert(key, value): Result<()>
  +search(key): Option<Value>
  +range_query(start, end): Vec<Value>
  +delete(key): Result<()>
  +bulk_load(keys, values): Result<()>
}

class Node {
  -is_leaf: bool
  -keys: Vec<Key>
  -num_keys: usize
}

class LeafNode {
  -keys: Vec<Key>
  -values: Vec<Value>
  -next: Option<NodePtr>
}

class InternalNode {
  -keys: Vec<Key>
  -children: Vec<NodePtr>
}

Node <|-- LeafNode
Node <|-- InternalNode
BPlusTree --> Node

@enduml
```

---

## 3. 页面格式设计

### 3.1 页面结构

```
┌─────────────────────────────────────────────────────────────────────┐
│                        页面格式 (8KB)                                │
├─────────────────────────────────────────────────────────────────────┤
│  Page Header (16 bytes)                                             │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ PageID (4) │ LSN (8) │ PageType (2) │ FreeSpace (2)        │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  Cell Pointers (可变)                                               │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ Pointer[0] (2) │ Pointer[1] (2) │ ... │ Pointer[N] (2)     │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  Cell Data (可变)                                                  │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ Cell 0: KeyLen(2) │ Key │ ValueLen(2) │ Value              │   │
│  │ Cell 1: KeyLen(2) │ Key │ ValueLen(2) │ Value              │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 页面类型

```rust
enum PageType {
    Leaf,        // B+Tree 叶子节点
    Internal,    // B+Tree 内部节点
    Data,        // 数据页面
    Index,       // 索引页面
    Header,      // 文件头页面
    Free,        // 空闲页面
}
```

---

## 4. WAL 设计

### 4.1 日志结构

```uml
@startuml

class WAL {
  -log_file: File
  -lsn: Lsn
  -buffer: Vec<WALEntry>
  --
  +append(entry): Lsn
  +flush(): Result<()>
  +recover(): Vec<WALEntry>
}

class WALEntry {
  -lsn: Lsn
  -tx_id: TransactionId
  -op_type: OperationType
  -table: String
  -key: Vec<u8>
  -value: Option<Vec<u8>>
  -timestamp: Timestamp
}

class LogSequenceNumber {
  -file_id: u32
  -offset: u64
}

WAL --> WALEntry
WALEntry --> LogSequenceNumber

@enduml
```

---

## 5. 与代码对应检查

### 5.1 模块文件对应

| 设计内容 | 代码文件 | 状态 |
|----------|----------|------|
| 存储引擎接口 | `engine.rs` | ✅ 对应 |
| Buffer Pool | `buffer_pool.rs` | ✅ 对应 |
| 页面结构 | `page.rs` | ✅ 对应 |
| 堆文件 | `heap.rs` | ✅ 对应 |
| 文件存储 | `file_storage.rs` | ✅ 对应 |
| 列式存储 | `columnar.rs` | ✅ 对应 |
| 二进制格式 | `binary_format.rs` | ✅ 对应 |
| 备份恢复 | `backup.rs` | ✅ 对应 |
| B+Tree 索引 | `bplus_tree/index.rs` | ✅ 对应 |

### 5.2 功能覆盖检查

| 功能 | 代码实现 | 状态 |
|------|----------|------|
| 内存存储 | ✅ | ✅ |
| 文件存储 | ✅ | ✅ |
| 列式存储 | ✅ | ✅ |
| Buffer Pool | ✅ | ✅ |
| LRU 淘汰 | ✅ | ✅ |
| B+Tree 索引 | ✅ | ✅ |
| 唯一索引 | ✅ | ✅ |
| 复合索引 | ✅ | ✅ |
| WAL 日志 | ✅ | ✅ |
| 崩溃恢复 | ✅ | ✅ |
| 备份恢复 | ✅ | ✅ |

---

## 6. 测试设计

### 6.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_buffer_pool_allocate() {
        let pool = BufferPool::new(100);
        let page_id = pool.allocate_page();
        assert!(page_id > 0);
    }
    
    #[test]
    fn test_bplus_tree_insert() {
        let mut tree = BPlusTree::new(4);
        tree.insert(1, b"value1").unwrap();
        
        let result = tree.search(1);
        assert!(result.is_some());
    }
    
    #[test]
    fn test_bplus_tree_range() {
        let mut tree = BPlusTree::new(4);
        for i in 1..=10 {
            tree.insert(i, format!("value{}", i)).unwrap();
        }
        
        let results = tree.range_query(3, 7);
        assert_eq!(results.len(), 5);
    }
    
    #[test]
    fn test_wal_recovery() {
        let wal = WAL::new("test.wal").unwrap();
        wal.append(test_entry()).unwrap();
        
        let recovered = wal.recover();
        assert!(!recovered.is_empty());
    }
}
```

---

**文档版本历史**

| 版本 | 日期 | 作者 | 变更 |
|------|------|------|------|
| 1.0 | 2026-03-26 | OpenCode | 初始版本 |

**文档状态**: ✅ 已完成
