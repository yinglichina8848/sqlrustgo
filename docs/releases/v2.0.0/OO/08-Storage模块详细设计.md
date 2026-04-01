# SQLRustGo v2.0.0 Storage 模块详细设计

> **版本**: v2.0.0
> **日期**: 2026-03-29
> **模块**: sqlrustgo-storage

---

## 1. 模块概述

Storage 模块负责数据存储，包括行式存储 (Row Storage) 和列式存储 (Columnar Storage)。

## 2. 核心 Trait

### 2.1 StorageEngine Trait

```rust
pub trait StorageEngine: Send + Sync {
    fn scan(&self, table: &TableRef) -> Result<Box<dyn RecordBatchReader>>;
    fn scan_with_projection(&self, table: &TableRef, projection: &[usize]) -> Result<Box<dyn RecordBatchReader>>;
    fn insert(&self, table: &TableRef, batch: RecordBatch) -> Result<usize>;
    fn delete(&self, table: &TableRef, keys: &[Key]) -> Result<usize>;
    fn update(&self, table: &TableRef, keys: &[Key], batch: RecordBatch) -> Result<usize>;
    fn flush(&self) -> Result<()>;
    fn checkpoint(&self) -> Result<()>;
    fn table_names(&self) -> Result<Vec<String>>;
    fn drop_table(&self, table: &TableRef) -> Result<()>;
}
```

---

## 3. 行式存储

### 3.1 RowStorage

```rust
pub struct RowStorage {
    buffer_pool: Arc<BufferPool>,
    wal_manager: Arc<WALManager>,
    catalog: Arc<dyn Catalog>,
    index_manager: Arc<IndexManager>,
}
```

### 3.2 BufferPool

```rust
pub struct BufferPool {
    pages: RwLock<LruCache<PageId, Arc<Page>>>,
    capacity: usize,
    disk_manager: Arc<DiskManager>,
}

impl BufferPool {
    pub fn new(capacity: usize) -> Self;
    pub fn get_page(&self, page_id: PageId) -> Result<Arc<Page>>;
    pub fn pin_page(&self, page_id: PageId) -> Result<Arc<Page>>;
    pub fn unpin_page(&self, page_id: PageId);
    pub fn allocate_page(&self) -> Result<PageId>;
    pub fn flush_dirty_pages(&self) -> Result<()>;
}

pub struct Page {
    pub page_id: PageId,
    pub data: Vec<u8>,
    pub pin_count: usize,
    pub is_dirty: bool,
    pub checksum: u32,
}
```

### 3.3 WALManager

```rust
pub struct WALManager {
    log_file: Arc<Mutex<File>>,
    position: Arc<AtomicU64>,
    enable_group_commit: bool,
    group_commit_interval: Duration,
    wal_buffer: Vec<WALEntry>,
}

impl WALManager {
    pub fn new(path: PathBuf) -> Result<Self>;
    pub fn append(&self, entry: WALEntry) -> Result<u64>;
    pub fn read(&self, offset: u64, length: usize) -> Result<Vec<WALEntry>>;
    pub fn flush(&self) -> Result<()>;
    pub fn truncate(&self, offset: u64) -> Result<()>;
    pub fn replay(&self, storage: &dyn StorageEngine) -> Result<()>;
}

pub struct WALEntry {
    pub txid: TransactionId,
    pub op_type: WALOpType,
    pub table_id: TableId,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

pub enum WALOpType {
    Insert,
    Update,
    Delete,
    Commit,
    Rollback,
}
```

### 3.4 WAL Group Commit

```rust
pub struct GroupCommitter {
    wal_manager: Arc<WALManager>,
    max_group_size: usize,
    max_wait_time: Duration,
    queue: Vec<WALEntry>,
}

impl GroupCommitter {
    pub fn new(wal: Arc<WALManager>, max_size: usize, max_wait: Duration) -> Self;
    pub fn add_entry(&self, entry: WALEntry) -> Result<u64>;
    pub fn flush_group(&self) -> Result<()>;
}
```

---

## 4. 列式存储

### 4.1 ColumnarStorage

```rust
pub struct ColumnarStorage {
    base_path: PathBuf,
    segments: RwLock<HashMap<SegmentId, Arc<ColumnSegment>>>,
    row_count: AtomicU64,
    config: ColumnarConfig,
}

pub struct ColumnarConfig {
    pub chunk_size: usize,
    pub compression: CompressionType,
    pub enable_parquet: bool,
}

impl ColumnarStorage {
    pub fn new(base_path: PathBuf) -> Result<Self>;
    pub fn write_batch(&self, table: &str, batch: RecordBatch) -> Result<()>;
    pub fn read_segment(&self, segment_id: SegmentId) -> Result<Arc<ColumnChunk>>;
    pub fn scan(&self, table: &str, projection: &[ColumnId]) -> Result<Box<dyn RecordBatchReader>>;
    pub fn flush(&self) -> Result<()>;
}
```

### 4.2 ColumnChunk

```rust
pub struct ColumnChunk {
    pub chunk_id: ChunkId,
    pub column_id: ColumnId,
    pub num_rows: usize,
    pub data: ArrayRef,
    pub null_bitmap: Bitmap,
    pub statistics: ColumnStatistics,
}

impl ColumnChunk {
    pub fn new(column_id: ColumnId, data: ArrayRef) -> Self;
    pub fn with_null_bitmap(mut self, bitmap: Bitmap) -> Self;
    pub fn num_rows(&self) -> usize;
    pub fn null_count(&self) -> usize;
}
```

### 4.3 ColumnSegment

```rust
pub struct ColumnSegment {
    pub segment_id: SegmentId,
    pub column_id: ColumnId,
    pub row_offset: u64,
    pub num_rows: u64,
    pub file_path: PathBuf,
    pub offset: u64,
    pub length: u64,
    pub compression: CompressionType,
    pub statistics: ColumnStatistics,
}
```

### 4.4 Projection Pushdown

```rust
pub struct ProjectionPushdownOptimizer {
    required_columns: Vec<ColumnId>,
}

impl ProjectionPushdownOptimizer {
    pub fn new(required_columns: Vec<ColumnId>) -> Self;
    pub fn optimize(&self, storage: &ColumnarStorage) -> Result<Vec<ColumnId>>;
}
```

---

## 5. Parquet 支持

### 5.1 ParquetCompat

```rust
pub struct ParquetCompat {
    reader: Option<ParquetRecordBatchReader>,
    writer: Option<ParquetWriter>,
    schema: SchemaRef,
}

impl ParquetCompat {
    pub fn read_from_file(path: &Path, projection: &[ColumnId]) -> Result<Box<dyn RecordBatchReader>>;
    pub fn write_to_file(path: &Path, batch: RecordBatch) -> Result<()>;
    pub fn import_from_parquet(table: &str, path: &Path, storage: &ColumnarStorage) -> Result<usize>;
    pub fn export_to_parquet(table: &str, path: &Path, storage: &ColumnarStorage) -> Result<usize>;
}
```

### 5.2 COPY 语句实现

```rust
pub struct ParquetCopyHandler {
    storage: Arc<dyn StorageEngine>,
}

impl ParquetCopyHandler {
    pub fn copy_from(&self, table: &str, path: &Path) -> Result<usize>;
    pub fn copy_to(&self, table: &str, path: &Path) -> Result<usize>;
}
```

---

## 6. 索引管理

### 6.1 IndexManager

```rust
pub struct IndexManager {
    indexes: RwLock<HashMap<IndexId, Arc<dyn Index>>>,
}

pub trait Index: Send + Sync {
    fn insert(&self, key: &Key, rid: RecordId) -> Result<()>;
    fn search(&self, key: &Key) -> Result<Vec<RecordId>>;
    fn range_scan(&self, start: &Key, end: &Key) -> Result<Vec<RecordId>>;
    fn delete(&self, key: &Key, rid: RecordId) -> Result<()>;
}
```

### 6.2 BTreeIndex

```rust
pub struct BTreeIndex {
    root: Option<Arc<BTreeNode>>,
    key_comparator: Box<dyn KeyComparator>,
}

impl Index for BTreeIndex {
    fn insert(&self, key: &Key, rid: RecordId) -> Result<()>;
    fn search(&self, key: &Key) -> Result<Vec<RecordId>>;
    fn range_scan(&self, start: &Key, end: &Key) -> Result<Vec<RecordId>>;
    fn delete(&self, key: &Key, rid: RecordId) -> Result<()>;
}
```

---

## 7. Catalog

### 7.1 Catalog Trait

```rust
pub trait Catalog: Send + Sync {
    fn get_table(&self, name: &str) -> Result<TableRef>;
    fn list_tables(&self) -> Result<Vec<String>>;
    fn create_table(&self, table: CreateTableRequest) -> Result<()>;
    fn drop_table(&self, name: &str) -> Result<()>;
    fn get_schema(&self, table: &str) -> Result<SchemaRef>;
    fn get_database(&self) -> Result<String>;
}
```

---

*文档生成日期: 2026-03-29*
*版本: v2.0.0*
