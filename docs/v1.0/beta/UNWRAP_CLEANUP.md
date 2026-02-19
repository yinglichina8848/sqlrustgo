# Unwrap 清理策略

> 生产代码 unwrap/panic 清理指南

---

## 定位方法

```bash
# 查找 unwrap
rg "\.unwrap\(" -n src --type rust

# 查找 expect
rg "\.expect\(" -n src --type rust

# 查找 panic
rg "panic!" -n src --type rust
```

---

## 分类处理

### A 类：可安全传播错误

使用 `?` 操作符传播错误。

```rust
// 原代码
let result = some_function().unwrap();

// 修改后
let result = some_function()?;
```

### B 类：Option 类型

使用 `ok_or()` 转换为 Result。

```rust
// 原代码
let value = map.get(key).unwrap();

// 修改后
let value = map
    .get(key)
    .ok_or(DbError::ExecutionError("key not found".into()))?;
```

### C 类：测试代码

保留，但添加注释标记。

```rust
// allow unwrap in test
let result = some_option.unwrap();
```

---

## 标准替换模式

### 模式 1：Option → Result

```rust
// 原
let value = map.get(key).unwrap();

// 改
let value = map
    .get(key)
    .ok_or(DbError::ExecutionError("key not found".into()))?;
```

### 模式 2：Result unwrap

```rust
// 原
let parsed = parse(input).unwrap();

// 改
let parsed = parse(input)?;
```

### 模式 3：错误转换

```rust
fn parse_sql(sql: &str) -> DbResult<Ast> {
    parser::parse(sql)
        .map_err(|e| DbError::ParserError(e.to_string()))
}
```

---

## CI 防回归

```yaml
- name: Forbid unwrap in src
  run: |
    if grep -R "unwrap(" src --include="*.rs" | grep -v "// allow unwrap"; then
      echo "unwrap found in production code"
      exit 1
    fi

- name: Forbid panic in src
  run: |
    if grep -R "panic!" src --include="*.rs" | grep -v "// allow panic"; then
      echo "panic found in production code"
      exit 1
    fi
```

---

## 验收标准

- [ ] `grep -rn "\.unwrap()" src` 返回空（除测试）
- [ ] `grep -rn "panic!" src` 返回空（除测试）
- [ ] CI 检查通过
- [ ] 所有测试通过

---

## 常见场景

### HashMap 访问

```rust
// 不安全
let v = map[&key];

// 安全
let v = map.get(&key).ok_or(DbError::NotFound)?;
```

### Vec 索引

```rust
// 不安全
let item = vec[i];

// 安全
let item = vec.get(i).ok_or(DbError::IndexOutOfBounds)?;
```

### 整数解析

```rust
// 不安全
let n: i32 = s.parse().unwrap();

// 安全
let n: i32 = s.parse().map_err(|e| DbError::ParseError(e.to_string()))?;
```

---

## 测试代码例外

测试代码允许使用 unwrap，但必须添加注释：

```rust
#[test]
fn test_something() {
    // allow unwrap in test
    let result = some_function().unwrap();
    assert_eq!(result, expected);
}
```
