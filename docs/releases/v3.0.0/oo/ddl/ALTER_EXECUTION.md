# ALTER TABLE 执行链路

> ALTER TABLE 操作深度分析 - ADD/DROP/MODIFY COLUMN, RENAME, 约束管理

## 1. ALTER TABLE 架构

```
┌─────────────────────────────────────────────────────────────┐
│                   ALTER TABLE 执行架构                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ALTER TABLE users ADD COLUMN phone VARCHAR(20)             │
│                        │                                    │
│                        ▼                                    │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                    Parser                              │  │
│  │  ParseAlterTableStmt {                                │  │
│  │    operation: ADD_COLUMN,                             │  │
│  │    table: "users",                                   │  │
│  │    column: ColumnDef { name: "phone", type: VARCHAR }│  │
│  │  }                                                   │  │
│  └──────────────────────────────────────────────────────┘  │
│                        │                                    │
│                        ▼                                    │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                 Catalog Validator                      │  │
│  │  - 检查表是否存在                                     │  │
│  │  - 检查列名是否重复                                   │  │
│  │  - 检查列类型合法性                                   │  │
│  │  - 检查默认值表达式                                   │  │
│  └──────────────────────────────────────────────────────┘  │
│                        │                                    │
│                        ▼                                    │
│  ┌──────────────────────────────────────────────────────┐  │
│  │               DDL Executor                             │  │
│  │  ┌────────────────────────────────────────────────┐   │  │
│  │  │ ALTER_TYPE = ADD_COLUMN (Instant DDL)          │   │  │
│  │  │   - 更新系统表 schema                          │   │  │
│  │  │   - 不移动现有数据                             │   │  │
│  │  │   - 新列默认值写入新读取的行                    │   │  │
│  │  ├────────────────────────────────────────────────┤   │  │
│  │  │ ALTER_TYPE = DROP/MODIFY COLUMN                │   │  │
│  │  │   - 表级排他锁                                 │   │  │
│  │  │   - 全表数据重构                               │   │  │
│  │  │   - 索引重建                                   │   │  │
│  │  └────────────────────────────────────────────────┘   │  │
│  └──────────────────────────────────────────────────────┘  │
│                        │                                    │
│                        ▼                                    │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                  DDL WAL Log                          │  │
│  │  - 记录完整 ALTER 语句                               │  │
│  │  - 用于崩溃恢复时重做                                │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 2. 操作分类与复杂度

| 操作类型 | 复杂度 | 数据移动 | 并发影响 |
|----------|--------|----------|----------|
| ADD COLUMN | O(1) | 无 | 阻塞读取 |
| DROP COLUMN | O(n) | 重构数据页 | 阻塞所有 |
| MODIFY COLUMN | O(n) | 转换数据 | 阻塞所有 |
| ADD INDEX | O(n) | 构建索引 | 阻塞写入 |
| DROP INDEX | O(1) | 无 | 无 |
| RENAME TABLE | O(1) | 无 | 阻塞所有 |
| ADD CONSTRAINT | O(1) | 无 | 验证数据 |
| DROP CONSTRAINT | O(1) | 无 | 无 |

## 3. ADD COLUMN 执行链路 (Instant DDL)

### 3.1 时序图

```
ALTER TABLE users ADD COLUMN phone VARCHAR(20) DEFAULT 'unknown'
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                    Parser                                    │
│  AlterTableStmt {                                             │
│    operation: AddColumn,                                      │
│    column: ColumnDef {                                        │
│      name: "phone",                                           │
│      data_type: VARCHAR(20),                                  │
│      default: Some("'unknown'")                               │
│    }                                                          │
│  }                                                            │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                 Catalog Check                                │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 1. table_exists("users") → true                       │ │
│  │ 2. column_not_exists("phone") → true                 │ │
│  │ 3. data_type_valid(VARCHAR) → true                   │ │
│  │ 4. default_expression_valid() → true                  │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                 Metadata Update                              │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 1. 锁住表 metadata (metadata_lock)                    │ │
│  │ 2. 更新 system_table.users_schema:                    │ │
│  │    - 添加 column_def: phone VARCHAR(20) DEFAULT..     │ │
│  │ 3. 递增 table_version                                │ │
│  │ 4. 释放 metadata_lock                                │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                   Storage Layer                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ ADD COLUMN (Instant):                                 │ │
│  │   - 不修改现有数据页                                  │ │
│  │   - 新读取的行应用新列默认值                          │ │
│  │   - 旧行读取时忽略已删除的列                           │ │
│  │                                                       │ │
│  │ 读取流程:                                             │ │
│  │   if row_version < schema_version:                    │ │
│  │       apply_default_values()  // 填充新列             │ │
│  │   else:                                              │ │
│  │       read_full_row()                                │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                     DDL WAL                                 │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ LogType::DDL:                                        │ │
│  │   "ALTER TABLE users ADD COLUMN phone VARCHAR(20)"   │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 状态图

```
                    ┌─────────────────┐
                    │   PARSING       │
                    │  Validating     │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │ACQUIRE METADATA │
                    │     LOCK       │
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              │                              │
     ┌────────▼────────┐          ┌────────▼────────┐
     │   ADD COLUMN    │          │DROP/MODIFY COL  │
     │   (Instant)     │          │  (Table Scan)   │
     └────────┬────────┘          └────────┬────────┘
              │                            │
     ┌────────▼────────┐          ┌────────▼────────┐
     │UPDATE SCHEMA    │          │ TABLE EXCLUSIVE │
     │   VERSION      │          │     LOCK       │
     └────────┬────────┘          └────────┬────────┘
              │                            │
     ┌────────▼────────┐          ┌────────▼────────┐
     │ REBUILD INDEXES│          │  REWRITE ALL    │
     │   (if needed)  │          │   DATA PAGES    │
     └────────┬────────┘          └────────┬────────┘
              │                            │
              └──────────────┬─────────────┘
                             │
                    ┌────────▼────────┐
                    │  WRITE DDL WAL  │
                    │  COMMIT TX      │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │    COMPLETE    │
                    └────────────────┘
```

## 4. DROP/MODIFY COLUMN 执行链路 (Table Rewrite)

### 4.1 时序图

```
ALTER TABLE users DROP COLUMN age
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                    Parser                                    │
│  AlterTableStmt { operation: DropColumn, column: "age" }    │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                 Catalog Validation                           │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 1. column_exists("age") → true                       │ │
│  │ 2. column_not_referenced_by_index("age") → false     │ │
│  │    (需要先删除相关索引)                                │ │
│  │ 3. column_not_referenced_by FK("age") → true        │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│               Table Rewrite (Full Scan)                      │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 1. 获取表级排他锁 (WRITE_EXCLUSIVE)                   │ │
│  │ 2. 创建临时表 temp_users                              │ │
│  │ 3. 扫描原表所有数据页:                               │ │
│  │    for each page in table:                           │ │
│  │        for each row in page:                         │ │
│  │            new_row = row.remove_column("age")         │ │
│  │            write_to_temp(new_row)                     │ │
│  │ 4. 重建所有受影响的索引                               │ │
│  │ 5. atomically swap(原表, 临时表)                     │ │
│  │ 6. 删除原表                                          │ │
│  │ 7. 释放排他锁                                        │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                    DDL WAL + Commit                          │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 数据重构算法

```
DROP/MODIFY COLUMN 重构算法
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

输入: 原表 T, 要删除的列集合 C_drop
输出: 新表 T'

1. LOCK T (WRITE_EXCLUSIVE)
2. 创建临时表 T_temp, schema = T.schema - C_drop

3. for each data_page in T:
4.     for each row in data_page:
5.         new_row = Row {}
6.         for each column in T.schema:
7.             if column not in C_drop:
8.                 new_row[column] = row[column]
9.         write_row_to_page(T_temp, new_row)

10. for each index in T.indexes:
11.    if index.columns ∩ C_drop ≠ ∅:
12.        rebuild_index(index)  // 重建受影响的索引
13.    else:
14.        迁移未受影响的索引到 T_temp

15. atomically:
16.    rename(T, T_old)
17.    rename(T_temp, T)

18. DROP TABLE T_old  // 延迟删除，避免回滚问题

19. UNLOCK T
```

## 5. ADD INDEX 执行链路

### 5.1 时序图

```
CREATE INDEX idx_name ON users(age)
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                    Parser                                    │
│  CreateIndexStmt {                                         │
│    index_name: "idx_name",                                 │
│    table: "users",                                        │
│    columns: ["age"],                                       │
│    unique: false                                          │
│  }                                                        │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                 Catalog Validation                          │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 1. table_exists("users") → true                     │ │
│  │ 2. index_not_exists("idx_name") → true              │ │
│  │ 3. column_exists("age") → true                      │ │
│  │ 4. 生成 index_id                                    │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│               Index Build (Offline)                          │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ Phase 1: Schema Update                               │ │
│  │   - 添加 index_def 到 system_table.indexes           │ │
│  │   - index 状态 = BUILDING                           │ │
│  │                                                       │ │
│  │ Phase 2: Index Construction                          │ │
│  │   - 获取表 SHARE锁 (允许读取，阻塞写入)               │ │
│  │   - 扫描表建立 B+Tree:                              │ │
│  │                                                       │ │
│  │   for each row in users:                            │ │
│  │       key = row[age]                                │ │
│  │       value = row_id (page_id, slot_id)            │ │
│  │       bptree_insert(index_bptree, key, value)       │ │
│  │                                                       │ │
│  │ Phase 3: Finalize                                    │ │
│  │   - 索引状态 = ACTIVE                               │ │
│  │   - 释放锁                                          │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                     DDL WAL + Commit                         │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 并行索引构建

```
CREATE INDEX idx_age ON users(age)  -- 利用多核并行

                        ┌─────────────────┐
                        │  INDEX BUILD   │
                        │   COORDINATOR  │
                        └────────┬────────┘
                                 │
              ┌──────────────────┼──────────────────┐
              │                  │                  │
     ┌────────▼────────┐ ┌────────▼────────┐ ┌────────▼────────┐
     │   WORKER 1     │ │   WORKER 2     │ │   WORKER N     │
     │  Pages 0-999   │ │ Pages 1000-1999│ │ Pages N*1000...│
     └────────┬────────┘ └────────┬────────┘ └────────┬────────┘
              │                  │                  │
              └──────────────────┼──────────────────┘
                                 │
                        ┌────────▼────────┐
                        │  MERGE SORT    │
                        │ (Key-Sorted)   │
                        └────────┬────────┘
                                 │
                        ┌────────▼────────┐
                        │  B+Tree Bulk   │
                        │    Load        │
                        └─────────────────┘
```

## 6. RENAME TABLE 执行链路

### 6.1 状态图

```
                ┌─────────────────┐
                │     PARSE       │
                │   RENAME stmt   │
                └────────┬────────┘
                         │
                ┌────────▼────────┐
                │  ACQUIRE LOCK   │
                │ (METADATA_ONLY) │
                └────────┬────────┘
                         │
                ┌────────▼────────┐
                │  CHECK NEW NAME │
                │ not_exists(name)│
                └────────┬────────┘
                         │
                ┌────────▼────────┐
                │  UPDATE CATALOG │
                │ system_table    │
                │ (table_name)    │
                └────────┬────────┘
                         │
                ┌────────▼────────┐
                │ UPDATE VIEWS    │
                │ referencing tbl │
                └────────┬────────┘
                         │
                ┌────────▼────────┐
                │  WRITE DDL WAL  │
                └────────┬────────┘
                         │
                ┌────────▼────────┐
                │    COMPLETE     │
                └─────────────────┘
```

## 7. 约束管理

### 7.1 ADD CONSTRAINT 时序图

```
ALTER TABLE orders ADD CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id)
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                    Parser                                    │
│  AlterTableStmt {                                             │
│    operation: AddConstraint,                                 │
│    constraint: ForeignKey {                                  │
│      columns: ["user_id"],                                  │
│      ref_table: "users",                                    │
│      ref_columns: ["id"]                                    │
│    }                                                        │
│  }                                                          │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                 Constraint Validation                         │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 1. ref_table exists → true                           │ │
│  │ 2. ref_column is PRIMARY KEY or UNIQUE → true        │ │
│  │ 3. data_type_match(user_id, id) → true              │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                 Data Validation                              │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 检查所有现有行:                                       │ │
│  │   for each row in orders:                            │ │
│  │       user_id = row.user_id                          │ │
│  │       if not exists(users, id=user_id):              │ │
│  │           throw FKViolation(user_id)                 │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                 Catalog Update                               │
│  - 添加 constraint_def 到 system_table.constraints          │
│  - constraint状态 = ENABLED                                │
└─────────────────────────────────────────────────────────────┘
```

## 8. DDL 与并发控制

### 8.1 锁层级

```
┌─────────────────────────────────────────────────────────────┐
│                    DDL 锁层级                                │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  LOCK LAYER (从高到低)                                       │
│  ─────────────────────────────────                         │
│  ┌─────────────────┐                                      │
│  │ TABLE_EXCLUSIVE  │  DROP TABLE, ALTER TABLE (rewrite)  │
│  │                 │  获取排他锁，完全阻塞所有访问           │
│  └─────────────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌─────────────────┐                                      │
│  │TABLE_WRITE_LOCK │  CREATE INDEX (build phase)          │
│  │                 │  允许读取，阻塞写入                    │
│  └─────────────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌─────────────────┐                                      │
│  │TABLE_READ_LOCK  │  ADD COLUMN (instant)               │
│  │                 │  允许读写，阻塞 DDL                   │
│  └─────────────────┘                                      │
│           │                                                 │
│           ▼                                                 │
│  ┌─────────────────┐                                      │
│  │METADATA_LOCK    │  RENAME TABLE, ADD CONSTRAINT        │
│  │                 │  仅锁元数据，不锁数据                  │
│  └─────────────────┘                                      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 8.2 DDL 与 DML 并发

| DDL 操作 | 允许的 DML | 阻塞的 DML |
|----------|-----------|-----------|
| ADD COLUMN (Instant) | SELECT, INSERT, UPDATE, DELETE | 无 |
| ADD INDEX | SELECT, INSERT, UPDATE, DELETE | 无 |
| DROP COLUMN | 无 | 所有 |
| MODIFY COLUMN | 无 | 所有 |
| RENAME TABLE | 无 | 所有 |

## 9. 崩溃恢复

### 9.1 DDL 恢复状态机

```
                    ┌─────────────────┐
                    │   CRASH OCCURS  │
                    │  (during DDL)   │
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              │                              │
     ┌────────▼────────┐          ┌────────▼────────┐
     │  DDL COMPLETE   │          │DDL NOT COMPLETE │
     │  (WAL flushed)  │          │ (WAL not flushed)│
     └────────┬────────┘          └────────┬────────┘
              │                              │
              │                     ┌────────▼────────┐
              │                     │   ROLLBACK    │
              │                     │ (undo DDL)    │
              │                     └────────────────┘
              │                              │
              ▼                              ▼
     ┌─────────────────┐          ┌─────────────────┐
     │     REDO DDL    │          │   SKIP DDL     │
     │  (replay from   │          │  (operation    │
     │   WAL)          │          │   was rolled   │
     │                 │          │   back)         │
     └─────────────────┘          └─────────────────┘
```

### 9.2 DDL WAL 格式

```rust
enum DDLLogEntry {
    CreateTable {
        table_id: u64,
        schema: TableSchema,
    },
    DropTable {
        table_id: u64,
        // 用于回滚：存储原始数据路径
        backup_path: PathBuf,
    },
    AlterTable {
        table_id: u64,
        operation: AlterOperation,
        // Instant DDL: 记录 schema 变更
        new_schema: TableSchema,
    },
    CreateIndex {
        index_id: u64,
        table_id: u64,
        columns: Vec<String>,
    },
    DropIndex {
        index_id: u64,
        // 用于回滚：存储索引定义
        index_def: IndexDef,
    },
    RenameTable {
        table_id: u64,
        old_name: String,
        new_name: String,
    },
}
```

## 10. 关键测试用例

### 10.1 Instant DDL 测试

```rust
#[test]
fn test_add_column_instant() {
    // 创建表，插入数据
    let tbl = create_table("t1", vec![("id", INT), ("name", VARCHAR)]);
    insert_into("t1", vec![(1, "a"), (2, "b")]);

    // Instant ADD COLUMN
    alter_table_add_column("t1", "age", INT, Some(18));

    // 验证：现有数据应用默认值
    let rows = select_from("t1");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].age, 18);  // 默认值生效

    // 验证：新插入的行有新列
    insert_into("t1", vec![(3, "c")]);
    let rows = select_from("t1");
    assert_eq!(rows.len(), 3);
}

#[test]
fn test_add_column_concurrent_read() {
    // 验证 ADD COLUMN 期间读取不阻塞
    let tbl = create_table("t1", vec![("id", INT)]);
    spawn_many_readers("t1", 10);

    // ADD COLUMN 应该不阻塞读取
    alter_table_add_column("t1", "name", VARCHAR, Some("default"));

    assert_all_readers_completed();
}
```

### 10.2 Table Rewrite 测试

```rust
#[test]
fn test_drop_column_rewrite() {
    let tbl = create_table("t1", vec![
        ("id", INT),
        ("name", VARCHAR),
        ("age", INT),  // 将被删除
        ("email", VARCHAR),
    ]);

    insert_into("t1", vec![(1, "a", 20, "a@test")]);

    // DROP COLUMN 触发表重构
    alter_table_drop_column("t1", "age");

    // 验证数据正确迁移
    let rows = select_from("t1");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, 1);
    assert_eq!(rows[0].name, "a");
    assert_eq!(rows[0].email, "a@test");
    assert!(rows[0].get("age").is_none());
}

#[test]
fn test_modify_column_type() {
    // VARCHAR(10) → VARCHAR(20)
    alter_table_modify_column("t1", "name", VARCHAR(20));

    // 验证类型变更
    let schema = get_table_schema("t1");
    assert_eq!(schema["name"].data_type, VARCHAR(20));
}
```

### 10.3 索引构建测试

```rust
#[test]
fn test_create_index_build() {
    let tbl = create_table("t1", vec![("id", INT), ("val", INT)]);
    insert_random_rows("t1", 10000);

    // 创建索引
    create_index("idx_val", "t1", vec!["val"]);

    // 验证索引存在且可用
    let index = get_index("idx_val");
    assert_eq!(index.status, IndexStatus::Active);

    // 验证索引工作正常
    let rows = select_where("t1", "val > 500");
    assert!(rows.iter().all(|r| r.val > 500));
}

#[test]
fn test_create_unique_index_constraint() {
    let tbl = create_table("t1", vec![("id", INT), ("code", VARCHAR)]);

    insert_into("t1", vec![(1, "ABC")]);

    // 创建唯一索引
    create_unique_index("idx_code", "t1", vec!["code"]);

    // 验证唯一性约束
    let result = insert_into("t1", vec![(2, "ABC")]);
    assert!(result.is_err());  // 唯一性冲突
}
```

### 10.4 崩溃恢复测试

```rust
#[test]
fn test_ddl_crash_recovery_redo() {
    let tbl = create_table("t1", vec![("id", INT)]);
    insert_into("t1", vec![(1), (2)]);

    // DDL 执行中崩溃
    crash_after_ddl_write();

    // 重启后应该重做 DDL
    restart_and_recover();

    // 验证 DDL 已应用
    let schema = get_table_schema("t1");
    assert!(schema.has_column("phone"));
}

#[test]
fn test_ddl_crash_recovery_undo() {
    let tbl = create_table("t1", vec![("id", INT)]);

    // DROP TABLE 执行中崩溃
    let result = execute_with_crash("DROP TABLE t1");

    // 重启后应该回滚
    restart_and_recover();

    // 验证表仍然存在
    assert!(table_exists("t1"));
}
```

## 11. 性能优化

### 11.1 当前性能瓶颈

| 操作 | 数据量 | 当前耗时 | 优化目标 |
|------|--------|----------|----------|
| ADD COLUMN | 1M rows | <100ms | <50ms |
| DROP COLUMN | 1M rows | 30s | 5s |
| MODIFY COLUMN | 1M rows | 45s | 8s |
| CREATE INDEX | 1M rows | 20s | 3s |

### 11.2 优化策略

```
┌─────────────────────────────────────────────────────────────┐
│                   DDL 性能优化策略                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Instant DDL (已实现)                                    │
│     ─────────────────────                                   │
│     ADD COLUMN 只改元数据，不移动数据                         │
│     - 新列默认值在读取时应用                                 │
│     - O(1) 复杂度                                           │
│                                                              │
│  2. 并行索引构建                                            │
│     ─────────────────────                                   │
│     - 分片扫描，多核并行                                     │
│     - merge sort 后 bulk load                              │
│     - 目标: 1M rows < 3s                                   │
│                                                              │
│  3. 在线 DDL (未来)                                        │
│     ─────────────────────                                   │
│     - DDL 不阻塞 DML                                        │
│     - 读写并发                                              │
│     - 需要 shadow table + double write                      │
│                                                              │
│  4. 增量索引构建 (未来)                                     │
│     ─────────────────────                                   │
│     - 不重建整个索引                                        │
│     - 增量更新 B+Tree                                       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 12. 覆盖率差距

| 操作 | 当前测试覆盖 | 目标覆盖 | 差距 |
|------|-------------|----------|------|
| ADD COLUMN | 70% | 95% | 25% |
| DROP COLUMN | 60% | 90% | 30% |
| MODIFY COLUMN | 50% | 85% | 35% |
| RENAME TABLE | 80% | 95% | 15% |
| ADD INDEX | 75% | 95% | 20% |
| DROP INDEX | 70% | 90% | 20% |
| ADD CONSTRAINT | 65% | 90% | 25% |
| FK Validation | 60% | 90% | 30% |
| DDL Recovery | 55% | 90% | 35% |

### 12.1 缺失的测试场景

```rust
// 缺失测试
#[test]
fn test_alter_table_add_column_concurrent_dml() {
    // ADD COLUMN 期间的并发 INSERT/UPDATE/DELETE
}

#[test]
fn test_alter_table_drop_column_with_indexes() {
    // 删除被索引引用的列
}

#[test]
fn test_alter_table_modify_column_data_loss() {
    // VARCHAR(10) → VARCHAR(5) 数据截断
}

#[test]
fn test_create_index_on_existing_table() {
    // 在已有数据的表上建索引
}

#[test]
fn test_fk_constraint_validation_huge_table() {
    // 大表外键验证性能
}

#[test]
fn test_ddl_rollback_on_crash() {
    // DDL 执行中崩溃的回滚
}
```

## 13. 相关文件

| 文件 | 说明 |
|------|------|
| `executor/src/ddl_executor.rs` | DDL 执行器主实现 |
| `catalog/src/table.rs` | 表元数据管理 |
| `catalog/src/schema.rs` | Schema 管理 |
| `storage/src/page.rs` | 数据页结构 |
| `storage/src/bptree.rs` | B+Tree 索引 |
| `transaction/src/wal.rs` | WAL 日志 |
| `transaction/src/mvcc.rs` | MVCC 版本管理 |

---

*文档版本: v3.0.0*
*最后更新: 2026-05-11*