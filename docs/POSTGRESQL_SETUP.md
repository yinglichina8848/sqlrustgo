# PostgreSQL Benchmark 环境配置指南

## 1. 概述

本文档说明如何配置 PostgreSQL 环境以运行 SQLRustGo 的 PostgreSQL 对比基准测试。

## 2. 前置条件

### 2.1 安装 PostgreSQL

**macOS (使用 Homebrew):**
```bash
brew install postgresql@15
brew services start postgresql@15
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install postgresql postgresql-client
sudo systemctl start postgresql
```

**验证安装:**
```bash
psql --version
```

### 2.2 Rust 环境

```bash
rustc --version
cargo --version
```

## 3. 数据库配置

### 3.1 创建数据库和用户

```bash
psql -U postgres
```

在 psql 中执行：

```sql
CREATE DATABASE tpch;
GRANT ALL PRIVILEGES ON DATABASE tpch TO postgres;
\c tpch

CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY,
    balance INTEGER NOT NULL DEFAULT 100
);

INSERT INTO accounts (id, balance)
SELECT i, 100
FROM generate_series(1, 10000) AS i
ON CONFLICT (id) DO NOTHING;
```

### 3.2 Docker 环境

```bash
docker run -d \
  --name sqlrustgo-pg \
  -e POSTGRES_DB=tpch \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -p 5432:5432 \
  postgres:15
```

## 4. 连接配置

### 4.1 默认配置

| 参数 | 默认值 |
|------|--------|
| Host | localhost |
| Port | 5432 |
| Database | tpch |
| User | postgres |
| Password | postgres |

### 4.2 连接字符串

```rust
"host=localhost port=5432 dbname=tpch user=postgres password=postgres"
```

## 5. 运行测试

```bash
# 运行 PostgreSQL 测试
cargo test -p sqlrustgo-bench -- db::postgres

# 运行 Mock 测试
cargo test -p sqlrustgo-bench --test postgres_mock_test

# 运行所有 bench 测试
cargo test -p sqlrustgo-bench
```

## 6. 常见问题

### Q1: 连接被拒绝
```bash
pg_isready -h localhost -p 5432
brew services start postgresql@15  # macOS
sudo systemctl start postgresql    # Linux
```

### Q2: 认证失败
```sql
ALTER USER postgres WITH PASSWORD 'your_password';
```

### Q3: 数据库不存在
```sql
CREATE DATABASE tpch;
```

### Q4: 表不存在
```sql
\c tpch
CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance INTEGER);
INSERT INTO accounts SELECT i, 100 FROM generate_series(1,10000) AS i;
```

## 7. 验证脚本

```bash
#!/bin/bash
CONN_STR="host=localhost port=5432 dbname=tpch user=postgres password=postgres"
psql "$CONN_STR" -c "SELECT COUNT(*) FROM accounts;"
```

---

*最后更新: 2026-03-20*