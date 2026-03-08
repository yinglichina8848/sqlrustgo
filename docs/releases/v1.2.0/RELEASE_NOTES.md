# SQLRustGo v1.2.0 发行说明

> **版本**: v1.2.0
> **代号**: Vector Engine
> **发布日期**: 待定
> **状态**: 🔄 开发中
> **版本类型**: 🏗️ 架构重构 + 接口抽象

---

## 版本亮点

v1.2.0 是**架构重构和接口抽象版本**，核心目标是为 v2.0 分布式架构打基础，同时提升性能到百万行级。

### 🚀 核心特性

- **🏗️ 架构重构**: 抽象存储引擎、执行器、统计信息接口，为分布式做准备
- **📐 接口抽象**: 定义 RecordBatch、Array、Operator、CostModel 等核心 trait
- **🚀 性能提升**: 向量化执行优化，支持百万行数据处理
- **🔄 向后兼容**: 现有 API 保持兼容，平滑升级

---

## 架构变更说明

### 架构演进

```
v1.1.x 架构                    v1.2.0 架构 (抽象化)
┌─────────────┐               ┌─────────────────────┐
│ FileStorage │               │ StorageEngine trait │
└─────────────┘               │   ├── FileStorage   │
                              │   └── MemoryStorage │
┌─────────────┐               └─────────────────────┘
│ RowExecutor │               ┌─────────────────────┐
└─────────────┘               │  VectorizedExecutor │
                              │  (基于 RecordBatch) │
┌─────────────┐               └─────────────────────┘
│ SimpleStats │               ┌─────────────────────┐
└─────────────┘               │  CostModel trait    │
                              │  + StatsCollector   │
┌─────────────┐               └─────────────────────┐
│ Hardcoded   │               ┌─────────────────────┐
│ Optimizer   │               │ Optimizer (Memo)    │
└─────────────┘               └─────────────────────┘
```

### 接口抽象层

| 模块 | v1.1.0 | v1.2.0 | 变化类型 |
|------|--------|--------|----------|
| 存储 |FileStorage (具体实现)|存储引擎特征| 抽象化 |
| 执行 |RowExecutor (行式)|矢量化执行器 + RecordBatch| 重构 |
| 统计 | 简单计数器 |表统计 + 列统计 + 统计收集器| 新增 |
| 优化 | 硬编码规则 |CostModel trait + Memo 结构| 抽象化 |
| 网络 | 同步处理 | 异步框架预留 | 预留 |

---

## 新增功能

### 向量化执行 (P0)

| 功能 | 说明 | 状态 |
|------|------|------|
|记录批次| 列式内存布局 + 网络传输支持 | ✅ 已完成 |
|ColumnarArray 特征| 列式数组抽象 | ✅ 已完成 |
| 向量化表达式 | 批量表达式计算 | ✅ 已完成 |
| 向量化 Filter | 批量过滤 | ✅ 已完成 |
|向量化 Projection| 批量投影 | ✅ 已完成 |
| 向量化聚合 | 批量聚合 | ✅ 已完成 |

### 统计信息 (P0)

| 功能 | 说明 | 状态 |
|------|------|------|
|表格统计| 表级统计信息 | ✅ 已完成 |
|列统计| 列级统计信息 | ✅ 已完成 |
| 统计信息收集器 | 自动收集统计 | ✅ 已完成 |
| 统计信息持久化 | 存储到磁盘 | ✅ 已完成 |
|ANALYZE 命令| 手动收集统计 | ✅ 已完成 |

### 简化 CBO (P1)

| 功能 | 说明 | 状态 |
|------|------|------|
|成本模型| 成本模型接口 | ✅ 已完成 |
| 基础成本估算 | 扫描/过滤成本 | ✅ 已完成 |
| Join 选择优化 | Join 顺序优化 | ✅ 已完成 |
| 索引选择优化 | 索引使用决策 | ⏳ 待完善 |

### 存储抽象 (P1)

| 功能 | 说明 | 状态 |
|------|------|------|
|存储引擎特征| 存储引擎抽象接口 | ✅ 已完成 |
|文件存储| 文件存储实现 | ✅ 已完成 |
|内存存储| 内存存储实现 | ✅ 已完成 |

---

## 性能改进

| 指标 | v1.1.0 | v1.2.0 目标 | 改进 |
|------|--------|-------------|------|
| 数据规模 | 10万行 | 100万行 | 10x |
| 简单查询延迟 | - | <100ms | - |
| 复杂查询延迟 | - | <1s | - |
| 内存效率 | 基准 | 优化 | - |

---

## API 变更

### 新增 API

```rust
// Storage Engine Trait
pub trait StorageEngine {
    fn read(&self, table: &str) -> Result<Vec<Record>>;
    fn write(&self, table: &str, records: Vec<Record>) -> Result<()>;
    fn scan(&self, table: &str, filter: Option<Filter>) -> Result<Vec<Record>>;
}

// RecordBatch
pub struct RecordBatch {
    schema: Arc<Schema>,
    columns: Vec<ArrayRef>,
    row_count: usize,
}

// Array trait
pub trait Array: Send + Sync {
    fn data_type(&self) -> &DataType;
    fn len(&self) -> usize;
    fn is_null(&self, index: usize) -> bool;
}

// TableStats
pub struct TableStats {
    pub row_count: usize,
    pub total_bytes: usize,
    pub column_stats: HashMap<String, ColumnStats>,
}

// CostModel
pub struct CostModel {
    pub seq_scan_cost: f64,
    pub idx_scan_cost: f64,
    pub filter_cost: f64,
    pub join_cost: f64,
}
```

### 兼容性

- ✅ 向后兼容 v1.1.0 API
- ✅ 现有代码无需修改
- ✅ 存储引擎可替换

---

## 为 v2.0 做准备

v1.2.0 的接口抽象为 v2.0 分布式架构打基础：

| 接口 | v2.0 扩展方向 |
|------|--------------|
|存储引擎| 可扩展为分布式存储 (Sharding) |
|记录批次|适合网络传输 (Serialization)|
|成本模型| 可扩展网络成本计算 |
| Memo | 支持分布式优化 |
|操作员| 可转换为远程执行 |

---

## 升级指南

### 从 v1.1.0 升级

1. 更新依赖版本
2. 无需修改现有代码
3. 可选：使用新的向量化 API
4. 可选：使用 StorageEngine trait 切换存储实现

详见 [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md)

---

## 贡献者

- @yinglichina8848
- @openheart-openheart
- @sonaheartopen
- @TRAE人工智能助手

---

## 已知问题

暂无

---

## 下一步计划

v1.3.0 计划：

- 完整 CBO 实现
- 分布式查询支持
- 事务隔离级别

---

## 变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-04 | 初始版本（计划中） |
| 1.1 | 2026-03-05 | 添加架构重构说明 |
