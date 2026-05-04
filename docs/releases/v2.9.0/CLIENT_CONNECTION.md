# SQLRustGo 客户端连接指南

> **版本**: v2.9.0
> **代号**: 企业级韧性 (Enterprise Resilience)
> **更新日期**: 2026-05-04

---

## 1. 概述

SQLRustGo 支持多种客户端连接方式，兼容 MySQL 5.7/8.0 协议。

### 连接方式概览

| 连接方式 | 协议 | 端口 | 适用场景 |
|----------|------|------|----------|
| MySQL CLI | MySQL Wire Protocol | 3306 | 命令行操作 |
| TCP (标准驱动) | MySQL Wire Protocol | 3306 | 应用程序 |
| REST API | HTTP | 8080 | Web 应用、跨语言 |

---

## 2. 启动服务器

### 2.1 MySQL 协议服务器

```bash
# 启动 MySQL 协议服务器 (端口 3306)
cargo run --release --bin sqlrustgo-mysql-server

# 指定主机和端口
cargo run --release --bin sqlrustgo-mysql-server -- --host 0.0.0.0 --port 3306
```

### 2.2 REST API 服务器

```bash
# 启动 HTTP REST API 服务器 (端口 8080)
cargo run --release --bin sqlrustgo-server
```

---

## 3. MySQL CLI 连接

### 3.1 使用 mysql 客户端连接

```bash
# 基本连接
mysql -h 127.0.0.1 -P 3306 -u root

# 指定数据库
mysql -h 127.0.0.1 -P 3306 -u root -D default

# 交互式模式
mysql -h 127.0.0.1 -P 3306 -u root -p
```

### 3.2 连接参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `-h` | 服务器地址 | 127.0.0.1 |
| `-P` | 服务器端口 | 3306 |
| `-u` | 用户名 | root |
| `-p` | 密码 (可选) | 无 |
| `-D` | 默认数据库 | 无 |

### 3.3 连接示例

```bash
# 启动服务器
$ cargo run --release --bin sqlrustgo-mysql-server
SQLRustGo MySQL Server v2.9.0
Listening on 127.0.0.1:3306

# 新终端连接
$ mysql -h 127.0.0.1 -P 3306 -u root
Welcome to the MySQL monitor. Commands end with ; or \g.

mysql> CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(100));
Query OK, 0 rows affected (0.01 sec)

mysql> INSERT INTO users VALUES (1, 'Alice');
Query OK, 1 row affected (0.00 sec)

mysql> SELECT * FROM users;
+----+-------+
| id | name  |
+----+-------+
|  1 | Alice |
+----+-------+
1 row in set (0.00 sec)
```

---

## 4. TCP 连接 (应用程序)

### 4.1 Python (mysql-connector-python)

```python
import mysql.connector

conn = mysql.connector.connect(
    host='127.0.0.1',
    port=3306,
    user='root',
    database='default'
)

cursor = conn.cursor()
cursor.execute("SELECT * FROM users")

for row in cursor.fetchall():
    print(row)

conn.close()
```

### 4.2 Node.js (mysql2)

```javascript
const mysql = require('mysql2');

const connection = mysql.createConnection({
    host: '127.0.0.1',
    port: 3306,
    user: 'root',
    database: 'default'
});

connection.query('SELECT * FROM users', (err, rows) => {
    if (err) throw err;
    console.log(rows);
});

connection.end();
```

### 4.3 Go (go-sql-driver/mysql)

```go
package main

import (
    "database/sql"
    "fmt"
    _ "github.com/go-sql-driver/mysql"
)

func main() {
    db, err := sql.Open("mysql", "root:@tcp(127.0.0.1:3306)/default")
    if err != nil {
        panic(err)
    }
    defer db.Close()

    rows, err := db.Query("SELECT * FROM users")
    if err != nil {
        panic(err)
    }
    defer rows.Close()

    for rows.Next() {
        var id int
        var name string
        rows.Scan(&id, &name)
        fmt.Printf("ID: %d, Name: %s\n", id, name)
    }
}
```

### 4.4 Rust (sqlx)

```rust
use sqlx::{mysql::MySqlPoolOptions, Row};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect("mysql://root@127.0.0.1:3306/default")
        .await?;

    let rows = sqlx::query("SELECT id, name FROM users")
        .fetch_all(&pool)
        .await?;

    for row in rows {
        let id: i32 = row.get("id");
        let name: String = row.get("name");
        println!("ID: {}, Name: {}", id, name);
    }

    Ok(())
}
```

### 4.5 Java (JDBC)

```java
import java.sql.*;

public class SqlrustgoConnection {
    public static void main(String[] args) {
        String url = "jdbc:mysql://127.0.0.1:3306/default";
        String user = "root";
        String password = "";

        try (Connection conn = DriverManager.getConnection(url, user, password);
             Statement stmt = conn.createStatement()) {

            ResultSet rs = stmt.executeQuery("SELECT * FROM users");
            while (rs.next()) {
                System.out.println("ID: " + rs.getInt("id") + ", Name: " + rs.getString("name"));
            }
        } catch (SQLException e) {
            e.printStackTrace();
        }
    }
}
```

---

## 5. REST API 连接

### 5.1 API 端点

| 方法 | 端点 | 说明 |
|------|------|------|
| GET | /health | 健康检查 |
| GET | /metrics | Prometheus 指标 |
| POST | /query | 执行 SQL 查询 |

### 5.2 REST API 使用示例

```bash
# 健康检查
curl http://localhost:8080/health

# 执行查询
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users"}'
```

详细 API 文档请参考 [API_REFERENCE.md](./API_REFERENCE.md)。

---

## 6. 分布式架构 (v2.9.0)

### 6.1 Semi-sync 复制

```sql
-- 配置 semi-sync 复制
SET GLOBAL rpl_semi_sync_master_enabled = 1;
SET GLOBAL rpl_semi_sync_slave_enabled = 1;

-- 查看复制状态
SHOW SLAVE STATUS\G
SHOW MASTER STATUS\G
```

### 6.2 XA 事务

```sql
-- 两阶段提交事务
XA BEGIN 'txn_001';
UPDATE accounts SET balance = balance - 100 WHERE id = 1;
UPDATE accounts SET balance = balance + 100 WHERE id = 2;
XA PREPARE 'txn_001';
XA COMMIT 'txn_001';
```

---

## 7. 连接故障排查

### 7.1 常见错误

| 错误 | 原因 | 解决方案 |
|------|------|----------|
| `Can't connect to MySQL server` | 服务器未启动 | 启动 `sqlrustgo-mysql-server` |
| `Access denied` | 认证失败 | 检查用户名密码 |
| `Unknown database` | 数据库不存在 | 创建数据库 |

### 7.2 调试步骤

```bash
# 1. 检查服务器是否运行
ps aux | grep sqlrustgo

# 2. 检查端口监听
netstat -an | grep 3306

# 3. 测试 TCP 连接
telnet 127.0.0.1 3306
```

---

## 8. 相关文档

- [快速开始](./QUICK_START.md)
- [REST API 参考](./API_REFERENCE.md)
- [安全报告](./SECURITY_REPORT.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-05-04*