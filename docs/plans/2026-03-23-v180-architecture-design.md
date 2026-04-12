# SQLRustGo v1.8.0 架构与功能设计

## 一、设计思想

### 1.1 核心原则

- 增量扩展：在现有执行引擎上"外挂"新能力，避免对核心模块大改。
- 复用优先：存储过程/触发器复用已有的表达式、执行器、事务管理基础设施。
- 教学友好：语法与常见教材（如MySQL/PostgreSQL）保持近似，降低学习成本。
- 性能可控：新增功能带来的开销应可衡量，并有明确的性能目标。

### 1.2 架构演进方向

```
v1.7.0                               v1.8.0
┌─────────────────┐                  ┌─────────────────────┐
│ SQL Engine      │                  │ SQL Engine          │
│ (火山模型)       │                  │ + 过程化引擎         │
│ MVCC            │                  │ + 触发器管理器       │
│ B+Tree          │                  │ + 系统表扩展         │
│ WAL             │                  │ + 新数据类型         │
└─────────────────┘                  └─────────────────────┘
```

---

## 二、总体架构

### 2.1 新增模块

```
crates/
├── sqlrustgo-procedure/            # 存储过程与函数
│   ├── ast/                        # 过程化语句 AST
│   ├── compiler/                   # 编译为内部指令
│   ├── executor/                   # 指令执行器
│   ├── cache/                      # 过程缓存
│   └── variable.rs                 # 变量上下文
├── sqlrustgo-trigger/               # 触发器管理
│   ├── manager.rs                  # 触发器注册/触发
│   ├── executor.rs                  # 触发器执行包装
│   └── recursion.rs                 # 递归深度控制
├── sqlrustgo-datatype/              # 新数据类型
│   ├── decimal.rs
│   ├── json.rs
│   └── functions.rs                # 相关函数
├── sqlrustgo-catalog/               # 扩展元数据
│   ├── procedure.rs
│   ├── trigger.rs
│   └── system_tables.rs
└── sqlrustgo-executor/             # 原有执行器扩展
    ├── dml_trigger.rs              # DML 触发器钩子
    └── procedure_call.rs           # CALL 语句执行
```

### 2.2 模块依赖关系

```
sqlrustgo-server (入口)
 │
 ├── sqlrustgo-executor
 │   ├── sqlrustgo-procedure (调用存储过程)
 │   ├── sqlrustgo-trigger (触发触发器)
 │   └── sqlrustgo-datatype
 │
 ├── sqlrustgo-catalog
 │   ├── sqlrustgo-procedure
 │   └── sqlrustgo-trigger
 │
 └── sqlrustgo-storage
     └── sqlrustgo-datatype
```

---

## 三、存储过程设计

### 3.1 语法（参考MySQL/PL/pgSQL）

```sql
-- 创建
CREATE PROCEDURE get_customer(IN cust_id INT, OUT name VARCHAR)
BEGIN
    SELECT c_name INTO name FROM customer WHERE c_custkey = cust_id;
END;

-- 调用
CALL get_customer(10, @name);
SELECT @name;

-- 过程内支持语句
CREATE PROCEDURE transfer(IN from INT, IN to INT, IN amount DECIMAL)
BEGIN
    DECLARE from_bal DECIMAL;
    SELECT balance INTO from_bal FROM accounts WHERE id = from;
    IF from_bal >= amount THEN
        UPDATE accounts SET balance = balance - amount WHERE id = from;
        UPDATE accounts SET balance = balance + amount WHERE id = to;
        COMMIT;
    ELSE
        ROLLBACK;
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'Insufficient funds';
    END IF;
END;
```

### 3.2 内部表示

过程指令集（简化版）：

```rust
enum ProcInstruction {
    // 变量操作
    DeclareVar { name: String, type: DataType, default: Option<Value> },
    AssignVar { name: String, expr: ExprNode },
    
    // 控制流
    If { condition: ExprNode, then_block: Vec<Self>, else_block: Option<Vec<Self>> },
    Loop { block: Vec<Self> },  // 无限循环，需内部 break
    While { condition: ExprNode, block: Vec<Self> },
    Break,
    Return { value: Option<ExprNode> },
    
    // SQL 执行
    ExecuteSql { sql: String, into_var: Option<String> },  // SELECT INTO
    ExecuteDml { node: PlanNode },  // 已优化的 DML 计划
    
    // 事务控制
    Commit,
    Rollback,
    
    // 异常处理（简化）
    Signal { sqlstate: String, message: String },
}
```

### 3.3 执行模型

- 解释执行：首次编译为指令序列，后续调用直接执行。
- 变量作用域：块级作用域（支持嵌套）。
- 参数传递：IN（值传递）、OUT（引用传递）、INOUT（两者）。
- 返回值：通过 OUT 参数或返回结果集（暂不支持多结果集）。
- 与事务集成：
  - 过程默认在调用事务中运行。
  - 过程内 COMMIT 提交当前事务并开启新事务（过程继续）。
  - 过程内 ROLLBACK 回滚当前事务并终止过程。

### 3.4 缓存与并发

- 过程缓存：全局 RwLock<HashMap<ProcedureKey, Arc<CompiledProcedure>>>，支持并发读。
- 变量上下文：每个调用独立，不共享。

---

## 四、触发器设计

### 4.1 语法

```sql
CREATE TRIGGER update_timestamp
BEFORE UPDATE ON orders
FOR EACH ROW
BEGIN
    SET NEW.updated_at = NOW();
END;

CREATE TRIGGER prevent_negative_stock
BEFORE INSERT ON inventory
FOR EACH ROW
BEGIN
    IF NEW.quantity < 0 THEN
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'Negative stock not allowed';
    END IF;
END;
```

### 4.2 内部表示

触发器元数据：

```rust
pub struct Trigger {
    pub name: String,
    pub table: TableId,
    pub event: TriggerEvent,       // Insert | Update | Delete
    pub timing: TriggerTiming,    // Before | After
    pub body: ProcInstructions,   // 复用存储过程的指令集
    pub for_each_row: bool,        // 目前仅支持 FOR EACH ROW
}
```

### 4.3 执行集成点

在 DML 执行器中插入钩子：

```rust
// InsertExecutor 伪代码
fn execute(&self, ctx: &ExecutionContext) -> Result<Vec<Row>> {
    let trigger_mgr = ctx.catalog.get_trigger_manager();
    let table_id = self.table.id();
    
    for row in self.rows {
        // BEFORE 触发器可修改行
        if let Some(trigger) = trigger_mgr.get_before_insert(table_id) {
            let new_row = trigger.execute(ctx, TriggerContext::new(None, Some(row)))?;
            row = new_row;
        }
        
        // 执行实际插入
        let inserted = storage.insert(row);
        
        // AFTER 触发器（不可修改）
        if let Some(trigger) = trigger_mgr.get_after_insert(table_id) {
            trigger.execute(ctx, TriggerContext::new(None, Some(inserted)))?;
        }
    }
    Ok(...)
}
```

### 4.4 递归与循环保护

- 深度计数器：每次触发前检查当前深度，超过 max_trigger_depth（默认10）时报错。
- 避免无限循环：触发器若修改同一张表，可能再次触发自身。通过深度限制即可阻止。

---

## 五、新数据类型设计

### 5.1 DECIMAL

内部表示：

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decimal {
    value: i128,       // 缩放后的整数
    scale: u8,         // 小数位数 (0-38)
    precision: u8,     // 总精度 (1-38)
}

impl Decimal {
    pub fn new(value: i128, precision: u8, scale: u8) -> Result<Self> { ... }
    pub fn parse(s: &str) -> Result<Self> { ... }
    // 运算
    pub fn add(&self, other: &Self) -> Result<Self> { ... }
    pub fn sub(&self, other: &Self) -> Result<Self> { ... }
    pub fn mul(&self, other: &Self) -> Result<Self> { ... }
    pub fn div(&self, other: &Self) -> Result<Self> { ... }
}
```

运算规则：

- 加法/减法：将两数 scale 对齐，结果 scale = max(s1, s2)，若溢出则报错。
- 乘法：结果 scale = s1 + s2，结果 precision = p1 + p2，若 >38 则报错。
- 除法：结果 scale = max(s1, s2) + 4，结果 precision 按 SQL 标准计算（最多 38）。

与浮点数转换：支持 CAST 显式转换，隐式转换仅在与 DECIMAL 比较时按精度提升。

### 5.2 JSON

内部表示：

```rust
pub struct JsonValue(serde_json::Value);

impl JsonValue {
    pub fn parse(s: &str) -> Result<Self> { ... }
    pub fn get(&self, path: &str) -> Option<&Self> { ... }
    pub fn to_string(&self) -> String { ... }
}
```

运算符：

- ->：返回 JSON 值（类型 JSON）
- ->>：返回字符串（类型 TEXT）
- 支持路径表达式如 '$.name'、'$[0]'。

索引支持：可为 JSON 字段的某个键创建虚拟列索引（通过表达式索引），例如 CREATE INDEX idx_name ON t ((data->>'name'))。

---

## 六、DDL 增强设计

### 6.1 ALTER TABLE 扩展

支持语法：

```sql
ALTER TABLE table_name ADD COLUMN column_def [FIRST|AFTER col];
ALTER TABLE table_name DROP COLUMN column_name;
ALTER TABLE table_name MODIFY COLUMN column_def; -- 修改类型/约束
```

实现要点：

- ADD COLUMN：仅允许添加到末尾（FIRST/AFTER 可稍后实现），需更新表元数据，并为已有行填充默认值（若 NOT NULL 则要求指定 DEFAULT）。
- DROP COLUMN：标记列删除，实际存储中保留数据（未来可 VACUUM）。更新表元数据，移除该列在索引中的引用。
- MODIFY COLUMN：仅允许安全的类型转换（如 INT→BIGINT），或增加长度（VARCHAR(10)→VARCHAR(20)）。若改变精度/有损转换则拒绝。

### 6.2 CREATE INDEX 独立

```sql
CREATE [UNIQUE] INDEX index_name ON table_name (col1, col2);
CREATE INDEX idx_expr ON t ((data->>'name')); -- 表达式索引
```

实现：复用现有 BTreeIndex 逻辑，索引定义存储于系统表。

---

## 七、DML 补全

### 7.1 LIMIT / OFFSET

```sql
SELECT ... FROM ... ORDER BY ... LIMIT n [OFFSET m];
```

执行器增加 LimitExecutor 包装下游算子，计数返回。

### 7.2 INSERT SET 语法

```sql
INSERT INTO table SET col1=val1, col2=val2;
```

解析为 INSERT ... VALUES (...) 形式，复用现有插入逻辑。

---

## 八、函数增强

### 8.1 字符串函数

- LENGTH(str) → 返回字符数
- UPPER(str) / LOWER(str)
- SUBSTR(str, start, len)
- TRIM(str)

### 8.2 时间函数

- NOW() → 当前时间戳（事务开始时）
- CURDATE() → 当前日期
- DATE_ADD(date, INTERVAL n unit) → 简单实现
- 格式化：DATE_FORMAT(date, format) 可暂缓

---

## 九、性能与测试要求

### 9.1 性能目标

- 触发器开销：单行 Insert/Update/Delete 延迟增加 ≤ 20%（相对于无触发器）。
- 存储过程调用（首次编译）≤ 100ms，后续调用（不含内部 SQL）≤ 1ms。
- 缓存命中后，过程执行指令流转开销可忽略。

### 9.2 测试套件

- 单元测试：各数据类型、过程指令、触发器递归检测。
- 集成测试：模拟真实场景，如银行转账过程、审计触发器。
- 回归测试：确保已有 SQL-92 特性不被破坏。

---

## 十、实现路线图（v1.8.0）

| 阶段 | 时间 | 内容 |
|------|------|------|
| Phase 1 | Week 1-2 | LIMIT/OFFSET, INSERT SET, ALTER TABLE ADD/DROP |
| Phase 2 | Week 3-4 | DECIMAL 类型、JSON 类型及基本函数 |
| Phase 3 | Week 5 | 字符串/时间函数、CREATE INDEX 独立 |
| Phase 4 | Week 6-7 | 存储过程（语法、编译、执行、变量、控制流） |
| Phase 5 | Week 8 | 触发器（元数据、集成、递归控制） |
| Phase 6 | Week 9 | 集成测试、文档、性能调优、发布 |

---

## 十一、总结

v1.8.0 的设计遵循"增量扩展、复用核心、教学优先"的理念，通过新增过程化引擎、触发器管理器和数据类型，在不重构现有架构的前提下，补齐了 SQL-92 的绝大部分特性，使 SQLRustGo 能够胜任数据库课程教学和中小型生产应用。后续版本将在此基础上，继续向量化优化和分布式扩展。
