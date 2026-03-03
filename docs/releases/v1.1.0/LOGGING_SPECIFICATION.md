# SQLRustGo 日志规范

> 版本：v1.1.0
> 发布日期：2026-03-03

---

## 一、日志级别

### 1.1 级别定义

| 级别 | 用途 | 示例 |
|------|------|------|
| **ERROR** | 错误事件，需要立即处理 | 连接失败、查询执行错误 |
| **WARN** | 警告事件，可能存在问题 | 性能下降、配置缺失 |
| **INFO** | 重要业务事件 | 服务启动、请求处理完成 |
| **DEBUG** | 调试信息 | SQL 解析过程、执行计划 |
| **TRACE** | 详细跟踪信息 | 每行代码执行、变量值 |

### 1.2 使用原则

```rust
// ERROR: 不可恢复的错误
log::error!("Failed to connect to database: {}", error);

// WARN: 可恢复但需要注意的问题
log::warn!("Connection pool exhausted, waiting for available connection");

// INFO: 重要业务事件
log::info!("Server started on {}", addr);
log::info!("Query executed in {}ms", elapsed.as_millis());

// DEBUG: 调试信息
log::debug!("Parsing SQL: {}", sql);
log::debug!("Execution plan: {:?}", plan);

// TRACE: 详细跟踪
log::trace!("Processing row: {:?}", row);
```

---

## 二、日志格式

### 2.1 标准格式

```
[timestamp] [level] [module] [request_id] message
```

### 2.2 示例

```
[2026-03-03T10:30:45.123Z] [INFO] [sqlrustgo::network] [req-12345] Server started on 0.0.0.0:3306
[2026-03-03T10:30:46.456Z] [DEBUG] [sqlrustgo::parser] [req-12346] Parsing SQL: SELECT * FROM users
[2026-03-03T10:30:46.458Z] [INFO] [sqlrustgo::executor] [req-12346] Query executed in 2ms, returned 100 rows
[2026-03-03T10:30:47.789Z] [ERROR] [sqlrustgo::storage] [req-12347] Failed to read table: users - No such file
```

---

## 三、模块日志规范

### 3.1 网络模块 (network)

```rust
// 连接事件
log::info!("[conn-{}] New connection from {}", conn_id, peer_addr);
log::info!("[conn-{}] Connection closed, duration: {:?}", conn_id, duration);

// 请求处理
log::debug!("[conn-{}] Received query: {}", conn_id, query);
log::info!("[conn-{}] Query completed in {}ms", conn_id, elapsed_ms);

// 错误
log::error!("[conn-{}] Failed to handle request: {}", conn_id, error);
```

### 3.2 执行器模块 (executor)

```rust
// 查询执行
log::debug!("Executing plan: {:?}", plan);
log::info!("Query returned {} rows in {}ms", rows.len(), elapsed_ms);

// 表操作
log::info!("Created table: {}", table_name);
log::info!("Dropped table: {}", table_name);

// 索引操作
log::info!("Created index on {}.{}", table, column);
```

### 3.3 存储模块 (storage)

```rust
// 文件操作
log::debug!("Reading table: {}", table_name);
log::debug!("Writing {} rows to table: {}", rows.len(), table_name);

// 索引操作
log::trace!("Index lookup: {} on {}", key, index_name);

// 错误
log::error!("Failed to write to table {}: {}", table_name, error);
```

### 3.4 事务模块 (transaction)

```rust
// 事务生命周期
log::debug!("Transaction {} started", tx_id);
log::debug!("Transaction {} committed", tx_id);
log::warn!("Transaction {} rolled back: {}", tx_id, reason);

// WAL 操作
log::trace!("WAL append: {:?}", record);
```

---

## 四、结构化日志

### 4.1 JSON 格式

```json
{
  "timestamp": "2026-03-03T10:30:45.123Z",
  "level": "INFO",
  "module": "sqlrustgo::network",
  "request_id": "req-12345",
  "message": "Query executed",
  "fields": {
    "query": "SELECT * FROM users",
    "rows": 100,
    "elapsed_ms": 2
  }
}
```

### 4.2 配置

```rust
// 使用 tracing 实现结构化日志
use tracing::{info, instrument};

#[instrument(skip(self, query))]
fn execute(&self, query: &str) -> SqlResult<QueryResult> {
    info!(query, "Executing query");
    // ...
    info!(rows = result.rows.len(), "Query completed");
    Ok(result)
}
```

---

## 五、性能考虑

### 5.1 延迟格式化

```rust
// 推荐：只在日志级别启用时格式化
log::debug!("Complex data: {:?}", expensive_to_format());

// 避免：总是格式化
// println!("Complex data: {:?}", expensive_to_format());
```

### 5.2 日志级别控制

```rust
// 生产环境
RUST_LOG=sqlrustgo=info

// 开发环境
RUST_LOG=sqlrustgo=debug

// 调试特定模块
RUST_LOG=sqlrustgo::executor=trace
```

---

## 六、日志轮转

### 6.1 配置

```toml
# log4rs 配置示例
appenders:
  rolling_file:
    kind: rolling_file
    path: /var/log/sqlrustgo/sqlrustgo.log
    encoder:
      pattern: "{d} [{l}] {m}{n}"
    policy:
      kind: compound
      trigger:
        kind: size
        limit: 100 mb
      roller:
        kind: fixed_window
        pattern: /var/log/sqlrustgo/sqlrustgo.{}.log
        count: 10
```

---

## 七、监控集成

### 7.1 Prometheus 指标

```rust
// 日志计数器
lazy_static! {
    static ref LOG_COUNTER: CounterVec = register_counter_vec!(
        "sqlrustgo_log_total",
        "Total number of log entries",
        &["level"]
    ).unwrap();
}

// 使用
log::error!("Error occurred");
LOG_COUNTER.with_label_values(&["error"]).inc();
```

### 7.2 告警规则

```yaml
# Prometheus 告警规则
groups:
  - name: sqlrustgo
    rules:
      - alert: HighErrorRate
        expr: rate(sqlrustgo_log_total{level="error"}[5m]) > 10
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
```

---

## 八、最佳实践

### 8.1 DO

- ✅ 使用结构化日志
- ✅ 包含请求 ID 用于追踪
- ✅ 记录关键业务事件
- ✅ 使用适当的日志级别
- ✅ 在生产环境使用 INFO 级别

### 8.2 DON'T

- ❌ 记录敏感信息（密码、密钥）
- ❌ 在热路径使用 TRACE 级别
- ❌ 过度使用 ERROR 级别
- ❌ 在日志中执行耗时操作

---

*本文档由 TRAE (GLM-5.0) 创建*
*最后更新: 2026-03-03*
