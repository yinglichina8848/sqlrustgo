# SQLRustGo v2.9.0 Rust API 使用示例

> **版本**: v2.9.0
> **更新日期**: 2026-05-05

---

## 一、核心 Crate 导入

```rust
use sqlrustgo::prelude::*;
use sqlrustgo::parser::{Parser, SelectStmt};
use sqlrustgo::executor::{ExecutionEngine, ResultSet};
use sqlrustgo::storage::{BufferPool, PageId};
use sqlrustgo::transaction::{Transaction, TransactionManager};
```

---

## 二、基础连接

### 2.1 创建连接

```rust
use sqlrustgo::connection::Connection;

let conn = Connection::new("data/sqlrustgo.db")?;
println!("Connected to database");
```

### 2.2 执行查询

```rust
let result = conn.execute("SELECT * FROM users WHERE id = 1")?;
for row in result {
    println!("id: {}, name: {}", row.get("id")?, row.get("name")?);
}
```

---

## 三、事务管理

### 3.1 开启事务

```rust
let mut tx = conn.begin()?;
println!("Transaction started: {}", tx.id());
```

### 3.2 提交事务

```rust
tx.commit()?;
println!("Transaction committed");
```

### 3.3 回滚事务

```rust
tx.rollback()?;
println!("Transaction rolled back");
```

---

## 四、DDL 操作

### 4.1 CREATE TABLE

```rust
conn.execute(
    "CREATE TABLE users (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        email TEXT UNIQUE
    )"
)?;
println!("Table created");
```

### 4.2 DROP TABLE

```rust
conn.execute("DROP TABLE IF EXISTS users")?;
println!("Table dropped");
```

---

## 五、窗口函数

### 5.1 ROW_NUMBER

```rust
let result = conn.execute(
    "SELECT
        name,
        department,
        salary,
        ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary DESC) as rank
     FROM employees"
)?;
```

### 5.2 RANK / DENSE_RANK

```rust
let result = conn.execute(
    "SELECT
        product_name,
        category,
        RANK() OVER (PARTITION BY category ORDER BY sales DESC) as rank,
        DENSE_RANK() OVER (PARTITION BY category ORDER BY sales DESC) as dense_rank
     FROM products"
)?;
```

---

## 六、CTE (WITH)

### 6.1 非递归 CTE

```rust
let result = conn.execute(
    "WITH regional_sales AS (
        SELECT region, SUM(amount) AS total_sales
        FROM orders
        GROUP BY region
    )
    SELECT region, total_sales
    FROM regional_sales
    WHERE total_sales > 10000"
)?;
```

### 6.2 递归 CTE

```rust
let result = conn.execute(
    "WITH RECURSIVE org_tree AS (
        SELECT id, name, manager_id, 0 AS depth
        FROM employees
        WHERE manager_id IS NULL
        UNION ALL
        SELECT e.id, e.name, e.manager_id, ot.depth + 1
        FROM employees e
        JOIN org_tree ot ON e.manager_id = ot.id
    )
    SELECT * FROM org_tree"
)?;
```

---

## 七、JSON 操作

### 7.1 JSON 提取

```rust
let result = conn.execute(
    "SELECT
        id,
        json_extract(data, '$.author') as author,
        json_extract(data, '$.tags[0]') as first_tag
     FROM articles"
)?;
```

---

## 八、性能调优

### 8.1 设置缓冲区大小

```rust
let engine = ExecutionEngine::new()
    .with_buffer_pool_size(2048)  // MB
    .with_max_connections(256);
```

### 8.2 启用 WAL

```rust
let engine = ExecutionEngine::new()
    .with_wal_enabled(true)
    .with_checkpoint_interval(500);
```

---

*本文档由 Hermes Agent 维护*
*更新日期: 2026-05-05*
