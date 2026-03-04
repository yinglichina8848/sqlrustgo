# v1.2.0 任务分解矩阵（甘特图版）

> **版本**: v1.2.0
> **代号**: Architecture Stabilization
> **制定日期**: 2026-03-05
> **依据**: ChatGPT 架构评估报告 - 修正版

---

## ⚠️ 重要架构决策

### ❌ 错误路线（会导致 2.0 重写）

- 向量化 + CBO 同步推进
- 执行模型不稳定就做成本模型
- Rule 设计不支持 Memo

### ✅ 正确路线（推荐）

- 接口化优先
- 执行模型稳定后，再做 CBO
- Memo 雏形 + 分布式预留字段

---

## 零、1.2 冻结接口清单（宪法）

> 这些接口一旦发布 v1.2.0，不允许在 1.x 修改签名。

### 🔥 一级冻结（绝对不能动）

#### 1. Operator Trait（执行模型核心）

```rust
pub trait Operator: Send {
    fn open(&mut self);
    fn next_batch(&mut self) -> Option<RecordBatch>;
    fn close(&mut self);
}
```

**为什么必须冻结？**
- 向量化依赖它
- 分布式执行依赖它
- 未来 async 也必须通过包装，而不是改签名

#### 2. RecordBatch 结构

```rust
pub struct RecordBatch {
    pub schema: Arc<Schema>,
    pub columns: Vec<ArrayRef>,
}
```

**冻结规则**:
- 不允许加入生命周期参数
- 不允许加入执行状态
- 只允许新增可选字段

#### 3. PlanNode 抽象层

```rust
pub enum PlanNode {
    Local(PhysicalPlan),
    Distributed(DistributedPlan),
}
```

即使 Distributed 现在是空壳，也必须存在。

#### 4. Cost 结构

```rust
pub struct Cost {
    pub cpu: f64,
    pub io: f64,
    pub network: f64,  // 1.2 = 0
}
```

#### 5. Optimizer Trait

```rust
pub trait Optimizer {
    fn optimize(&self, memo: &mut Memo) -> GroupId;
}
```

⚠️ 不要再使用: `fn optimize(plan: LogicalPlan)`

#### 6. Executor Trait

```rust
pub trait Executor: Send + Sync {
    fn execute(&self, plan: PlanNode) -> Result<ResultSet>;
}
```

必须接受 PlanNode，而不是 PhysicalPlan。

#### 7. Catalog Trait

```rust
pub trait Catalog: Send + Sync {
    fn get_table(&self, name: &str) -> Option<TableMeta>;
}

pub struct TableMeta {
    pub name: String,
    pub schema: Schema,
    pub distribution: Option<DistributionInfo>,
}
```

#### 8. StatisticsProvider

```rust
pub trait StatisticsProvider: Send + Sync {
    fn table_stats(&self, table: &str) -> Option<TableStats>;
}
```rust
pub struct TableStats {
    pub row_count: usize,
    pub source: StatsSource,
}

#[derive(Clone, Debug)]
pub enum StatsSource {
    Local,
    Global,
}
```

### 二级冻结（谨慎修改）

- StorageEngine trait
- QueryService trait
- Array trait

允许扩展方法，但不能改已有签名。

---

## 一、甘特图（关键路径）

### Phase 1: 核心接口稳定（Week 1-2）

```
Week 1                              Week 2
┌─────────────────────────────┬─────────────────────────────┐
│ R-001 错误域重构              │ R-005 Executor trait       │
│ R-002 Optimizer trait        │ R-006 StorageEngine trait  │
│ R-003 Statistics trait       │ R-007 QueryService trait  │
│ R-004 Catalog trait          │                             │
└─────────────────────────────┴─────────────────────────────┘
```

### Phase 2: 执行模型定型（Week 3-4）

```
Week 3                              Week 4
┌─────────────────────────────┬─────────────────────────────┐
│ E-001 RecordBatch 结构        │ E-003 LocalExecutor 实现   │
│ E-002 Operator trait         │ E-004 执行器集成            │
└─────────────────────────────┴─────────────────────────────┘
                           ↑
                    ⚠️ 冻结执行模型
```

### Phase 3: 统计系统（Week 5）

```
Week 5
┌─────────────────────────────┬─────────────────────────────┐
│ S-001 TableStats 结构         │ S-003 统计收集器            │
│ S-002 ColumnStats 结构       │ S-004 ANALYZE 命令         │
└─────────────────────────────┴─────────────────────────────┘
```

### Phase 4: CBO 框架（Week 6-7）

```
Week 6                              Week 7
┌─────────────────────────────┬─────────────────────────────┐
│ O-001 Memo 结构               │ O-005 Join 重排序          │
│ O-002 谓词下推规则            │ O-006 成本模型             │
│ O-003 投影裁剪规则            │                             │
│ O-004 常量折叠                │                             │
└─────────────────────────────┴─────────────────────────────┘
```

### Phase 5: 工程化 + 文档（Week 8）

```
Week 8
┌─────────────────────────────┬─────────────────────────────┐
│ C-001~C-004 Catalog 完善     │ D-001~D-002 文档            │
└─────────────────────────────┴─────────────────────────────┘
```

---

## 🎯 对比：原计划 vs 新计划

| 项目 | 原计划 | 新计划 |
|------|--------|--------|
| 向量化时间 | Week 1 | Week 8 |
| CBO 时间 | Week 7 | Week 6 |
| 接口冻结 | 未明确 | Week 1 完成 |
| 2.0 重写概率 | 高 | 低 |

### 战略原则

**先冻结接口 → 再写执行 → 再写优化 → 最后做性能**

顺序错了，必重写。

---

## 二、架构隐患分析

### 🚨 核心风险

| 风险 | 后果 | 解决方案 |
|------|------|----------|
| 执行模型和 CBO 同时重构 | CostModel 假设错误 | Phase 2 冻结执行模型 |
| Rule 与 Memo 不兼容 | 2.0 重写 Optimizer | O-001 先做 Memo 雏形 |
| Catalog 无分布式字段 | 2.0 扩展影响 LogicalPlan | C-001 加 distribution 预留 |
| RecordBatch 无序列化协议 | 2.0 重写传输层 | B-001 加 BinaryFormat 预留 |

### 🔴 重写概率评估

| 模块 | 重写概率 | 原因 |
|------|----------|------|
| Optimizer | 70% | 执行模型不稳 |
| CostModel | 80% | 假设错误 |
| 执行层 | 60% | 向量化过早 |
| Catalog | 50% | 无分布式字段 |

---

## 三、详细任务分解

### 轨道 R: 核心接口 (P0)

| ID | 任务 | Week | 依赖 | 负责人 | 工时 |
|----|------|------|------|--------|------|
| R-001 | 错误域重构 | 1 | - | openheart | 6h |
| R-002 | Optimizer trait | 1 | - | openheart | 8h |
| R-003 | Statistics trait | 1 | - | openheart | 4h |
| R-004 | Catalog trait | 1 | - | heartopen | 6h |
| R-005 | Executor trait | 2 | R-002 | heartopen | 8h |
| R-006 | StorageEngine trait | 2 | - | heartopen | 6h |
| R-007 | QueryService trait | 2 | R-005 | heartopen | 4h |

### 轨道 E: 执行层 (P0)

| ID | 任务 | Week | 依赖 | 负责人 | 工时 |
|----|------|------|------|--------|------|
| E-001 | RecordBatch 结构 | 3 | R-005 | heartopen | 4h |
| E-002 | Operator trait | 3 | E-001 | heartopen | 8h |
| E-003 | LocalExecutor | 4 | E-002 | heartopen | 6h |
| E-004 | 执行器集成 | 4 | E-003, R-007 | heartopen | 4h |

**⚠️ 此阶段结束后执行模型必须冻结**

### 轨道 S: 统计信息 (P1)

| ID | 任务 | Week | 依赖 | 负责人 | 工时 |
|----|------|------|------|--------|------|
| S-001 | TableStats 结构 | 5 | R-003 | openheart | 4h |
| S-002 | ColumnStats 结构 | 5 | S-001 | openheart | 4h |
| S-003 | 统计收集器 | 5 | S-002 | openheart | 6h |
| S-004 | ANALYZE 命令 | 5 | S-003 | heartopen | 4h |

### 轨道 O: 优化器 (P1)

| ID | 任务 | Week | 依赖 | 负责人 | 工时 |
|----|------|------|------|--------|------|
| O-001 | Memo 结构 | 6 | R-002 | openheart | 4h |
| O-002 | 谓词下推 | 6 | O-001 | openheart | 6h |
| O-003 | 投影裁剪 | 6 | O-001 | openheart | 4h |
| O-004 | 常量折叠 | 6 | O-001 | openheart | 4h |
| O-005 | Join 重排序 | 7 | O-001, S-004 | openheart | 8h |
| O-006 | 成本模型 | 7 | O-005 | openheart | 6h |

### 轨道 C: Catalog 完善 (P1)

| ID | 任务 | Week | 依赖 | 负责人 | 工时 |
|----|------|------|------|--------|------|
| C-001 | TableMeta + distribution | 8 | R-004 | heartopen | 4h |
| C-002 | InMemoryCatalog | 8 | C-001 | heartopen | 6h |
| C-003 | Catalog 持久化 | 8 | C-002 | heartopen | 6h |
| C-004 | 系统表 | 8 | C-002 | heartopen | 4h |

### 轨道 B: 桥接层 (P1)

| ID | 任务 | Week | 依赖 | 负责人 | 工时 |
|----|------|------|------|--------|------|
| B-001 | BinaryFormat 预留 | 8 | E-001 | openheart | 4h |
| B-002 | NetworkCost 预留 | 8 | O-006 | openheart | 4h |

---

## 四、1.2 → 2.0 桥接层设计

### 4.1 DistributedPlan 占位

```rust
pub enum PlanNode {
    Local(PhysicalPlan),
    Distributed(DistributedPlan),  // 1.2 空壳，2.0 实现
}

pub struct DistributedPlan {
    // 1.2 为空
}
```

### 4.2 Catalog 预留分布式字段

```rust
pub struct TableMeta {
    pub name: String,
    pub schema: Schema,
    pub distribution: Option<DistributionInfo>,  // 预留
}

pub struct DistributionInfo {
    pub partition_key: Vec<String>,
    pub replicas: usize,
}
```

### 4.3 Cost 预留网络成本

```rust
pub struct Cost {
    pub cpu: f64,
    pub io: f64,
    pub network: f64,  // 1.2 为 0
}
```

### 4.4 Executor 预留远程实现

```rust
trait Executor {
    fn execute(&self, plan: PlanNode) -> Result<ResultSet>;
}

// 未来实现:
// - LocalExecutor
// - RemoteExecutor
// - DistributedExecutor
```

### 4.5 RecordBatch 预留序列化

```rust
impl RecordBatch {
    pub fn to_bytes(&self) -> Vec<u8>;      // 预留
    pub fn from_bytes(bytes: &[u8]) -> Self; // 预留
}
```

---

## 五、安全线检查

| 接口 | 2.0 必需 | 预留字段 |
|------|----------|----------|
| Executor trait | ✅ | PlanNode |
| StorageEngine trait | ✅ | - |
| StatisticsProvider | ✅ | - |
| Catalog | ✅ | distribution |
| Optimizer | ✅ | Memo |
| CostModel | ✅ | network |
| RecordBatch | ✅ | to_bytes |

---

## 六、版本发布流程

```
Week 1-2 → Week 3-4 → Week 5 → Week 6-7 → Week 8
   接口        执行       统计      CBO       收尾
    ↓           ↓          ↓         ↓          ↓
v1.2.0-draft → alpha → beta → rc → GA
```

---

## 七、向量化延后说明

**向量化任务 (V-001~V-007) 延后到 v1.3.0**

原因：
1. 执行模型必须先稳定
2. 向量化应建立在稳定 Operator trait 之后
3. 避免 2.0 重写执行层

v1.3.0 将包含：
- 向量化执行
- 节点抽象
- WAL 预留

---

## 八、相关文档

- [ARCHITECTURE_REFACTORING_PLAN.md](./ARCHITECTURE_REFACTORING_PLAN.md)
- [VERSION_PLAN.md](./VERSION_PLAN.md)
- [VERSION_ROADMAP_1X_TO_2.md](../VERSION_ROADMAP_1X_TO_2.md)

---

*本文档由 AI 助手生成*
*制定日期: 2026-03-05*
