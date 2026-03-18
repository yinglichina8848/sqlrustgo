# SQLRustGo v1.4.0 用户手册

> 版本：v1.4.0
> 发布日期：2026-03-16
> 适用用户：数据库使用者、应用开发者、数据库学习者

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
docker pull minzuuniversity/sqlrustgo:v1.4.0

# 运行 REPL
docker run -it minzuuniversity/sqlrustgo:v1.4.0

# 运行服务器模式
docker run -p 5432:5432 minzuuniversity/sqlrustgo:v1.4.0 --server
```

#### 使用预编译二进制

```bash
# macOS (Apple Silicon)
curl -L -o sqlrustgo-v1.4.0-darwin-arm64.tar.gz \
  https://github.com/minzuuniversity/sqlrustgo/releases/download/v1.4.0/sqlrustgo-v1.4.0-darwin-arm64.tar.gz
tar -xzf sqlrustgo-v1.4.0-darwin-arm64.tar.gz
./sqlrustgo-v1.4.0-darwin-arm64/sqlrustgo

# Linux (x86_64)
curl -L -o sqlrustgo-v1.4.0-linux-x86_64.tar.gz \
  https://github.com/minzuuniversity/sqlrustgo/releases/download/v1.4.0/sqlrustgo-v1.4.0-linux-x86_64.tar.gz
tar -xzf sqlrustgo-v1.4.0-linux-x86_64.tar.gz
./sqlrustgo-v1.4.0-linux-x86_64/sqlrustgo
```

---

## 二、SQL 使用

### 2.1 REPL 模式

```bash
# 启动 REPL
cargo run --release

# 或使用预编译二进制
./sqlrustgo
```

### 2.2 基本 SQL 操作

```sql
-- 创建表
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER);

-- 插入数据
INSERT INTO users VALUES (1, 'Alice', 30);
INSERT INTO users VALUES (2, 'Bob', 25);

-- 查询
SELECT * FROM users;
SELECT * FROM users WHERE age > 28;

-- 更新
UPDATE users SET age = 31 WHERE id = 1;

-- 删除
DELETE FROM users WHERE id = 2;

-- 聚合
SELECT COUNT(*) FROM users;
SELECT AVG(age) FROM users;
```

### 2.3 Join 操作

```sql
-- 创建测试表
CREATE TABLE orders (id INTEGER, user_id INTEGER, amount REAL);
INSERT INTO orders VALUES (1, 1, 100.0), (2, 1, 200.0), (3, 2, 150.0);

-- Inner Join
SELECT u.name, o.amount FROM users u JOIN orders o ON u.id = o.user_id;

-- Left Join
SELECT u.name, o.amount FROM users u LEFT JOIN orders o ON u.id = o.user_id;

-- Cross Join (笛卡尔积)
SELECT * FROM users, orders;
```

---

## 三、API 使用

### 3.1 编程 API

```rust
use sqlrustgo::{Session, Result};

fn main() -> Result<()> {
    let mut session = Session::new();
    
    // 执行查询
    let result = session.execute("SELECT * FROM users")?;
    
    for row in result {
        println!("{:?}", row);
    }
    
    Ok(())
}
```

### 3.2 HTTP API

启动服务器后访问：

```bash
# 健康检查
curl http://localhost:5432/health

# 就绪检查
curl http://localhost:5432/health/ready

# 存活检查
curl http://localhost:5432/health/live

# Prometheus 指标
curl http://localhost:5432/metrics
```

---

## 四、监控配置

### 4.1 Prometheus 配置

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'sqlrustgo'
    static_configs:
      - targets: ['localhost:5432']
```

### 4.2 Grafana 导入

1. 访问 Grafana Dashboard 导入
2. 上传 `docs/monitoring/grafana-dashboard.json`
3. 配置 Prometheus 数据源

---

## 五、性能优化

### 5.1 CBO 优化

v1.4.0 自动使用基于代价的优化：

- 索引选择优化
- Join 顺序优化
- 扫描方式优化

### 5.2 Join 算法选择

| 数据量 | 推荐算法 |
|--------|----------|
| 小表 | HashJoin |
| 大表+有序 | SortMergeJoin |
| Cross Join | NestedLoopJoin |

---

## 六、版本升级

### 从 v1.3.0 升级

v1.4.0 保持向后兼容，直接升级：

```bash
cargo update
cargo build --release
```

---

## 七、常见问题

### Q1: 如何启用 CBO 优化？
A: CBO 默认启用，无需额外配置。

### Q2: 如何查看执行计划？
A: 使用 EXPLAIN 命令（待实现）。

### Q3: 支持哪些 Join 类型？
A: Inner, Left, Right, Full, Cross, Semi, Anti Join。

---

## 八、更多资源

- 文档: https://github.com/minzuuniversity/sqlrustgo/docs
- 问题反馈: https://github.com/minzuuniversity/sqlrustgo/issues
- 版本发布: https://github.com/minzuuniversity/sqlrustgo/releases

---

**文档版本**: 1.0
**最后更新**: 2026-03-16
