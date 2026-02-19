# 覆盖率提升优先级

> 测试覆盖率提升策略

---

## 优先级排序

| 优先级 | 模块 | 原因 |
|--------|------|------|
| 1 | parser | 核心，输入验证 |
| 2 | executor | 核心，执行逻辑 |
| 3 | planner | 核心，查询规划 |
| 4 | storage | 核心，数据持久化 |
| 5 | network | 重要，通信层 |
| 6 | auth | 重要，安全层 |

---

## 覆盖率目标

| 模块 | 当前 | 目标 |
|------|------|------|
| parser | - | ≥ 80% |
| executor | - | ≥ 80% |
| planner | - | ≥ 80% |
| storage | - | ≥ 80% |
| network | - | ≥ 75% |
| auth | - | ≥ 75% |
| **整体** | - | **≥ 75%** |

---

## 提升策略

### 1. 边界测试优先

```rust
#[test]
fn test_empty_sql() {
    let result = parse("");
    assert!(result.is_err());
}

#[test]
fn test_invalid_sql() {
    let result = parse("INVALID");
    assert!(result.is_err());
}

#[test]
fn test_null_value() {
    let result = execute("SELECT NULL");
    assert!(result.is_ok());
}

#[test]
fn test_long_input() {
    let long_sql = "SELECT ".to_string() + &"a, ".repeat(1000);
    let result = parse(&long_sql);
    assert!(result.is_ok());
}
```

### 2. 错误路径测试

大多数人只测成功路径，应该补充错误路径：

```rust
#[test]
fn test_table_not_found() {
    let result = execute("SELECT * FROM nonexistent");
    assert!(matches!(result, Err(DbError::TableNotFound(_))));
}

#[test]
fn test_column_not_found() {
    let result = execute("SELECT nonexistent FROM users");
    assert!(matches!(result, Err(DbError::ColumnNotFound(_))));
}
```

### 3. 覆盖 panic 风险点

```rust
// Option 访问
#[test]
fn test_option_none() {
    let result = get_optional_value(None);
    assert!(result.is_err());
}

// Vec 索引
#[test]
fn test_index_out_of_bounds() {
    let result = get_element(&vec![], 0);
    assert!(result.is_err());
}

// HashMap get
#[test]
fn test_key_not_found() {
    let result = lookup(&HashMap::new(), "key");
    assert!(result.is_err());
}
```

---

## 测试模板

### 解析器测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_select() {
        let result = parse("SELECT * FROM users");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_insert() {
        let result = parse("INSERT INTO users VALUES (1, 'test')");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid() {
        let result = parse("INVALID SQL");
        assert!(result.is_err());
    }
}
```

### 执行器测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Database {
        let mut db = Database::new();
        db.execute("CREATE TABLE test (id INT, name TEXT)").unwrap();
        db
    }

    #[test]
    fn test_execute_select_empty() {
        let db = setup();
        let result = db.execute("SELECT * FROM test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_insert() {
        let db = setup();
        let result = db.execute("INSERT INTO test VALUES (1, 'test')");
        assert!(result.is_ok());
    }
}
```

---

## 覆盖率检查命令

```bash
# 生成覆盖率报告
cargo llvm-cov --all-features --workspace --ignore-filename-regex="(tests|benches)"

# 生成 HTML 报告
cargo llvm-cov --all-features --workspace --html

# 检查特定模块
cargo llvm-cov --all-features --workspace --ignore-filename-regex="(tests|benches)" | grep parser
```

---

## CI 配置

```yaml
- name: Coverage Check
  run: |
    cargo llvm-cov --all-features --workspace --ignore-filename-regex="(tests|benches)"
    
- name: Coverage Report
  run: |
    cargo llvm-cov --all-features --workspace --html
```

---

## 验收标准

- [ ] 核心模块覆盖率 ≥ 80%
- [ ] 整体覆盖率 ≥ 75%
- [ ] 无新增代码未覆盖
- [ ] 错误路径有测试
