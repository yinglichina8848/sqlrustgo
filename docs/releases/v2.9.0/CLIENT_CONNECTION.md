# SQLRustGo 客户端连接指南

> **版本**: v2.9.0
> **代号**: Enterprise Resilience
> **更新日期**: 2026-05-05

---

## 1. 概述

SQLRustGo 支持多种客户端连接方式，兼容 MySQL 5.7 协议。您可以使用标准的 MySQL 客户端工具、ODBC、JDBC 等方式连接 SQLRustGo 数据库。v2.9.0 版本新增了分布式连接支持（Semi-sync、MTS、Multi-source、XA 事务）。

### 连接方式概览

| 连接方式 | 协议 | 端口 | 驱动要求 | 适用场景 |
|----------|------|------|----------|----------|
| MySQL CLI | MySQL Wire Protocol | 3306 | mysql-client | 命令行操作 |
| ODBC | MySQL Wire Protocol | 3306 | MySQL ODBC Driver | Windows 应用 |
| JDBC | MySQL Wire Protocol | 3306 | MySQL Connector/J | Java 应用 |
| REST API | HTTP | 8080 | 无 | Web 应用、跨语言 |

### v2.9.0 新增分布式连接

| 连接方式 | 协议 | 端口 | 说明 |
|----------|------|------|------|
| XA 事务 | MySQL Wire Protocol | 3306 | 两阶段提交分布式事务 |
| Semi-sync | MySQL Wire Protocol | 3306/3307 | 半同步复制确保数据同步 |
| MTS | MySQL Wire Protocol | 3307 | 多线程从库复制 |

---

## 2. 启动服务器

### 2.1 MySQL 协议服务器

```bash
# 启动 MySQL 协议服务器 (端口 3306)
cargo run --release --bin sqlrustgo-mysql-server

# 指定主机和端口
sqlrustgo-mysql-server --host 0.0.0.0 --port 3306
```

### 2.2 REST API 服务器

```bash
# 启动 HTTP REST API 服务器 (端口 8080)
cargo run --release --bin sqlrustgo-server
```

### 2.3 分布式服务器

```bash
# 启动主节点 (支持 Semi-sync 复制)
cargo run --release --bin sqlrustgo-server --role primary --port 3306

# 启动从节点 (MTS 并行复制)
cargo run --release --bin sqlrustgo-server --role replica --port 3307 --source 127.0.0.1:3306
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
Your MySQL connection id is 1

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

## 4. ODBC 连接

### 4.1 安装 MySQL ODBC 驱动

**Windows:**
1. 下载 [MySQL ODBC Connector](https://dev.mysql.com/downloads/connector/odbc/)
2. 安装并配置 ODBC 数据源

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install unixodbc unixodbc-dev
sudo apt-get install libmyodbc
```

**macOS:**
```bash
brew install mysql-connector-c
```

### 4.2 配置 ODBC 数据源

**Linux (odbc.ini):**
```ini
[SQLRustGo]
Description = SQLRustGo Database
Driver = /usr/lib/x86_64-linux-gnu/odbc/libmyodbc.so
Server = 127.0.0.1
Port = 3306
Database = default
User = root
Password =
```

### 4.3 ODBC 连接字符串

```
Driver={MySQL ODBC 8.0 Driver};Server=127.0.0.1;Port=3306;Database=default;User=root;Password=
```

### 4.4 使用示例

**Python (pyodbc):**
```python
import pyodbc

conn = pyodbc.connect(
    'DRIVER={MySQL ODBC 8.0 Driver};'
    'SERVER=127.0.0.1;'
    'PORT=3306;'
    'DATABASE=default;'
    'USER=root;'
    'PASSWORD='
)

cursor = conn.cursor()
cursor.execute("SELECT * FROM users")

for row in cursor.fetchall():
    print(row)
```

---

## 5. JDBC 连接

### 5.1 添加 MySQL Connector/J 依赖

**Maven:**
```xml
<dependency>
    <groupId>mysql</groupId>
    <artifactId>mysql-connector-java</artifactId>
    <version>8.0.33</version>
</dependency>
```

**Gradle:**
```groovy
implementation 'mysql:mysql-connector-java:8.0.33'
```

### 5.2 JDBC 连接字符串

```
jdbc:mysql://127.0.0.1:3306/default?useSSL=false&allowPublicKeyRetrieval=true
```

### 5.3 Java 使用示例

```java
import java.sql.*;

public class SqlrustgoConnection {
    public static void main(String[] args) {
        String url = "jdbc:mysql://127.0.0.1:3306/default";
        String user = "root";
        String password = "";

        try (Connection conn = DriverManager.getConnection(url, user, password);
             Statement stmt = conn.createStatement()) {

            // 创建表
            stmt.execute("CREATE TABLE IF NOT EXISTS users (id INT PRIMARY KEY, name VARCHAR(100))");

            // 插入数据
            stmt.execute("INSERT INTO users VALUES (1, 'Alice')");

            // 查询
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

### 5.4 Spring Boot 配置

```yaml
spring:
  datasource:
    url: jdbc:mysql://127.0.0.1:3306/default?useSSL=false&allowPublicKeyRetrieval=true
    driver-class-name: com.mysql.cj.jdbc.Driver
    username: root
    password:
```

---

## 6. 其他编程语言连接

### 6.1 Python (mysql-connector-python)

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

### 6.2 Node.js (mysql2)

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

### 6.3 Go (go-sql-driver/mysql)

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

### 6.4 Rust (sqlx)

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

---

## 7. REST API 连接

### 7.1 启动 REST API 服务器

```bash
cargo run --release --bin sqlrustgo-server
```

### 7.2 API 端点

| 方法 | 端点 | 说明 |
|------|------|------|
| GET | /health | 健康检查 |
| GET | /metrics | Prometheus 指标 |
| GET | /ready | 就绪检查 |
| POST | /query | 执行 SQL 查询 |
| POST | /query/params | 带参数的查询 |
| GET | /tables | 列出表 |
| GET | /tables/{name} | 表结构 |

### 7.3 REST API 使用示例

```bash
# 健康检查
curl http://localhost:8080/health

# 执行查询
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users"}'

# 执行 DDL
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"sql": "CREATE TABLE t (id INT PRIMARY KEY)"}'
```

详细 API 文档请参考 [API_REFERENCE.md](./API_REFERENCE.md)。

---

## 8. 分布式连接 (v2.9.0)

### 8.1 Semi-sync 复制连接

半同步复制确保数据在主从之间同步后才返回成功。

```bash
# 连接到主节点 (读写)
mysql -h 127.0.0.1 -P 3306 -u root

# 连接到从节点 (只读，自动接收 Semi-sync 复制)
mysql -h 127.0.0.1 -P 3307 -u root
```

**配置 Semi-sync 复制：**

```toml
[replication]
type = "semi-sync"
source = "192.168.0.100:3306"
ack_timeout = 10  # 秒
```

### 8.2 MTS (Multi-Threaded Slave) 并行复制

```bash
# 启动 MTS 从节点
sqlrustgo-server --role replica --port 3307 \
  --source 127.0.0.1:3306 \
  --mts-workers 4
```

### 8.3 Multi-source 复制

连接多个主源进行复制：

```bash
# 启动多源复制从节点
sqlrustgo-server --role replica --port 3308 \
  --source-list 192.168.0.101:3306,192.168.0.102:3306
```

### 8.4 XA 事务连接

两阶段提交分布式事务支持：

```sql
-- 启动 XA 事务
XA START 'transaction_id';

-- 执行 SQL
INSERT INTO orders (id, amount) VALUES (1, 100);

-- 结束 XA 事务
XA END 'transaction_id';

-- 准备提交
XA PREPARE 'transaction_id';

-- 提交 XA 事务
XA COMMIT 'transaction_id';
```

**Java/JDBC XA 示例：**

```java
import javax.sql.XAConnection;
import javax.transaction.xa.*;

public class SqlrustgoXA {
    public static void main(String[] args) throws Exception {
        // 获取 XA 数据源
        javax.sql.DataSource ds = new com.mysql.cj.jdbc.MysqlXADataSource();
        ((com.mysql.cj.jdbc.MysqlXADataSource) ds).setUrl("jdbc:mysql://127.0.0.1:3306/default");

        XAConnection xaConn = ds.getXAConnection();
        XAResource xaRes = xaConn.getXAResource();

        // 启动 XA 事务
        Xid xid = new MyXid(1, new byte[]{0x01}, new byte[]{0x02});
        xaRes.start(xid, XAResource.TMNOFLAGS);

        // 执行 SQL
        // ...

        // 结束 XA 事务
        xaRes.end(xid, XAResource.TMSUCCESS);

        // 准备提交
        xaRes.prepare(xid);

        // 提交
        xaRes.commit(xid, false);

        xaConn.close();
    }
}
```

---

## 9. 主从复制连接 (v2.8.0 兼容)

### 9.1 复制架构

```
┌─────────────┐      ┌─────────────┐
│   Primary   │ ───> │   Replica   │
│  (主节点)   │ GTID │  (从节点)   │
└─────────────┘      └─────────────┘
```

### 9.2 连接主节点

```bash
# 连接到主节点 (读写)
mysql -h 127.0.0.1 -P 3306 -u root
```

### 9.3 连接从节点

```bash
# 连接到从节点 (只读)
mysql -h 127.0.0.1 -P 3307 -u root
```

---

## 10. 连接故障排查

### 10.1 常见错误

| 错误 | 原因 | 解决方案 |
|------|------|----------|
| `Can't connect to MySQL server` | 服务器未启动 | 启动 `sqlrustgo-mysql-server` |
| `Access denied` | 认证失败 | 检查用户名密码 |
| `Unknown database` | 数据库不存在 | 创建数据库 |
| `Connection timed out` | 防火墙阻止 | 检查防火墙设置 |
| `XAER_NOTA` | XA 事务 ID 不存在 | 检查事务状态 |

### 10.2 调试步骤

```bash
# 1. 检查服务器是否运行
ps aux | grep sqlrustgo

# 2. 检查端口监听
netstat -an | grep 3306

# 3. 测试 TCP 连接
telnet 127.0.0.1 3306

# 4. 查看服务器日志
tail -f /var/log/sqlrustgo/server.log

# 5. 检查 XA 事务状态
mysql -h 127.0.0.1 -P 3306 -u root -e "XA RECOVER"
```

---

## 11. 连接参数参考

### 11.1 MySQL Wire Protocol 参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `max_allowed_packet` | 最大数据包大小 | 64MB |
| `net_buffer_length` | 网络缓冲区大小 | 16KB |
| `connect_timeout` | 连接超时 | 10s |
| `read_timeout` | 读取超时 | 0 (无限制) |
| `write_timeout` | 写入超时 | 0 (无限制) |

### 11.2 SSL/TLS 配置

```bash
# 启用 SSL (需要配置证书)
sqlrustgo-mysql-server --tls-cert /path/to/server.crt --tls-key /path/to/server.key

# JDBC SSL 连接
jdbc:mysql://127.0.0.1:3306/default?useSSL=true&requireSSL=true
```

### 11.3 分布式复制参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `semi_sync_timeout` | Semi-sync ACK 超时 | 10s |
| `mts_workers` | MTS 工作线程数 | 4 |
| `replica_delay` | 复制延迟 (秒) | 0 |

---

## 12. 相关文档

- [快速开始](./QUICK_START.md)
- [REST API 参考](./API_REFERENCE.md)
- [分布式设计](./DISTRIBUTED_DESIGN.md)
- [安全加固指南](./SECURITY_HARDENING.md)
- [用户手册](./USER_MANUAL.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-05-05*
