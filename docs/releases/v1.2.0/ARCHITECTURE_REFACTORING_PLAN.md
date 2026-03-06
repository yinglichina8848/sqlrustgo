# v1.2.0 架构重构计划

> **版本**: v1.2.0
> **代号**: Architecture Stabilization
> **制定日期**: 2026-03-05
> **依据**: ChatGPT 架构评估报告

---

## 一、v1.1.0 现状评估

### 1.1 当前架构能力

| 维度 | 评分 | 说明 |
|------|------|------|
| 架构设计 | ⭐⭐⭐⭐☆ | 分层清晰 |
| 工程规范 | ⭐⭐⭐☆☆ | 版本治理需修复 |
| 扩展能力 | ⭐⭐⭐☆☆ | 未完全接口化 |
| 分布式可演进性 | ⭐⭐☆☆☆ | 需要结构重构 |

### 1.2 关键问题清单

#### 🚨 必须立即修复

| 问题 | 严重程度 | 说明 |
|------|----------|------|
| 版本信息不一致 | 严重 | README 显示 v1.1.0，代码输出 v1.0.0 |
| REPL 与内核耦合过高 | 高 | 直接实例化 ExecutionEngine |
| ExecutionEngine 单实例内存模型 | 高 | 无可替换执行层 |

#### ⚠️ 必须改进

| 问题 | 影响 | 说明 |
|------|------|------|
| 事务与存储边界未隔离 | 高 | TransactionManager 与 WAL 耦合 |
| Catalog 尚未实现 | 高 | 无系统表、无 schema registry |
| 执行层未接口化 | 高 | 无法注入远程执行代理 |

---

## 二、v1.2.0 战略定位

### ❌ 错误定位（避免）

- 只做性能优化
- 只做 CBO
- 只做向量化执行

这会导致结构崩坏。

### ✅ 正确定位

**v1.2.0 = 架构接口化 + 优化器体系成型版本**

核心目标：

1. 把"单机数据库内核"改造成"可扩展数据库平台内核"
2. 把 CBO 设计成可插拔优化框架
3. 把执行引擎与优化器彻底解耦

---

## 三、v1.2.0 必须完成的 6 大重构

### 3.1 重构①：引入 Optimizer 框架层

**当前结构**:
```
Parser → LogicalPlan → 直接 PhysicalPlan
```

**必须改成**:
```
Parser
   ↓
LogicalPlan
   ↓
Optimizer (Rule + Cost)
   ↓
PhysicalPlan
   ↓
Executor
```

**新增核心 Trait**:

```rust
trait Optimizer {
    fn optimize(&self, plan: LogicalPlan) -> Result<LogicalPlan>;
}

trait Rule {
    fn apply(&self, plan: &mut LogicalPlan) -> bool;
}

trait CostModel {
    fn estimate(&self, plan: &LogicalPlan) -> f64;
}
```

**设计思想**:
- Rule 负责重写
- CostModel 负责评分
- Optimizer 负责搜索

### 3.2 重构②：引入 Statistics 子系统

**没有统计信息，CBO 是假的。**

**新增结构**:

```rust
struct TableStats {
    row_count: usize,
    column_stats: HashMap<String, ColumnStats>,
}

struct ColumnStats {
    distinct_count: usize,
    null_fraction: f64,
    min_value: Option<ScalarValue>,
    max_value: Option<ScalarValue>,
}
```

**新增 Trait**:

```rust
trait StatisticsProvider {
    fn table_stats(&self, table: &str) -> Option<TableStats>;
}
```

**⚠️ 未来演进**:
- 分布式统计同步接口
- 分片统计汇总接口

### 3.3 重构③：Catalog 正式落地

**当前问题**: REPL 显示 "Tables: (catalog not yet implemented)"

**必须拆成**:

```
catalog/
├── table.rs      # 表元数据
├── index.rs      # 索引元数据
├── schema.rs     # Schema 管理
└── stats.rs      # 统计信息
```

**新增 Trait**:

```rust
trait Catalog: Send + Sync {
    fn get_table(&self, name: &str) -> Option<TableMeta>;
    fn list_tables(&self) -> Vec<String>;
    fn add_table(&self, meta: TableMeta) -> Result<()>;
    fn drop_table(&self, name: &str) -> Result<()>;
}
```

**必须满足**:
- ✅ 可序列化
- ✅ 可持久化
- ✅ 可替换存储

### 3.4 重构④：执行层接口化

**当前问题**: `ExecutionEngine::execute(plan)` 无法替换

**必须改成**:

```rust
trait Executor: Send {
    fn execute(&self, plan: PhysicalPlan) -> Result<ExecutionResult>;
}
```

**未来可实现**:
- `LocalExecutor` - 本地执行
- `RemoteExecutor` - 远程执行代理
- `DistributedExecutor` - 分布式执行

### 3.5 重构⑤：错误域重构

**当前问题**: 错误混合在一起

**必须分域**:

```
error/
├── parser_error.rs     # 解析错误
├── optimizer_error.rs  # 优化错误
├── execution_error.rs  # 执行错误
└── storage_error.rs   # 存储错误
```

**定义错误枚举**:

```rust
enum SQLError {
    Parser(ParserError),
    Optimizer(OptimizerError),
    Execution(ExecutionError),
    Storage(StorageError),
    Catalog(CatalogError),
}
```

### 3.6 重构⑥：Pipeline 执行模型

**当前问题**: Iterator 模型 `next()`

**引入**:

```rust
trait Operator: Send {
    fn open(&mut self, ctx: &dyn ExecutionContext);
    fn next_batch(&mut self) -> Option<RecordBatch>;
    fn close(&mut self) -> Result<()>;
}
```

**优势**:
- 可做向量化
- 可做并行执行
- 可做远程批量传输

---

## 四、CBO 实现规范

### ❌ 错误方式

- 把 cost 逻辑写死在 join 里面
- 把统计信息直接从 storage 读
- 在 LogicalPlan 中塞 cost 字段

### ✅ 正确方式

#### 1. 使用 Memo 结构

```rust
struct Memo {
    groups: Vec<Group>,
}

struct Group {
    expressions: Vec<LogicalPlan>,
    cost: Option<f64>,
}
```

#### 2. 采用"两阶段优化"

| 阶段 | 类型 | 说明 |
|------|------|------|
| 第一阶段 | Rule-based rewrite | 谓词下推、投影裁剪 |
| 第二阶段 | Cost-based selection | Join 顺序、索引选择 |

**不要混在一起。**

---

## 五、目标目录结构

### 5.1 当前实际目录结构 (crates/ workspace)

> ⚠️ **重要**: v1.2.0 已通过 PR #305 实施 workspace 结构，以下是实际结构:

```
crates/
├── common/              # 通用错误类型 (SqlError)
├── types/              # 类型系统 (Value, DataType, error types)
├── parser/             # SQL 解析 (Lexer, Parser, Token)
├── planner/            # 逻辑计划 (占位)
├── optimizer/          # 优化器 (Rule, Cost, Memo)
├── executor/           # 执行器 (Operator, RecordBatch)
├── storage/            # 存储引擎 (StorageEngine trait, FileStorage)
├── catalog/            # 元数据管理
├── transaction/        # 事务 (WAL, TransactionManager)
└── server/             # 网络服务 (REPL, QueryService)
```

### 5.2 目录说明

| 目录 | 目的 | 优先级 | 状态 |
|------|------|--------|------|
| `optimizer/` | CBO 优化框架 | P0 | ✅ 已实现 |
| `catalog/` | 元数据管理 | P0 | ✅ 已实现 |
| `executor/` | 执行器抽象 | P0 | ✅ 已实现 |
| `storage/` | 存储引擎抽象 | P0 | ✅ 已实现 |
| `transaction/` | 事务管理 | P0 | ✅ 已实现 |
| `types/` | 类型系统 | P0 | ✅ 已实现 |
| `common/` | 通用错误 | P1 | ✅ 已实现 |
| `planner/` | 逻辑计划 | P1 | 🔄 开发中 |

---

## 六、v1.2.0 当前能力评估

> ⚠️ **更新**: 截至 v1.2.0-draft 阶段，以下接口已完成:

| 能力 | 是否具备 | 实现位置 | 说明 |
|------|----------|----------|------|
| CBO 可扩展 | ✅ | crates/optimizer/ | Optimizer trait + Rule + CostModel |
| 多优化器支持 | ✅ | crates/optimizer/ | 可插拔优化框架 |
| 执行层替换 | ✅ | crates/executor/ | Executor trait |
| 统计独立 | ✅ | crates/types/ | StatisticsProvider trait |
| Catalog 可共享 | ✅ | crates/catalog/ | Catalog trait + 可序列化 |
| 为 2.0 预留接口 | ✅ | 各 crate | 所有核心 trait 已定义 |
| 存储抽象 | ✅ | crates/storage/ | StorageEngine trait |
| 事务管理 | ✅ | crates/transaction/ | WAL + TransactionManager |

---

## 七、v1.3.0 演进路径

> ⚠️ **重要**: 根据 TASK_MATRIX.md，向量化执行已延后到 v1.3.0

### 如果 1.2.0 做好，1.3.0 只需：

1. **节点模型**: `struct Node { id: NodeId, role: Leader | Follower }`
2. **远程执行代理**: `trait ExecutionTransport`
3. **日志复制接口**: `trait LogReplicator`
4. **网络层抽象**: 支持分布式

**不会推翻核心。**

---

## 八、重构与功能开发并行

### 轨道 A: 架构重构 (P0)

| 任务 | 依赖 | 负责人 |
|------|------|--------|
| R-001: 错误域重构 | - | openheart |
| R-002: Optimizer 框架 | - | openheart |
| R-003: Statistics 子系统 | R-002 | openheart |
| R-004: Catalog 系统 | - | heartopen |
| R-005: Executor trait | - | heartopen |
| R-006: StorageEngine trait | - | heartopen |

### 轨道 B: 功能开发 (P1)

| 任务 | 依赖 | 负责人 |
|------|------|--------|
| V-001: RecordBatch | R-005 | openheart |
| V-002: ColumnarArray | V-001 | openheart |
| V-003: 向量化表达式 | V-002 | openheart |
| V-004: 执行器重构 | R-005, V-003 | heartopen |
| C-001: CBO 成本模型 | R-002, R-003 | openheart |
| C-002: 谓词下推 | R-002 | heartopen |

---

## 九、与现有 v1.2.0 任务的整合

### 现有任务 → 新架构 (实际完成状态)

| 现有任务 | 对应重构 | 状态 |
|----------|----------|------|
| V-001~V-007 (向量化) | R-005 + R-006 + Pipeline | ⚠️ 延后到 v1.3.0 |
| S-001~S-006 (统计信息) | crates/types/ | ✅ 已实现 |
| C-001~C-006 (CBO) | crates/optimizer/ | ✅ 已实现 |
| R-001~R-007 (核心接口) | 各 crate trait | ✅ 已实现 |
| 目录重构 | crates/ workspace | ✅ 已完成 (PR #305) |

---

## 十、验收标准

### 架构验收

- [ ] Optimizer trait 定义完成
- [ ] Rule trait 定义完成
- [ ] CostModel trait 定义完成
- [ ] StatisticsProvider trait 定义完成
- [ ] Catalog trait 定义完成
- [ ] Executor trait 定义完成
- [ ] StorageEngine trait 定义完成
- [ ] 错误域分离完成
- [ ] Pipeline 执行模型完成

### 功能验收

- [ ] CBO 优化生效
- [ ] 统计信息收集正常
- [ ] Catalog 可持久化
- [ ] 向量化执行可用

---

## 十一、相关文档

- [VERSION_PLAN.md](./VERSION_PLAN.md)
- [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md)
- [TASK_MATRIX.md](./TASK_MATRIX.md)
- [AI_CLI_GUIDE.md](./AI_CLI_GUIDE.md)

---

## 十二、最小可运行 CBO 示例 (Rust)

> 这是一个极简可运行示例，可以此为基础扩展成真正 CBO

```rust
use std::collections::HashMap;

#[derive(Clone)]
enum LogicalPlan {
    Scan { table: String },
    Join { left: Box<LogicalPlan>, right: Box<LogicalPlan> },
}

struct TableStats {
    row_count: usize,
}

struct StatsProvider {
    stats: HashMap<String, TableStats>,
}

impl StatsProvider {
    fn get(&self, table: &str) -> usize {
        self.stats.get(table).unwrap().row_count
    }
}

trait CostModel {
    fn estimate(&self, plan: &LogicalPlan) -> usize;
}

struct SimpleCostModel<'a> {
    stats: &'a StatsProvider,
}

impl<'a> CostModel for SimpleCostModel<'a> {
    fn estimate(&self, plan: &LogicalPlan) -> usize {
        match plan {
            LogicalPlan::Scan { table } => self.stats.get(table),
            LogicalPlan::Join { left, right } => {
                self.estimate(left) * self.estimate(right)
            }
        }
    }
}

fn main() {
    let mut map = HashMap::new();
    map.insert("users".into(), TableStats { row_count: 1000 });
    map.insert("orders".into(), TableStats { row_count: 100 });

    let stats = StatsProvider { stats: map };

    let plan = LogicalPlan::Join {
        left: Box::new(LogicalPlan::Scan { table: "users".into() }),
        right: Box::new(LogicalPlan::Scan { table: "orders".into() }),
    };

    let cost_model = SimpleCostModel { stats: &stats };

    println!("Estimated cost: {}", cost_model.estimate(&plan));
}
```

**特点**:
- ✅ 有 LogicalPlan
- ✅ 有 Stats
- ✅ 有 CostModel
- ✅ 可运行
- ✅ 可扩展

---

## 十三、1.2 → 2.0 架构断层风险图

### 🚨 高风险断层点

| 风险点 | 如果 1.2 没做会怎样 | 影响 |
|--------|---------------------|------|
| 无 Catalog | 无法做分布式元数据 | 🔴 高 |
| 无 Stats 接口 | 无法做全局优化 | 🔴 高 |
| 无 Executor trait | 无法远程执行 | 🔴 高 |
| 无 Storage trait | 无法做复制 | 🔴 高 |
| 无错误分域 | 分布式错误爆炸 | 🟠 中 |
| 无 Pipeline 模型 | 无法向量化 | 🟠 中 |

### 安全线

只要完成以下 6 个 trait，2.0 演进安全：

```
✅ Executor trait      → 远程执行
✅ StorageEngine trait → 分布式存储
✅ StatisticsProvider  → 全局优化
✅ Catalog            → 元数据共享
✅ Optimizer trait    → 可扩展优化
✅ Error 分类         → 错误传播
```

---

## 十四、30 天执行计划

### 第 1 周：结构重构

| Day | 任务 |
|-----|------|
| Day 1-2 | 抽离 Executor trait，移除 main.rs 中硬编码依赖 |
| Day 3-4 | 创建 optimizer/ 目录，定义 Rule trait |
| Day 5-7 | 定义 CostModel trait，定义 Optimizer trait，写 NoOpOptimizer |

### 第 2 周：Statistics + Catalog

| Day | 任务 |
|-----|------|
| Day 8-10 | 创建 statistics 模块，实现 TableStats |
| Day 11-12 | 创建 catalog trait，实现 InMemoryCatalog |
| Day 13-14 | 优化器改为依赖 StatsProvider |

### 第 3 周：CBO 最小实现

| Day | 任务 |
|-----|------|
| Day 15-16 | 实现 JoinCostModel |
| Day 17-18 | 实现 Join Reorder Rule |
| Day 19-21 | 实现两阶段优化流程 |

### 第 4 周：工程化稳定

| Day | 任务 |
|-----|------|
| Day 22-23 | 错误域重构 |
| Day 24-25 | 单元测试覆盖 CBO |
| Day 26-27 | Benchmark（对比优化前后） |
| Day 28-30 | 写 1.2 白皮书，更新架构图，准备 1.3 节点模型草图 |

---

## 十五、Cascades Optimizer 架构设计

> ⚠️ **当前状态**: v1.2.0 实现了简化版 CBO，完整 Cascades 优化器是 v1.3.0 目标

### 核心思想

> Rule 生成等价表达式，Memo 负责去重，Cost 负责选择。

### 1. Memo 结构

```rust
pub struct Memo {
    pub groups: Vec<Group>,
}

pub struct Group {
    pub logical_exprs: Vec<LogicalExpr>,
    pub physical_exprs: Vec<PhysicalExpr>,
    pub best_plan: Option<BestPlan>,
}
```

**设计原则**:
- 每个 Group 代表"语义等价"的表达式集合
- Rule 只往 Group 里添加表达式
- Cost 只选最优

### 2. 表达式分层

```rust
pub enum LogicalExpr {
    Scan { table: String },
    Join { left: GroupId, right: GroupId },
}

pub enum PhysicalExpr {
    HashJoin { left: GroupId, right: GroupId },
    NestedLoopJoin { left: GroupId, right: GroupId },
}
```

**注意**:
- Logical 不知道物理实现
- Physical 不影响等价性

### 3. Rule 分两类

| 类型 | 作用 | 示例 |
|------|------|------|
| Transformation Rule | 逻辑等价变换 | Join Reorder, Predicate Pushdown |
| Implementation Rule | 物理实现生成 | Join → HashJoin |

```rust
pub trait TransformationRule {
    fn apply(&self, memo: &mut Memo, group_id: GroupId);
}

pub trait ImplementationRule {
    fn apply(&self, memo: &mut Memo, group_id: GroupId);
}
```

### 4. 扩展性

| 扩展能力 | 如何支持 |
|----------|----------|
| 新 Join 算法 | 新增 ImplementationRule |
| 分布式 Join | 新增 PhysicalExpr 变种 |
| GPU 执行 | 新增 CostModel 分支 |
| 列存支持 | 新增 Scan 物理实现 |

---

## 十六、2.0 分布式执行原型结构

### 1. DistributedPlan 设计

```rust
pub enum DistributedPlan {
    Local(PhysicalPlan),
    Remote {
        node_id: NodeId,
        plan: PhysicalPlan,
    },
    Shuffle {
        input: Box<DistributedPlan>,
        partition_key: String,
    },
    Gather {
        inputs: Vec<DistributedPlan>,
    },
}
```

### 2. 目录结构

```
distributed/
├── node.rs              # 节点模型
├── rpc.rs              # RPC 通信
├── distributed_plan.rs  # 分布式计划
├── global_optimizer.rs # 全局优化器
└── scheduler.rs         # 任务调度
```

### 3. 预留接口

| 接口 | 作用 |
|------|------|
| Executor trait | 可替换执行 |
| Storage trait | 可远程复制 |
| Statistics trait | 全局优化 |
| Catalog trait | 元数据共享 |

---

## 十七、1.x → 3.0 五年演进路线图

### 第一阶段 (Year 1): 单机内核成熟

| 版本 | 目标 |
|------|------|
| 1.2 | 架构接口化 |
| 1.3 | 节点抽象 |
| 1.4 | WAL 抽象 |
| 1.5 | 可复制日志 |

**成果**: 可嵌入数据库内核

### 第二阶段 (Year 2): 2.0 分布式数据库

| 实现 |
|------|
| Raft 共识 |
| 分布式执行 |
| 分布式优化 |
| 分区表 |

**对标**: CockroachDB, TiDB

### 第三阶段 (Year 3): 高性能优化

| 能力 |
|------|
| 向量化执行 |
| 列存支持 |
| JIT 编译 |
| 成本模型升级 |

**对标**: DuckDB

### 第四阶段 (Year 4): 自治数据库

| 能力 |
|------|
| 自动索引 |
| 自动统计 |
| 自适应执行 |
| 查询缓存 |

### 第五阶段 (Year 5): AI + 数据库内核

| 实现 |
|------|
| 查询计划自学习 |
| Workload 预测 |
| 成本模型自动调参 |
| 智能数据分布 |

---

## 十八、最终判断

### 🚨 如果 1.2 架构不稳

- 2.0 会重写 optimizer
- 3.0 会重写 execution

### ✅ 如果 1.2 做对

- 2.0 只是增加分布式层
- 3.0 只是增加智能层

### 🎯 战略总结

> 你现在的位置不是"写数据库"，而是在设计一个可扩展数据库内核平台。

---

*本文档基于 ChatGPT 架构评估报告整合*
*制定日期: 2026-03-05*
*更新日期: 2026-03-05*
