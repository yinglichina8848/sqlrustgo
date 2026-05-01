# MySQL Wire Protocol Server 重构开发文档

## Issue

**Issue**: [#1536 - refactor: MySQL Wire Protocol Server 重构以支持 Sysbench 测试](https://github.com/minzuuniversity/sqlrustgo/issues/1536)

**目标**: 修复 `sqlrustgo-mysql-server` crate，使 SQLRustGo 能够接受标准 MySQL 客户端连接，从而支持 sysbench 兼容性测试。

---

## 1. 问题分析

### 1.1 当前状态

| 组件 | 状态 | 问题 |
|------|------|------|
| `sqlrustgo-mysql-server` | ❌ 无法编译 | API 不兼容 |
| `sqlrustgo-server` | ❌ 无法编译 | serde_json 缺失 |
| `sqlrustgo` (根库) | ⚠️ 编译错误 | ExecutionEngine 存在 bug |
| `sqlrustgo-sql-cli` | ❌ 无法编译 | API 不匹配 |

### 1.2 根本原因

#### 问题 1: `ExecutionEngine` 编译错误

**位置**: `src/lib.rs:29`

```rust
pub fn new(storage: Arc<RwLock<MemoryStorage>>) -> Self {
    let storage_ref: &'static dyn StorageEngine = unsafe {
        &*(storage.read().unwrap() as *const MemoryStorage as *const dyn StorageEngine)
        //^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        // ERROR: Non-primitive cast
    };
    ...
}
```

**问题**: `RwLockReadGuard<MemoryStorage>` 不能用 `as` 转换为 `*const MemoryStorage`

#### 问题 2: API 签名不匹配

**mysql-server 期望**:
```rust
pub fn execute(&mut self, statement: Statement) -> SqlResult<ExecutorResult>
```

**实际 LocalExecutor 签名**:
```rust
pub fn execute(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult>
```

**问题**: LocalExecutor 期望 `PhysicalPlan`，而 mysql-server 传入 `Statement`

#### 问题 3: Statement → PhysicalPlan 转换缺失

当前代码无法将 `sqlrustgo_parser::Statement` 转换为 `sqlrustgo_planner::PhysicalPlan`。

### 1.3 影响范围

```
sqlrustgo-mysql-server
├── 依赖: sqlrustgo (根库)
│   └── ExecutionEngine (有 bug)
├── 依赖: sqlrustgo-parser
├── 依赖: sqlrustgo-storage
└── 问题:
    ├── ExecutionEngine::new() 编译错误
    ├── ExecutionEngine::execute() 签名不匹配
    └── 无法创建 PhysicalPlan from Statement
```

---

## 2. 解决方案设计

### 2.1 架构图

```
                    ┌─────────────────────────────────────────────────────────┐
                    │              MySQL Wire Protocol Server                 │
                    │  (crates/mysql-server/src/lib.rs)                     │
                    │                                                          │
                    │  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐  │
                    │  │  handshake  │→ │ query parser │→ │ result writer│  │
                    │  └─────────────┘  └──────────────┘  └──────────────┘  │
                    └────────────────────────┬──────────────────────────────────┘
                                             │
                                             ▼
                    ┌─────────────────────────────────────────────────────────┐
                    │              MySqlServerAdapter                         │
                    │  (新创建)                                                │
                    │                                                          │
                    │  - parse() → Statement                                  │
                    │  - plan() → PhysicalPlan                                │
                    │  - execute() → ExecutorResult                           │
                    │  - convert_result() → MySQL packets                     │
                    └────────────────────────┬─────────────────────────────────┘
                                             │
                    ┌────────────────────────▼────────────────────────────────┐
                    │              LocalExecutor                              │
                    │  (sqlrustgo_executor::LocalExecutor)                    │
                    │                                                          │
                    │  - execute(plan: &dyn PhysicalPlan)                      │
                    │  - 需要 Statement → PhysicalPlan 转换                    │
                    └──────────────────────────────────────────────────────────┘
```

### 2.2 核心组件

#### 2.2.1 MySqlServerAdapter (新创建)

```rust
pub struct MySqlServerAdapter {
    planner: DefaultPlanner,
    optimizer: DefaultOptimizer,
    storage: Arc<RwLock<dyn StorageEngine>>,
    session_manager: SessionManager,
}

impl MySqlServerAdapter {
    /// 创建 adapter
    pub fn new(storage: Arc<RwLock<MemoryStorage>>) -> Self { ... }

    /// 解析 SQL
    pub fn parse(&self, sql: &str) -> SqlResult<Statement> { ... }

    /// 生成执行计划
    pub fn plan(&self, stmt: Statement) -> SqlResult<Box<dyn PhysicalPlan>> { ... }

    /// 执行查询
    pub fn execute(&self, sql: &str) -> SqlResult<ExecutorResult> {
        let stmt = self.parse(sql)?;
        let plan = self.plan(stmt)?;
        self.executor.execute(plan.as_ref())
    }
}
```

#### 2.2.2 Statement → PhysicalPlan 转换

```rust
impl MySqlServerAdapter {
    fn plan(&self, stmt: Statement) -> SqlResult<Box<dyn PhysicalPlan>> {
        // 1. 转换为 LogicalPlan
        let logical_plan = self.planner.create_plan(stmt)?;

        // 2. 优化 LogicalPlan
        let optimized_plan = self.optimizer.optimize(logical_plan)?;

        // 3. 转换为 PhysicalPlan
        let physical_plan = self.planner.create_physical_plan(optimized_plan)?;

        Ok(physical_plan)
    }
}
```

### 2.3 修复计划

#### Phase 1: 修复 ExecutionEngine (src/lib.rs)

**文件**: `src/lib.rs`

**问题 1a - 修复 unsafe cast**:
```rust
// 错误写法
let storage_ref: &'static dyn StorageEngine = unsafe {
    &*(storage.read().unwrap() as *const MemoryStorage as *const dyn StorageEngine)
};

// 正确写法 - 使用 Arc::clone 和 Box
pub fn new(storage: Arc<RwLock<MemoryStorage>>) -> Self {
    // 方案 1: 直接使用 Arc<&dyn StorageEngine>
    let storage_ref = Arc::clone(&storage);
    let storage_engine: Arc<dyn StorageEngine> = storage_ref;
    // 或方案 2: 使用 mutex/guard 模式
}
```

**问题 1b - 修复 execute 签名**:
```rust
// 当前 (错误)
pub fn execute(&mut self, statement: Statement) -> SqlResult<ExecutorResult> {
    self.executor.execute(&statement)  // 期望 PhysicalPlan
}

// 需要添加 planner 转换
pub fn execute(&mut self, statement: Statement) -> SqlResult<ExecutorResult> {
    let plan = self.planner.statement_to_plan(statement)?;
    self.executor.execute(plan.as_ref())
}
```

#### Phase 2: 更新 mysql-server crate

**文件**: `crates/mysql-server/src/lib.rs`

**更新导入**:
```rust
// 删除
use sqlrustgo::{parse, ExecutionEngine, ExecutorResult, SqlError, Value};

// 添加
use sqlrustgo::{
    parse,
    MySqlServerAdapter,  // 新 adapter
    Value,
};
use sqlrustgo_types::SqlError;
```

**更新执行逻辑**:
```rust
// 旧代码
let mut engine = ExecutionEngine::new(storage.clone());
let result = engine.execute(statement)?;

// 新代码
let adapter = MySqlServerAdapter::new(storage.clone());
let result = adapter.execute(sql)?;
```

#### Phase 3: 修复 server crate (可选)

**文件**: `crates/server/Cargo.toml`

**添加依赖**:
```toml
serde_json = "1"
```

---

## 3. 实施步骤

### 3.1 任务分解

| 任务 | 优先级 | 估计时间 | 依赖 |
|------|--------|----------|------|
| T1: 修复 src/lib.rs ExecutionEngine | P0 | 2h | 无 |
| T2: 创建 MySqlServerAdapter | P0 | 4h | T1 |
| T3: 更新 mysql-server 使用新 adapter | P0 | 2h | T2 |
| T4: 修复 server crate serde_json | P1 | 1h | 无 |
| T5: 添加 workspace members | P1 | 0.5h | T3 |
| T6: 集成测试 | P0 | 2h | T3 |
| T7: sysbench 验证 | P0 | 2h | T6 |

### 3.2 验证标准

- [ ] `cargo build -p sqlrustgo-mysql-server` 成功
- [ ] `cargo build -p sqlrustgo-server` 成功
- [ ] MySQL 客户端可以连接
- [ ] `sysbench oltp_read_write --db-driver=mysql` 可以运行
- [ ] QPS 基线测试通过 (≥500 for read_write)

---

## 4. 风险评估

| 风险 | 影响 | 缓解 |
|------|------|------|
| Statement → PhysicalPlan 转换复杂 | 高 | 使用现有的 planner |
| StorageEngine trait 对象安全 | 中 | 使用 Arc<dyn StorageEngine> |
| Session 管理需要适配 | 中 | 复用 security crate |
| 测试环境配置 | 低 | 文档化步骤 |

---

## 5. 参考资料

- [MySQL Wire Protocol](https://dev.mysql.com/doc/internals/en/client-server-protocol.html)
- [现有 mysql-server 实现](../../crates/mysql-server/src/lib.rs)
- [LocalExecutor 实现](../../crates/executor/src/local_executor.rs)
- [Planner API](../../crates/planner/src/lib.rs)
