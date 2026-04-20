# MVCC SSI 完整集成设计

> **日期**: 2026-04-20
> **状态**: Implemented
> **目标**: 将 MVCC SSI 完整集成到执行引擎，支持标准 SQL 事务语句

## 1. 概述

将 MVCC SSI (Serializable Snapshot Isolation) 完整集成到 SQLRustGo，使其支持标准 SQL 事务语句和冲突检测。

### 1.1 目标

- Parser 支持 `BEGIN`, `COMMIT`, `ROLLBACK`, `SET TRANSACTION ISOLATION LEVEL`
- ExecutionEngine 管理事务生命周期，使用 SSI 检测器
- Storage 层使用 MVCC 版本链
- 完整的集成测试覆盖

### 1.2 当前状态

| 组件 | 状态 |
|-------|------|
| SSI 检测器 (`crates/transaction/src/ssi.rs`) | ✅ 完成 |
| MVCC 快照 (`crates/transaction/src/mvcc.rs`) | ✅ 完成 |
| 版本链 (`crates/transaction/src/version_chain.rs`) | ✅ 完成 |
| SSI 单元测试 | ✅ 28 tests |
| 执行引擎集成 | ✅ 完成 |
| SQL 事务语句 (Parser) | ✅ 完成 |
| TransactionManager | ✅ 完成 |
| 集成测试 | ✅ 20+ tests |

## 2. 架构设计

### 2.1 组件层次

```
┌─────────────────────────────────────┐
│         SQL Statement                │
├─────────────────────────────────────┤
│         Parser                      │ ← 新增事务语句解析
├─────────────────────────────────────┤
│      ExecutionEngine                │ ← 新增事务管理
├─────────────────────────────────────┤
│    TransactionManager               │ ← 使用 SSI 检测器
├─────────────────────────────────────┤
│    MVCC Storage Engine              │ ← 已有，待集成
└─────────────────────────────────────┘
```

### 2.2 Parser 层变更

新增 `TransactionStatement` 枚举：

```rust
pub enum TransactionStatement {
    Begin(Option<IsolationLevel>),
    Commit,
    Rollback,
    SetTransaction { isolation_level: IsolationLevel },
}
```

### 2.3 ExecutionEngine 层变更

```rust
pub struct ExecutionEngine {
    storage: Arc<RwLock<dyn StorageEngine>>,
    // 新增
    transaction_manager: TransactionManager,
    current_tx: Option<TransactionContext>,
}

impl ExecutionEngine {
    pub fn execute_transaction(&mut self, stmt: &TransactionStatement) -> SqlResult<()>;
    pub fn begin_transaction(&mut self, isolation: IsolationLevel) -> SqlResult<()>;
    pub fn commit(&mut self) -> SqlResult<()>;
    pub fn rollback(&mut self) -> SqlResult<()>;
}
```

### 2.4 TransactionManager

```rust
pub struct TransactionManager {
    ssi_detector: SsiDetector,
    mvcc: MVCC,
    active_tx: HashMap<TxId, TransactionState>,
}

pub enum IsolationLevel {
    ReadCommitted,
    SnapshotIsolation,
    Serializable,
}
```

## 3. 实现步骤

### Step 1: Parser 层

- [ ] 新增 `TransactionStatement` 枚举
- [ ] 新增 `IsolationLevel` 枚举
- [ ] 实现 `parse_begin()`, `parse_commit()`, `parse_rollback()`
- [ ] 实现 `parse_set_transaction()`
- [ ] 单元测试

### Step 2: TransactionManager

- [ ] 新增 `TransactionManager` 结构体
- [ ] 集成 `SsiDetector` 和 `MVCC`
- [ ] 实现 `begin()`, `commit()`, `rollback()`
- [ ] 实现 SSI 冲突检测

### Step 3: ExecutionEngine 集成

- [ ] 新增事务管理字段
- [ ] 实现 `execute_transaction()` 分发
- [ ] 支持嵌套事务
- [ ] 错误处理 (事务冲突时回滚)

### Step 4: Storage 层集成

- [ ] MVCC Storage 与 WAL 集成
- [ ] 版本链 GC
- [ ] 事务日志

### Step 5: 测试

- [ ] SSI 冲突检测集成测试
- [ ] 并发事务正确性测试
- [ ] 回归测试框架集成
- [ ] TPC-C 风格测试

## 4. 测试计划

### 4.1 SSI 冲突检测测试

```rust
// 经典 WriteSkew 测试
// T1: READ X, WRITE Y
// T2: READ Y, WRITE X
// 应该有一个事务失败
```

### 4.2 并发事务测试

- 并发读取同一行
- 并发写入同一行
- 并发写入不同行但有依赖
- 长事务与短事务交错

### 4.3 回归测试

- `BEGIN; SELECT; UPDATE; COMMIT;`
- `BEGIN SERIALIZABLE; ... ; ROLLBACK;`
- `SET TRANSACTION ISOLATION LEVEL SERIALIZABLE; ...`

## 5. 风险和缓解

| 风险 | 缓解 |
|------|------|
| SSI 性能开销 | 使用只读快照优化 |
| 版本链 GC 复杂性 | 实现保守 GC |
| 测试覆盖不足 | TPC-C 风格测试 |

## 6. 验收标准

- [ ] `BEGIN SERIALIZABLE; ... ; COMMIT;` 可执行
- [ ] WriteSkew 冲突被正确检测
- [ ] SSI 集成测试 > 20 tests
- [ ] 回归测试框架包含 MVCC 测试
- [ ] Clippy 和 fmt 通过
