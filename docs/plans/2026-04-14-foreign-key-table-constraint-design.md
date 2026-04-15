# Foreign Key Table Constraint 设计方案

**Issue**: #1379 - FOREIGN KEY 约束 - parser + executor 实现  
**日期**: 2026-04-14  
**状态**: 设计完成，等待实现

## 背景

当前 SQLRustGo 的 FOREIGN KEY 支持仅限列级语法：

```sql
CREATE TABLE orders (
    id INTEGER,
    user_id INTEGER REFERENCES users(id)  -- 列级 FK ✓
);
```

缺少 SQL 标准的表级约束语法：

```sql
CREATE TABLE orders (
    id INTEGER,
    user_id INTEGER,
    FOREIGN KEY (user_id) REFERENCES users(id)  -- 表级 FK ✗
);
```

## 目标

实现完整的表级约束语法，支持：
- `FOREIGN KEY (col,...) REFERENCES table(col,...)`
- `PRIMARY KEY (col,...)` 表级语法
- `ON DELETE/UPDATE CASCADE/SET NULL/RESTRICT`
- Multi-column (复合) 外键

## 设计方案

### 1. Parser 层

#### 1.1 新增 TableConstraint 枚举

位置: `crates/parser/src/parser.rs`

```rust
/// Table-level constraints for CREATE TABLE
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TableConstraint {
    /// PRIMARY KEY (col1, col2, ...)
    PrimaryKey {
        columns: Vec<String>,
    },
    /// FOREIGN KEY (col1, col2, ...) REFERENCES table(col1, col2, ...)
    ForeignKey {
        columns: Vec<String>,           // 子表列
        reference_table: String,         // 父表名
        reference_columns: Vec<String>,  // 父表列
        on_delete: Option<ForeignKeyAction>,
        on_update: Option<ForeignKeyAction>,
    },
    /// UNIQUE (col1, col2, ...)
    Unique {
        columns: Vec<String>,
    },
}
```

#### 1.2 扩展 CreateTableStatement

```rust
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub if_not_exists: bool,
    pub constraints: Vec<TableConstraint>,  // 新增
}
```

#### 1.3 解析流程

```rust
fn parse_create_table(&mut self) -> Result<Statement, String> {
    // ... 解析 name 和 columns ...
    
    // 解析表级约束 (在列定义之后)
    let mut constraints = Vec::new();
    if matches!(self.current(), Some(Token::RParen)) {
        self.next();  // consume ')'
        
        // 解析表级约束
        loop {
            match self.current() {
                Some(Token::Foreign) => {
                    self.next();  // consume FOREIGN
                    self.parse_foreign_key_constraint(&mut constraints)?;
                }
                Some(Token::Primary) => {
                    self.next();  // consume PRIMARY
                    self.parse_primary_key_constraint(&mut constraints)?;
                }
                Some(Token::Unique) => {
                    self.next();
                    self.parse_unique_constraint(&mut constraints)?;
                }
                _ => break,
            }
        }
    }
}
```

#### 1.4 语法支持

```sql
-- 单列 FK
FOREIGN KEY (user_id) REFERENCES users(id)

-- 多列 FK
FOREIGN KEY (order_id, product_id) REFERENCES order_items(id, product_id)

-- 带级联动作
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE SET NULL

-- 表级 PRIMARY KEY
PRIMARY KEY (id, version)

-- 表级 UNIQUE
UNIQUE (email, tenant_id)
```

### 2. Catalog 层

已有 `ForeignKeyRef` 支持 multi-column，无需修改：

```rust
pub struct ForeignKeyRef {
    pub table: String,
    pub column: String,  // 注意：这是单列，复合 FK 需要多行
    pub on_delete: Option<ForeignKeyAction>,
    pub on_update: Option<ForeignKeyAction>,
}
```

**注意**: 复合 FK 需要在 catalog 中存储多条 FK 记录，或扩展 ForeignKeyRef 支持多列。

### 3. Executor 层

FK 约束检查逻辑已有实现，multi-column 支持：

```rust
// 检查复合 FK
fn validate_foreign_key(
    child_row: &Row,
    parent_row: &Row,
    child_cols: &[ColumnId],
    parent_cols: &[ColumnId],
) -> bool {
    child_cols.iter()
        .zip(parent_cols.iter())
        .all(|(c, p)| child_row[c] == parent_row[p])
}
```

## 实现步骤

### Phase 1: Parser 扩展
1. 新增 `TableConstraint` 枚举
2. 扩展 `CreateTableStatement` 添加 `constraints` 字段
3. 实现 `parse_table_constraint()` 方法
4. 实现 `FOREIGN KEY (...) REFERENCES` 解析
5. 实现 `PRIMARY KEY (...)` 表级解析
6. 支持 `ON DELETE/UPDATE` 动作

### Phase 2: Catalog 增强
1. 确保 multi-column FK 正确存储
2. FK 引用完整性验证

### Phase 3: 测试
1. 添加集成测试 `tests/integration/foreign_key_table_constraint_test.rs`
2. 在 `regression_test.rs` 中注册

## 提交计划

| Commit | 内容 |
|--------|------|
| 1 | 新增 TableConstraint 枚举，扩展 CreateTableStatement |
| 2 | Parser 支持 FOREIGN KEY (...) REFERENCES 表级语法 |
| 3 | Parser 支持 PRIMARY KEY 表级语法 |
| 4 | Catalog 层 multi-column FK 支持 |
| 5 | 集成测试 |

## 风险与注意事项

1. **向后兼容**: 现有 `REFERENCES` 列级语法保持不变
2. **AST Normalization**: 考虑将列级 REFERENCES 规范化为 TableConstraint（未来优化）
3. **依赖版本问题**: 当前 Arrow 依赖有编译问题，需先解决（预先存在）

## 验证条件

- [ ] `CREATE TABLE t (a INT, b INT, FOREIGN KEY (a, b) REFERENCES p(x, y))` 解析成功
- [ ] `CREATE TABLE t (a INT, b INT, PRIMARY KEY (a, b))` 解析成功
- [ ] `ON DELETE CASCADE ON UPDATE SET NULL` 正确处理
- [ ] 复合 FK 约束检查正确
- [ ] 现有 FK 测试继续通过
