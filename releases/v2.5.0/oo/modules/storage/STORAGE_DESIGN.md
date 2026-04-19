# Storage 模块设计

**版本**: v2.5.0
**模块**: Storage (存储引擎)

---

## 一、What (是什么)

Storage 是 SQLRustGo 的存储引擎核心模块，负责数据的持久化存储、索引管理、缓冲池管理和页面调度。

## 二、Why (为什么)

- **数据持久化**: 确保数据不丢失
- **高效访问**: B+树索引和缓冲池
- **事务支持**: MVCC 和 WAL 集成
- **多种存储**: 行式、列式、向量存储

## 三、How (如何实现)

### 3.1 存储架构

```
┌─────────────────────────────────────────┐
│           StorageEngine                    │
├─────────────────────────────────────────┤
│  - BufferPool                           │
│  - PageManager                          │
│  - IndexManager                          │
│  - WALIntegration                        │
└─────────────────────────────────────────┘
         │              │
         ▼              ▼
┌────────────────┐  ┌────────────────┐
│  RowStorage    │  │ ColumnarStorage │
└────────────────┘  └────────────────┘
         │
         ▼
┌─────────────────────────────────────────┐
│           BPlusTree                        │
│  - Clustered Index                       │
│  - Secondary Index                      │
└─────────────────────────────────────────┘
```

### 3.2 缓冲池

```rust
pub struct BufferPool {
    pages: HashMap<PageId, PageFrame>,
    replacer: Box<dyn PageReplacer>,
    free_list: Vec<PageId>,
    capacity: usize,
}

pub struct PageFrame {
    page_id: PageId,
    data: Vec<u8>,
    pin_count: usize,
    is_dirty: bool,
    last_access: u64,
}

pub trait PageReplacer: Send + Sync {
    fn victim(&mut self) -> Option<PageId>;
    fn record_access(&mut self, page_id: PageId);
}
```

### 3.3 B+树索引

```rust
pub struct BPlusTree {
    root: Option<PageId>,
    comparator: Box<dyn Comparator>,
}

pub enum BTreeNode {
    Internal {
        children: Vec<PageId>,
        keys: Vec<Key>,
    },
    Leaf {
        next: Option<PageId>,
        entries: Vec<(Key, Value)>,
    },
}

impl BPlusTree {
    // 插入
    pub fn insert(&mut self, key: Key, value: Value) -> Result<()> {
        let leaf = self.find_leaf(key)?;
        self.insert_into_leaf(leaf, key, value)
    }

    // 查找
    pub fn get(&self, key: &Key) -> Result<Option<Value>> {
        let leaf = self.find_leaf(key)?;
        Ok(leaf.find(key))
    }

    // 删除
    pub fn delete(&mut self, key: &Key) -> Result<()> {
        let leaf = self.find_leaf(key)?;
        self.delete_from_leaf(leaf, key)
    }
}
```

### 3.4 列式存储

```rust
pub struct ColumnarStorage {
    segments: Vec<ColumnSegment>,
    schema: Schema,
}

pub struct ColumnSegment {
    column_type: ColumnType,
    nulls: Bitmap,
    data: ColumnData,
    stats: ColumnStats,
}

pub enum ColumnData {
    Int32(Vec<i32>),
    Int64(Vec<i64>),
    Float32(Vec<f32>),
    Float64(Vec<f64>),
    String(Vec<String>),
    ByteArray(Vec<Vec<u8>>),
}

pub struct ColumnStats {
    min: Value,
    max: Value,
    null_count: usize,
    distinct_count: usize,
}
```

## 四、接口设计

### 4.1 公开 API

```rust
pub trait StorageEngine: Send + Sync {
    // 表操作
    fn create_table(&self, schema: Schema) -> Result<TableId>;
    fn drop_table(&self, table_id: TableId) -> Result<()>;
    fn get_table(&self, table_id: TableId) -> Result<Table>;

    // 页面操作
    fn get_page(&self, page_id: PageId) -> Result<PageGuard>;
    fn allocate_page(&self) -> Result<PageId>;
    fn deallocate_page(&self, page_id: PageId) -> Result<()>;

    // 索引操作
    fn create_index(&self, table_id: TableId, columns: Vec<ColumnId>) -> Result<IndexId>;
    fn drop_index(&self, index_id: IndexId) -> Result<()>;

    // 事务支持
    fn begin_transaction(&self) -> Result<TransactionId>;
    fn commit(&self, tx_id: TransactionId) -> Result<()>;
    fn rollback(&self, tx_id: TransactionId) -> Result<()>;
}
```

### 4.2 表接口

```rust
pub trait Table: Send + Sync {
    // 插入
    fn insert(&self, row: Row) -> Result<RowId>;

    // 更新
    fn update(&self, row_id: RowId, row: Row) -> Result<()>;

    // 删除
    fn delete(&self, row_id: RowId) -> Result<()>;

    // 扫描
    fn scan(&self, scan_request: ScanRequest) -> Result<Box<dyn Iterator<Item = Row>>>;

    // 索引查找
    fn get_by_index(&self, index_id: IndexId, key: &Key) -> Result<Option<Row>>;
}
```

## 五、存储格式

### 5.1 页面格式

```
┌─────────────────────────────────────────┐
│ Page Header (16 bytes)                   │
├─────────────────────────────────────────┤
│  - page_id: u32                         │
│  - checksum: u32                        │
│  - page_type: u8                        │
│  - item_count: u16                      │
│  - free_space_offset: u16               │
├─────────────────────────────────────────┤
│ Item Pointers                            │
│  [offset: u16, length: u16] × n         │
├─────────────────────────────────────────┤
│ Free Space                               │
├─────────────────────────────────────────┤
│ Items                                    │
│  [data] × n                             │
└─────────────────────────────────────────┘
```

### 5.2 数据文件结构

```
database/
├── catalog/
│   └── tables.dat    # 系统表
├── t{table_id}/
│   ├── primary.dat   # 主数据文件
│   └── index_{idx}.dat  # 索引文件
└── wal/
    └── wal.dat      # WAL 文件
```

## 六、性能考虑

| 操作 | 时间复杂度 | 说明 |
|------|------------|------|
| 页面读取 | O(1) | 缓冲池命中 |
| B+树查找 | O(log n) | 树高度 |
| 顺序扫描 | O(n) | 全表扫描 |
| 索引扫描 | O(log n + k) | k 为结果数 |

### 优化策略

1. **缓冲池**: LRU/CLOCK 淘汰策略
2. **预读**: 顺序访问预读相邻页面
3. **压缩**: 列式存储 LZ4/Zstd 压缩
4. **并行扫描**: 多线程并行读取

## 七、相关文档

- [ARCHITECTURE_V2.5.md](../architecture/ARCHITECTURE_V2.5.md) - 整体架构
- [MVCC_DESIGN.md](./mvcc/MVCC_DESIGN.md) - MVCC 集成

---

*Storage 模块设计 v2.5.0*
