# v3.1.0 快速入门指南

> **版本**: 3.1.0
> **预计完成时间**: 约10分钟

---

## 1. 安装 (2分钟)

```bash
# 从源码构建
git clone http://192.168.0.252:3000/openclaw/sqlrustgo.git
cd sqlrustgo
git checkout develop/v3.1.0
cargo build --release --all-features
```

---

## 2. 启动服务器 (1分钟)

```bash
# 创建数据目录
mkdir -p /tmp/sqlrustgo_data

# 后台启动服务器
./target/release/sqlrustgo \
  --data-dir /tmp/sqlrustgo_data \
  --bind 127.0.0.1:3306 &

# 等待启动完成
sleep 2
```

---

## 3. 连接 (1分钟)

```bash
# 使用 MySQL 客户端连接
mysql -h 127.0.0.1 -P 3306 -u root

# 或使用 sqlrustgo CLI
./target/release/sqlrustgo cli --data-dir /tmp/sqlrustgo_data
```

---

## 4. 创建数据库和表 (2分钟)

```sql
-- 创建数据库
CREATE DATABASE IF NOT EXISTS demo;

-- 使用数据库
USE demo;

-- 创建表
CREATE TABLE users (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 插入数据
INSERT INTO users (name, email) VALUES
    ('Alice', 'alice@example.com'),
    ('Bob', 'bob@example.com'),
    ('Charlie', 'charlie@example.com');

-- 查询
SELECT * FROM users WHERE id = 1;
```

---

## 5. 体验新功能 (3分钟)

### INFORMATION_SCHEMA

```sql
-- 查看表元数据
SELECT TABLE_NAME, TABLE_ROWS
FROM information_schema.TABLES
WHERE TABLE_SCHEMA = 'demo';

-- 查看列详情
SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = 'demo';
```

### MERGE 语句

```sql
-- 创建源表
CREATE TABLE users_staging (
    id INT,
    name VARCHAR(100),
    email VARCHAR(255)
);

INSERT INTO users_staging VALUES
    (1, 'Alice Updated', 'alice.new@example.com'),
    (4, 'Dave', 'dave@example.com');

-- 合并操作
MERGE INTO users AS t
USING users_staging AS s
ON t.id = s.id
WHEN MATCHED THEN
    UPDATE SET t.name = s.name, t.email = s.email
WHEN NOT MATCHED THEN
    INSERT (id, name, email) VALUES (s.id, s.name, s.email);
```

### SAVEPOINT

```sql
BEGIN;
INSERT INTO users (name, email) VALUES ('Test', 'test@test.com');
SAVEPOINT sp1;
UPDATE users SET name = 'Updated' WHERE email = 'test@test.com';
ROLLBACK TO SAVEPOINT sp1;
COMMIT;

-- 验证: 'Test' 行存在，而不是 'Updated'
SELECT * FROM users WHERE email = 'test@test.com';
```

### EXPLAIN 与 CostModel

```sql
-- 创建索引
CREATE INDEX idx_email ON users(email);

-- 查看查询计划
EXPLAIN SELECT * FROM users WHERE email = 'alice@example.com';
```

---

## 6. 运行测试 (1分钟)

```bash
# 运行 SQL 语料库测试
cargo test -p sqlrustgo-sql-corpus

# 运行特定测试
cargo test --test merge_test

# 运行覆盖率
cargo llvm-cov --all-features --lib --summary-only
```

---

## 下一步

| 主题 | 文档 |
|-------|-----|
| 完整安装说明 | `INSTALL.md` |
| 配置 | `DEPLOYMENT_GUIDE.md` |
| 性能调优 | `PERFORMANCE_TARGETS.md` |
| GMP 功能 | `GMP_COMPLIANCE_ROADMAP.md` |
| 完整功能列表 | `FEATURE_MATRIX.md` |

---

## 常用命令

```bash
# 停止服务器
pkill sqlrustgo

# 重置数据
rm -rf /tmp/sqlrustgo_data
mkdir /tmp/sqlrustgo_data

# 查看版本
./target/release/sqlrustgo --version

# 查看日志
tail -f /tmp/sqlrustgo_data/log/sqlrustgo.log
```
