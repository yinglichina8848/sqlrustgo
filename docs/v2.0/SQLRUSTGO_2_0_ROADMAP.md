# sqlrustgo 2.0 路线图

> 版本：v2.0
> 日期：2026-03-05
> 目标：从"项目代码"升级为"数据库内核架构"，支持 Client-Server 模式
> 状态：架构稳定，为 3.0 分布式演进做准备

---

## 〇、架构设计概览

详见 [ARCHITECTURE_V2.md](./ARCHITECTURE_V2.md)

### 核心设计原则

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          核心设计原则                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   1. Trait 设计稳定                                                          │
│      └── Executor trait 接口冻结                                            │
│      └── StorageEngine trait 接口冻结                                       │
│      └── ExecutionContext trait 接口冻结                                    │
│                                                                              │
│   2. 内存管理模型稳定                                                        │
│      └── RecordBatch 内存布局稳定                                           │
│      └── BufferPool 接口稳定                                                │
│                                                                              │
│   3. 网络协议可扩展                                                          │
│      └── MySQL 协议版本字段                                                 │
│      └── 扩展字段预留                                                       │
│                                                                              │
│   核心原则: 2.0 不会在 3.0 被推翻，只会扩展                                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.0 分布式接口预留

| 接口 | 2.0 状态 | 3.0 扩展 |
|------|----------|----------|
|执行上下文| Local 实现 |Distributed 实现|
|数据交换| Noop 实现 | 真实网络传输 |
|计划片段| 不使用 | 分布式调度 |
|存储引擎| 单机存储 | 分布式存储 |

---

## 一、开发轨道概览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          2.0 双轨道开发                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   轨道 A: 内核架构重构                    轨道 B: 网络层增强                │
│   ─────────────────────                  ─────────────────────              │
│                                                                              │
│   Phase 1: 核心架构 (1-2月)              Phase 1: 基础C/S (1周)             │
│   ├── LogicalPlan 重构                   ├── server.rs 可执行程序           │
│   ├── PhysicalPlan trait                 ├── client.rs 可执行程序           │
│   ├── Executor 插件化                    └── 本地通信测试                   │
│   └── HashJoin 实现                                                         │
│                                          Phase 2: 功能完善 (1-2周)          │
│   Phase 2: 性能升级 (2-3月)              ├── 异步服务器                     │
│   ├── 向量化执行                         ├── 连接池                         │
│   ├── 统计信息                           ├── 会话管理                       │
│   └── 简化 CBO                           └── 交互模式                       │
│                                                                              │
│   Phase 3: 内核能力 (3-6月)              Phase 3: 生产就绪 (2-3周)          │
│   ├── 完整 CBO                           ├── 认证机制                       │
│   ├── Memory Pool                        ├── SSL/TLS                        │
│   └── Spill to Disk                      └── 性能优化                       │
│                                                                              │
│   依赖关系: Phase 1 (轨道A) → Phase 2-3 (轨道B)                              │
│   说明: 网络层需要稳定的执行引擎支持                                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 二、对标 Apache DataFusion

### 2.1 DataFusion 核心设计

| 模块 | 说明 |
|:-----|:-----|
| Arrow 列式内存 | 高效内存布局 |
|LogicalPlan / PhysicalPlan 分离| 清晰的执行层次 |
| 向量化执行 | 批量处理 |
| CBO（部分实现） | 成本优化 |
| 插件式数据源 | 可扩展存储 |

### 2.2 对比分析

| 模块 |sqlrustgo|数据融合|
|:-----|:----------|:-----------|
|解析器| ✅ | ✅ |
|逻辑计划| 部分 | ✅ |
| CBO | ❌ | 部分 |
| 向量化 | ❌ | ✅ |
| 插件 | 基础 | ✅ |
| 分布式 | ❌ |通过 Ballista|
|**客户端-服务器**| ⚠️ 基础 | ✅ |

---

## 三、升级路径

### 第一阶段：核心架构重构

**时间**：1-2 个月

| 任务 | 说明 |
|:-----|:-----|
|LogicalPlan 独立模块| 清晰的逻辑计划 |
|PhysicalPlan trait 化| 可扩展的物理计划 |
|Executor 插件化| 可替换执行器 |
|HashJoin 实现| 解决 Join 性能 |

**结果**：L3 → L4

### 第二阶段：性能升级

**时间**：2-3 个月

| 任务 | 说明 |
|:-----|:-----|
| 向量化执行 | 批量处理 |
| 批处理表达式 | 列式计算 |
| 基础统计信息 | 表/列统计 |
| 简化 CBO | 成本优化 |

**结果**：可处理 100万行级数据

### 第三阶段：内核级能力

**时间**：3-6 个月

| 任务 | 说明 |
|:-----|:-----|
| 完整 CBO | 成本优化器 |
|加入再订购| Join 重排序 |
| 插件动态加载 | 运行时扩展 |
|内存池| 内存管理 |
|溢出到磁盘| 磁盘溢出 |

**结果**：可商用级内核

### 第四阶段：企业级能力

**时间**：6-12 个月

| 任务 | 说明 |
|:-----|:-----|
| 分布式执行 | 多节点 |
| 任务调度 | 任务分发 |
| 事务支持 | ACID |
| 高可用 | 故障恢复 |

---

## 三、Phase 1 详细计划

### 3.1 LogicalPlan 重构

```rust
pub enum LogicalPlan {
    Projection { input: Box<LogicalPlan>, expr: Vec<Expr> },
    Filter { input: Box<LogicalPlan>, predicate: Expr },
    Join { left: Box<LogicalPlan>, right: Box<LogicalPlan>, on: Vec<(Expr, Expr)> },
    Aggregate { input: Box<LogicalPlan>, group_expr: Vec<Expr>, aggr_expr: Vec<Expr> },
    TableScan { table_name: String, projection: Option<Vec<usize>> },
}
```

### 3.2 PhysicalPlan trait 化

```rust
pub trait PhysicalPlan: Send + Sync {
    fn schema(&self) -> &Schema;
    fn execute(&self, partition: usize) -> Result<Box<dyn ExecutionPlan>>;
    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>>;
}
```

### 3.3 HashJoin 实现

```rust
pub struct HashJoinExec {
    left: Arc<dyn PhysicalPlan>,
    right: Arc<dyn PhysicalPlan>,
    on: Vec<(Column, Column)>,
    join_type: JoinType,
}
```

---

## 四、Phase 2 详细计划

### 4.1 向量化执行

```rust
pub struct RecordBatch {
    columns: Vec<ArrayRef>,
    row_count: usize,
}

pub trait PhysicalExpr {
    fn evaluate(&self, batch: &RecordBatch) -> Result<ArrayRef>;
}
```

### 4.2 统计信息

```rust
pub struct TableStats {
    pub row_count: usize,
    pub total_bytes: usize,
}

pub struct ColumnStats {
    pub distinct_count: usize,
    pub null_count: usize,
    pub min_value: Option<ScalarValue>,
    pub max_value: Option<ScalarValue>,
}
```

---

## 五、Phase 3 详细计划

### 5.1 完整 CBO

```rust
pub struct CostBasedOptimizer {
    statistics: Arc<dyn StatisticsCollector>,
    cost_estimator: Arc<dyn CostEstimator>,
    plan_enumerator: Arc<dyn PlanEnumerator>,
}
```

### 5.2 内存池

```rust
pub trait MemoryPool: Send + Sync {
    fn allocate(&self, size: usize) -> Result<MemoryAllocation>;
    fn free(&self, allocation: MemoryAllocation);
    fn available(&self) -> usize;
}
```

### 5.3 溢出到磁盘

```rust
pub trait SpillManager: Send + Sync {
    fn spill(&self, batch: &RecordBatch) -> Result<SpillHandle>;
    fn read(&self, handle: &SpillHandle) -> Result<RecordBatch>;
}
```

---

## 六、Phase 4 详细计划

### 6.1 分布式执行

```rust
pub struct DistributedExecutor {
    scheduler: Arc<dyn Scheduler>,
    workers: Vec<WorkerInfo>,
}

pub trait Scheduler: Send + Sync {
    fn schedule(&self, task: Task) -> Result<TaskHandle>;
    fn status(&self, handle: &TaskHandle) -> Result<TaskStatus>;
}
```

### 6.2 事务支持

```rust
pub trait TransactionManager: Send + Sync {
    fn begin(&self) -> Result<Transaction>;
    fn commit(&self, txn: Transaction) -> Result<()>;
    fn rollback(&self, txn: Transaction) -> Result<()>;
}
```

---

## 七、里程碑

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          里程碑规划                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   v1.1.0 (Phase 1 - 内核架构)                                               │
│   ├── LogicalPlan 重构                                                      │
│   ├── PhysicalPlan trait                                                    │
│   ├── Executor 插件化                                                       │
│   ├── HashJoin                                                              │
│   └── 目标：L4 架构                                                         │
│                                                                              │
│   v1.1.1 (Phase 1 - 网络层)                                                 │
│   ├── server.rs 可执行程序                                                  │
│   ├── client.rs 可执行程序                                                  │
│   ├── 本地通信测试                                                          │
│   └── 目标：基础 Client-Server                                              │
│                                                                              │
│   v1.2.0 (Phase 2 - 内核性能)                                               │
│   ├── 向量化执行                                                            │
│   ├── 统计信息                                                              │
│   ├── 简化 CBO                                                              │
│   └── 目标：100万行级                                                       │
│                                                                              │
│   v1.2.1 (Phase 2 - 网络增强)                                               │
│   ├── 异步服务器                                                            │
│   ├── 连接池                                                                │
│   ├── 会话管理                                                              │
│   └── 目标：多连接支持                                                      │
│                                                                              │
│   v2.0.0 (Phase 3 - 内核能力)                                               │
│   ├── 完整 CBO                                                              │
│   ├── Memory Pool                                                           │
│   ├── Spill to Disk                                                         │
│   └── 目标：商用级内核                                                      │
│                                                                              │
│   v2.0.1 (Phase 3 - 网络生产)                                               │
│   ├── 认证机制                                                              │
│   ├── SSL/TLS                                                               │
│   ├── 性能优化                                                              │
│   └── 目标：生产就绪                                                        │
│                                                                              │
│   v3.0.0 (Phase 4 - 企业级)                                                 │
│   ├── 分布式执行                                                            │
│   ├── 事务支持                                                              │
│   ├── 高可用                                                                │
│   └── 目标：企业级                                                          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 八、网络层详细计划

> 详细文档见: [网络增强开发计划](./网络设计/NETWORK_ENHANCEMENT_PLAN.md)

### 8.1 Phase 1: 基础 Client-Server (1周)

| ID | 任务 | 预估时间 |
|----|------|----------|
| N-001 | 创建 server.rs 基础框架 | 2h |
| N-002 | 实现命令行参数解析 | 2h |
| N-003 | 集成存储引擎初始化 | 2h |
| N-004 | 创建 client.rs 基础框架 | 2h |
| N-005 | 实现单次查询执行 | 3h |
| N-006 | 服务器-执行器集成 | 4h |
| N-007 | 本地通信测试 | 2h |

### 8.2 Phase 2: 功能完善 (1-2周)

| ID | 任务 | 预估时间 |
|----|------|----------|
| N-011 | 异步服务器实现 | 4h |
| N-012 | 连接池实现 | 4h |
| N-013 | 会话管理实现 | 3h |
| N-014 | 交互模式 (REPL) | 3h |
| N-015 | 配置文件支持 | 2h |

### 8.3 Phase 3: 生产就绪 (2-3周)

| ID | 任务 | 预估时间 |
|----|------|----------|
| N-019 | 认证机制实现 | 4h |
| N-020 |SSL/TLS 支持| 4h |
| N-021 | 性能测试和优化 | 4h |
| N-022 | 文档编写 | 3h |

---

## 九、关键决策

### 9.1 如果不做这些会怎样？

**3 年后**：
- 执行者2000行
- Join 逻辑混乱
- 性能不可控
- 插件难以插入
- 无独立部署能力

### 9.2 如果做完这些

**你将拥有**：
- 可演进内核
- 可融资故事
- 可对标 DataFusion
- 可嵌入产品
- 可独立部署的 Client-Server

---

## 十、总结

```
现在 sqlrustgo 的关键不是"多写功能"。

而是：

把它从"项目代码"升级为"数据库内核架构"。

同时支持 Client-Server 模式，实现跨系统通信。
```

---

*本文档由 TRAE (GLM-5.0) 创建*
