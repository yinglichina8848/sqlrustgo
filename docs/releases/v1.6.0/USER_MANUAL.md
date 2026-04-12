# SQLRustGo v1.6.0 用户手册

> **版本**: v1.6.0
> **发布日期**: 2026-03-19
> **适用用户**: 数据库使用者、应用开发者、数据库学习者

---

## 一、快速开始

### 1.1 安装

#### 从源码构建

```bash
# 克隆项目
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 构建 (Debug 模式)
cargo build

# 构建 (Release 模式，推荐)
cargo build --release

# 运行测试
cargo test --workspace
```

#### 使用 Docker

```bash
# 拉取镜像
docker pull minzuuniversity/sqlrustgo:v1.6.0

# 运行 REPL
docker run -it minzuuniversity/sqlrustgo:v1.6.0

# 运行服务器模式
docker run -p 5432:5432 minzuuniversity/sqlrustgo:v1.6.0 --server
```

---

## 二、基本操作

### 2.1 REPL 模式

```bash
# 启动 REPL
cargo run --release

# 在 REPL 中执行 SQL
SQL > CREATE TABLE users (id INTEGER, name TEXT);
SQL > INSERT INTO users VALUES (1, 'Alice');
SQL > SELECT * FROM users;
```

### 2.2 服务器模式

```bash
# 启动服务器
cargo run --release -- --server --port 5432
```

---

## 三、事务支持

### 3.1 开始事务

```rust
let mut tx = transaction_manager.begin()?;
```

### 3.2 提交事务

```rust
let commit_ts = transaction_manager.commit()?;
```

### 3.3 回滚事务

```rust
transaction_manager.rollback()?;
```

### 3.4 隔离级别

```rust
// READ COMMITTED (默认)
let tx = transaction_manager.begin_with_isolation(
    IsolationLevel::ReadCommitted
)?;
```

---

## 四、查询缓存

### 4.1 配置缓存

```rust
let config = QueryCacheConfig {
    enabled: true,
    max_entries: 1000,
    max_memory_bytes: 100 * 1024 * 1024,
    ttl_seconds: 30,
};
```

### 4.2 使用缓存

```rust
let cache = QueryCache::new(config);

// 获取缓存
let result = cache.get(&key);

// 设置缓存
cache.put(key, entry, tables)?;
```

---

## 五、索引操作

### 5.1 创建索引

```sql
CREATE INDEX idx_id ON users(id);
CREATE UNIQUE INDEX idx_email ON users(email);
```

### 5.2 复合索引

```sql
CREATE INDEX idx_name_age ON users(name, age);
```

### 5.3 全文索引

```sql
CREATE FULLTEXT INDEX idx_content ON articles(content);
```

---

## 六、数据类型

### 6.1 支持的数据类型

| 类型 | 说明 | 示例 |
|------|------|------|
| INTEGER | 64位整数 | 123 |
| TEXT | 文本 | 'hello' |
| FLOAT | 浮点数 | 1.23 |
| BOOLEAN | 布尔值 | TRUE/FALSE |
| DATE | 日期 | DATE '2024-01-01' |
| TIMESTAMP | 时间戳 | TIMESTAMP '2024-01-01 12:00:00' |

### 6.2 日期操作

```sql
SELECT * FROM orders WHERE order_date = DATE '2024-01-01';
SELECT * FROM events WHERE created_at > TIMESTAMP '2024-01-01 00:00:00';
```

---

## 七、并发控制

### 7.1 行级锁

```rust
// 获取共享锁 (读锁)
lock_manager.acquire_lock(tx_id, key, LockMode::Shared)?;

// 获取排他锁 (写锁)
lock_manager.acquire_lock(tx_id, key, LockMode::Exclusive)?;
```

### 7.2 死锁检测

死锁检测自动启用，检测到死锁时会自动回滚一个事务。

---

## 八、TPC-H 基准测试

### 8.1 运行基准测试

```bash
cargo bench --bench tpch_bench
```

### 8.2 测试结果

| 查询 | 延迟 |
|------|------|
| Q1 | < 0.5ms |
| Q6 | < 0.2ms |

---

## 九、配置参考

### 9.1 连接池配置

```rust
let config = PoolConfig {
    size: 50,  // 最大连接数
    timeout: Duration::from_secs(5),
};
```

### 9.2 查询缓存配置

```rust
let config = QueryCacheConfig {
    enabled: true,
    max_entries: 1000,
    max_memory_bytes: 100 * 1024 * 1024,
    ttl_seconds: 30,
};
```

---

## 十、故障排除

### 10.1 常见问题

| 问题 | 解决方案 |
|------|----------|
| 编译失败 | 运行 `cargo update` |
| 测试失败 | 检查日志 `cargo test -- --nocapture` |
| 性能问题 | 检查查询缓存命中率 |

### 10.2 日志

```bash
# 启用日志
RUST_LOG=debug cargo run
```

---

*本文档由 AI 辅助分析生成*
*生成日期: 2026-03-19*
