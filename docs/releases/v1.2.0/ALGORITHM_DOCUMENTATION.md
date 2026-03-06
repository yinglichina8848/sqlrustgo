# v1.2.0 核心算法文档

> ⚠️ **重要更新**: 代码已迁移到 crates/ workspace 结构，以下路径可能已变更。
> 实际位置请参考 crates/ 目录下的对应模块。

本文档记录 v1.2.0 中核心数据结构和算法的实现细节，基于实际代码。

---

## 一、B+ Tree 索引算法

### 1.1 数据结构

**位置**: `crates/storage/src/bplus_tree/tree.rs`

```rust
pub struct BPlusTree {
    root: Option<Arc<Node>>,
    size: usize,
}

pub enum Node {
    Internal(InternalNode),
    Leaf(LeafNode),
}

pub struct InternalNode {
    keys: Vec<i64>,
    children: Vec<Arc<Node>>,
}

pub struct LeafNode {
    keys: Vec<i64>,
    values: Vec<u32>,
    next: Option<Arc<Node>>,
}
```

### 1.2 核心算法

#### 1.2.1 插入算法 (insert)

```
插入(key, value):
1. 找到对应的叶子节点
2. 如果叶子节点未满:
   - 按 key 排序插入
3. 如果叶子节点已满:
   - 分裂为两个节点
   - 将中间 key 上推到父节点
   - 递归处理父节点分裂（如需要）
```

**代码位置**: `src/storage/bplus_tree/tree.rs:33-123`

| 方法 | 说明 | 时间复杂度 |
|------|------|-----------|
| `insert(key, value)` | 插入键值对 | O(log n) |
| `search(key)` | 单点查询 | O(log n) |
| `range_query(start, end)` | 范围查询 | O(log n + k) |

#### 1.2.2 分裂算法

当叶子节点键数量超过阈值时：

```
分裂(node):
1. 创建两个新节点 (left, right)
2. 将原节点前半部分键值复制到 left
3. 将原节点后半部分键值复制到 right
4. 如果是叶子节点:
   - 设置 right.next = node.next
   - node.next = right
5. 返回 middle_key（用于插入父节点）
```

### 1.3 当前限制

- **键类型**: 仅支持 `i64`
- **值类型**: 仅支持 `u32` (行指针)
- **未实现**: 删除操作、持久化

---

## 二、Hash Join 算法

### 2.1 数据结构

**位置**: `src/planner/physical_plan.rs:488-627`

```rust
pub struct HashJoinExec {
    left: Arc<dyn PhysicalPlan>,
    right: Arc<dyn PhysicalPlan>,
    on: Vec<(Column, Column)>,      // Join 条件
    join_type: JoinType,
    schema: Schema,
    // 内部状态
    build_table: HashMap<Column, Vec<RecordBatch>>,
    build_side_done: bool,
}
```

### 2.2 Join 算法

#### 2.2.1 执行流程

```
HashJoin(left_plan, right_plan, join_condition):
1. Build Phase (构建阶段):
   a. 执行 right_plan，获取右表数据
   b. 对每条记录，按 join_condition 的列构建 hash table
   c. hash_table[join_key] = [records...]

2. Probe Phase (探测阶段):
   a. 执行 left_plan，获取左表数据
   b. 对每条记录，计算 join key
   c. 在 hash table 中查找匹配
   d. 组合左右记录输出

3. Join Types:
   - Inner Join: 只输出匹配记录
   - Left Join: 左表记录始终输出，无匹配时填充 NULL
   - Right Join: 右表记录始终输出
   - Full Join: 左右表记录都输出
```

#### 2.2.2 算法图解

```
Build Phase (右表):
┌─────────────────────────────────────────┐
│  right_plan 执行                        │
│  ┌─────┬─────┬─────┐                  │
│  │ k=1 │ k=2 │ k=3 │                  │
│  └─────┴─────┴─────┘                  │
│         │                              │
│         ▼                              │
│  ┌──────────────────────────────┐      │
│  │     Hash Table              │      │
│  │  key=1 → [r1, r4]          │      │
│  │  key=2 → [r2]              │      │
│  │  key=3 → [r3, r5]         │      │
│  └──────────────────────────────┘      │
└─────────────────────────────────────────┘

Probe Phase (左表):
┌─────────────────────────────────────────┐
│  left_plan 执行                         │
│  ┌─────┬─────┬─────┐                  │
│  │ k=1 │ k=2 │ k=4 │                  │
│  └─────┴─────┴─────┘                  │
│    │     │     │                      │
│    ▼     ▼     ▼                      │
│  [r1]  [r2]  (no match)              │
│  +r1   +r2                            │
└─────────────────────────────────────────┘
```

### 2.3 时间复杂度

| 阶段 | 时间复杂度 | 空间复杂度 |
|------|-----------|-----------|
| Build Phase | O(n) | O(n) |
| Probe Phase | O(m) | - |
| Total | O(n + m) | O(n) |

其中 n = 右表行数, m = 左表行数

### 2.4 当前限制

- **Join 条件**: 仅支持等值连接 (equi-join)
- **哈希表**: 全部加载到内存
- **未实现**: 外连接优化、广播连接

---

## 三、查询优化器 (Optimizer)

### 3.1 统计信息系统

**位置**: `src/optimizer/stats.rs`

```rust
pub trait StatisticsProvider: Send + Sync {
    fn get_table_stats(&self, table: &str) -> Option<TableStats>;
    fn get_column_stats(&self, table: &str, column: &str) -> Option<ColumnStats>;
}

pub struct TableStats {
    pub row_count: u64,
    pub data_size: u64,
    pub statistics: HashMap<String, ColumnStats>,
}

pub struct ColumnStats {
    pub null_count: u64,
    pub distinct_count: u64,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
    pub histogram: Vec<f64>,
}
```

### 3.2 成本模型

**位置**: `src/optimizer/cost.rs`

```rust
pub struct Cost {
    pub cpu_cost: f64,
    pub io_cost: f64,
    pub memory_cost: f64,
}

impl Cost {
    pub fn total(&self) -> f64 {
        self.cpu_cost + self.io_cost * IO_COST_FACTOR + self.memory_cost * MEM_COST_FACTOR
    }
}
```

**成本估算公式**:

```
Scan Cost = base_cost + rows * row_cost
Join Cost = build_cost + probe_cost + output_cost
```

### 3.3 优化规则

**位置**: `src/optimizer/rules.rs`

| 规则 | 说明 | 状态 |
|------|------|------|
| PredicatePushdown | 谓词下推到存储层 | 待实现 (C-003) |
| ProjectionPruning | 裁剪不需要的列 | 待实现 (C-004) |
| ConstantFolding | 常量折叠 | 待实现 |
| JoinReordering | Join 重排序 | 待实现 (C-002) |

### 3.4 当前实现状态

- ✅ `StatisticsProvider` trait 定义
- ✅ `TableStats` / `ColumnStats` 结构
- ✅ `InMemoryStatisticsProvider` 内存实现
- ⏳ CBO 成本模型完善
- ⏳ 优化规则实现

---

## 四、执行器 (Executor)

### 4.1 物理算子

**位置**: `src/planner/physical_plan.rs`

| 算子 | 说明 | 实现状态 |
|------|------|----------|
| `SeqScanExec` | 全表扫描 | ✅ |
| `FilterExec` | 过滤 | ✅ |
| `ProjectionExec` | 投影 | ✅ |
| `HashJoinExec` | Hash 连接 | ✅ |
| `AggregateExec` | 聚合 | ✅ |
| `SortExec` | 排序 | ✅ |
| `LimitExec` | 限制数量 | ✅ |

### 4.2 算子接口

```rust
pub trait PhysicalPlan: Send + Sync {
    fn schema(&self) -> &Schema;
    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>>;
    fn execute(&self) -> Result<RecordBatch>;
}
```

### 4.3 执行模型

```
execute():
┌─────────────────────────────────────────────────┐
│  对于每个算子:                                   │
│  1. 先执行子算子获取数据                        │
│  2. 应用本算子逻辑                              │
│  3. 输出 RecordBatch                          │
└─────────────────────────────────────────────────┘
```

### 4.4 RecordBatch

```rust
pub struct RecordBatch {
    columns: Vec<Array>,
    row_count: usize,
}
```

**特性**:
- 列式存储
- 固定行数 (默认 1024)
- 惰性求值

---

## 五、版本状态

### 已实现算法

| 模块 | 算法 | 状态 |
|------|------|------|
| B+Tree | 插入、分裂、搜索 | ✅ |
| Hash Join | Build/Probe | ✅ |
| 执行器 | 物理算子 | ✅ |
| 统计信息 | Provider 接口 | ✅ |

### 待实现算法

| 模块 | 算法 | Issue |
|------|------|-------|
| 统计信息 | ANALYZE 命令 | S-005 |
| 统计信息 | 持久化 | S-004 |
| CBO | 谓词下推 | C-003 |
| CBO | 投影裁剪 | C-004 |
| CBO | Join 重排序 | C-002 |

---

## 六、文档版本

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0.0 | 2026-03-05 | 初始版本 |

---

*本文档基于 v1.2.0 develop 分支实际代码生成*
