# stored_proc.rs 模块拆分实施计划

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 4539 行的 stored_proc.rs 拆分为 6 个独立模块，提升可测试性，目标 executor 覆盖率从 70% 提升至 85%

**Architecture:**
- 将 stored_proc.rs 拆分为 `context.rs`, `cursor.rs`, `handler.rs`, `cte.rs`, `expression.rs`, `execution.rs`
- 保持 `StoredProcExecutor` 公用 API 不变
- 每个模块独立的 `#[cfg(test)]` 测试

**Tech Stack:** Rust, cargo, cargo llvm-cov

---

## 文件结构

```
crates/executor/src/stored_proc/
├── mod.rs              # StoredProcExecutor, StoredProcError (主入口)
├── context.rs          # ProcedureContext (变量/标签/作用域/Cursor)
├── cursor.rs           # Cursor 相关逻辑
├── handler.rs          # ExceptionHandler
├── cte.rs              # CTE/递归查询执行
├── expression.rs       # 表达式求值
└── execution.rs        # 语句执行
```

---

## 任务分解

### Task 1: 创建目录结构和空模块

**Files:**
- Create: `crates/executor/src/stored_proc/mod.rs` (初始为空)
- Create: `crates/executor/src/stored_proc/context.rs` (空)
- Create: `crates/executor/src/stored_proc/cursor.rs` (空)
- Create: `crates/executor/src/stored_proc/handler.rs` (空)
- Create: `crates/executor/src/stored_proc/cte.rs` (空)
- Create: `crates/executor/src/stored_proc/expression.rs` (空)
- Create: `crates/executor/src/stored_proc/execution.rs` (空)

- [ ] **Step 1: 创建目录**
  ```bash
  mkdir -p crates/executor/src/stored_proc
  ```

- [ ] **Step 2: 创建 mod.rs (初始导入所有子模块)**
  ```rust
  //! Stored Procedure Executor
  //!
  //! This module provides stored procedure execution support with control flow.

  mod context;
  mod cursor;
  mod handler;
  mod cte;
  mod expression;
  mod execution;

  pub use context::ProcedureContext;
  pub use handler::ExceptionHandler;
  pub use self::StoredProcError;
  ```

- [ ] **Step 3: 创建各子模块空壳文件**

- [ ] **Step 4: 验证编译**
  ```bash
  cd /home/ai/dev/sqlrustgo && cargo check -p sqlrustgo-executor 2>&1
  ```
  Expected: 编译成功（只是空模块）

- [ ] **Step 5: 提交**
  ```bash
  git add crates/executor/src/stored_proc/
  git commit -m "feat(executor): scaffold stored_proc module structure"
  ```

---

### Task 2: 提取 context.rs (ProcedureContext, Cursor)

**Files:**
- Modify: `crates/executor/src/stored_proc/context.rs` (添加 ProcedureContext, Cursor, StoredProcError)
- Modify: `crates/executor/src/stored_proc/mod.rs` (导出)

- [ ] **Step 1: 从 stored_proc.rs 提取 StoredProcError, ProcedureContext, Cursor**
  提取行: 13-400 (约)
  ```rust
  // context.rs
  use std::collections::HashMap;
  use sqlrustgo_types::Value;
  use sqlrustgo_catalog::HandlerCondition;
  use sqlrustgo_catalog::StoredProcStatement;

  /// Stored procedure execution error
  #[derive(Debug, Clone)]
  pub struct StoredProcError { ... }

  /// Exception handler registered by DECLARE HANDLER
  #[derive(Debug, Clone)]
  pub struct ExceptionHandler { ... }

  /// Cursor state for stored procedure cursors
  #[derive(Debug, Clone)]
  struct Cursor { ... }

  /// Session-level variables for stored procedure execution
  #[derive(Debug, Clone, Default)]
  pub struct ProcedureContext { ... }

  impl ProcedureContext {
      // 所有方法...
  }
  ```

- [ ] **Step 2: 更新 mod.rs 导出**
  ```rust
  pub use context::{ProcedureContext, StoredProcError, ExceptionHandler};
  ```

- [ ] **Step 3: 验证编译**
  ```bash
  cargo check -p sqlrustgo-executor 2>&1
  ```

- [ ] **Step 4: 运行 context 相关测试**
  ```bash
  cargo test -p sqlrustgo-executor procedure_context 2>&1
  ```

- [ ] **Step 5: 提交**
  ```bash
  git add crates/executor/src/stored_proc/
  git commit -m "feat(executor): extract ProcedureContext to context.rs"
  ```

---

### Task 3: 提取 handler.rs

**Files:**
- Modify: `crates/executor/src/stored_proc/handler.rs` (补充完整内容)
- Modify: `crates/executor/src/stored_proc/mod.rs`

- [ ] **Step 1: 确认 handler.rs 已通过 context.rs 提取完成**
  - ExceptionHandler 已在 context.rs 中

- [ ] **Step 2: 更新 mod.rs 导出（如需要）**

- [ ] **Step 3: 验证编译**
  ```bash
  cargo check -p sqlrustgo-executor 2>&1
  ```

- [ ] **Step 4: 提交**
  ```bash
  git commit -m "feat(executor): extract handler module"
  ```

---

### Task 4: 提取 expression.rs

**Files:**
- Modify: `crates/executor/src/stored_proc/expression.rs`
- Modify: `crates/executor/src/stored_proc/mod.rs`

- [ ] **Step 1: 从 stored_proc.rs 提取表达式相关函数**
  提取函数:
  - `expression_to_value` (行 1411-1655, 244行)
  - `evaluate_row_expression` (行 1656-1882, 226行)
  - `evaluate_expression_with_row` (行 2283-2353)
  - `evaluate_expression_with_binding` (行 2431-2504)
  - `bind_row_to_context` (行 2504-2519)
  - `value_is_true` (行 2519-2528)
  - `expression_to_value_with_context` (行 2528-2595)
  - `evaluate_binary_op` (行 2819-2979)
  - `like_match` (行 2979-3065)
  - `between_match` (行 3066-3074)
  - `regexp_match` (行 3075-3086)
  - `ge_values` (行 3087-3096)
  - `le_values` (行 3097-3106)
  - `evaluate_condition` (行 3107-3128)
  - `evaluate_expression` (行 3129-3164)
  - `expand_variable` (行 3165-3178)
  - `expand_variables_in_sql` (行 3179-3205)
  - `escape_sql_value` (行 3206-3222)
  - `evaluate_constant` (行 3223-3255)
  - `compare_values` (行 3256-3274)
  - `partial_cmp` (行 3275-3288)
  - `arithmetic_op` (行 3289-3318)

- [ ] **Step 2: 更新 mod.rs**
  ```rust
  pub mod expression;
  pub use expression::*;
  ```

- [ ] **Step 3: 验证编译**
  ```bash
  cargo check -p sqlrustgo-executor 2>&1
  ```

- [ ] **Step 4: 提交**
  ```bash
  git commit -m "feat(executor): extract expression.rs module"
  ```

---

### Task 5: 提取 cte.rs

**Files:**
- Modify: `crates/executor/src/stored_proc/cte.rs`
- Modify: `crates/executor/src/stored_proc/mod.rs`

- [ ] **Step 1: 从 stored_proc.rs 提取 CTE 相关函数**
  提取函数:
  - `execute_cte_subquery` (行 1909-2049, ~140行)
  - `execute_recursive_cte` (行 2049-2095, ~46行)
  - `extract_select_columns` (行 2095-2130)
  - `execute_cte_anchor` (行 2130-2206)
  - `execute_cte_recursive` (行 2206-2283)
  - `execute_with_cte` (主入口, 行 471-502)

- [ ] **Step 2: 更新 mod.rs**
  ```rust
  pub mod cte;
  pub use cte::*;
  ```

- [ ] **Step 3: 验证编译**
  ```bash
  cargo check -p sqlrustgo-executor 2>&1
  ```

- [ ] **Step 4: 提交**
  ```bash
  git commit -m "feat(executor): extract cte.rs module"
  ```

---

### Task 6: 提取 execution.rs

**Files:**
- Modify: `crates/executor/src/stored_proc/execution.rs`
- Modify: `crates/executor/src/stored_proc/mod.rs`

- [ ] **Step 1: 从 stored_proc.rs 提取执行相关函数**
  提取函数:
  - `execute_call` (行 417-471)
  - `execute_body` (行 502-558)
  - `execute_statement` (行 558-872, ~314行)
  - `execute_sql` (行 872-910)
  - `execute_statement_storage` (行 910-1411)
  - `execute_subquery` (行 1882-1909)
  - `extract_column_binding` (行 2373-2431)
  - `extract_identifiers_from_expr` (行 2373-2431)
  - `validate_foreign_keys` (行 2595-2660)
  - `validate_unique_constraints` (行 2660-2723)
  - `validate_primary_key` (行 2723-2819)

- [ ] **Step 2: 更新 mod.rs**
  ```rust
  pub mod execution;
  pub use execution::*;
  ```

- [ ] **Step 3: 验证编译**
  ```bash
  cargo check -p sqlrustgo-executor 2>&1
  ```

- [ ] **Step 4: 提交**
  ```bash
  git commit -m "feat(executor): extract execution.rs module"
  ```

---

### Task 7: 重构 mod.rs 为真正的入口

**Files:**
- Modify: `crates/executor/src/stored_proc/mod.rs`

- [ ] **Step 1: 添加 StoredProcExecutor 主结构**
  ```rust
  pub struct StoredProcExecutor {
      catalog: Arc<Catalog>,
      storage: Arc<RwLock<dyn StorageEngine>>,
  }
  ```

- [ ] **Step 2: 导入所有子模块并重新导出**
  ```rust
  pub use context::{ProcedureContext, StoredProcError, ExceptionHandler};
  pub use expression::*;
  pub use cte::*;
  pub use execution::*;
  ```

- [ ] **Step 3: 验证编译**
  ```bash
  cargo check -p sqlrustgo-executor 2>&1
  ```

- [ ] **Step 4: 提交**
  ```bash
  git commit -m "feat(executor): finalize stored_proc module structure"
  ```

---

### Task 8: 验证测试和覆盖率

**Files:**
- Modify: `crates/executor/tests/patch_stored_proc_coverage.rs` (如需要调整 import)

- [ ] **Step 1: 运行所有 stored_proc 相关测试**
  ```bash
  cargo test -p sqlrustgo-executor stored_proc 2>&1
  ```

- [ ] **Step 2: 检查覆盖率**
  ```bash
  cargo llvm-cov --all-features 2>&1 | grep -A5 "executor"
  ```

- [ ] **Step 3: 验证格式化**
  ```bash
  cargo fmt --check 2>&1
  ```

- [ ] **Step 4: 验证 clippy**
  ```bash
  cargo clippy -p sqlrustgo-executor --all-features -- -D warnings 2>&1
  ```

- [ ] **Step 5: 提交**
  ```bash
  git commit -m "test(executor): verify stored_proc module tests pass"
  ```

---

## 验收标准

1. `cargo check -p sqlrustgo-executor` 编译通过
2. `cargo test -p sqlrustgo-executor` 所有测试通过
3. `cargo fmt --check` 通过
4. `cargo clippy -p sqlrustgo-executor --all-features -- -D warnings` 零警告
5. executor crate 覆盖率目标: Alpha 阶段 75%

---

## 风险缓解

| 风险 | 缓解措施 |
|------|----------|
| 编译失败 | 每步完成后立即验证 |
| 测试失败 | 保持 import 路径兼容 |
| 覆盖率下降 | 分步提交，便于回滚 |
