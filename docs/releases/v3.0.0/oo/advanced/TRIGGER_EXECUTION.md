# 触发器执行链路

> Trigger: BEFORE/AFTER, INSERT/UPDATE/DELETE, ROW/STATEMENT

## 1. 触发器概述

### 1.1 触发器类型

```
┌─────────────────────────────────────────────────────────────┐
│                    触发器分类                               │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  按时间 (Timing):                                           │
│  ─────────────────────────────────────────────────────────  │
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │   BEFORE       │  │   AFTER        │                   │
│  │                 │  │                 │                   │
│  │  操作执行前     │  │  操作执行后     │                   │
│  │  触发器         │  │  触发器         │                   │
│  └─────────────────┘  └─────────────────┘                   │
│                                                              │
│  按事件 (Event):                                            │
│  ─────────────────────────────────────────────────────────  │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐               │
│  │  INSERT   │  │  UPDATE   │  │  DELETE   │               │
│  └───────────┘  └───────────┘  └───────────┘               │
│                                                              │
│  按级别 (Level):                                            │
│  ─────────────────────────────────────────────────────────  │
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │   ROW          │  │  STATEMENT     │                   │
│  │                 │  │                 │                   │
│  │ 每行触发一次    │  │ 每语句触发一次  │                   │
│  └─────────────────┘  └─────────────────┘                   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 触发器组合

```
BEFORE INSERT ROW      - 插入行前触发
AFTER INSERT ROW        - 插入行后触发
BEFORE UPDATE ROW      - 更新行前触发
AFTER UPDATE ROW        - 更新行后触发
BEFORE DELETE ROW      - 删除行前触发
AFTER DELETE ROW        - 删除行后触发

BEFORE INSERT          - 插入语句前触发 (STATEMENT)
AFTER INSERT           - 插入语句后触发 (STATEMENT)
```

## 2. 触发器执行架构

### 2.1 执行流程

```
┌─────────────────────────────────────────────────────────────┐
│                 触发器执行流程                                │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  CREATE TRIGGER order_update                                │
│  BEFORE UPDATE ON orders                                     │
│  FOR EACH ROW                                               │
│  BEGIN                                                      │
│    NEW.total = OLD.quantity * NEW.price;                    │
│  END                                                        │
│      │                                                        │
│      ▼                                                        │
│  ┌─────────────────────────────────────────────┐           │
│  │              Parser                           │           │
│  │  CREATE TRIGGER Statement {                  │           │
│  │    name: "order_update",                     │           │
│  │    timing: BEFORE,                           │           │
│  │    event: UPDATE,                            │           │
│  │    level: ROW,                              │           │
│  │    table: "orders",                         │           │
│  │    body: [assignment, ...]                   │           │
│  │  }                                          │           │
│  └─────────────────────────────────────────────┘           │
│      │                                                        │
│      ▼                                                        │
│  ┌─────────────────────────────────────────────┐           │
│  │              Catalog                         │           │
│  │  - 验证触发器名称唯一                        │           │
│  │  - 保存到系统表 mysql.trigger               │           │
│  │  - 注册到表的触发器列表                      │           │
│  └─────────────────────────────────────────────┘           │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 触发器执行时序

```
UPDATE orders SET price = 150 WHERE id = 1
    │
    ▼
┌─────────────────────────────────────────────┐
│              DML 执行器                       │
│  ┌─────────────────────────────────────┐  │
│  │ FOR EACH ROW (old_row, new_row):   │  │
│  │                                     │  │
│  │   if BEFORE trigger exists:          │  │
│  │       execute BEFORE trigger         │  │
│  │       if trigger modifies row:       │  │
│  │           use trigger-modified row   │  │
│  │                                     │  │
│  │   execute DML operation             │  │
│  │                                     │  │
│  │   if AFTER trigger exists:           │  │
│  │       execute AFTER trigger         │  │
│  └─────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

## 3. 触发器数据结构

### 3.1 触发器定义

```rust
// crates/executor/src/trigger.rs

/// 触发器时间: BEFORE 或 AFTER
pub enum TriggerTiming {
    Before,
    After,
}

/// 触发器事件: INSERT, UPDATE, DELETE
pub enum TriggerEvent {
    Insert,
    Update,
    Delete,
}

/// 触发器类型 (时间 + 事件)
pub enum TriggerType {
    BeforeInsert,
    AfterInsert,
    BeforeUpdate,
    AfterUpdate,
    BeforeDelete,
    AfterDelete,
}

/// 触发器信息
pub struct TriggerInfo {
    pub name: String,
    pub table_name: String,
    pub timing: TriggerTiming,
    pub event: TriggerEvent,
    pub level: TriggerLevel,
    pub body: Vec<TriggerStmt>,
}

pub enum TriggerLevel {
    Row,
    Statement,
}
```

### 3.2 触发器执行上下文

```rust
/// 触发器执行时的上下文
pub struct TriggerContext {
    /// 触发该触发器的 DML 操作类型
    pub dml_type: DmlType,
    /// 旧行 (for UPDATE/DELETE)
    pub old_row: Option<Row>,
    /// 新行 (for INSERT/UPDATE)
    pub new_row: Option<Row>,
    /// 触发器是否修改了 new_row
    pub row_modified: bool,
}

/// 触发器执行结果
pub struct TriggerResult {
    /// 是否继续执行 DML
    pub proceed: bool,
    /// 修改后的新行
    pub modified_row: Option<Row>,
}
```

## 4. 触发器状态机

### 4.1 BEFORE 触发器状态机

```
                  ┌──────────────────┐
                  │    ROW_START    │
                  └────────┬─────────┘
                           │ BEFORE trigger exists?
                           ▼
                  ┌──────────────────┐
                  │  LOAD_OLD_ROW   │
                  └────────┬─────────┘
                           │ load OLD row
                           ▼
                  ┌──────────────────┐
                  │  LOAD_NEW_ROW   │
                  └────────┬─────────┘
                           │ load NEW row
                           ▼
                  ┌──────────────────┐
                  │  EXEC_TRIGGER   │
                  └────────┬─────────┘
                           │ execute trigger body
                           ▼
                  ┌──────────────────┐
                  │ ROW_MODIFIED?   │
                  └────────┬─────────┘
                           │
            ┌─────────────┼─────────────┐
            │             │             │
            ▼             │             ▼
     ┌──────────┐        │      ┌──────────┐
     │  USE     │        │      │  USE     │
     │ MODIFIED │        │      │ ORIGINAL │
     │  ROW     │        │      │  ROW     │
     └──────────┘        │      └──────────┘
                          │
                          ▼
                  ┌──────────────────┐
                  │    PROCEED      │
                  └──────────────────┘
```

### 4.2 AFTER 触发器状态机

```
                  ┌──────────────────┐
                  │    ROW_COMPLETE  │
                  └────────┬─────────┘
                           │ DML operation completed
                           ▼
                  ┌──────────────────┐
                  │  AFTER_TRIGGER  │
                  └────────┬─────────┘
                           │ execute AFTER trigger
                           ▼
                  ┌──────────────────┐
                  │    NEXT_ROW      │
                  └────────┬─────────┘
                           │ more rows?
                           │
            ┌─────────────┴─────────────┐
            │                           │
            ▼                           ▼
     ┌──────────┐              ┌──────────┐
     │   YES    │              │    NO    │
     └──────────┘              └──────────┘
          │                           │
          ▼                           ▼
     ┌──────────┐              ┌──────────┐
     │   BACK   │              │   DONE   │
     │   TO     │              └──────────┘
     │ ROW_START│
     └──────────┘
```

## 5. 触发器表达式

### 5.1 OLD 和 NEW 引用

```sql
-- OLD 引用: 更新/删除前的行
-- NEW 引用: 插入/更新后的行

-- UPDATE 触发器
CREATE TRIGGER update_check
BEFORE UPDATE ON orders
FOR EACH ROW
BEGIN
    -- OLD.quantity 是更新前的数量
    -- NEW.quantity 是更新后的数量
    IF NEW.quantity > 100 THEN
        SET NEW.total = OLD.total * 1.1;  -- 增加 10%
    END IF;
END;

-- INSERT 触发器
CREATE TRIGGER set_default
BEFORE INSERT ON orders
FOR EACH ROW
BEGIN
    IF NEW.discount IS NULL THEN
        SET NEW.discount = 0;  -- 默认折扣
    END IF;
END;

-- DELETE 触发器
CREATE TRIGGER log_delete
BEFORE DELETE ON orders
FOR EACH ROW
BEGIN
    INSERT INTO delete_log (id, deleted_at)
    VALUES (OLD.id, NOW());
END;
```

### 5.2 条件触发

```sql
-- WHEN 子句: 只在条件满足时触发
CREATE TRIGGER high_value_update
AFTER UPDATE ON orders
FOR EACH ROW
WHEN (OLD.total > 10000 AND NEW.total > OLD.total)
BEGIN
    INSERT INTO audit_log (action, table_name, row_id)
    VALUES ('HIGH_VALUE_UPDATE', 'orders', NEW.id);
END;
```

## 6. 触发器链

### 6.1 触发器链执行

```
┌─────────────────────────────────────────────────────────────┐
│                    触发器链                                   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  表: orders                                                 │
│  ─────────────────────────────────────────────────────────  │
│  触发器 1: validate_order (BEFORE INSERT)                 │
│  触发器 2: set_discount (BEFORE INSERT)                    │
│  触发器 3: log_insert (AFTER INSERT)                        │
│                                                              │
│  执行流程:                                                 │
│  ─────────────────────────────────────────────────────────  │
│  INSERT INTO orders ...                                    │
│       │                                                     │
│       ▼                                                     │
│  ┌─────────────────────────────────────────────┐           │
│  │  BEFORE INSERT 触发器链                      │           │
│  │  ─────────────────────────────────────────── │           │
│  │  1. validate_order: 验证数据               │           │
│  │     ↓ (通过)                               │           │
│  │  2. set_discount: 设置折扣                 │           │
│  │     ↓ (完成)                               │           │
│  └─────────────────────────────────────────────┘           │
│       │                                                     │
│       ▼                                                     │
│  ┌─────────────────────────────────────────────┐           │
│  │  执行 INSERT 操作                           │           │
│  └─────────────────────────────────────────────┘           │
│       │                                                     │
│       ▼                                                     │
│  ┌─────────────────────────────────────────────┐           │
│  │  AFTER INSERT 触发器链                       │           │
│  │  ─────────────────────────────────────────── │           │
│  │  3. log_insert: 记录日志                   │           │
│  │     ↓ (完成)                               │           │
│  └─────────────────────────────────────────────┘           │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 递归触发器检测

```rust
/// 检测递归触发器
fn detect_recursive_trigger(
    trigger: &TriggerInfo,
    chain: &[String],
) -> Result<(), TriggerError> {
    if chain.contains(&trigger.name) {
        return Err(TriggerError::RecursiveTrigger {
            trigger: trigger.name.clone(),
            chain: chain.clone(),
        });
    }

    // 检查是否超过最大嵌套深度
    if chain.len() >= MAX_TRIGGER_DEPTH {
        return Err(TriggerError::MaxDepthExceeded {
            depth: chain.len(),
        });
    }

    Ok(())
}
```

## 7. 触发器与事务

### 7.1 触发器在事务中

```
┌─────────────────────────────────────────────────────────────┐
│                    触发器与事务                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  BEGIN;                                                     │
│    INSERT INTO orders ...;  -- 触发 trigger1              │
│    UPDATE orders ...;    -- 触发 trigger2              │
│  COMMIT;                                                    │
│                                                              │
│  ─────────────────────────────────────────────────────────  │
│                                                              │
│  触发器在事务中执行:                                        │
│  ─────────────────────────────────────────────────────────  │
│  1. 触发器执行成功 → 事务继续                               │
│  2. 触发器执行失败 → 整个事务回滚                          │
│  3. BEFORE 触发器修改了行 → 修改后的行用于 DML            │
│  4. AFTER 触发器失败 → 已执行的 DML 不会回滚              │
│                                                              │
│  注意: AFTER 触发器失败不会导致 DML 回滚!                  │
│  如果需要原子性，应该使用 BEFORE 触发器或存储过程           │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 8. 测试计划

### 8.1 基本触发器测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| TRIG-T01 | BEFORE INSERT 触发器 | 插入前执行 |
| TRIG-T02 | AFTER INSERT 触发器 | 插入后执行 |
| TRIG-T03 | BEFORE UPDATE 触发器 | 更新前执行 |
| TRIG-T04 | AFTER UPDATE 触发器 | 更新后执行 |
| TRIG-T05 | BEFORE DELETE 触发器 | 删除前执行 |
| TRIG-T06 | AFTER DELETE 触发器 | 删除后执行 |

### 8.2 行级别触发器测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| TRIG-T10 | OLD/NEW 引用 UPDATE | 正确获取旧值和新值 |
| TRIG-T11 | OLD 引用 INSERT | 返回 NULL |
| TRIG-T12 | NEW 引用 DELETE | 返回 NULL |
| TRIG-T13 | 修改 NEW 行 | DML 使用修改后的值 |

### 8.3 触发器链测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| TRIG-T20 | 多触发器顺序 | 按创建顺序执行 |
| TRIG-T21 | BEFORE/AFTER 混合 | 正确阶段执行 |
| TRIG-T22 | 递归触发器检测 | 报错防止无限循环 |

### 8.4 事务测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| TRIG-T30 | 触发器失败事务回滚 | 整个事务回滚 |
| TRIG-T31 | AFTER 触发器失败 | DML 已提交 |
| TRIG-T32 | 嵌套事务 | 正确处理 savepoint |

## 9. 覆盖率差距分析

### 9.1 当前覆盖率

| 组件 | 行覆盖率 | 说明 |
|------|----------|------|
| trigger.rs | ~55% | 基础结构 |
| trigger_eval/ | ~50% | 表达式执行 |
| 触发器与 DML 集成 | ~45% | 执行时机 |

### 9.2 差距原因

1. **WHEN 子句**: 未实现条件触发
2. **INSTEAD OF 触发器**: 未实现 (用于视图)
3. ** STATEMENT 级别**: 部分支持
4. **触发器链深度限制**: 未实现
5. **触发器权限检查**: 不完整

### 9.3 提升计划

| 阶段 | 任务 | 目标覆盖率 |
|------|------|-----------|
| v3.1.0 | 实现 WHEN 子句 | 70% |
| v3.1.0 | 完善 STATEMENT 级别 | 70% |
| v3.2.0 | 实现 INSTEAD OF | 65% |

## 10. 核心文件索引

| 文件 | 行数 | 说明 |
|------|------|------|
| `crates/executor/src/trigger.rs` | ~2040 | 触发器执行器 |
| `crates/executor/src/trigger_eval/` | ~500 | 触发器表达式执行 |
| `crates/storage/src/trigger.rs` | ~300 | 触发器存储 |

## 11. 相关文档

| 文档 | 说明 |
|------|------|
| [DML_EXECUTION.md](./DML_EXECUTION.md) | DML 执行链路 |
| [STORED_PROCEDURE.md](./STORED_PROCEDURE.md) | 存储过程 |
