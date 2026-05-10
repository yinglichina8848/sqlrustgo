# 存储过程执行链路

> Stored Procedure: CREATE PROCEDURE, CALL, 控制流, 异常处理

## 1. 存储过程概述

### 1.1 存储过程结构

```
┌─────────────────────────────────────────────────────────────┐
│                 存储过程结构                                  │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  CREATE PROCEDURE procedure_name(                            │
│      IN param1 INT,           -- 输入参数                   │
│      OUT param2 VARCHAR(100), -- 输出参数                   │
│      INOUT param3 BIGINT      -- 输入输出参数               │
│  )                                                          │
│  BEGIN                                                      │
│      -- 局部变量                                            │
│      DECLARE local_var INT DEFAULT 0;                       │
│                                                              │
│      -- 条件处理                                            │
│      DECLARE CONTINUE HANDLER FOR SQLEXCEPTION              │
│          SET @error_code = 1;                              │
│                                                              │
│      -- 执行逻辑                                            │
│      SET local_var = param1 + 100;                         │
│      SELECT * FROM t WHERE id = local_var;                 │
│  END                                                        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 支持的控制流

```
┌─────────────────────────────────────────────────────────────┐
│                    控制流支持                                 │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  条件:                                                     │
│  ─────────────────────────────────────────────────────────  │
│  IF condition THEN                                          │
│      statements;                                          │
│  [ELSEIF condition THEN                                   │
│      statements;]                                          │
│  [ELSE                                                   │
│      statements;]                                          │
│  END IF;                                                  │
│                                                              │
│  循环:                                                     │
│  ─────────────────────────────────────────────────────────  │
│  WHILE condition DO                                        │
│      statements;                                          │
│  END WHILE;                                               │
│                                                              │
│  REPEAT                                                    │
│      statements;                                          │
│  UNTIL condition END REPEAT;                              │
│                                                              │
│  LOOP                                                      │
│      statements;                                          │
│  END LOOP;                                                │
│                                                              │
│  流程控制:                                                 │
│  ─────────────────────────────────────────────────────────  │
│  LEAVE label;      -- 退出带标签的块                       │
│  ITERATE label;   -- 继续下一次循环                         │
│  RETURN value;     -- 返回值                               │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 2. 存储过程执行架构

### 2.1 调用流程

```
┌─────────────────────────────────────────────────────────────┐
│                 存储过程调用流程                             │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  CALL procedure_name(1, @output);                          │
│      │                                                        │
│      ▼                                                        │
│  ┌─────────────────────────────────────────────┐           │
│  │              Parser                           │           │
│  │  CALL Statement {                            │           │
│  │    procedure: "procedure_name",             │           │
│  │    args: [1, @output]                      │           │
│  │  }                                          │           │
│  └─────────────────────────────────────────────┘           │
│      │                                                        │
│      ▼                                                        │
│  ┌─────────────────────────────────────────────┐           │
│  │              Catalog                         │           │
│  │  - 查找存储过程定义                         │           │
│  │  - 验证参数类型                             │           │
│  │  - 加载过程体                               │           │
│  └─────────────────────────────────────────────┘           │
│      │                                                        │
│      ▼                                                        │
│  ┌─────────────────────────────────────────────┐           │
│  │              Executor                        │           │
│  │  ProcedureExecutor {                        │           │
│  │    - 创建 ProcedureContext                   │           │
│  │    - 绑定参数                               │           │
│  │    - 执行过程体                             │           │
│  │    - 返回结果                             │           │
│  │  }                                         │           │
│  └─────────────────────────────────────────────┘           │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 3. 存储过程上下文

### 3.1 ProcedureContext 结构

```rust
// crates/executor/src/stored_proc.rs

pub struct ProcedureContext {
    /// 局部变量 (不含 @)
    local_variables: HashMap<String, Value>,
    /// 会话级变量 (不含 @)
    session_variables: HashMap<String, Value>,
    /// 返回值
    return_value: Option<Value>,
    /// 是否退出当前循环
    leave: bool,
    /// 是否继续下一次循环
    iterate: bool,
    /// 当前标签
    current_label: Option<String>,
    /// 标签栈 (嵌套块)
    label_stack: Vec<String>,
    /// BEGIN/END 块作用域栈
    scope_stack: Vec<HashMap<String, Value>>,
    /// 异常处理器栈
    handler_stack: Vec<ExceptionHandler>,
    /// 是否正在处理异常
    exception_handling: bool,
    /// 当前异常 (RESIGNAL 用)
    current_exception: Option<StoredProcError>,
    /// 游标
    cursors: HashMap<String, Cursor>,
    /// CTE 结果
    cte_tables: HashMap<String, Vec<Vec<Value>>>,
}
```

### 3.2 异常处理

```rust
/// 异常处理器
pub struct ExceptionHandler {
    condition: HandlerCondition,
    body: Vec<StoredProcStatement>,
}

/// 处理器条件
pub enum HandlerCondition {
    NotFound,        -- 游标未找到
    Sqlexception,    -- SQL 异常
    Sqlwarning,       -- SQL 警告
}
```

## 4. 存储过程状态机

```
                  ┌──────────────────┐
                  │     INITIAL     │
                  └────────┬─────────┘
                           │ CALL received
                           ▼
                  ┌──────────────────┐
                  │   LOAD_PROC      │
                  └────────┬─────────┘
                           │ load definition
                           ▼
                  ┌──────────────────┐
                  │  BIND_PARAMS    │
                  └────────┬─────────┘
                           │ bind IN/OUT/INOUT
                           ▼
                  ┌──────────────────┐
                  │   INIT_CONTEXT   │
                  └────────┬─────────┘
                           │ create ProcedureContext
                           ▼
                  ┌──────────────────┐
                  │   EXEC_STMTS    │
                  └────────┬─────────┘
                           │ execute statements
                           ▼
                  ┌──────────────────┐
                  │   HANDLER_CHECK  │
                  └────────┬─────────┘
                           │ exception occurred?
                           │
            ┌─────────────┼─────────────┐
            │             │             │
            ▼             │             ▼
     ┌──────────┐        │      ┌──────────┐
     │ EXEC_HNDLR│        │      │  RETURN  │
     └──────────┘        │      └──────────┘
            │             │
            │  continue?
            │
            └────────────┘
```

## 5. 游标支持

### 5.1 游标生命周期

```
┌─────────────────────────────────────────────────────────────┐
│                    游标生命周期                               │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  DECLARE cursor_name CURSOR FOR                             │
│      SELECT ... FROM ... WHERE ...;                         │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  声明 (DECLARE)                                     │   │
│  │  ─────────────────────────────────────────────────── │   │
│  │  定义游标和关联的查询                               │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                   │
│                          ▼                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  打开 (OPEN)                                        │   │
│  │  ─────────────────────────────────────────────────── │   │
│  │  执行查询，结果集存入游标                           │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                   │
│                          ▼                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  取数 (FETCH)                                       │   │
│  │  ─────────────────────────────────────────────────── │   │
│  │  从结果集取一行到变量                               │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                   │
│                          ▼                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  关闭 (CLOSE)                                       │   │
│  │  ─────────────────────────────────────────────────── │   │
│  │  释放结果集                                         │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 游标实现

```rust
struct Cursor {
    name: String,
    query: String,
    records: Vec<Vec<Value>>,  // 预编译的结果集
    position: usize,
    is_open: bool,
}

impl Cursor {
    fn fetch(&mut self) -> Option<Vec<Value>> {
        if !self.is_open || self.position >= self.records.len() {
            return None;
        }
        let row = self.records[self.position].clone();
        self.position += 1;
        Some(row)
    }
}
```

## 6. 存储过程与函数

### 6.1 存储过程 vs 存储函数

| 特性 | 存储过程 | 存储函数 |
|------|----------|----------|
| RETURN | 无 (用 OUT 参数) | 有 (必须) |
| 调用方式 | CALL | SELECT 或 FROM |
| SQL 中使用 | 独立调用 | 表达式中 |
| 事务控制 | 可开始/提交/回滚 | 不可 |


## 7. 测试计划

### 7.1 基本测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| PROC-T01 | 简单 CALL | 执行成功 |
| PROC-T02 | IN 参数 | 值传入过程 |
| PROC-T03 | OUT 参数 | 值返回调用者 |
| PROC-T04 | INOUT 参数 | 双向传递 |

### 7.2 控制流测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| PROC-T10 | IF THEN ELSE | 条件分支 |
| PROC-T11 | WHILE 循环 | 循环执行 |
| PROC-T12 | LEAVE 语句 | 退出块 |
| PROC-T13 | ITERATE 语句 | 继续循环 |

### 7.3 异常处理测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| PROC-T20 | CONTINUE HANDLER | 异常后继续 |
| PROC-T21 | EXIT HANDLER | 异常后退出 |
| PROC-T22 | 多个 HANDLER | 正确匹配条件 |
| PROC-T23 | RESIGNAL | 重新抛出异常 |

### 7.4 游标测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| PROC-T30 | 声明游标 | 定义成功 |
| PROC-T31 | OPEN 游标 | 查询执行 |
| PROC-T32 | FETCH | 获取一行 |
| PROC-T33 | CLOSE 游标 | 释放资源 |

## 8. 覆盖率差距分析

### 8.1 当前覆盖率

| 组件 | 行覆盖率 | 说明 |
|------|----------|------|
| stored_proc.rs | ~50% | 基础控制流 |
| 游标支持 | ~40% | 声明/打开/取数/关闭 |
| 异常处理 | ~45% | CONTINUE/EXIT HANDLER |
| RETURN 函数 | ~50% | 函数支持 |

### 8.2 差距原因

1. **游标 FOR 循环**: 未实现简化的 FOR 循环语法
2. **异常条件值**: 未实现获取异常详情的 SQLSTATE
3. **动态 SQL**: 未实现 PREPARE/EXECUTE
4. **存储函数限制**: 某些上下文中不可用

### 8.3 提升计划

| 阶段 | 任务 | 目标覆盖率 |
|------|------|-----------|
| v3.1.0 | 完善游标 FOR 循环 | 70% |
| v3.1.0 | 动态 SQL 支持 | 60% |
| v3.2.0 | 完整异常条件 | 75% |

## 9. 核心文件索引

| 文件 | 行数 | 说明 |
|------|------|------|
| `crates/executor/src/stored_proc.rs` | ~3380 | 存储过程执行器 |
| `crates/catalog/src/stored_proc.rs` | ~500 | 存储过程存储 |
| `tests/stored_proc_catalog_test.rs` | ~300 | 测试 |

## 10. 相关文档

| 文档 | 说明 |
|------|------|
| [TRIGGER_EXECUTION.md](./TRIGGER_EXECUTION.md) | 触发器执行 |
| [SUBQUERY_EXECUTION.md](../query/SUBQUERY_EXECUTION.md) | 子查询执行 |
