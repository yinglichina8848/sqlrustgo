# SQLRustGo 1.2 接口冻结白皮书

> **版本**: 1.2
> **代号**: Vector Engine
> **类型**: 🏗️ 架构重构 + 接口抽象
> **日期**: 2026-03-05

---

## 1. 为什么需要接口冻结

数据库系统的核心问题不是功能，而是 **接口稳定性**。

一旦执行接口改变：

- 优化器需要重写
- 执行器需要重写
- 分布式调度需要重写

因此 SQLRustGo 在 **1.2 冻结核心接口**，确保：

1. 现有代码不会因为 API 变化而失效
2. 开发者可以基于稳定接口进行开发
3. 为 v2.0 分布式架构提供坚实的基础

---

## 2. 执行算子接口

这是执行引擎的核心接口：

```rust
pub trait Operator: Send {
    fn open(&mut self);
    fn next_batch(&mut self) -> Option<RecordBatch>;
    fn close(&mut self);
}
```

### 接口设计原则

1. **Batch 模式**: 使用 `next_batch` 而非 `next`，提高吞吐量
2. **无状态设计**: 通过 `open`/`close` 管理资源
3. **Send + Sync**: 支持并发执行

### 未来扩展

此接口支持未来扩展：

- **Pipeline**: 流式处理
- **Async**: 异步执行
- **Distributed**: 分布式执行

所有扩展都必须兼容此接口。

---

## 3. RecordBatch

向量化执行的核心数据结构：

```rust
pub struct RecordBatch {
    pub schema: Arc<Schema>,
    pub columns: Vec<ArrayRef>,
    pub row_count: usize,
}
```

### 设计特点

- **列式存储**: 适合向量化处理
- **网络友好**: 适合分布式传输
- **内存高效**: 批量处理减少分配

### Array 抽象

```rust
pub trait Array: Send + Sync {
    fn data_type(&self) -> &DataType;
    fn len(&self) -> usize;
    fn is_null(&self, index: usize) -> bool;
    fn as_any(&self) -> &dyn Any;
}
```

具体实现：

- `Int32Array`
- `Int64Array`
- `FloatArray`
- `DoubleArray`
- `StringArray`
- `BooleanArray`

---

## 4. PlanNode

物理计划节点：

```rust
pub enum PlanNode {
    /// 本地执行计划
    Local(PhysicalPlan),
    /// 分布式执行计划（预留）
    Distributed(DistributedPlan),
}
```

### 设计意图

- **Local**: 当前单机执行
- **Distributed**: v2.0 分布式执行预留

### DistributedPlan 结构

```rust
pub struct DistributedPlan {
    pub local_plan: PhysicalPlan,
    pub exchange_nodes: Vec<ExchangeNode>,
    pub shuffle_strategy: ShuffleStrategy,
}
```

---

## 5. Optimizer

优化器接口：

```rust
pub trait Optimizer {
    fn optimize(&self, memo: &mut Memo) -> GroupId;
}
```

### Memo 结构

```rust
pub struct Memo {
    pub groups: Vec<Group>,
}

pub struct Group {
    pub group_id: GroupId,
    pub expressions: Vec<Expr>,
    pub physical_properties: PhysicalProperties,
}
```

### 设计意图

支持未来升级为 **Cascades Optimizer**：

- 规则优化 + 成本优化
- 自底向上/自顶向下搜索
- 表达式等价转换

---

## 6. Executor

执行器接口：

```rust
pub trait Executor {
    fn execute(&self, plan: PlanNode) -> Result<ResultSet>;
}
```

### 执行流程

```
PlanNode
   │
   ▼
Executor::execute()
   │
   ├──► Operator::open()
   │
   ├──► while let Some(batch) = Operator::next_batch()
   │         └──► 处理数据
   │
   └──► Operator::close()
```

---

## 7. Catalog

目录接口：

```rust
pub trait Catalog {
    fn get_table(&self, name: &str) -> Option<TableMeta>;
}
```

### TableMeta 结构

```rust
pub struct TableMeta {
    pub name: String,
    pub schema: Schema,
    pub distribution: Distribution,
    pub statistics: Option<TableStats>,
}

pub enum Distribution {
    Single,
    Sharded { shard_count: usize },
    Replicated { replica_count: usize },
}
```

---

## 8. StorageEngine

存储引擎抽象：

```rust
pub trait StorageEngine {
    fn read(&self, table: &str) -> Result<Vec<Record>>;
    fn write(&self, table: &str, records: Vec<Record>) -> Result<()>;
    fn scan(&self, table: &str, filter: Option<Filter>) -> Result<Vec<Record>>;
    fn get_stats(&self, table: &str) -> Result<TableStats>;
}
```

### 当前实现

- **FileStorage**: 文件存储（持久化）
- **MemoryStorage**: 内存存储（测试/缓存）

---

## 9. 接口兼容性矩阵

| 接口 | 1.2 | 2.0 扩展 |
|------|-----|----------|
| Operator | ✅ 冻结 | + async |
| RecordBatch | ✅ 冻结 | + 分片支持 |
| PlanNode | ✅ 冻结 | + Distributed |
| Optimizer | ✅ 冻结 | + Cascades |
| Executor | ✅ 冻结 | + Distributed |
| Catalog | ✅ 冻结 | + Remote |
| StorageEngine | ✅ 冻结 | + Sharding |

---

## 10. 冻结原则

### 已冻结接口

1. `Operator` trait 的三个方法
2. `RecordBatch` 结构
3. `PlanNode` 枚举
4. 核心 `Optimizer` 方法签名
5. `Executor::execute` 签名

### 冻结规则

- **不删除**: 不会移除已发布的接口
- **不修改**: 不会改变接口签名
- **可扩展**: 可以添加新方法（默认实现）

---

## 附录：快速链接

- [发布白皮书](./sqlrustgo_1.2_release_whitepaper.md)
- [技术债预测](./sqlrustgo_tech_debt_forecast.md)
- [2.0 分布式框架](./sqlrustgo_2.0_distributed_framework.md)
- [版本计划](../releases/v1.2.0/VERSION_PLAN.md)
