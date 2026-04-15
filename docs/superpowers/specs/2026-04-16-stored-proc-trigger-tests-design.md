# 存储过程增强与触发器测试设计

> 日期: 2026-04-16
> Issue: #1434
> 状态: Approved

## 1. 概述

为 v2.5.0 补充存储过程和触发器的测试用例，实现缺失的 CASE/REPEAT 功能。

## 2. 存储过程增强

### 2.1 新增语句类型

在 `StoredProcStatement` 枚举中添加：

```rust
/// CASE case_value WHEN value1 THEN result1 ... ELSE result END
Case {
    case_value: Option<String>,
    when_clauses: Vec<(String, String)>,
    else_result: Option<String>,
}

/// CASE WHEN condition1 THEN result1 ... ELSE result END
CaseWhen {
    when_clauses: Vec<(String, String)>,
    else_result: Option<String>,
}

/// REPEAT statements UNTIL condition END REPEAT
Repeat {
    body: Vec<StoredProcStatement>,
    condition: String,
}
```

### 2.2 实现要求

**CASE 语句**
- 支持 `CASE expr WHEN value THEN result ... END`
- 支持 `CASE WHEN condition THEN result ... END`
- 支持嵌套 CASE
- 支持 ELSE 子句

**CASE WHEN 表达式**
- 条件表达式求值
- 返回匹配条件的结果

**REPEAT 循环**
- 先执行 body，再判断 UNTIL 条件
- 条件为真时退出循环
- 支持 LEAVE 退出

## 3. 触发器测试

### 3.1 新建文件

`crates/executor/tests/test_trigger.rs`

### 3.2 测试覆盖

| 测试 | 说明 |
|------|------|
| `test_before_insert_trigger` | BEFORE INSERT 触发器 |
| `test_after_insert_trigger` | AFTER INSERT 触发器 |
| `test_before_update_trigger` | BEFORE UPDATE 触发器 |
| `test_after_update_trigger` | AFTER UPDATE 触发器 |
| `test_before_delete_trigger` | BEFORE DELETE 触发器 |
| `test_after_delete_trigger` | AFTER DELETE 触发器 |
| `test_trigger_modifies_new_row` | 触发器修改 NEW row 值 |
| `test_multiple_triggers_order` | 多触发器执行顺序 |
| `test_trigger_transaction_handling` | 触发器中的事务处理 |

### 3.3 触发器接口

```rust
// TriggerExecutor<S>
pub fn execute_before_insert(&self, table: &str, new_row: &Record) -> SqlResult<Record>
pub fn execute_after_insert(&self, table: &str, new_row: &Record) -> SqlResult<()>
pub fn execute_before_update(&self, table: &str, old_row: &Record, new_row: &Record) -> SqlResult<Record>
pub fn execute_after_update(&self, table: &str, old_row: &Record, new_row: &Record) -> SqlResult<()>
pub fn execute_before_delete(&self, table: &str, old_row: &Record) -> SqlResult<()>
pub fn execute_after_delete(&self, table: &str, old_row: &Record) -> SqlResult<()>
```

## 4. 文件结构

```
crates/
├── catalog/src/stored_proc.rs     # 添加 Case, CaseWhen, Repeat 变体
├── executor/src/
│   ├── stored_proc.rs            # 实现 execute_case, execute_case_when, execute_repeat
│   └── trigger.rs              # 已有实现
└── executor/tests/
    ├── test_stored_proc.rs      # 添加 CASE/REPEAT 测试
    └── test_trigger.rs          # 新建触发器测试
```

## 5. 实现计划

### Task 1: 存储过程 - 添加 CASE 语句类型
- 修改 `crates/catalog/src/stored_proc.rs`
- 添加 `Case` 和 `CaseWhen` 变体到 `StoredProcStatement`

### Task 2: 存储过程 - 实现 CASE 执行
- 修改 `crates/executor/src/stored_proc.rs`
- 实现 `execute_case` 和 `execute_case_when` 方法

### Task 3: 存储过程 - 添加 REPEAT 语句类型和执行
- 修改 `crates/catalog/src/stored_proc.rs`
- 修改 `crates/executor/src/stored_proc.rs`
- 实现 `execute_repeat` 方法

### Task 4: 存储过程 - 添加测试
- 修改 `crates/executor/tests/test_stored_proc.rs`
- 添加 CASE 和 REPEAT 测试用例

### Task 5: 触发器测试
- 创建 `crates/executor/tests/test_trigger.rs`
- 实现所有触发器测试用例

## 6. 验收标准

| 功能 | 验收标准 |
|------|---------|
| CASE 语句 | `CASE x WHEN 1 THEN 'one' WHEN 2 THEN 'two' ELSE 'other' END` 正确执行 |
| CASE WHEN | `CASE WHEN x > 10 THEN 'big' ELSE 'small' END` 正确执行 |
| REPEAT 循环 | `REPEAT SET x = x + 1; UNTIL x >= 10 END REPEAT` 循环 10 次 |
| 触发器 | BEFORE/AFTER + INSERT/UPDATE/DELETE 所有组合测试通过 |
