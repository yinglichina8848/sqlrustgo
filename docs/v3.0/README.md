# v3.0 分布式执行平台

> Distributed Execution Platform

---

## 版本定位

v3.0 不是"加功能"，而是：

**执行平台重构**

核心目标：

- 向量化默认
- 并行默认
- 分布式可选
- 自适应优化基础打好
- 性能可持续进化

---

## 执行模型演进

### Volcano Model（v1.x）

```rust
parent.next()
    → child.next()
        → child.next()
```

特点：
- Tuple-at-a-time
- 虚函数频繁调用
- 分支预测差
- Cache miss 多

### Pipeline Model（v2.x）

```
Scan → Filter → Project → HashJoin → Output
```

转化为：

```rust
while batch:
    fused_function(batch)
```

特点：
- Operator Fusion
- 消除虚调用
- 编译期 inline

### Vectorized Model（v3.0 默认）

```rust
next_batch() → ColumnarBatch(1024 rows)
```

特点：
- 1024–4096 rows
- 列式存储
- SIMD friendly

---

## 3.0 执行模型矩阵

| 模型 | 3.0 状态 |
|------|----------|
| Volcano | 保留兼容 |
| Pipeline | 中间层 |
| Vector | 默认执行引擎 |

---

## 可扩展算子注册系统

### Operator Registry

```rust
trait PhysicalOperator {
    fn create(&self) -> Box<dyn Executor>;
}

struct OperatorRegistry {
    map: HashMap<String, Box<dyn PhysicalOperator>>,
}
```

### 注册机制

```rust
registry.register("hash_join", Box::new(HashJoinFactory));
registry.register("filter", Box::new(FilterFactory));
```

### 动态选择

优化器只输出：

```rust
PhysicalPlan {
    operator: "hash_join"
}
```

执行时查 registry。

---

## 并行执行线程调度

### 线程模型

```
Coordinator
   ↓
Task Scheduler
   ↓
Worker Threads
```

### 任务抽象

```rust
trait Task {
    fn execute(&self);
}
```

### Exchange Operator

并行核心：

```
Producer → Exchange → Consumer
```

Exchange 负责：

- 分区
- shuffle
- backpressure

---

## 分布式执行 DAG

### 从树到 DAG

单机：

```
Scan → Join → Agg
```

分布式：

```
Scan_A  Scan_B
    ↓      ↓
  Shuffle  Shuffle
       ↓
     HashJoin
       ↓
     Aggregate
```

### 组件划分

- Coordinator
- Worker Nodes
- Shuffle Service
- Metadata Service

### 执行阶段

1. Logical Plan
2. Fragmentation
3. DAG Build
4. Task Scheduling
5. Shuffle
6. Result Merge

---

## 自适应优化

### 问题背景

CBO 依赖：
- row_count
- selectivity
- histogram

但真实执行时：

```
estimated_rows ≠ actual_rows
```

### 运行时反馈

```rust
struct ExecStats {
    input_rows: u64,
    output_rows: u64,
    elapsed_ms: u128,
}

trait Instrumented {
    fn stats(&self) -> ExecStats;
}
```

### 动态重规划

```rust
if actual_rows > estimated_rows * 5 {
    trigger_replan();
}
```

### Re-Optimization 模式

- 模式 A：Join 侧交换
- 模式 B：Hash → Nested Loop
- 模式 C：Pipeline 分裂

---

## 列式存储引擎

### 列块结构

```rust
struct ColumnBlock<T> {
    values: Vec<T>,
    null_bitmap: Vec<u8>,
}
```

### 数据组织

```
Segment
   ↓
Column Chunk (1MB)
   ↓
Vector (1024 rows)
```

### 编码策略

- RLE
- Dictionary
- Delta Encoding
- Bit-Packing

---

## 性能基准体系

### Micro Benchmark

- 单算子性能
- Filter 吞吐
- Hash 构建速度

### Operator Benchmark

- Hash Join TPS
- Sort Throughput

### Query Benchmark

- TPC-H
- TPC-DS

### Regression Benchmark

CI 自动跑，允许 ±3% 波动

---

## 测试分层体系

| 层级 | 内容 |
|------|------|
| 单元测试 | Parser, Cost Model, Hash Table |
| Operator 测试 | HashJoin correctness, Spill correctness |
| Plan 测试 | SQL → 计划结构 |
| Fuzz 测试 | 随机 SQL |
| Crash 测试 | WAL 重放, 崩溃一致性 |

---

## 技术战略图

```
v1.0 → 稳定
v2.0 → 执行模型升级
v3.0 → 分布式执行平台
v4.0 → AI 驱动优化
```
