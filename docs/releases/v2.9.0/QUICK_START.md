# v2.9.0 快速开始

> **版本**: v2.9.0
> **代号**: 企业级韧性 (Enterprise Resilience)
> **发布日期**: 2026-05-04

---

## 1. 安装

### 1.1 前置要求

- Rust 1.85+
- Cargo (随 Rust 安装)

### 1.2 安装步骤

```bash
# 克隆仓库
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 切换到 v2.9.0 分支
git checkout v2.9.0-rc.1

# 构建
cargo build --all-features --release
```

### 1.3 验证安装

```bash
# 运行测试
cargo test --all-features

# 启动 REPL
cargo run --release --bin sqlrustgo
```

---

## 2. 连接方式

SQLRustGo 支持多种连接方式，兼容 MySQL 5.7/8.0 协议。

### 2.1 MySQL CLI 连接 (推荐)

```bash
# 终端1: 启动 MySQL 协议服务器
cargo run --release --bin sqlrustgo-mysql-server --host 127.0.0.1 --port 3306

# 终端2: 使用 mysql 客户端连接
mysql -h 127.0.0.1 -P 3306 -u root
```

### 2.2 TCP 连接 (应用程序)

```bash
# 使用标准 MySQL 驱动连接
# 连接字符串: mysql://127.0.0.1:3306/default
```

详细连接方式请参考 [客户端连接指南](./CLIENT_CONNECTION.md)。

---

## 3. REST API

### 3.1 启动 REST API 服务器

```bash
# 启动 HTTP 服务器 (端口 8080)
cargo run --release --bin sqlrustgo-server
```

### 3.2 API 端点

| 端点 | 说明 |
|------|------|
| `GET /health` | 健康检查 |
| `GET /metrics` | Prometheus 指标 |
| `POST /query` | 执行 SQL 查询 |

详细 API 文档请参考 [API 参考](./API_REFERENCE.md)。

---

## 4. 基础 SQL 操作

### 4.1 创建表

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(255)
);
```

### 4.2 插入数据

```sql
INSERT INTO users (id, name, email) VALUES
(1, 'Alice', 'alice@example.com'),
(2, 'Bob', 'bob@example.com');

-- INSERT ON DUPLICATE KEY UPDATE (MySQL 兼容)
INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice_new@example.com')
ON DUPLICATE KEY UPDATE email = VALUES(email);
```

### 4.3 查询

```sql
SELECT * FROM users;
SELECT name, email FROM users WHERE id = 1;
```

### 4.4 聚合查询

```sql
SELECT COUNT(*) FROM users;
SELECT AVG(age) FROM users GROUP BY department;
```

### 4.5 CTE/WITH 语法

```sql
-- 非递归 CTE
WITH active_users AS (
    SELECT * FROM users WHERE status = 'active'
)
SELECT * FROM active_users WHERE id > 10;

-- 递归 CTE
WITH RECURSIVE cte AS (
    SELECT 1 AS n
    UNION ALL
    SELECT n + 1 FROM cte WHERE n < 10
)
SELECT * FROM cte;
```

### 4.6 窗口函数

```sql
-- ROW_NUMBER
SELECT name, department,
       ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary DESC) as rank
FROM employees;

-- RANK / DENSE_RANK
SELECT name, score, RANK() OVER (ORDER BY score DESC) as rank
FROM leaderboard;

-- 聚合作为窗口函数
SELECT year, product,
       SUM(amount) OVER (PARTITION BY year ORDER BY amount) as running_total
FROM sales;
```

### 4.7 CASE/WHEN

```sql
SELECT
    name,
    CASE
        WHEN age < 18 THEN '未成年'
        WHEN age < 65 THEN '成年'
        ELSE '老年'
    END as age_group
FROM users;
```

---

## 5. 分布式架构 (v2.9.0 新增)

### 5.1 Semi-sync 复制

```sql
-- 配置 semi-sync 复制 (需要在 MySQL 客户端执行)
SET GLOBAL rpl_semi_sync_master_enabled = 1;
SET GLOBAL rpl_semi_sync_slave_enabled = 1;
```

### 5.2 XA 事务

```sql
-- 两阶段提交 (XA) 事务
XA BEGIN 'transaction_id';
UPDATE accounts SET balance = balance - 100 WHERE id = 1;
UPDATE accounts SET balance = balance + 100 WHERE id = 2;
XA PREPARE 'transaction_id';
XA COMMIT 'transaction_id';
```

### 5.3 主从复制状态

```sql
-- 查看复制状态
SHOW SLAVE STATUS\G
SHOW MASTER STATUS\G
```

---

## 6. 高级 SQL 特性

### 6.1 TRUNCATE TABLE

```sql
-- 清空表数据（比 DELETE 更快）
TRUNCATE TABLE users;
```

### 6.2 REPLACE INTO

```sql
-- 如果唯一键冲突，替换现有行
REPLACE INTO users (id, name, email) VALUES (1, 'Alice', 'alice_new@example.com');
```

### 6.3 JSON 操作

```sql
-- JSON 提取
SELECT JSON_EXTRACT(data, '$.name') FROM t;
SELECT data->>'$.name' FROM t;

-- JSON 路径
SELECT * FROM t WHERE JSON_CONTAINS(data, '"value"', '$.tags');
```

---

## 7. 形式化验证 (v2.9.0 亮点)

### 7.1 TLA+ 证明

v2.9.0 包含 18 个形式化验证证明：

| Proof ID | 标题 | 工具 |
|----------|------|------|
| PROOF-001 | Parser SELECT 语义 | TLA+ |
| PROOF-002 | 类型推导 | TLA+ |
| PROOF-003 | WAL 恢复 | TLA+ |
| PROOF-004 | B+Tree 查询完整性 | TLA+ |
| PROOF-005 | MVCC 快照隔离 | TLA+ |
| ... | ... | ... |

### 7.2 验证方法

```bash
# 运行全部形式化证明
bash scripts/verify/run_all_proofs.sh

# 运行快速验证
bash docs/formal/formal_smoke.sh
```

---

## 8. 性能基准

### 8.1 SQL Corpus

```
=== Summary ===
Total: 59 cases, 59 passed, 0 failed
Pass rate: 100.0%
```

### 8.2 覆盖率

| 模块 | 覆盖率 |
|------|--------|
| 总覆盖率 | 84.18% |
| Executor | 71.08% |
| Transaction | 90.99% |
| Storage | 81.77% |

### 8.3 性能目标

| 指标 | 目标 | 状态 |
|------|------|------|
| QPS | ≥10,000 | ⏳ 进行中 |
| 延迟 P99 | <10ms | ⏳ 待测 |

---

## 9. 文档

| 文档 | 说明 |
|------|------|
| [README.md](./README.md) | 文档索引 |
| [CLIENT_CONNECTION.md](./CLIENT_CONNECTION.md) | 客户端连接指南 |
| [API_REFERENCE.md](./API_REFERENCE.md) | REST API 参考 |
| [SECURITY_REPORT.md](./SECURITY_REPORT.md) | 安全报告 |
| [CHANGELOG.md](./CHANGELOG.md) | 变更日志 |

---

## 10. 故障排除

### 10.1 常见问题

**Q: 构建失败**
```bash
# 清理并重新构建
cargo clean
cargo build --all-features
```

**Q: 测试失败**
```bash
# 运行单个测试
cargo test <test_name> --all-features
```

**Q: 连接被拒绝**
```bash
# 检查服务器是否运行
cargo run --release --bin sqlrustgo-mysql-server
```

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-05-04*