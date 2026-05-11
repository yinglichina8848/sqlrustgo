# v3.1.0 架构重构详细计划

> **版本**: 1.0  
> **日期**: 2026-05-11  
> **状态**: 🟡 规划中  
> **目标**: 系统性重构核心存储和事务层，支持聚簇索引、Gap Locking 和存储加密

---

## 一、重构概览

### 1.1 重构范围

| 模块 | 当前状态 | 重构目标 | 风险等级 |
|------|---------|---------|---------|
| **B+Tree 存储层** | 无聚簇索引 | 聚簇索引 + 二级索引联动 | 🔴 高 |
| **事务锁管理器** | 表级/行级锁 | Gap Locking + Next-Key | 🔴 高 |
| **审计日志** | 部分完成 | Crash-safe SHA-256 链 | 🟠 中 |
| **存储加密** | 无 | AES-256-GCM 页级加密 | 🟠 中 |
| **权限执行** | 解析层 | RBAC 执行层 | 🟡 低 |

### 1.2 重构原则

```
R1. 增量重构：每个模块独立可测试，不破坏现有功能
R2. 接口不变：StorageEngine trait 保持兼容
R3. 零停机：所有重构支持回滚
R4. 测试先行：每个重构先写契约测试，再实现
R5. 性能基准：记录重构前 QPS，重构后回归 ≤ 5%
```

---

## 二、B+Tree 聚簇索引重构

### 2.1 当前架构

```
┌─────────────────────────────────────────────────────────────┐
│              当前 B+Tree 存储                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Non-Leaf Node:                                             │
│  [P0 | K1 | P1 | K2 | P2 | ...]                          │
│                                                             │
│  Leaf Node:                                                 │
│  [K1 | Ptr(V1)] [K2 | Ptr(V2)] [K3 | Ptr(V3)] ...        │
│           ↓              ↓              ↓                   │
│        Data Page 1   Data Page 2   Data Page 3             │
│                                                             │
│  问题: 主键查询需要 2 次 I/O (索引 + 数据页)              │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 目标架构

```
┌─────────────────────────────────────────────────────────────┐
│              聚簇 B+Tree 存储                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Non-Leaf Node:                                             │
│  [P0 | K1 | P1 | K2 | P2 | ...]                          │
│                                                             │
│  Leaf Node (Clustered):                                     │
│  [K1 | RowData1] [K2 | RowData2] [K3 | RowData3] ...     │
│   └─ 主键    └─ 完整行数据（直接在叶子节点）              │
│                                                             │
│  二级索引 Leaf Node:                                        │
│  [IdxKey1 | PK1] [IdxKey2 | PK2] [IdxKey3 | PK3] ...   │
│                ↓           ↓            ↓                  │
│         查主键索引 → 聚簇叶节点 → RowData                 │
│                                                             │
│  优势: 主键查询 1 次 I/O，范围扫描更优的数据局部性         │
└─────────────────────────────────────────────────────────────┘
```

### 2.3 数据结构变更

#### LeafNode 新布局

```rust
// 当前: KeyPointer
pub struct KeyPointer {
    pub key: Key,
    pub value_page_id: PageId,
    pub value_offset: u16,
}

// 重构后: ClusteredLeafEntry
pub struct ClusteredLeafEntry {
    pub key: Key,           // 主键
    pub row_data: Vec<u8>,   // 行数据直接存储
    pub tx_id: u64,         // 事务 ID（MVCC）
    pub visible: bool,      // 可见性标记
}

pub struct SecondaryIndexEntry {
    pub index_key: Key,      // 二级索引键
    pub primary_key: Key,    // 指向主键
}
```

#### TableMeta 新增字段

```rust
pub struct TableMeta {
    pub id: TableId,
    pub name: String,
    pub schema: Schema,
    pub primary_key: Vec<ColumnId>,    // 主键列
    pub clustered_index_id: IndexId,   // 聚簇索引 ID
    pub secondary_indexes: Vec<IndexId>, // 二级索引列表
    pub hidden_pk: ColumnId,           // 隐藏主键（无主键时生成 UUID）
}
```

### 2.4 重构步骤

#### Phase 1: 隐藏主键支持（不破坏现有功能）

```
步骤 1.1: 在 Column 表中增加 hidden_pk_column
步骤 1.2: TableMeta 读取时若无主键自动生成 hidden_pk
步骤 1.3: INSERT 时自动填充 hidden_pk（UUID）
步骤 1.4: 契约测试: 无主键表的 hidden_pk 自动生成且唯一
```

#### Phase 2: 聚簇 LeafNode 实现

```
步骤 2.1: 定义 ClusteredLeafNode trait
步骤 2.2: 实现 ClusteredLeafNode::new() / get() / put() / delete()
步骤 2.3: 修改 BplusTree::insert() 区分聚簇/非聚簇路径
步骤 2.4: 契约测试: 聚簇表主键查询返回完整行数据
```

#### Phase 3: 二级索引联动

```
步骤 3.1: INSERT 时同步更新所有二级索引
步骤 3.2: UPDATE 时同步更新所有二级索引（old_pk → new_pk）
步骤 3.3: DELETE 时同步删除所有二级索引条目
步骤 3.4: 契约测试: 二级索引正确指向主键，无孤立索引
```

#### Phase 4: 聚簇/非聚簇路径切换

```
步骤 4.1: TableMeta 标志 clustered: bool
步骤 4.2: StorageEngine trait 增加 put_clustered() / get_clustered()
步骤 4.3: 通过 feature flag 切换新旧实现
步骤 4.4: 回归测试: 聚簇/非聚簇路径行为一致
```

### 2.5 核心文件变更

```
crates/storage/src/bplus_tree/
├── mod.rs              [重构] 统一 LeafNode trait
├── index.rs            [重构] 支持聚簇/非聚簇
├── clustered_leaf.rs    [新建] 聚簇叶子节点实现
├── secondary_index.rs   [新建] 二级索引联动
└── bplus_tree.rs       [重构] insert/get 路径分支

crates/storage/src/
├── storage_engine.rs    [重构] StorageEngine trait 增加 clustered 方法
├── table.rs            [重构] TableMeta 增加聚簇字段
└── clustered_table.rs   [新建] 聚簇表管理层
```

### 2.6 性能影响分析

| 操作 | 当前 | 聚簇后 | 变化 |
|------|------|--------|------|
| 主键点查 | 2 I/O | 1 I/O | **-50%** |
| 范围扫描 | N+1 I/O | N I/O | **-1 I/O** |
| 插入 | 1 I/O | 1 I/O | 不变 |
| 更新索引列 | 更新数据页 + 索引页 | 更新聚簇叶 + 所有二级索引 | **+N I/O** |

---

## 三、事务锁管理器重构（Gap Locking）

### 3.1 当前锁实现

```rust
// 当前: 简单行锁
pub enum LockMode { Shared, Exclusive }

pub struct RowLock {
    pub page_id: PageId,
    pub slot_id: SlotId,
    pub mode: LockMode,
    pub tx_id: u64,
}
```

**缺失能力**:
- Gap Lock（间隙锁）
- Next-Key Lock（记录+间隙）
- 谓词锁（Predicate Lock）

### 3.2 目标锁架构

```
┌─────────────────────────────────────────────────────────────┐
│              Gap Locking 架构                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  LockManager                                                │
│  ├── lock_table: HashMap<LockKey, LockEntry>              │
│  ├── wait_graph: DiGraph<LockKey, TxId>  // 死锁检测     │
│  └── gap_locks: BTreeMap<(PageId, KeyRange), GapLock>     │
│                                                             │
│  LockKey = (table_id, lock_type, key_or_range)            │
│                                                             │
│  GapLock 类型:                                              │
│  ├── RECORD_LOCK(table, row_key)          // 记录锁        │
│  ├── GAP_LOCK(table, key_range)            // 间隙锁        │
│  ├── NEXT_KEY_LOCK(table, key)             // Next-Key      │
│  └── PREDICATE_LOCK(table, predicate)      // 谓词锁（未来）│
│                                                             │
│  LockEntry:                                                  │
│  ├── holders: Vec<TxId>           // 锁持有者              │
│  ├── waiters: Vec<TxId>           // 等待队列              │
│  ├── mode: LockMode                                         │
│  └── gap_range: Option<(Key, Key)>  // 间隙范围           │
└─────────────────────────────────────────────────────────────┘
```

### 3.3 Next-Key Lock 算法

```rust
// 等值查询: SELECT * FROM t WHERE idx = 5 FOR UPDATE
// 1. 锁定 idx = 5 这个记录 → RECORD_LOCK(t, 5)
// 2. 锁定 (5, next_key) 间隙 → GAP_LOCK(t, (5, +∞))

// 范围查询: SELECT * FROM t WHERE idx BETWEEN 3 AND 7 FOR UPDATE
// 1. 锁定 [3, 7] 区间所有记录 → NEXT_KEY_LOCK(t, 3...7)
// 2. 锁定 (7, +∞) 间隙 → GAP_LOCK(t, (7, +∞))

// 插入: INSERT INTO t VALUES (6)
// 1. 检查 6 所在间隙是否有 GAP_LOCK
// 2. 检查 6 这个记录是否有 RECORD_LOCK
// 3. 无冲突则获 AUTO_INC_LOCK
```

### 3.4 重构步骤

#### Phase 1: LockManager 扩展

```
步骤 1.1: LockKey 增加 gap_range 字段
步骤 1.2: 实现 LockManager::lock_gap(table, range, mode)
步骤 1.3: 实现 LockManager::lock_next_key(table, key, mode)
步骤 1.4: 契约测试: 间隙锁互相兼容/互斥规则
```

#### Phase 2: 锁协议集成

```
步骤 2.1: 修改 RowLockManager 在范围扫描时请求 Next-Key Lock
步骤 2.2: 修改 RowLockManager 在插入时请求 Gap Lock
步骤 2.3: 实现锁升级（SHARED → EXCLUSIVE）
步骤 2.4: 契约测试: Gap Lock 防止幻读
```

#### Phase 3: SSI 死锁检测增强

```
步骤 3.1: 扩展 wait_graph 支持 gap_locks
步骤 3.2: 实现 wait_for_graph 的偏序检测
步骤 3.3: SSI 检测 Write-Read / Write-Write 偏序
步骤 3.4: 契约测试: SSI 死锁 < 100ms 检测
```

### 3.5 核心文件变更

```
crates/transaction/src/
├── lock_manager.rs      [重构] LockManager 扩展 gap 支持
├── gap_lock.rs          [新建] GapLock 类型与逻辑
├── next_key_lock.rs     [新建] Next-Key Lock 算法
├── ssi_detector.rs      [重构] SSI 偏序检测增强
└── lock_integration.rs  [新建] 锁协议集成测试
```

---

## 四、存储加密架构

### 4.1 加密设计

```
┌─────────────────────────────────────────────────────────────┐
│              页级加密架构                                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  PageLayout (16KB):                                        │
│  ┌────────┬──────────────────────────────────┬─────────┐ │
│  │  IV    │        Encrypted Data             │  Tag    │ │
│  │ 12 B   │         ~16360 B                  │  12 B   │ │
│  └────────┴──────────────────────────────────┴─────────┘ │
│                                                             │
│  Key Derivation:                                            │
│  master_key (32B) → HKDF-SHA256 → page_key (32B)         │
│  page_key + IV → AES-256-GCM → ciphertext + tag           │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 密钥管理

```rust
pub trait KeyProvider: Send + Sync {
    fn get_master_key(&self) -> Result<[u8; 32], CryptoError>;
    fn rotate_key(&mut self) -> Result<(), CryptoError>;
}

pub struct EnvKeyProvider;    // 环境变量 SQLRUSTGO_MASTER_KEY
pub struct FileKeyProvider;   // 文件存储 (0600)
```

### 4.3 重构步骤

```
Phase 1: 加密原语
  1.1 实现 Aes256Gcm cipher
  1.2 实现 KeyProvider trait (Env, File)
  1.3 实现 PageCipher (encrypt/decrypt_page)
  1.4 契约测试: encrypt → decrypt = original

Phase 2: 存储层集成
  2.1 修改 FileStorage::read_page 调用 PageCipher.decrypt
  2.2 修改 FileStorage::write_page 调用 PageCipher.encrypt
  2.3 添加 encrypt_storage feature flag
  2.4 回归测试: 加密前后数据完全一致

Phase 3: 密钥轮转
  3.1 实现 key_version 和 re-encrypt
  3.2 启动时检测密钥版本，不一致时重新加密
  3.3 契约测试: 密钥轮转后数据可读
```

### 4.4 核心文件变更

```
crates/storage/src/encryption/
├── aes_cipher.rs      [新建] AES-256-GCM 实现
├── key_manager.rs     [新建] 密钥管理 trait
├── page_cipher.rs     [新建] 页级加解密
└── mod.rs

crates/storage/src/
└── file_storage.rs    [重构] 集成 PageCipher
```

---

## 五、审计日志重构

### 5.1 当前问题

```
当前审计: 分散在 gmp crate，无 SHA-256 链，无崩溃安全保证
```

### 5.2 目标架构

```
┌─────────────────────────────────────────────────────────────┐
│              不可变审计链                                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  AuditEntry:                                                │
│  ┌─────────────────────────────────────────────────────┐  │
│  │  seq: u64              // 序列号                     │  │
│  │  prev_hash: [u8; 32]   // 前一条 SHA-256            │  │
│  │  timestamp: i64         // Unix 时间戳                │  │
│  │  user_id: u64                                         │  │
│  │  session_id: u64                                     │  │
│  │  action: ActionType   // DML/DDL/AUTH/...           │  │
│  │  sql: String                                         │  │
│  │  row_count: u64                                      │  │
│  │  tx_id: u64                                          │  │
│  │  hash: [u8; 32]     // 当前条 SHA-256               │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
│  链式: Entry[N].prev_hash = SHA256(Entry[N-1])            │
│                                                             │
│  WAL Integration:                                           │
│  1. 审计条目作为 WAL record 写入                            │
│  2. checkpoint 时审计链 snapshot 一起刷盘                    │
│  3. 崩溃恢复时 WAL replay 验证链完整性                      │
└─────────────────────────────────────────────────────────────┘
```

### 5.3 重构步骤

```
Phase 1: 审计链核心
  1.1 定义 AuditEntry 结构（含 seq, prev_hash, hash）
  1.2 实现 hash_chain(entry) → SHA-256
  1.3 实现 AuditLog::append(entry) 验证 prev_hash
  1.4 实现 AuditLog::verify_chain() 启动时全链验证

Phase 2: WAL 集成
  2.1 事务 commit 时写审计 WAL record
  2.2 checkpoint 包含审计链 state
  2.3 崩溃恢复 replay 验证审计链

Phase 3: 篡改检测
  3.1 后台线程定期扫描链完整性
  3.2 检测到断链时触发 alarm
  3.3 证据导出（含完整链 + 签名）
```

### 5.4 核心文件变更

```
crates/gmp/src/
├── audit_chain.rs      [新建] 不可变审计链
├── audit_wal.rs        [新建] WAL 集成
├── tamper_detector.rs  [新建] 篡改检测
└── evidence.rs         [重构] 证据导出

crates/transaction/src/
└── audit_integration.rs [新建] 事务层审计 hook
```

---

## 六、重构测试策略

### 6.1 契约测试（重构前必须通过）

```rust
// 每个重构模块的契约测试必须先通过，再实现新逻辑

#[test]
fn test_clustered_pk_lookup_single_io() {
    // 聚簇主键查询必须返回完整行数据
    let table = create_clustered_table("test", PRIMARY_KEY);
    let row = table.insert(("k1", "v1"));
    let retrieved = table.get_primary_key(row.pk);
    assert_eq!(retrieved, row);  // 数据完全一致
}

#[test]
fn test_gap_lock_prevents_phantom_read() {
    // Gap Lock 必须防止幻读
    let lm = LockManager::new();
    let tx1 = spawn_tx();
    let tx2 = spawn_tx();

    // tx1: SELECT * FROM t WHERE idx BETWEEN 1 AND 10 FOR UPDATE
    lm.lock_range(&tx1, table, (1, 10), LockMode::Exclusive);

    // tx2: INSERT INTO t VALUES (5, ...) — 必须被阻塞
    let result = lm.try_lock_gap(&tx2, table, 5, LockMode::Exclusive);
    assert!(result.is_none()); // 间隙被锁定
}
```

### 6.2 回归测试

```bash
# 重构前后对比
cargo test --all-features  # 必须全部通过
cargo bench --樯戴 --release  # QPS 回归 ≤ 5%
```

### 6.3 混沌测试

```bash
# 崩溃注入（见 GMP_COMPLIANCE_ROADMAP.md BP2-2 矩阵）
cargo test --test chaos_crash_wal_before
cargo test --test chaos_crash_wal_after
cargo test --test chaos_crash_precommit
```

---

## 七、里程碑

```
Week 1-2:   BP2-1~BP2-6 审计链 + WAL 集成
Week 3-7:   Gap Locking + SERIALIZABLE 完整实现
Week 8-13:  聚簇索引 Phase 1-4
Week 14-18: 存储加密 Phase 1-3
Week 19-22: 列级权限 + RBAC 执行层
Week 23-24: 集成测试 + 性能调优
```

---

*本文档由 hermes agent 创建。*
*每次架构重构里程碑后更新状态。*
