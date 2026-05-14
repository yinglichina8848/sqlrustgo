# v3.1.0 聚簇索引 + 存储加密 实现报告

> **日期**: 2026-05-14
> **PR**: #778
> **状态**: ✅ 已合并到 develop/v3.1.0

---

## 一、概述

本次实现完成了 v3.1.0 聚簇索引和存储加密的核心功能，为后续 Phase 2/3 奠定了基础。

### 完成内容

| 组件 | 功能点 | 文件 |
|------|--------|------|
| **聚簇索引扫描** | ClusteredIndexScanExec + VolcanoExecutor | planner, executor |
| **二级索引联动** | SecondaryIndex + IndexOnlyScan 优化 | storage |
| **存储加密 MVP** | AES-256-GCM + EncryptedBufferPool | storage |

---

## 二、聚簇索引实现

### 2.1 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                      Executor Layer                          │
│  ClusteredIndexScanVolcanoExecutor                          │
│  - init(): 使用 ClusteredLeafPage.lower_bound/upper_bound  │
│  - next(): 返回 Rows                                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Planner Layer                           │
│  ClusteredIndexScanExec                                     │
│  - cluster_key_expr, cluster_key_range                      │
│  - schema, projection                                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Storage Layer                          │
│  ClusteredLeafPage                                         │
│  - insert(cluster_key, row_data) → slot_idx                │
│  - get(slot_idx) → ClusteredLeafRecord                     │
│  - lower_bound(key) / upper_bound(key) → range scan        │
│  - split() → page split                                   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 核心 API

**ClusteredLeafPage** (`leaf.rs`):
```rust
pub fn insert(&mut self, cluster_key: &ClusterKey, ...) -> Result<u16>
pub fn get(&self, slot_idx: u16) -> Result<Option<ClusteredLeafRecord>>
pub fn get_cluster_key(&self, slot_idx: u16) -> Result<Option<ClusterKey>>
pub fn lower_bound(&self, key: &ClusterKey) -> Option<u16>
pub fn upper_bound(&self, key: &ClusterKey) -> Option<u16>
pub fn delete(&mut self, slot_idx: u16) -> Result<()>
pub fn split(&mut self, split_pos: u16) -> (ClusteredLeafPage, ClusterKey)
```

### 2.3 物理计划节点

**ClusteredIndexScanExec** (`planner/src/physical_plan.rs`):
```rust
pub struct ClusteredIndexScanExec {
    pub table_name: String,
    pub index_name: String,
    pub cluster_key_expr: Option<Expr>,
    pub cluster_key_range: Option<(i64, i64)>,
    pub schema: Schema,
    pub projection: Option<Vec<usize>>,
    pub row_count: u64,
    pub page_count: u64,
}
```

### 2.4 执行器实现

**ClusteredIndexScanVolcanoExecutor** (`executor/src/clustered_index_scan.rs`):
```rust
impl<S: StorageEngine> VolcanoExecutor for ClusteredIndexScanVolcanoExecutor<S> {
    fn open(&mut self) -> SqlResult<()>
    fn next(&mut self) -> SqlResult<Option<Vec<Value>>>
    fn close(&mut self) -> SqlResult<()>
}
```

---

## 三、二级索引联动

### 3.1 设计目标

1. **存储 cluster_key 而非 row_id** - 与聚簇索引协调工作
2. **IndexOnlyScan 优化** - 查询仅需索引时跳过回表
3. **唯一约束检查** - 唯一索引插入时验证

### 3.2 核心结构

**SecondaryIndex** (`storage/src/clustered_index/secondary_index.rs`):
```rust
pub struct SecondaryIndex {
    data: BTreeMap<SecondaryIndexKey, Vec<ClusterKey>>,
    metadata: SecondaryIndexMetadata,
}

pub struct SecondaryIndexMetadata {
    pub name: String,
    pub table_name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
}

pub enum SecondaryIndexKey {
    Single(Value),
    Composite(Vec<Value>),
}
```

### 3.3 核心方法

| 方法 | 签名 | 说明 |
|------|------|------|
| `insert` | `insert(key: &Value, cluster_key: &ClusterKey)` | 插入索引条目 |
| `search` | `search(key: &Value) -> Vec<ClusterKey>` | 搜索返回 cluster_key 列表 |
| `search_unique` | `search_unique(key: &Value) -> Option<ClusterKey>` | 唯一索引搜索 |
| `range_query` | `range_query(start: &Value, end: &Value) -> Vec<ClusterKey>` | 范围查询 |
| `covers_query` | `covers_query(required: &[String]) -> bool` | IndexOnlyScan 检查 |
| `delete` | `delete(cluster_key: &ClusterKey)` | 删除条目 |

### 3.4 协调流程

**插入流程**:
```
INSERT INTO users (id, email, name) VALUES (1, "a@b.com", "Alice")
  │
  ▼
1. 插入聚簇索引 → 获得 cluster_key = PrimaryKey(Value::Integer(1))
  │
  ▼
2. 插入二级索引 (email) → 存储 ("a@b.com", cluster_key)
```

**搜索流程**:
```
SELECT * FROM users WHERE email = "a@b.com"
  │
  ▼
1. 搜索二级索引 → 获得 cluster_key = PrimaryKey(Value::Integer(1))
  │
  ▼
2. 用 cluster_key 搜索聚簇索引 → 获得完整行
```

**IndexOnlyScan 优化**:
```
SELECT email FROM users WHERE email = "a@b.com"
  │
  ▼
1. 检查二级索引 covers_query(["email"]) → true
  │
  ▼
2. 直接返回二级索引结果，无需回表
```

---

## 四、存储加密实现

### 4.1 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    EncryptedBufferPool                      │
│  - get_or_load_encrypted(): 加载并解密                      │
│  - flush_encrypted(): 加密并刷新                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    EncryptionManager                         │
│  - AesEncryptionManager (AES-256-GCM)                       │
│  - Crypt trait (抽象接口)                                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    EncryptedFileStorage                      │
│  - 加密页面格式: nonce(12) + ciphertext + tag(16)           │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 加密页面格式

**EncryptedPage** (`encryption.rs`):
```rust
pub struct EncryptedPage {
    pub page_id: u32,
    pub nonce: [u8; 12],      // GCM nonce
    pub ciphertext: Vec<u8>,
    pub tag: [u8; 16],       // GCM auth tag
    pub key_version: u32,
}
```

### 4.3 核心 API

**AesEncryptionManager**:
```rust
impl AesEncryptionManager {
    pub fn new(master_key: &[u8; 32]) -> Result<Self, EncryptionError>
    pub fn encrypt(&self, page_id: u32, data: &[u8], key_version: u32)
        -> Result<EncryptedPage, EncryptionError>
    pub fn decrypt(&self, page: &EncryptedPage)
        -> Result<DecryptedPage, EncryptionError>
    pub fn encrypt_bytes(&self, page_id: u32, data: &[u8], key_version: u32)
        -> Result<Vec<u8>, EncryptionError>
    pub fn decrypt_bytes(&self, page_id: u32, data: &[u8], key_version: u32)
        -> Result<Vec<u8>, EncryptionError>
}
```

**Crypt trait** (抽象接口):
```rust
pub trait Crypt: Send + Sync {
    fn encrypt_bytes(&self, page_id: u32, data: &[u8], key_version: u32)
        -> Result<Vec<u8>, EncryptionError>;
    fn decrypt_bytes(&self, page_id: u32, data: &[u8], key_version: u32)
        -> Result<Vec<u8>, EncryptionError>;
}
```

### 4.4 EncryptedBufferPool

**EncryptedBufferPool** (`buffer_pool.rs`):
```rust
pub struct EncryptedBufferPool<S: StorageEngine, C: Crypt> {
    inner: Arc<BufferPool<S>>,
    encryption: Arc<C>,
}

impl<S: StorageEngine, C: Crypt> EncryptedBufferPool<S, C> {
    pub fn new(inner: Arc<BufferPool<S>>, encryption: Arc<C>) -> Self
    pub fn get_or_load_encrypted(&self, page_id: u32) -> Result<Arc<Page>>
    pub fn flush_encrypted(&self, page_id: u32) -> Result<()>
}
```

---

## 五、测试结果

### 5.1 聚簇索引测试

**ClusteredLeafPage 测试** (`leaf.rs`):
- ✅ `test_clustered_leaf_insert_and_get` - 插入和获取
- ✅ `test_clustered_leaf_delete` - 删除标记
- ✅ `test_clustered_leaf_lower_bound` - 下界查找
- ✅ `test_clustered_leaf_upper_bound` - 上界查找
- ✅ `test_clustered_leaf_split` - 页面分裂
- ✅ `test_clustered_leaf_compact` - 碎片整理

**WAL 恢复测试** (`wal_recovery_tests.rs`):
- ✅ `test_recover_uncommitted_inserts` - 未提交插入恢复
- ✅ `test_recover_to_timestamp` - 时间点恢复
- ✅ `test_recovery_with_primary_key_cluster_key` - 主键簇键恢复

### 5.2 二级索引测试

**SecondaryIndex 测试** (`secondary_index_tests.rs`):
- ✅ `test_secondary_index_insert_and_search` - 插入和搜索
- ✅ `test_secondary_index_unique_constraint` - 唯一约束
- ✅ `test_secondary_index_range_query` - 范围查询
- ✅ `test_secondary_index_composite` - 复合索引
- ✅ `test_secondary_index_delete` - 删除
- ✅ `test_secondary_index_null_handling` - NULL 处理
- ✅ `test_secondary_index_coordination` - 协调测试
- ✅ `test_secondary_index_covers_query` - IndexOnlyScan 检查

**测试覆盖**:
```
secondary_index.rs:
  test_secondary_index_insert_and_search ........ ok
  test_secondary_index_unique_constraint ........ ok
  test_secondary_index_range_query ............. ok
  test_secondary_index_composite .............. ok
  test_secondary_index_delete ................. ok
  test_secondary_index_null_handling .......... ok
  test_secondary_index_coordination ........... ok
  test_secondary_index_covers_query .......... ok
  test_secondary_index_multiple_cluster_keys .. ok
  test_secondary_index_delete_by_value ....... ok
  test_secondary_index_search_all ............ ok
  test_secondary_index_various_value_types .... ok
  test_secondary_index_clear .................. ok
  test_secondary_index_stats ................. ok
  test_secondary_index_from_metadata ......... ok
  test_secondary_index_partial_range ......... ok
  test_secondary_index_insert_twice_different_keys ... ok
  test_secondary_index_reinsert_deleted ...... ok
  test_secondary_index_in_memory_only ....... ok
  test_secondary_index_constraints ........... ok
  test_secondary_index_cluster_key_lifecycle . ok
  test_cluster_key_comparison ............... ok
  test_cluster_key_ordering ................. ok
  test_cluster_key_serialization ............ ok
```

### 5.3 加密测试

**AesEncryptionManager 测试** (`encryption.rs`):
- ✅ `test_aes_encryption_manager_new_invalid_key_size` - 密钥大小验证
- ✅ `test_encrypt_decrypt_roundtrip` - 加解密往返
- ✅ `test_gcm_tag_verification` - GCM 认证标签验证
- ✅ `test_different_pages_different_nonces` - 每页不同 nonce
- ✅ `test_encrypted_page_serialization` - 序列化
- ✅ `test_decrypt_with_wrong_key` - 错误密钥检测
- ✅ `test_encrypt_decrypt_bytes` - 字节级加解密

### 5.4 测试统计

```
cargo test -p sqlrustgo-storage
  346 tests passed, 0 failed
```

---

## 六、文件清单

### 新增文件

| 文件 | 行数 | 说明 |
|------|------|------|
| `executor/src/clustered_index_scan.rs` | 216 | ClusteredIndexScan 执行器 |
| `storage/src/clustered_index/secondary_index.rs` | 420 | 二级索引实现 |
| `storage/src/clustered_index/secondary_index_tests.rs` | 500+ | 二级索引测试 |
| `storage/src/encryption.rs` | 390 | 加密核心 |
| `storage/src/encrypted_file.rs` | 250+ | 加密文件存储 |
| `storage/src/key_manager.rs` | 100+ | 密钥管理接口 |

### 修改文件

| 文件 | 修改 | 说明 |
|------|------|------|
| `planner/src/physical_plan.rs` | +80 | ClusteredIndexScanExec |
| `executor/src/local_executor.rs` | +40 | 调度逻辑 |
| `executor/src/lib.rs` | +5 | 模块导出 |
| `storage/src/buffer_pool.rs` | +200 | EncryptedBufferPool |
| `storage/src/page.rs` | +6 | 加密字段 |
| `storage/src/lib.rs` | +15 | 导出加密类型 |

---

## 七、验证命令

```bash
# 编译
cargo build --all-features

# 测试
cargo test -p sqlrustgo-storage

# Lint
cargo clippy --all-features -- -D warnings

# 格式化
cargo fmt --check --all
```

---

## 八、已知限制

### Phase 1 已完成，Phase 2/3 待做

| 功能 | 状态 | 说明 |
|------|------|------|
| 聚簇索引扫描 | ✅ 完成 | 基础功能 |
| 二级索引联动 | ✅ 完成 | 基础功能 |
| IndexOnlyScan | ✅ 完成 | 覆盖检查 |
| EncryptedBufferPool | ✅ 完成 | MVP |
| ALTER TABLE ADD PRIMARY KEY | ❌ 待做 | DDL 支持 |
| 无主键表处理 | ❌ 待做 | 隐藏 rowid |
| 密钥轮换 | ❌ 待做 | KeyManager |
| WAL 加密 | ❌ 待做 | 事务日志 |
| KMS 集成 | ❌ 待做 | 生产环境 |

---

## 九、总结

本次实现完成了 v3.1.0 聚簇索引和存储加密的核心 MVP：

1. **聚簇索引扫描**: 从存储层到执行器的完整链路
2. **二级索引联动**: 支持 IndexOnlyScan 优化的协调机制
3. **存储加密**: AES-256-GCM + BufferPool 加密包装器

所有代码通过了编译、clippy 零警告、cargo fmt 检查，346 个测试全部通过。

---

**关联 Issue**: #669
**合并 PR**: #778