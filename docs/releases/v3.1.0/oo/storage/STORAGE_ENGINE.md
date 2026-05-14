# Buffer Pool 与存储引擎 (v3.1.0)

> **基于 GitNexus 分析** | 67,755 符号, 102,165 关系
> Buffer Pool / MemoryStorage / FileStorage / ClusteredIndex

## 1. Buffer Pool 架构

### 1.1 核心数据结构

```rust
pub struct BufferPool {
    pages: Mutex<HashMap<u32, Arc<Page>>>,
    lru: Mutex<VecDeque<u32>>,
    capacity: usize,
    prefetch_window: usize,
    stats: RwLock<BufferPoolStats>,
}

pub struct BufferPoolStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub prefetch_hits: u64,
}

pub struct MemoryPool {
    free_list: Mutex<Vec<Vec<u8>>>,
    block_size: usize,
}

pub struct EncryptedBufferPool {
    inner: Arc<BufferPool>,
    encryption: Arc<dyn Crypt>,
}
```

### 1.2 关键文件

| 文件 | 作用 |
|------|------|
| [buffer_pool.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/storage/src/buffer_pool.rs) | Buffer Pool 核心 |
| [engine.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/storage/src/engine.rs) | StorageEngine trait |
| [file_storage.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/storage/src/file_storage.rs) | 文件存储实现 |

## 2. Buffer Pool 操作链路

### 2.1 页面获取时序图

```
get(page_id=42)
    │
    ▼
┌──────────────────────────────────────────────┐
│ 1. pages.lock() → 检查 page_id 是否在缓存   │
└──────────────────┬───────────────────────────┘
                   │
        ┌──────────┴──────────┐
        HIT                   MISS
        │                      │
        ▼                      ▼
┌──────────────┐    ┌──────────────────────────┐
│ 更新 LRU     │    │ 2. 从磁盘加载 Page       │
│ lru.retain() │    │    ├── pages.insert()     │
│ ⚠️ O(N)     │    │    ├── lru.push_front()   │
│              │    │    └── 检查容量            │
│ stats.hits++ │    │        ├── 超容量: evict  │
└──────┬───────┘    │        └── 未超: OK       │
       │            │    stats.misses++         │
       │            └────────────┬──────────────┘
       │                         │
       └────────────┬────────────┘
                    │
                    ▼
              Return Arc<Page>
```

### 2.2 LRU 淘汰状态图

```
            ┌──────────────┐
            │  GET PAGE    │
            └──────┬───────┘
                   │
                   ▼
            ┌──────────────┐
            │  IN CACHE?   │
            └──┬───────┬───┘
            YES│       │NO
               │       │
               ▼       ▼
        ┌──────────┐ ┌──────────────┐
        │UPDATE LRU│ │LOAD FROM DISK│
        │O(N)⚠️   │ └──────┬───────┘
        └────┬─────┘        │
             │              ▼
             │      ┌──────────────┐
             │      │OVER CAPACITY?│
             │      └──┬───────┬───┘
             │      YES│       │NO
             │         │       │
             │         ▼       │
             │  ┌────────────┐ │
             │  │EVICT LRU   │ │
             │  │pop_back()  │ │
             │  │O(1)        │ │
             │  └──────┬─────┘ │
             │         │       │
             └─────────┼───────┘
                       │
                       ▼
                ┌──────────────┐
                │ RETURN PAGE  │
                └──────────────┘
```

## 3. 存储引擎

### 3.1 StorageEngine Trait

```rust
pub trait StorageEngine: Send + Sync {
    fn scan(&self, table: &str) -> Result<Vec<Record>>;
    fn insert(&self, table: &str, records: Vec<Record>) -> Result<()>;
    fn update(&self, table: &str, filters: Vec<Filter>, updates: Vec<(usize, Value)>) -> Result<u64>;
    fn delete(&self, table: &str, filters: Vec<Filter>) -> Result<u64>;
    fn create_table(&self, info: &TableInfo) -> Result<()>;
    fn drop_table(&self, name: &str) -> Result<()>;
    fn create_index(&self, table: &str, column: &str, col_idx: usize) -> Result<()>;
}
```

### 3.2 MemoryStorage

```rust
pub struct MemoryStorage {
    tables: HashMap<String, Vec<Record>>,
    table_infos: HashMap<String, TableInfo>,
    triggers: HashMap<String, TriggerInfo>,
    views: HashSet<String>,
    indexes: HashMap<String, (usize, HashMap<i64, Vec<usize>>)>,
    predicate_cache: HashMap<String, (Vec<String>, Predicate)>,
    auto_increment_counters: HashMap<String, i64>,
}
```

### 3.3 FileStorage

```rust
pub struct FileStorage {
    data_dir: PathBuf,
    tables: HashMap<String, TableData>,
    indexes: RwLock<HashMap<(String, String), BPlusTree>>,
    insert_buffer: HashMap<String, Vec<Record>>,
    buffer_threshold: usize,
    enable_buffer: bool,
}
```

## 4. 算法复杂度与性能分析

### 4.1 Buffer Pool 操作复杂度

| 操作 | 复杂度 | 说明 | 优化建议 |
|------|--------|------|---------|
| get (命中) | O(N) ⚠️ | lru.retain() 遍历 | HashMap+双向链表 O(1) |
| get (未命中) | O(1) | 统计更新 | ✅ |
| insert | O(N) ⚠️ | lru.retain() 遍历 | HashMap+双向链表 O(1) |
| evict | O(1) | pop_back() | ✅ |
| prefetch | O(W) | W=window_size | ✅ |

### 4.2 存储引擎操作复杂度

| 操作 | MemoryStorage | FileStorage |
|------|---------------|-------------|
| scan | O(N) | O(N) + 磁盘IO |
| insert | O(1) 均摊 | O(1) + WAL + 可能 flush |
| update | O(N) | O(N) + WAL |
| delete | O(N) | O(N) + WAL |
| create_index | O(N) | O(N log N) B+Tree 构建 |

### 4.3 ⚠️ 已知问题

| 问题 | 严重性 | 影响 | 修复建议 |
|------|--------|------|---------|
| **LRU 更新 O(N)** | 🔴 严重 | 每次页面访问都遍历 LRU | HashMap+双向链表 |
| **双重锁** | 🟡 中等 | pages+lru 分别 Mutex | 合并为单锁或使用 RwLock |
| **MemoryPool 无容量限制** | 🟡 中等 | 可能无限增长 | 添加容量上限 |
| **EncryptedBufferPool 克隆** | 🟡 中等 | 每次解密都 clone Page | 零拷贝解密 |
| **FileStorage 无页面缓存** | 🟡 中等 | 每次都读磁盘 | 集成 BufferPool |

### 4.4 Buffer Pool 优化方案

```
当前 LRU: VecDeque<u32> + retain()
  get(page_id=42):
    1. pages.lock() → 查找 → 命中
    2. lru.lock() → retain(|&id| id != 42) → push_front(42)  // O(N)
    3. 解锁

优化 LRU: HashMap<u32, Node> + 双向链表
  get(page_id=42):
    1. pages.lock() → 查找 → 命中
    2. lru.lock() → map[42].move_to_front()  // O(1)
    3. 解锁

  预期: 每次页面访问从 O(N) 降到 O(1)
  在 1000 页 Buffer Pool 中: 1000x 提升
```

## 5. 与其他模块的依赖

```
BufferPool
  ├── 依赖: storage::page::Page
  ├── 依赖: storage::encryption::Crypt
  ├── 被依赖: FileStorage (页面缓存)
  ├── 被依赖: BTreeIndex (节点缓存)
  └── 被依赖: ClusteredIndex (聚簇页面)

MemoryStorage
  ├── 被依赖: MemoryExecutionEngine
  ├── 被依赖: 几乎所有测试
  └── 被依赖: mysql-server (⚠️ 紧耦合)

FileStorage
  ├── 被依赖: 72小时稳定性测试
  ├── 被依赖: 生产环境
  └── 被依赖: BTreeIndex (索引管理)
```

## 变更记录

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-05-15 | v2.0 | 基于 GitNexus 重新分析，补充 Buffer Pool LRU O(N) 问题、存储引擎对比 |
