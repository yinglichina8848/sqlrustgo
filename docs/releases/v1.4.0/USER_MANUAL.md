# SQLRustGo v1.4.0 用户手册

> **版本**: 1.4.0
> **更新日期**: 2026-03-18

---

## 一、快速开始

### 1.1 安装

```bash
cargo add sqlrustgo
```

### 1.2 基本使用

```rust
use sqlrustgo::*;

fn main() -> Result<()> {
    let mut db = Database::new()?;
    
    // 创建表
    db.execute("CREATE TABLE users (id INT, name TEXT)")?;
    
    // 插入数据
    db.execute("INSERT INTO users VALUES (1, 'Alice')")?;
    
    // 查询
    let results = db.query("SELECT * FROM users")?;
    
    for row in results {
        println!("{:?}", row);
    }
    
    Ok(())
}
```

---

## 二、SQL 支持

### 2.1 DDL

```sql
CREATE TABLE table_name (
    column_name data_type,
    ...
);

DROP TABLE table_name;
```

### 2.2 DML

```sql
INSERT INTO table_name VALUES (...);

SELECT column_list FROM table_name
WHERE condition
GROUP BY column
HAVING condition
ORDER BY column [ASC|DESC]
LIMIT count;
```

### 2.3 Join

```sql
-- Hash Join (默认)
SELECT * FROM a JOIN b ON a.id = b.id;

-- Sort Merge Join
SELECT * FROM a SORT_MERGE JOIN b ON a.id = b.id;

-- Nested Loop Join
SELECT * FROM a NESTED_LOOP JOIN b ON 1=1;
```

---

## 三、CBO 优化器

### 3.1 启用 CBO

```rust
let db = Database::builder()
    .enable_cbo(true)
    .build()?;
```

### 3.2 统计信息

```rust
// 收集统计信息
db.execute("ANALYZE table_name")?;
```

### 3.3 查看执行计划

```rust
let plan = db.explain("SELECT * FROM users WHERE id > 100")?;
println!("{}", plan);
```

---

## 四、监控

### 4.1 Prometheus 指标

访问 `/metrics` 端点:

```bash
curl http://localhost:8080/metrics
```

### 4.2 可用指标

| 指标名称 | 类型 | 说明 |
|----------|------|------|
| sqlrustgo_queries_total | Counter | 查询总数 |
| sqlrustgo_query_duration_seconds | Histogram | 查询耗时 |
| sqlrustgo_rows_scanned | Counter | 扫描行数 |

### 4.3 Grafana

导入 `grafana-dashboard.json` 到 Grafana 查看监控面板。

---

## 五、性能调优

### 5.1 缓冲池配置

```rust
let db = Database::builder()
    .buffer_pool_size(1024)  // MB
    .build()?;
```

### 5.2 索引使用

CBO 会自动选择是否使用索引:

```sql
-- 强制使用索引
SELECT * FROM users USING INDEX idx_id WHERE id > 100;
```

---

## 六、基准测试

### 6.1 运行 TPC-H

```bash
cargo bench --bench bench_tpch
```

### 6.2 性能对比

查看 [PERFORMANCE_BENCHMARK_REPORT.md](./PERFORMANCE_BENCHMARK_REPORT.md)

---

## 七、版本信息

- **版本**: v1.4.0
- **代号**: CBO & Vectorization Ready
- **发布日期**: 2026-03-18

---

## 八、相关链接

- [架构文档](./ARCHITECTURE.md)
- [发布说明](./RELEASE_NOTES.md)
- [变更日志](./CHANGE_LOG.md)
