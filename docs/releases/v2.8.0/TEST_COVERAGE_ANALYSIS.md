# SQLRustGo 测试覆盖全面分析报告

> **版本**: v2.8.0
> **日期**: 2026-04-23
> **状态**: 进行中

---

## 一、执行摘要

本报告对 SQLRustGo 的测试覆盖率进行了全面的白盒分析，涵盖 Parser、Planner、Optimizer 和 Storage 四个核心模块。

### 当前测试状态

| Crate | 测试数 | 行覆盖率(估) | 状态 |
|-------|--------|--------------|------|
| sqlrustgo-parser | 67 passed | ~55% | 需增强 |
| sqlrustgo-planner | 81 passed | ~45% | 有 bug |
| sqlrustgo-optimizer | 233 passed | ~60% | 占位符 |
| sqlrustgo-storage | 159 passed | ~70% | 基本良好 |
| **总计** | **540 passed** | **~55%** | **中等** |

### 目标
- 短期目标: 70% 覆盖率
- 长期目标: 85% 覆盖率

---

## 二、Parser 覆盖率分析

### 2.1 已覆盖 SQL 语法

| 类别 | 覆盖的语法 |
|------|------------|
| SELECT | 列、`*`、限定名 (table.column)、WHERE |
| JOIN | INNER、LEFT、RIGHT、FULL (OUTER) |
| DML | INSERT、UPDATE、DELETE + WHERE |
| DDL | CREATE/DROP TABLE、INDEX、TRIGGER |
| AGGREGATE | COUNT、SUM、AVG、MIN、MAX + GROUP BY + HAVING |
| CTE | WITH、WITH RECURSIVE |
| TRANSACTION | BEGIN、COMMIT、ROLLBACK、START TRANSACTION |

### 2.2 覆盖缺口

| 缺口 | 严重度 | 文件位置 | 说明 |
|------|--------|----------|------|
| **ORDER BY** | P0 | parser.rs | Token 存在但 `parse_statement` 未实现 |
| **LIMIT/OFFSET** | P0 | parser.rs | Token 存在但未解析 |
| **UNION/INTERSECT/EXCEPT** | P1 | parser.rs | `UnionStatement` 类型已定义但未被调用 |
| **DISTINCT** | P1 | parser.rs | 只对聚合实现，`SELECT DISTINCT` 未实现 |
| **CASE/WHEN/ELSE** | P1 | token.rs | Token 不存在 |
| **IS NULL/IS NOT NULL** | P1 | expression.rs | Expression 变体不存在 |
| **BETWEEN** | P2 | token.rs | Token 存在但未解析 |
| **WINDOW functions** | P2 | token.rs | Token 存在但未使用 |

### 2.3 被忽略的测试

```
test_parse_create_with_table_constraint_fk     - Table-level FOREIGN KEY
test_parse_create_procedure_with_in_param       - IN 参数模式
test_parse_create_procedure_with_multiple_params - 多参数
test_parse_create_procedure_with_out_param      - OUT 参数模式
test_parse_call_with_null                       - CALL 中 NULL
```

---

## 三、Planner 覆盖率分析

### 3.1 已覆盖功能

- 基础 SELECT 计划生成
- WHERE 条件计划
- JOIN 计划 (Hash Join、Sort Merge Join)
- GROUP BY 计划
- INSERT/UPDATE/DELETE 计划

### 3.2 发现的 Bug

#### Bug 1: UNION 右半边被忽略 (P0)

**文件**: `planner/src/planner.rs:210-212`

```rust
LogicalPlan::Union { left, .. } => {
    self.create_physical_plan_internal(left)  // right 被丢弃!
}
```

**影响**: `SELECT ... UNION SELECT ...` 只执行左半边查询。

#### Bug 2: Join Schema 计算错误 (P1)

**文件**: `planner/src/planner.rs:153`

```rust
let schema = Schema::new(vec![]);  // 错误：应为左右子节点的组合
```

**影响**: JOIN 操作的 schema 为空，导致列引用失败。

### 3.3 占位符优化规则

**文件**: `optimizer/src/rules.rs`

| 规则 | 行号 | 状态 |
|------|------|------|
| PredicatePushdown | 67 | `apply()` 返回 false |
| ProjectionPruning | 95 | `apply()` 返回 false |
| ConstantFolding | 122 | `apply()` 返回 false |

这些规则已注册但未实现任何优化逻辑。

### 3.4 覆盖缺口

| 缺口 | 严重度 |
|------|--------|
| **UNION 物理计划** | P0 - 右孩子被忽略 |
| **CreateTrigger/CreateProcedure/Call** | P1 - 无测试 |
| **has_storage()/select_scan()** | P1 - 无索引测试 |
| **DefaultPlanner::with_storage()** | P1 - 无测试 |
| **optimizer iterations** | P2 - 无规则效果测试 |

---

## 四、Storage 覆盖率分析

### 4.1 已充分覆盖

| 模块 | 测试数 | 覆盖率 |
|------|--------|--------|
| B+ Tree | ~73 | ~85% |
| WAL | 36 | ~80% |
| Buffer Pool | 9 | ~70% |
| FileStorage | 23 | ~65% |

### 4.2 覆盖缺口

| 缺口 | 文件:行 | 说明 |
|------|---------|------|
| **WalArchiveManager::archive_wal()** | backup.rs | 未测试 |
| **WalArchiveManager::recover_from_archive()** | backup.rs | 未实现，返回 Err |
| **FileStorage::create_index()** | file_storage.rs:330 | "table not found" 分支未测试 |
| **FileStorage::drop_table()** | file_storage.rs | I/O 错误未测试 |
| **FileStorage::load_table()** | file_storage.rs | JSON 解析错误未测试 |
| **WalEntry::from_bytes** | wal.rs:173,183 | 边界条件未测试 |
| **clock_replacer** | clock_replacer.rs:248-357 | 测试存在但未被发现 |

### 4.3 未测试的错误路径

```rust
// WalEntry::from_bytes
- bytes.len() < 34 (line 116-117) - 已测试
- offset + 4 > bytes.len() (line 173) - 未测试
- offset + data_len <= bytes.len() (line 183) - 未测试边界

// WalArchiveManager
- archive_wal() 文件年龄计算 - 未测试
- compress_file() - 未测试
- recover_from_archive() 压缩路径 - 未测试

// FileStorage
- create_index() table not found (line 330) - 未测试
- drop_table() fs::remove_file 错误 - 未测试
- load_table() serde 错误 - 未测试
```

---

## 五、覆盖率改进路线图

```
当前状态: ~55% 覆盖率

Phase 1 (快速提升 1-2 周):
  +20% ORDER BY parser 测试
  +10% UNION bug 修复 + 测试
  +5%  FileStorage 错误路径测试

Phase 2 (中等工作量 2-4 周):
  +8%  UNION/INTERSECT/EXCEPT 完整实现
  +5%  IS NULL/DISTINCT 实现
  +5%  LIMIT/OFFSET 实现

Phase 3 (完善 4-8 周):
  +5%  Window functions
  +5%  CASE expressions
  +5%  优化器规则实现
```

目标: 从 55% 提升到 85%

---

## 六、具体任务清单

| # | 任务 | 优先级 | 预计时间 | 状态 |
|---|------|--------|----------|------|
| 1 | 修复 UNION bug (右孩子被忽略) | P0 | 30min | 待开始 |
| 2 | 添加 ORDER BY parser 测试 | P0 | 1h | 待开始 |
| 3 | 添加 LIMIT/OFFSET parser 测试 | P0 | 1h | 待开始 |
| 4 | 添加 FileStorage 错误路径测试 | P1 | 1h | 待开始 |
| 5 | 添加 UNION 端到端测试 | P1 | 1h | 待开始 |
| 6 | 实现 ORDER BY parser | P0 | 2h | 待开始 |
| 7 | 实现 LIMIT/OFFSET parser | P0 | 2h | 待开始 |
| 8 | 完善优化器占位规则 | P2 | 4h | 待开始 |

---

## 七、测试方法论

### 7.1 语句覆盖 (Statement Coverage)
确保每个可执行语句都被至少一个测试执行。

### 7.2 分支覆盖 (Branch Coverage)
确保每个条件分支 (if/else、match) 的每个方向都被测试。

### 7.3 条件覆盖 (Condition Coverage)
确保每个布尔表达式中的每个子表达式都被评估为 true 和 false。

### 7.4 路径覆盖 (Path Coverage)
确保每个可能的执行路径组合都被测试（对于有循环的代码较难实现）。

---

## 八、附录

### A. 测试命令

```bash
# 运行所有测试
cargo test --workspace

# 运行核心 crate 测试
cargo test -p sqlrustgo-parser
cargo test -p sqlrustgo-planner
cargo test -p sqlrustgo-storage

# 带覆盖率
cargo tarpaulin --out html

# 运行被忽略的测试
cargo test -- --ignored
```

### B. 覆盖率工具

- **cargo-tarpaulin**: Rust 专用覆盖率工具
- **llvm-profdata**: 处理 .profdata 文件
- **grcov**: 覆盖率报告聚合

### C. 参考资料

- [Rust 测试文档](https://doc.rust-lang.org/rust-by-example/testing.html)
- [cargo-tarpaulin 文档](https://github.com/xd009642/tarpaulin)
- [Peter's Rust 测试模式](https://www.youtube.com/watch?v=2EcFtB9bwLA)

---

## 版本历史

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-04-23 | 1.0 | 初始版本 |
