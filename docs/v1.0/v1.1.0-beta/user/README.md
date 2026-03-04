# SQLRustGo 用户指南

## 1. 快速开始

### 1.1 安装依赖

```bash
# 确保已安装 Rust
rustc --version
cargo --version
```

### 1.2 编译项目

```bash
# 克隆项目
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 编译
cargo build --release
```

### 1.3 运行 REPL

```bash
# 启动交互式命令行
cargo run --bin sqlrustgo

# 或者使用编译后的二进制
./target/release/sqlrustgo
```

## 2. REPL 使用

### 2.1 基本命令

```sql
-- 创建表
CREATE TABLE users (id INTEGER, name TEXT);

-- 插入数据
INSERT INTO users VALUES (1, 'Alice');
INSERT INTO users VALUES (2, 'Bob');

-- 查询数据
SELECT * FROM users;
SELECT id, name FROM users WHERE id = 1;

-- 更新数据
UPDATE users SET name = 'Charlie' WHERE id = 1;

-- 删除数据
DELETE FROM users WHERE id = 2;

-- 删除表
DROP TABLE users;
```

### 2.2 退出 REPL

```sql
-- 输入 quit 或 exit
QUIT;
```

## 3. SQL 支持

### 3.1 支持的 SQL 语句

| 语句 | 示例 |
|------|------|
| CREATE TABLE | `CREATE TABLE t (id INTEGER, name TEXT);` |
| INSERT | `INSERT INTO t VALUES (1, 'text');` |
| SELECT | `SELECT * FROM t WHERE id > 0;` |
| UPDATE | `UPDATE t SET name = 'new' WHERE id = 1;` |
| DELETE | `DELETE FROM t WHERE id = 1;` |
| DROP TABLE | `DROP TABLE t;` |

### 3.2 支持的数据类型

- `INTEGER` / `INT` - 64位整数
- `TEXT` / `VARCHAR` - 文本字符串
- `FLOAT` / `DOUBLE` - 浮点数
- `BOOLEAN` / `BOOL` - 布尔值
- `BLOB` - 二进制数据

### 3.3 聚合函数

```sql
SELECT COUNT(*) FROM users;
SELECT SUM(amount) FROM orders;
SELECT AVG(price) FROM products;
SELECT MIN(age) FROM users;
SELECT MAX(score) FROM tests;
```

## 4. 服务器模式

### 4.1 启动服务器

```bash
# 启动 TCP 服务器（默认端口 3306）
cargo run --bin sqlrustgo-server
```

### 4.2 连接到服务器

使用 MySQL 客户端连接：

```bash
mysql -h 127.0.0.1 -P 3306 -u root -p
```

## 5. 配置

### 5.1 数据目录

数据默认存储在 `data/` 目录。

### 5.2 日志

运行时日志输出到 stdout。

## 6. 常见问题

### 6.1 如何查看表结构？

```sql
-- 当前版本使用 SELECT 查看数据
SELECT * FROM table_name LIMIT 1;
```

### 6.2 如何备份数据？

复制 `data/` 目录即可备份。

### 6.3 如何重置数据库？

```bash
# 停止服务
# 删除数据目录
rm -rf data/*
# 重启服务
```

## 7. 退出

在 REPL 中输入以下命令退出：

```sql
QUIT;
-- 或
EXIT;
-- 或按 Ctrl+C
```
