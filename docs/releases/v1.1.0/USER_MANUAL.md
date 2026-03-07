# SQLRustGo v1.1.0 用户手册

> 版本：v1.1.0
> 发布日期：2026-03-05
> 适用用户：数据库使用者、应用开发者

---

## 一、快速开始

### 1.1 安装

```bash
# 克隆项目
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 构建
cargo build --release

# 运行测试
cargo test
```

### 1.2 启动模式

SQLRustGo 支持两种运行模式：

| 模式 | 命令 | 说明 |
|------|------|------|
| **REPL 模式** |__代码0__| 交互式命令行 |
| **服务器模式** |__代码0__|Client-Server 架构|

---

## 二、REPL 模式

### 2.1 启动 REPL

```bash
cargo run
```

```
SQLRustGo v1.1.0
Type 'help' for commands, 'exit' to quit.

sql>
```

### 2.2 支持的命令

| 命令 | 说明 |
|------|------|
| `help` | 显示帮助信息 |
| `exit` | 退出 REPL |
|__代码0__| 列出所有表 |
|__代码0__| 显示表结构 |

### 2.3 SQL 语法

#### DDL - 数据定义

```sql
-- 创建表
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    age INTEGER,
    email TEXT UNIQUE
);

-- 创建索引
CREATE INDEX idx_users_name ON users(name);

-- 删除表
DROP TABLE users;
```

#### DML - 数据操作

```sql
-- 插入数据
INSERT INTO users (id, name, age, email) VALUES (1, 'Alice', 30, 'alice@example.com');
INSERT INTO users VALUES (2, 'Bob', 25, 'bob@example.com');

-- 查询数据
SELECT * FROM users;
SELECT id, name FROM users WHERE age > 25;
SELECT * FROM users ORDER BY age DESC LIMIT 10;

-- 更新数据
UPDATE users SET age = 31 WHERE id = 1;

-- 删除数据
DELETE FROM users WHERE id = 2;
```

#### JOIN 查询

```sql
-- 创建关联表
CREATE TABLE orders (
    id INTEGER PRIMARY KEY,
    user_id INTEGER,
    amount FLOAT
);

-- Inner Join
SELECT users.name, orders.amount
FROM users
INNER JOIN orders ON users.id = orders.user_id;

-- Left Join
SELECT users.name, orders.amount
FROM users
LEFT JOIN orders ON users.id = orders.user_id;
```

#### 聚合函数

```sql
SELECT COUNT(*) FROM users;
SELECT AVG(age) FROM users;
SELECT SUM(age) FROM users;
SELECT MAX(age), MIN(age) FROM users;
```

---

## 三、Client-Server 模式

### 3.1 启动服务器

```bash
# 启动服务器（默认端口 3306）
cargo run --release -- --server

# 指定端口和数据目录
cargo run --release -- --server --port 3307 --data ./mydata
```

服务器启动后显示：

```
SQLRustGo Server v1.1.0
Listening on 0.0.0.0:3306
Data directory: ./data
```

### 3.2 连接服务器

#### 使用内置客户端

```bash
# 连接本地服务器
cargo run --release -- --client --host 127.0.0.1 --port 3306
```

#### 使用 netcat 测试

```bash
nc 127.0.0.1 3306
```

### 3.3 网络协议

SQLRustGo 使用简化的文本协议：

```
请求格式: SQL语句\n
响应格式: JSON格式的结果
```

#### 示例

```bash
$ echo "SELECT * FROM users" | nc 127.0.0.1 3306
{"columns":["id","name","age"],"rows":[[1,"Alice",30],[2,"Bob",25]],"rows_affected":0}
```

### 3.4 认证（基础）

```sql
-- 创建用户
CREATE USER 'myuser' IDENTIFIED BY 'mypassword';

-- 授权
GRANT ALL ON mydb TO 'myuser';
```

---

## 四、数据持久化

### 4.1 数据存储

数据存储在 `--data` 参数指定的目录中：

```
data/
├── users.json          # 表数据
├── orders.json         # 表数据
├── users_idx_id.json   # 索引数据
└── users_idx_name.json # 索引数据
```

### 4.2 存储格式

v1.1.0 使用 JSON 格式存储数据：

```json
{
  "info": {
    "name": "users",
    "columns": ["id", "name", "age"],
    "column_types": ["INTEGER", "TEXT", "INTEGER"]
  },
  "rows": [
    [1, "Alice", 30],
    [2, "Bob", 25]
  ]
}
```

> ⚠️ **注意**: JSON 存储格式适用于教学和小规模数据，不适合生产环境大数据量场景。
> v1.2.0 将实现二进制存储格式，支持 100万行级数据。

### 4.3 备份与恢复

```bash
# 备份
cp -r data/ data_backup/

# 恢复
cp -r data_backup/ data/
```

---

## 五、性能特性

### 5.1 v1.1.0 性能指标

| 操作 | 性能 | 说明 |
|------|------|------|
| Lexer | ~2 µs | 词法分析 |
|解析器| ~2 µs | 语法分析 |
|选择| ~2 µs | 内存查询 |
|插入| ~2 ms | 持久化写入 |
|哈希连接| 可用 | JOIN 查询 |

### 5.2 索引支持

| 类型 | 支持 | 说明 |
|------|------|------|
| INTEGER 单列索引 | ✅ | B+ 树实现 |
| TEXT 单列索引 | ✅ | Hash 索引 |
| 多列索引 | ❌ | v1.2.0 计划 |
| 唯一索引 | ⚠️ | 部分支持 |

### 5.3 已知限制

| 限制 | 说明 | 计划版本 |
|------|------|----------|
| JSON 存储 | 大数据量性能差 | v1.2.0 |
| 全表扫描 | 无查询优化 | v1.2.0 |
| 无 MVCC | 事务隔离级别有限 | v1.3.0 |
| 单线程存储 | 无并发保护 |v1.1.0-草案|

---

## 六、错误处理

### 6.1 常见错误

| 错误 | 原因 | 解决方法 |
|------|------|----------|
|__代码0__| 表不存在 | 先创建表 |
|__代码0__| 列名错误 | 检查列名 |
|__代码0__| 主键冲突 | 使用不同的主键值 |
|__代码0__| SQL 语法错误 | 检查 SQL 语法 |

### 6.2 错误示例

```sql
sql> SELECT * FROM nonexistent;
Error: Table 'nonexistent' not found

sql> INSERT INTO users (id) VALUES (1);
Error: Column 'name' is NOT NULL but no value provided
```

---

## 七、版本兼容性

### 7.1 系统要求

| 项目 | 要求 |
|------|------|
| Rust | ≥ 1.75 |
| 操作系统 |Linux / macOS / Windows|
| 内存 | ≥ 512 MB |

### 7.2 升级指南

从 v1.0.0 升级到 v1.1.0，请参考 [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md)。

---

## 八、获取帮助

- **文档**: [docs/](../../docs/)
- **Issue**: https://github.com/minzuuniversity/sqlrustgo/issues
- **讨论**: https://github.com/minzuuniversity/sqlrustgo/discussions

---

*本文档由 yinglichina8848 创建*
*最后更新: 2026-03-05*
