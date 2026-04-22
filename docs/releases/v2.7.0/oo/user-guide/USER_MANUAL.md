# SQLRustGo v2.7.0 用户手册

> **版本**: v2.7.0 (GA)
> **更新日期**: 2026-04-22

---

## 1. 快速开始

### 1.1 构建

```bash
# Debug 构建
cargo build

# Release 构建
cargo build --release

# 全特性构建
cargo build --all-features
```

### 1.2 运行

```bash
# 启动 REPL
cargo run --release

# 运行测试
cargo test --workspace
```

---

## 2. SQL 语法

### 2.1 基础操作

```sql
-- 创建表
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(255)
);

-- 插入数据
INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com');

-- 查询
SELECT * FROM users;

-- 更新
UPDATE users SET name = 'Bob' WHERE id = 1;

-- 删除
DELETE FROM users WHERE id = 1;
```

### 2.2 聚合查询

```sql
-- 聚合函数
SELECT COUNT(*) FROM orders;
SELECT SUM(amount) FROM orders;
SELECT AVG(price) FROM products;
SELECT MIN(created_at) FROM events;
SELECT MAX(score) FROM games;

-- 分组
SELECT department, AVG(salary) FROM employees GROUP BY department;

-- 分组过滤
SELECT department, AVG(salary) as avg_sal
FROM employees
GROUP BY department
HAVING AVG(salary) > 50000;
```

### 2.3 JOIN

```sql
-- 内连接
SELECT u.name, o.order_id
FROM users u
INNER JOIN orders o ON u.id = o.user_id;

-- 左连接
SELECT u.name, o.order_id
FROM users u
LEFT JOIN orders o ON u.id = o.user_id;

-- 右连接
SELECT u.name, o.order_id
FROM users u
RIGHT JOIN orders o ON u.id = o.user_id;
```

---

## 3. 企业级特性

### 3.1 WAL 崩溃恢复

WAL (预写日志) 自动启用，确保崩溃后数据可恢复：

```sql
-- 事务操作会自动记录到 WAL
BEGIN;
INSERT INTO users (id, name) VALUES (1, 'Alice');
COMMIT;
-- 如果系统崩溃，重启后会从 WAL 恢复
```

### 3.2 外键约束

```sql
-- 创建带外键的表
CREATE TABLE orders (
    id INTEGER PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    amount DECIMAL(10,2)
);

-- 级联删除
CREATE TABLE orders (
    id INTEGER PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE
);
```

### 3.3 备份恢复

```bash
# 备份
./target/release/sqlrustgo backup --output backup.db

# 恢复
./target/release/sqlrustgo restore --input backup.db
```

---

## 4. 检索功能

### 4.1 全文检索 (lex)

```sql
-- 全文搜索
SELECT * FROM documents
WHERE MATCH(content, 'keywords');
```

### 4.2 向量检索 (vec)

```sql
-- 语义搜索
SELECT * FROM documents
WHERE VECTOR_SEARCH(content, embedding, top_k=10);
```

### 4.3 图检索 (graph)

```sql
-- 关系查询
SELECT * FROM graph
WHERE PATH(start_id, end_id, hops=3);
```

### 4.4 混合检索 (hybrid)

```sql
-- 混合搜索 (RRF)
SELECT * FROM documents
WHERE HYBRID_SEARCH(
    content,
    embedding,
    strategy='RRF',
    weights=[0.3, 0.7]
);
```

---

## 5. GMP 审计支持

### 5.1 审计查询

```sql
-- GMP Top 10 审核查询
SELECT * FROM gmp_audit_log
WHERE action_type = 'SENSITIVE_ACCESS'
ORDER BY timestamp DESC
LIMIT 10;
```

### 5.2 证据链查询

```sql
-- 查看证据链
SELECT * FROM evidence_chain
WHERE tx_id = 12345;
```

---

## 6. 配置

### 6.1 配置文件

```toml
# sqlrustgo.toml
[database]
path = "./data"

[wal]
enabled = true
checkpoint_interval = 300

[search]
default_mode = "hybrid"
```

---

## 7. 故障排查

### 7.1 崩溃恢复

如果系统异常关闭，重启时会自动进行 WAL 重放：

```
=== WAL Recovery ===
Loading WAL file...
Found 150 uncommitted transactions
Recovering... 150/150 done
Database recovered successfully
```

### 7.2 常见问题

| 问题 | 解决方案 |
|------|----------|
| 启动失败 | 检查 WAL 文件完整性 |
| 性能下降 | 运行 `VACUUM` 清理 |
| 索引失效 | 重建索引 `REINDEX` |

---

*用户手册 v2.7.0*
*最后更新: 2026-04-22*
