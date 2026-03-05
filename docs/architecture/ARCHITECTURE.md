# SQLRustGo 架构总览

> **版本**: 1.2
> **代号**: Vector Engine
> **更新日期**: 2026-03-05

---

## 1. 系统架构总览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              SQLRustGo 架构                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│      SQL Query                                                               │
│         │                                                                    │
│         ▼                                                                    │
│   ┌─────────┐     ┌──────────────┐     ┌─────────────┐                   │
│   │ Parser  │────►│   Optimizer   │────►│  Executor   │                   │
│   └─────────┘     │ (Rule + CBO)  │     └──────┬──────┘                   │
│                   └──────────────┘              │                           │
│                         │                       ▼                           │
│                         │               ┌─────────────┐                     │
│                         └───────────────►│  Storage    │                     │
│                                           │  Engine     │                     │
│                                           └─────────────┘                     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 核心模块

| 模块 | 职责 | 关键组件 |
|------|------|----------|
| Parser | SQL 解析 | Lexer, AST |
| Optimizer | 查询优化 | Memo, Rule, CostModel |
| Executor | 查询执行 | Operator, RecordBatch |
| Storage | 数据存储 | StorageEngine, FileStorage |

---

## 2. 查询执行流程

### 2.1 完整流程

```
┌─────────┐    ┌────────────┐    ┌─────────┐    ┌───────────┐
│   SQL   │───►│   Parser   │───►│ Logical │───►│ Optimizer │
│ Query   │    │            │    │  Plan   │    │ (Rule+CBO)│
└─────────┘    └────────────┘    └─────────┘    └─────┬─────┘
                                                       │
                                                       ▼
┌─────────┐    ┌────────────┐    ┌─────────┐    ┌───────────┐
│ Result  │◄───│  Storage   │◄───│Physical │◄───│  Planner  │
│         │    │  Engine    │    │  Plan   │    │           │
└─────────┘    └────────────┘    └─────────┘    └───────────┘
```

### 2.2 阶段说明

| 阶段 | 输入 | 输出 | 说明 |
|------|------|------|------|
| Parser | SQL 文本 | AST | 词法分析 + 语法分析 |
| Logical Plan | AST | LogicalPlan | 关系代数表达式 |
| Optimizer | LogicalPlan | PhysicalPlan | 规则优化 + 成本优化 |
| Planner | PhysicalPlan | ExecutionPlan | 物理执行计划 |
| Executor | ExecutionPlan | ResultSet | 数据处理 |

---

## 3. 核心模块详解

### 3.1 Parser (解析器)

```
┌─────────────┐
│    SQL      │
└──────┬──────┘
       │
       ▼
┌─────────────┐    ┌─────────────┐
│   Lexer     │───►│    AST      │
│  (词法)     │    │  (语法树)   │
└─────────────┘    └─────────────┘
```

**关键组件**:

- Lexer: SQL 词法分析
- Parser: SQL 语法分析
- AST: 抽象语法树

### 3.2 Optimizer (优化器)

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Logical    │────►│    Memo     │────►│  Physical   │
│   Plan      │     │ (中间表示)   │     │    Plan     │
└─────────────┘     └─────────────┘     └─────────────┘
                         │
                         ▼
                  ┌─────────────┐
                  │   Rules     │
                  │  (规则优化)  │
                  └─────────────┘
                         │
                         ▼
                  ┌─────────────┐
                  │  CostModel  │
                  │ (成本优化)   │
                  └─────────────┘
```

**优化规则**:

- Predicate Pushdown (谓词下推)
- Projection Pushdown (投影裁剪)
- Constant Folding (常量折叠)
- Join Reordering (Join 重排)

### 3.3 Executor (执行器)

```
┌─────────────────────────────────────────────────────────────┐
│                     Iterator Model                           │
├─────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌─────────┐      ┌─────────────┐      ┌─────────┐        │
│   │  Scan   │─────►│   Filter    │─────►│Project  │        │
│   └─────────┘      └─────────────┘      └────┬────┘        │
│                                               │              │
│                                               ▼              │
│                                          ┌─────────┐         │
│                                          │  Join   │         │
│                                          └────┬────┘         │
│                                               │              │
│                                               ▼              │
│                                          ┌─────────┐         │
│                                          │Aggregate│         │
│                                          └─────────┘         │
│                                                                  │
└──────────────────────────────────────────────────────────────┘
```

**执行模型**:

```rust
pub trait Operator: Send {
    fn open(&mut self);                    // 初始化
    fn next_batch(&mut self) -> Option<RecordBatch>;  // 获取数据
    fn close(&mut self);                    // 清理资源
}
```

### 3.4 Storage Engine (存储引擎)

```
┌────────────────────────────────────────┐
│        StorageEngine Trait              │
├────────────────────────────────────────┤
│                                          │
│  ┌──────────────────────────────────┐   │
│  │         StorageEngine             │   │
│  ├──────────────────────────────────┤   │
│  │ + read(table) -> Vec<Record>     │   │
│  │ + write(table, records)         │   │
│  │ + scan(table, filter)           │   │
│  │ + get_stats(table)               │   │
│  └──────────────────────────────────┘   │
│           ▲                ▲             │
│           │                │             │
│    ┌──────┴─────┐   ┌──────┴─────┐      │
│    │FileStorage │   │MemoryStorage│      │
│    └────────────┘   └────────────┘      │
│                                          │
└──────────────────────────────────────────┘
```

**实现**:

- FileStorage: 持久化存储
- MemoryStorage: 内存存储 (测试/缓存)

---

## 4. 数据结构

### 4.1 RecordBatch

```rust
pub struct RecordBatch {
    pub schema: Arc<Schema>,
    pub columns: Vec<ArrayRef>,   // 列式存储
    pub row_count: usize,
}
```

### 4.2 Array

```rust
pub trait Array: Send + Sync {
    fn data_type(&self) -> &DataType;
    fn len(&self) -> usize;
    fn is_null(&self, index: usize) -> bool;
}
```

### 4.3 Statistics

```rust
pub struct TableStats {
    pub row_count: usize,
    pub total_bytes: usize,
    pub column_stats: HashMap<String, ColumnStats>,
}

pub struct ColumnStats {
    pub ndv: usize,           // 唯一值数量
    pub null_count: usize,    // 空值数量
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
}
```

---

## 5. 模块结构

```
src/
├── parser/          # SQL 解析
│   ├── lexer.rs
│   ├── parser.rs
│   └── ast.rs
├── planner/         # 计划生成
│   ├── logical.rs   # 逻辑计划
│   └── physical.rs  # 物理计划
├── optimizer/       # 查询优化
│   ├── memo.rs      # 中间表示
│   ├── rules.rs     # 优化规则
│   └── cost.rs      # 成本模型
├── executor/        # 执行引擎
│   ├── operator.rs  # 算子抽象
│   ├── batch.rs     # RecordBatch
│   ├── vectors/     # 向量化实现
│   └── engine.rs    # 执行器
├── storage/         # 存储引擎
│   ├── engine.rs   # 存储接口
│   ├── file.rs     # 文件存储
│   ├── memory.rs   # 内存存储
│   └── stats.rs    # 统计信息
├── catalog/         # 目录服务
│   └── catalog.rs
├── network/         # 网络层
│   └── server.rs
└── types/          # 类型系统
    └── value.rs
```

---

## 6. 版本演进

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        SQLRustGo 版本演进                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  v1.0        v1.1        v1.2          v1.3        v2.0               │
│  ──────►   ──────►    ──────►      ──────►    ──────►                │
│                                                                          │
│  SQL执行    基础引擎    向量化+CBO    完整向量    分布式               │
│  原型                    +统计         化执行      执行                  │
│                                                                          │
│  ┌─────┐   ┌─────┐   ┌─────────┐  ┌─────────┐ ┌─────────┐          │
│  │Parser│   │ +   │   │Vectorized│  │  Full   │ │  DAG    │          │
│  │     │   │Exec │   │Executor  │  │Vectorize│ │Executor │          │
│  └─────┘   └─────┘   └─────────┘  └─────────┘ └─────────┘          │
│                                                                          │
│  ┌─────┐   ┌─────┐   ┌─────────┐  ┌─────────┐ ┌─────────┐          │
│  │简单  │   │ +   │   │ +Stats  │  │ +Pipeline│ │Sharding │          │
│  │存储  │   │存储  │   │ +Cost   │  │  +Tuning │ │ +Fault  │          │
│  └─────┘   └─────┘   └─────────┘  └─────────┘ │Tolerance│          │
│                                                   └─────────┘          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 7. 关键接口

### 7.1 Operator Trait

```rust
pub trait Operator: Send {
    fn open(&mut self);
    fn next_batch(&mut self) -> Option<RecordBatch>;
    fn close(&mut self);
}
```

### 7.2 StorageEngine Trait

```rust
pub trait StorageEngine {
    fn read(&self, table: &str) -> Result<Vec<Record>>;
    fn write(&self, table: &str, records: Vec<Record>) -> Result<()>;
    fn scan(&self, table: &str, filter: Option<Filter>) -> Result<Vec<Record>>;
    fn get_stats(&self, table: &str) -> Result<TableStats>;
}
```

### 7.3 Optimizer Trait

```rust
pub trait Optimizer {
    fn optimize(&self, memo: &mut Memo) -> GroupId;
}
```

---

## 8. 扩展点

### 8.1 可扩展接口

| 接口 | 扩展方式 | 用途 |
|------|----------|------|
| StorageEngine | 实现 trait | 支持新存储后端 |
| Operator | 实现 trait | 添加新算子 |
| Optimizer | 添加规则 | 新优化策略 |
| Catalog | 实现 trait | 新数据源 |

### 8.2 Feature Flags

```toml
[features]
default = ["file-storage", "vectorized"]
file-storage = []
memory-storage = []
vectorized = []
distributed = []  # 2.0
```

---

## 9. 相关文档

- [发布白皮书](../whitepaper/sqlrustgo_1.2_release_whitepaper.md)
- [接口冻结](../whitepaper/sqlrustgo_1.2_interface_freeze.md)
- [技术债预测](../whitepaper/sqlrustgo_tech_debt_forecast.md)
- [2.0 分布式框架](../whitepaper/sqlrustgo_2.0_distributed_framework.md)
- [v1.2 发布说明](../releases/v1.2.0/RELEASE_NOTES.md)
