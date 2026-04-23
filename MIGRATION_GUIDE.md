# v2.8.0 迁移指南

> **版本**: v2.8.0-alpha  
> **从**: v2.7.0  
> **日期**: 2026-04-23

---

## 一、升级路径

### 兼容性说明

v2.8.0 是 **Alpha** 版本，与 v2.7.0 相比：

- ✅ 向后兼容：大多数 v2.7.0 SQL 语句无需修改即可运行
- ⚠️ 部分破坏性变更：见下方详细说明
- ⚠️ 新功能可能存在未知问题

---

## 二、破坏性变更

### 2.1 版本标识

| 变更项 | v2.7.0 | v2.8.0 |
|--------|--------|--------|
| 版本号 | v2.7.0 | v2.8.0-alpha |
| 状态 | Stable | Alpha |

### 2.2 SQL 语法变化

#### CHECK 约束（新增，兼容）

```sql
-- v2.8.0 新增支持 CHECK 约束
CREATE TABLE users (
    id INT PRIMARY KEY,
    age INT CHECK (age >= 0 AND age <= 150),
    name VARCHAR(100)
);

-- v2.7.0 中 CHECK 被解析但不会强制执行
-- v2.8.0 数据结构已支持，验证逻辑待完整实现
```

#### FULL OUTER JOIN（修复）

```sql
-- v2.7.0 可能不支持或结果不正确
-- v2.8.0 已修复，3/3 测试通过

SELECT a.id, b.name 
FROM table_a a
FULL OUTER JOIN table_b b ON a.id = b.id;
```

#### TRUNCATE/REPLACE INTO（新增）

```sql
-- v2.8.0 新增支持
TRUNCATE TABLE users;

REPLACE INTO users (id, name) VALUES (1, 'Alice');
```

---

## 三、API 变更

### 3.1 Rust API

#### TableInfo 新增字段

```rust
// v2.8.0 新增
pub struct CheckConstraint {
    pub name: String,
    pub expression: String,
}

pub struct TableInfo {
    // ... existing fields ...
    pub check_constraints: Vec<CheckConstraint>,  // 新增
}
```

#### 影响范围

如果您自定义了 `TableInfo` 初始化代码，需要添加 `check_constraints` 字段：

```rust
// v2.7.0
TableInfo {
    name: "users".to_string(),
    columns: vec![],
    // ...
}

// v2.8.0
TableInfo {
    name: "users".to_string(),
    columns: vec![],
    check_constraints: vec![],  // 新增字段
    // ...
}
```

---

## 四、配置变更

### 4.1 新增配置项

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `check_constraint_validation` | bool | false | 是否启用 CHECK 约束验证 |

### 4.2 弃用配置项

暂无。

---

## 五、行为变更

### 5.1 窗口函数

| 函数 | v2.7.0 | v2.8.0 |
|------|--------|--------|
| ROW_NUMBER | ✅ | ✅ |
| RANK | ✅ | ✅ |
| DENSE_RANK | ✅ | ✅ |
| LEAD/LAG | ⚠️ 部分 | ⚠️ 部分 |

### 5.2 分区表

| 分区类型 | v2.7.0 | v2.8.0 |
|----------|--------|--------|
| RANGE | ✅ | ✅ |
| LIST | ✅ | ✅ |
| HASH | ✅ | ✅ |
| KEY | ⏳ 未开始 | ⏳ 未开始 |

---

## 六、升级步骤

### 6.1 从 v2.7.0 升级

1. **备份数据**
   ```bash
   # 备份数据库
   cargo run --release --bin sqlrustgo-cli -- backup /path/to/db
   ```

2. **停止服务**
   ```bash
   # 停止所有 sqlrustgo 实例
   ```

3. **更新二进制**
   ```bash
   cargo build --release
   ```

4. **验证兼容性**
   ```bash
   # 运行迁移测试
   cargo test --test migration_tests
   ```

5. **启动服务**
   ```bash
   cargo run --release --bin sqlrustgo-server
   ```

### 6.2 回滚步骤

如遇问题，回滚到 v2.7.0：

1. 停止 v2.8.0 服务
2. 恢复 v2.7.0 二进制
3. 从备份恢复数据
4. 报告问题：https://github.com/minzuuniversity/sqlrustgo/issues

---

## 七、已知问题

| Issue | 描述 | 状态 | 解决方式 |
|-------|------|------|----------|
| #1791 | 触发器测试编译错误 | ⏳ 处理中 | 暂时跳过触发器功能 |
| - | CHECK 约束验证逻辑 | ⚠️ 部分实现 | 数据结构已添加，验证待完成 |

---

## 八、获取帮助

- **文档**: https://github.com/minzuuniversity/sqlrustgo/tree/develop/v2.8.0/docs
- **Issue**: https://github.com/minzuuniversity/sqlrustgo/issues
- **Discussions**: https://github.com/minzuuniversity/sqlrustgo/discussions
