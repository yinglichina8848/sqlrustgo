# 聚簇索引执行链路文档

> **版本**: v3.1.0
> **日期**: 2026-05-14
> **Issue**: #607, #784
> **状态**: ✅ 已完成

---

## 一、聚簇索引概述

### 1.1 什么是聚簇索引

聚簇索引（Clustered Index）是一种特殊的索引类型，其中索引的叶子节点存储完整的数据行，而不是仅存储指向数据行的指针。这意味着数据行的物理存储顺序与索引键的顺序相同。

```
MySQL InnoDB 聚簇索引结构:

主键索引叶子节点:
┌─────────────────────────────────────────────┐
│ key=1  →  [id=1, name="Alice", age=30]     │  ← 存储完整行数据
├─────────────────────────────────────────────┤
│ key=2  →  [id=2, name="Bob", age=25]       │
├─────────────────────────────────────────────┤
│ key=3  →  [id=3, name="Charlie", age=35]   │
└─────────────────────────────────────────────┘

数据页物理布局:
┌────────┬────────┬────────┬────────┐
│ Page 0 │ Page 1 │ Page 2 │ Page 3 │
│ id=1   │ id=2   │ id=3   │ id=4   │
│ "Alice"│ "Bob"  │ "Charlie"│ "Diana" │
└────────┴────────┴────────┴────────┘
         ↑ 按主键顺序物理存储
```

### 1.2 聚簇索引 vs 非聚簇索引

| 特性 | 聚簇索引 | 非聚簇索引 |
|------|----------|------------|
| 叶子节点存储 | 完整数据行 | 仅存储键值 + 主键 |
| 数据物理顺序 | 按索引键排序 | 与索引顺序无关 |
| 主键选择 | 影响数据物理布局 | - |
| 查询效率（主键查询） | O(1) 直接访问 | O(log n) + 回表 |
| 插入性能 | 需要维护物理顺序 | 仅更新索引 |
| 空间使用 | 较小（无需回表） | 较大（索引 + 数据） |

### 1.3 解决的问题

1. **范围查询优化**：主键范围扫描时数据物理连续，磁盘 I/O 更少
2. **主键查询优化**：主键查询无需回表，直接获取数据
3. **排序优化**：ORDER BY 主键天然有序
4. **覆盖索引优化**：更多查询可被索引覆盖

---

## 二、实际实现状态

### 2.1 聚簇索引模块 (crates/storage/src/clustered_index/)

| 文件 | 功能 |
|------|------|
| `mod.rs` | 模块导出 |
| `leaf.rs` | ClusteredLeafPage 实现 |
| `overflow.rs` | OverflowManager 溢出页管理 |
| `transaction.rs` | ClusteredPageTransaction WAL 事务 |
| `wal_integration.rs` | ClusteredWalEntry, ClusteredWalManager |
| `invariant_tests.rs` | 不变性测试 |
| `wal_recovery_tests.rs` | WAL 恢复测试 |

### 2.2 核心数据结构

```rust
// leaf.rs
pub struct ClusteredLeafPage {
    data: Vec<u8>,           // 页面数据
    slot_count: u16,         // 槽数量
    free_space_start: u16,   // 空闲空间起始位置
    data_end: u16,           // 数据结束位置
}

// transaction.rs
pub struct ClusteredPageTransaction<'a> {
    page: &'a mut ClusteredLeafPage,
    wal_manager: &'a ClusteredWalManager,
    tx_id: u64,
}
```

### 2.3 实现状态

| 特性 | 状态 | 说明 |
|------|------|------|
| ClusteredLeafPage | ✅ 已实现 | 存储完整行数据 |
| OverflowManager | ✅ 已实现 | 溢出页管理 |
| ClusteredPageTransaction | ✅ 已实现 | WAL 事务支持 |
| ClusteredWalManager | ✅ 已实现 | WAL 集成 |
| WAL 恢复 | ✅ 已实现 | 63 测试通过 |
| 页面分裂 | ✅ 已实现 | test_needs_split |

---

## 三、执行链路

### 3.1 INSERT 执行链路

```
INSERT INTO t VALUES (1, "Alice", 30)
  ↓
Parser: 解析 INSERT
  ↓
Planner: 生成 logical plan
  ↓
Optimizer: 选择聚簇索引
  ↓
ClusteredPageTransaction::insert():
  ├→ WAL log_insert()  ← 先写 WAL
  │   └→ 失败则不修改页面
  │
  └→ ClusteredLeafPage::insert()
      ├→ 编码固定列
      ├→ 编码变长列
      ├→ 更新槽目录
      └→ 返回 slot_idx
```

### 3.2 DELETE 执行链路

```
DELETE FROM t WHERE id = 5
  ↓
ClusteredPageTransaction::delete(slot_idx):
  ├→ 获取 cluster_key 用于 WAL
  ├→ WAL log_delete()
  │   └→ 失败则不修改页面
  │
  └→ ClusteredLeafPage::delete()
      └→ 标记槽为已删除
```

### 3.3 页面分裂

```
当 free_space < MIN_FREE_SPACE_THRESHOLD (128 bytes):
  ↓
ClusteredLeafPage::needs_split() → true
  ↓
页面分裂:
  ├→ 分配新页面
  ├→ 移动后半部分数据到新页
  ├→ 更新相邻页面指针
  └→ 更新父节点索引
```

---

## 四、WAL 集成

### 4.1 ClusteredWalEntry

```rust
pub enum ClusteredWalEntry {
    Insert {
        cluster_key: ClusterKey,
        fixed_columns: Vec<Value>,
        varlen_columns: Vec<Option<Vec<u8>>>,
        null_bitmap: Vec<bool>,
    },
    Delete {
        cluster_key: ClusterKey,
        slot_idx: u16,
    },
    Update {
        old_key: ClusterKey,
        new_key: ClusterKey,
        // ...
    },
}
```

### 4.2 恢复流程

```
崩溃恢复:
  ↓
读取 WAL 文件
  ↓
按 tx_id 分组日志
  ↓
过滤未提交事务
  ↓
重放已提交事务:
  ├→ INSERT → 重新插入
  ├→ DELETE → 标记删除
  └→ UPDATE → 删除旧值，插入新值
```

---

## 五、测试验证

### 5.1 单元测试 (63 tests)

| 测试类别 | 测试数 | 状态 |
|---------|--------|------|
| leaf::tests | 覆盖 insert/get/delete/split | ✅ |
| overflow::tests | 溢出链分配/释放 | ✅ |
| transaction::tests | WAL 日志/提交 | ✅ |
| wal_recovery_tests | 崩溃恢复场景 | ✅ |

### 5.2 关键测试用例

| 测试 | 说明 |
|------|------|
| `test_transaction_insert_logs_to_wal` | INSERT 写入 WAL |
| `test_transaction_delete_logs_to_wal` | DELETE 写入 WAL |
| `test_recover_primary_key_cluster` | 主键聚簇恢复 |
| `test_recover_to_timestamp` | PITR 时间点恢复 |
| `test_needs_split` | 页面分裂触发 |
| `test_encode_overflow_chain_multi_page` | 溢出链编解码 |

---

## 六、参考

- `crates/storage/src/clustered_index/` - 聚簇索引完整实现
- `crates/storage/src/row_format/` - Compact Row 编码格式
- `crates/storage/src/wal.rs` - WAL 基础设施

### 2.2 现有 B+Tree 实现

```rust
// crates/storage/src/bplus_tree/index.rs
pub struct BPlusTree {
    root_page_id: u32,
    buffer_pool: Arc<BufferPool>,
}

impl BPlusTree {
    pub fn search(&self, key: &[u8]) -> Option<Vec<u8>>;
    pub fn insert(&self, key: &[u8], value: &[u8]) -> Result<()>;
    pub fn delete(&self, key: &[u8]) -> Result<()>;
}
```

---

## 三、执行链路

### 3.1 当前表读取链路（非聚簇）

```
SELECT * FROM users WHERE id = 1
  ↓
Parser: 解析为 Get { table: "users", key: 1 }
  ↓
Planner: 生成 PhysicalPlan
  ↓
Optimizer: 选择最优索引（如果有）
  ↓
Executor:
  ├→ TableScan::new("users")
  │   └→ StorageEngine::scan()
  │       └→ BPlusTree::search(primary_key)
  │           └→ 返回 row_id
  │
  └→ HeapFetch::fetch(row_id)
      └→ 从 Heap 表中获取完整行
      └→ 可能跨多个页面（随机 I/O）
```

### 3.2 聚簇索引读取链路（规划）

```
SELECT * FROM users WHERE id = 1
  ↓
Optimizer: 选择 clustered_index_scan
  ↓
Executor:
  └→ ClusteredIndexScan::new("users", primary_key=1)
      └→ BPlusTree::search_in_clustered(key=1)
          └→ 直接在 B+Tree 叶子节点返回完整行
          └→ 无需回表（O(1) 访问）
```

### 3.3 聚簇索引插入链路（规划）

```
INSERT INTO users (id, name, age) VALUES (100, "Zara", 28)
  ↓
Executor:
  ├→ ClusteredIndex::locate_insert_position(key=100)
  │   └→ BPlusTree::search(gap_key=100)
  │       └→ 找到要插入的叶子节点
  │
  ├→ BPlusTree::insert_leaf(key=100, row_data)
  │   └→ 叶子节点存储完整行数据
  │   └→ 可能触发节点分裂（级联向上）
  │
  └→ 如果有二级索引
      └→ 同时更新所有二级索引
```

### 3.4 聚簇索引范围扫描链路（规划）

```
SELECT * FROM users WHERE id BETWEEN 10 AND 50
  ↓
Executor:
  └→ ClusteredRangeScan::new("users", start=10, end=50)
      ├→ BPlusTree::search_lower_bound(start=10)
      │   └→ 定位起始叶子节点
      │
      └→ 顺序扫描叶子节点直到 end=50
          └→ 数据物理连续 → 顺序 I/O → 高效
```

---

## 四、关键设计决策

### 4.1 主键选择策略

| 策略 | 优点 | 缺点 | 适用场景 |
|------|------|------|----------|
| 用户声明主键 | 业务语义清晰 | 可能非单调 | 有明确业务主键 |
| 自动生成 rowid | 插入高效 | 无业务含义 | 日志表、审计表 |
| UUID | 全局唯一 | 插入随机 | 分布式场景 |

### 4.2 聚簇索引结构设计

```rust
// 聚簇索引叶子节点
pub struct ClusteredLeafNode {
    pub is_leaf: bool,              // true
    pub num_keys: u16,             // 当前键数量
    pub keys: Vec<i64>,             // 主键数组
    pub row_data: Vec<RowData>,     // 完整行数据（直接在叶子节点）
    pub next_leaf: Option<u32>,    // 下一个叶子节点
}

// 行数据直接内联存储
pub struct RowData {
    pub columns: Vec<Value>,        // 所有列值
}
```

### 4.3 二级索引与聚簇索引的关系

```
表: users (id PRIMARY KEY, name, age)

聚簇索引:
└── BPlusTree
    └── 叶子: [id=1, name="Alice", age=30]
              [id=2, name="Bob", age=25]
              [id=3, name="Charlie", age=35]

二级索引 (name):
└── BPlusTree
    └── 叶子: [name="Alice", id=1]  ← 存储主键引用
              [name="Bob", id=2]
              [name="Charlie", id=3]

查询: SELECT * FROM users WHERE name = 'Bob'
  → 二级索引找到 id=2
  → 回表到聚簇索引获取完整行
```

---

## 五、实现计划

### 5.1 第一阶段：基础结构

| 任务 | 优先级 | 说明 |
|------|--------|------|
| 设计 ClusteredBPlusTree 结构 | P0 | 新结构或扩展现有 B+Tree |
| 实现 ClusteredLeafNode | P0 | 叶子节点存储完整行数据 |
| 修改 TableInfo 增加 clustered_index_id | P0 | 表元数据支持聚簇索引 |
| 实现 ClusteredIndexScan 算子 | P0 | 替代 IndexScan |
| 实现 ClusteredIndexInsert 算子 | P0 | 替代 IndexInsert |

### 5.2 第二阶段：二级索引联动

| 任务 | 优先级 | 说明 |
|------|--------|------|
| 修改 SecondaryIndex 支持聚簇回表 | P1 | 二级索引存储主键引用 |
| 实现 IndexOnlyScan 优化 | P1 | 覆盖索引无需回表 |
| 处理唯一索引/主键约束冲突检测 | P1 | 插入时检查唯一性 |

### 5.3 第三阶段：DDL 支持

| 任务 | 优先级 | 说明 |
|------|--------|------|
| ALTER TABLE ADD PRIMARY KEY | P2 | 添加主键（触发聚簇索引构建） |
| 处理无主键表（自动生成 rowid 聚簇） | P2 | 对于无主键表使用隐藏 rowid |
| DROP INDEX 维护 | P2 | 删除索引时的清理 |

---

## 六、测试计划

### 6.1 单元测试

| 测试 | 验证点 |
|------|--------|
| clustered_insert_test | 插入后数据正确存储 |
| clustered_search_test | 主键查询无需回表 |
| clustered_range_scan_test | 范围扫描顺序正确 |
| secondary_index_test | 二级索引正确回表 |

### 6.2 集成测试

| 测试 | 验证点 |
|------|--------|
| clustered_crud_test | 完整 CRUD 操作 |
| clustered_secondary_test | 二级索引 + 聚簇索引联动 |
| clustered_ddl_test | DDL 对聚簇索引的影响 |

### 6.3 性能测试

| 测试 | 目标 |
|------|------|
| clustered_select_qps | 主键查询 QPS 提升 |
| clustered_range_qps | 范围扫描延迟降低 |
| clustered_insert_qps | 插入性能对比 |

---

## 七、相关文件

| 文件 | 作用 |
|------|------|
| `crates/storage/src/clustered_index.rs` | 聚簇索引核心实现（待创建） |
| `crates/storage/src/bplus_tree/index.rs` | B+Tree 基础实现 |
| `crates/executor/src/operators/clustered_scan.rs` | 聚簇索引扫描算子（待创建） |
| `crates/planner/src/clustered_index_planner.rs` | 聚簇索引规划器（待创建） |
| `tests/integration/test_clustered_index.rs` | 集成测试（待创建） |