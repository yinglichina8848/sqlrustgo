# SQLRustGo 版本规划路线图

> 最后更新：2026-02-20

---

## 版本演进总览

```
v1.0 ──→ v1.1 ──→ v2.0 ──→ v3.0 ──→ v4.0 ──→ v5.0 ──→ v6.0 ──→ v7.0 ──→ v8.0
 │        │        │        │        │        │        │        │        │
稳定     小修复   架构升级  分布式   AI优化   自适应   智能系统  控制论   AI-Ready OS
```

---

## v1.0.0 - 工程合规版本

### 核心定位
**教科书级工程合规数据库雏形**

v1.0 不是功能强，而是：
- 可运行
- 不崩溃
- 有测试
- 有文档
- 有版本
- 有 CI
- 有 Release Note
- 符合工程规范

### 冻结原则
从 Beta 阶段开始：

| 禁止 | 允许 |
|------|------|
| 新功能 | panic 修复 |
| 性能优化 | unwrap 移除 |
| 架构重构 | 测试补充 |
| API 变更 | 文档完善 |
| 新模块 | CI 修复 |

### 发布标准
| 项目 | 状态 |
|------|------|
| 无 unwrap | 必须 |
| 无 panic | 必须 |
| CI 全绿 | 必须 |
| 测试通过 | 必须 |
| 有版本 tag | 必须 |
| 有 Release Note | 必须 |
| README 清晰 | 必须 |
| 依赖锁定 | 必须 |

### 7天封板计划
| 阶段 | 任务 |
|------|------|
| Day 1-2 | 清零 panic，全部 unwrap 清理 |
| Day 3 | 错误模型统一，引入 DbError |
| Day 4 | 覆盖率冲刺，核心模块 ≥ 85% |
| Day 5 | 文档完善，README、架构图、运行说明 |
| Day 6 | 打 RC (v1.0.0-rc.1)，运行 48 小时 |
| Day 7 | 正式发布 v1.0.0 |

---

## v1.1.0 - 小修复版本

### 核心定位
**v1.0 的稳定维护版本**

### 允许内容
- bug fix
- 文档修复
- 小幅性能优化（非破坏）
- 新增非破坏 API

### 禁止内容
- 架构升级
- 执行模型改变
- 优化器引入
- 大规模重构

---

## v2.0.0 - 架构升级版本

### 核心定位
**真正性能与架构跃迁版本**

### 执行模型升级
从 Row-based iterator 升级到 Vectorized batch execution

```
Volcano Model          Vectorized Model
next() -> one tuple    next_batch() -> ColumnarBatch(1024 rows)
```

### CBO 优化器
实现完整流程：
```
SQL → Logical Plan → Rule Rewriter → Cost Estimator → Physical Plan
```

核心数据结构：
```rust
pub enum LogicalPlan {
    Scan { table: String },
    Filter { predicate: Expr, input: Box<LogicalPlan> },
    Projection { columns: Vec<String>, input: Box<LogicalPlan> },
    Join { left: Box<LogicalPlan>, right: Box<LogicalPlan>, on: Expr },
}
```

### 存储抽象层
```rust
trait StorageEngine {
    fn scan(&self, table: &str) -> DbResult<RowSet>;
}
```

支持：InMemory / Disk-based / LSM

### 并行执行
- 阶段 1：Operator-level parallelism
- 阶段 2：Pipeline parallelism

### Join 算法演进
| 阶段 | 算法 | 复杂度 |
|------|------|--------|
| v1.x | Nested Loop Join | O(N²) |
| v2.0 | Hash Join | O(N) |
| v2.1 | Sort-Merge Join | O(N log N) |

---

## v3.0.0 - 分布式执行平台

### 核心定位
**从数据库 → 数据执行平台**

### 执行模型演进
| 模型 | 特点 | 适用场景 |
|------|------|----------|
| Volcano | Tuple-at-a-time | v1.x 兼容 |
| Pipeline | Operator Fusion | 中间层 |
| Vectorized | Batch 1024-4096 rows | 默认引擎 |

### 可扩展算子注册系统
```rust
trait PhysicalOperator {
    fn create(&self) -> Box<dyn Executor>;
}

struct OperatorRegistry {
    map: HashMap<String, Box<dyn PhysicalOperator>>,
}
```

### 并行执行线程调度
- Work-Stealing
- NUMA 感知
- Exchange Operator

### 分布式执行 DAG
```
Scan_A    Scan_B
    ↓        ↓
  Shuffle  Shuffle
       ↓
     HashJoin
       ↓
     Aggregate
```

### 性能基准体系
| 类型 | 内容 |
|------|------|
| Micro Benchmark | 单算子性能 |
| Operator Benchmark | Hash Join TPS |
| Query Benchmark | TPC-H, TPC-DS |
| Regression Benchmark | CI 自动对比 |

---

## v4.0.0 - AI 驱动优化

### 核心定位
**数据系统智能化 + 异构计算化**

### AI 优化器架构
```
Query → Logical Plan → Classical CBO → AI Optimizer Layer → Plan Selection
```

### 三层智能结构
| 层级 | 功能 |
|------|------|
| Layer 1 | Learned Cost Model: cost = ML(plan_features) |
| Layer 2 | Plan Ranking Model: 多候选计划排序 |
| Layer 3 | Runtime Reinforcement: 在线训练 |

### GPU 执行引擎
```
Planner → Device Selector → CPU Operator / GPU Operator
```

适合 GPU 的算子：Filter / Projection / Aggregation / Hash Join

### 列式存储压缩
| 编码 | 适用场景 |
|------|----------|
| RLE | 低基数列 |
| Dictionary | 字符串 |
| Delta | 时间戳、单调递增 |
| Bit-Packing | 小整数范围 |

---

## v5.0.0 - 自适应优化

### 核心定位
**运行时自优化系统**

### 自适应优化框架
```
Plan → Execute → Collect Metrics → Retrain → Update Model
```

### 运行时反馈
```rust
struct ExecStats {
    input_rows: u64,
    output_rows: u64,
    elapsed_ms: u128,
}
```

### 动态重规划
```rust
if actual_rows > estimated_rows * 5 {
    trigger_replan();
}
```

### 列式存储引擎
```rust
struct ColumnBlock<T> {
    values: Vec<T>,
    null_bitmap: Vec<u8>,
}
```

---

## v6.0.0 - 自进化智能系统

### 核心定位
**控制论驱动的自进化数据库**

### 状态空间模型
```
x(t+1) = f(x(t), u(t), w(t))
y(t) = g(x(t))
```

状态向量：[数据分布, 统计信息, Cache状态, 执行计划, 资源占用, 负载]

### RL 训练框架
```rust
// MDP 定义
状态 S: 统计特征向量 + Query Graph embedding
动作 A: 选择 join 顺序、算子类型、资源分配
奖励 R: -Latency - λ*MemoryPenalty
```

### Agent 协作框架
```
Agent Layer
    ↓
Semantic Planner
    ↓
Query Generator
    ↓
Adaptive DB Kernel
    ↓
Feedback to Agent
```

---

## v7.0.0 - 自主进化数据系统

### 核心定位
**自主进化智能数据操作系统**

### 控制论模型
三层控制环：
| 环路 | 周期 | 功能 |
|------|------|------|
| 快环 | 毫秒级 | Runtime Re-Optimization |
| 中环 | 分钟级 | 统计信息 + Plan Rebuild |
| 慢环 | 天级 | 索引生成 + 数据重分布 |

### AI + CBO 混合决策
```
Score(plan) = α * Cost_CBO + β * T_AI
```

### 安全策略
```
π_final = α * π_CBO + (1-α) * π_RL
```

渐进式接管，确保稳定性。

---

## v8.0.0 - AI-Ready Operating Environment

### 核心定位
**内置 AI 和数据库能力的操作系统环境**

### 设计原则
- 不是 Database = OS
- 不是 AI = OS
- 而是 OS 内建 AI 能力 + OS 内建数据能力
- 三者解耦但深度协同

### 分层架构
```
┌──────────────────────┐
│ Applications / Agents│
├──────────────────────┤
│ AI Runtime Layer     │
├──────────────────────┤
│ Embedded DB Engine   │
├──────────────────────┤
│ OS Kernel            │
├──────────────────────┤
│ Hardware             │
└──────────────────────┘
```

### 硬件分层适配
| 设备类型 | AI 形态 | 数据能力 |
|----------|---------|----------|
| 嵌入式 | Tiny model | KV 引擎 |
| 手机 | 本地推理 | 列式数据库 |
| PC | 混合推理 | 向量索引 |
| 服务器 | 训练+推理 | 分布式数据库 |
| 超算 | 分布式训练 | GPU 执行引擎 |

---

## 技术战略图

```
Year 1: v1.0-v2.0
├── 工程稳定
├── 向量化执行
├── CBO 初版
└── 插件式存储

Year 2: v3.0-v4.0
├── 并行执行
├── 分布式执行
├── AI 优化上线
└── GPU 实验性支持

Year 3: v5.0-v6.0
├── 自适应优化
├── 控制论模型
├── Agent 协作
└── 列式存储成熟

Year 4+: v7.0-v8.0
├── 自主进化
├── AI-Ready OS
├── 全球算力调度
└── 智能基础设施
```

---

## 版本推进流程

```
v1.0.0-alpha ──→ v1.0.0-beta ──→ v1.0.0-rc ──→ v1.0.0 ──→ v1.1.0 ──→ v2.0.0
     ✅               ✅              ⏳           ⏳          ⏳          ⏳
```

---

## 项目成熟度评分模型

| 维度 | 指标 | 满分 |
|------|------|------|
| 稳定性 | 无 panic、无 unwrap、CI 强制、覆盖率≥80% | 20 |
| 工程规范 | PR 规范、Commit 语义化、分支清晰、Release 节奏 | 20 |
| 架构清晰度 | 模块边界、错误模型、执行流程、无循环依赖 | 20 |
| 可维护性 | 技术债追踪、文档齐全、CHANGELOG、版本策略 | 20 |
| 技术深度 | 优化器设计、执行模型、性能 benchmark、扩展性 | 20 |

**评分区间**：
- 0-40：实验项目
- 40-60：工程雏形
- 60-80：稳定系统
- 80-90：成熟系统
- 90+：长期资产级项目

---

*本文档基于 AI 协作对话记录整理，由 TRAE (GLM-5.0) 更新*
