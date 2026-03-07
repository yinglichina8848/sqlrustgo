# SQLRustGo 1.2 发布白皮书

> **版本**: 1.2
> **代号**: Vector Engine
> **类型**: 🏗️ 架构重构 + 接口抽象
> **日期**: 2026-03-05

---

## 1. 项目愿景

SQLRustGo 是一个使用 Rust 构建的现代数据库执行引擎实验项目，目标是探索：

- **向量化执行** - 高效的列式数据处理
- **成本优化器（CBO）** - 基于统计信息的查询优化
- **可扩展查询架构** - 模块化设计，支持扩展
- **分布式 SQL 执行系统** - 最终演进目标

### 项目路线图

```
1.x ──────► 2.x ──────► 3.x
 │             │             │
 ▼             ▼             ▼
单机执行    分布式执行    云原生数据库
引擎
```

---

## 2. SQLRustGo 架构总览

```
         SQL
          │
          ▼
       Parser
          │
          ▼
    Logical Plan
          │
          ▼
   Optimizer (Rule + Cost)
          │
          ▼
   Physical Plan
          │
          ▼
   Executor Pipeline
          │
          ▼
   Storage Engine
```

---

## 3. 1.2 版本核心目标

SQLRustGo 1.2 是**架构重构和接口抽象版本**，核心目标是为 v2.0 分布式架构打基础。

| 模块 | 1.2 目标 |
|------|----------|
|解析器|SQL → 逻辑计划|
|优化器| 基础规则优化 + Memo 结构 |
|统计数据|表统计 + 列统计|
| CBO | 简化成本优化器 |
|执行| 向量化执行框架 (RecordBatch) |

---

## 4. 1.2 技术能力

### 4.1 SQL 支持

- 选择、来自、何处、连接、限制、排序、分组

### 4.2 执行模型

采用 **Iterator + Batch** 模型：

```rust
pub trait Operator: Send {
    fn open(&mut self);
    fn next_batch(&mut self) -> Option<RecordBatch>;
    fn close(&mut self);
}
```

### 4.3 统计

```rust
pub struct TableStats {
    pub row_count: usize,
    pub total_bytes: usize,
    pub column_stats: HashMap<String, ColumnStats>,
}
```

### 4.4 CBO

- Join 顺序选择
- 基于统计的成本计算
- 扫描方式选择

---

## 5. 架构演进

```
v1.1.x                    v1.2.0
┌─────────────┐           ┌─────────────────────┐
│ FileStorage │           │ StorageEngine trait │
└─────────────┘           │   ├── FileStorage   │
                           │   └── MemoryStorage │
┌─────────────┐           └─────────────────────┘
│ RowExecutor │           ┌─────────────────────┐
└─────────────┘           │  VectorizedExecutor │
                           │  (基于 RecordBatch) │
┌─────────────┐           └─────────────────────┐
│ SimpleStats │           ┌─────────────────────┐
└─────────────┘           │  CostModel trait    │
                           │  + StatsCollector   │
┌─────────────┐           └─────────────────────┐
│ Hardcoded   │           ┌─────────────────────┐
│ Optimizer   │           │ Optimizer (Memo)    │
└─────────────┘           └─────────────────────┘
```

---

## 6. 版本路线

| 版本 | 能力 | 状态 |
|------|------|------|
| 1.0 | SQL 执行原型 | ✅ 已发布 |
| 1.1 | 基础执行引擎 | ✅ 已发布 |
| **1.2** |**CBO + Statistics + 向量化**| 🔄 开发中 |
| 2.0 | 分布式执行 | 📋 计划中 |

---

## 7. 贡献者

- @yinglichina8848
- @openheart-openheart
- @sonaheartopen
- @TRAE人工智能助手
