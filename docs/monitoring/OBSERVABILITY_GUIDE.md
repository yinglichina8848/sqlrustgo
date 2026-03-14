# SQLRustGo 可观测性 API 开发指南

> **版本**: 1.0
> **创建日期**: 2026-03-15
> **状态**: 🔄 开发中

---

## 目录

1. [概述](#1-概述)
2. [健康检查端点](#2-健康检查端点)
3. [指标端点](#3-指标端点)
4. [Metrics Trait](#4-metrics-trait)
5. [自定义指标](#5-自定义指标)
6. [集成测试](#6-集成测试)
7. [检查清单](#7-检查清单)

---

## 1. 概述

本文档提供 SQLRustGo 可观测性功能的 API 使用指南，包括健康检查和指标监控。

### 1.1 可观测性架构

```
┌─────────────────────────────────────────────────────────────┐
│                    SQLRustGo Server                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │ Health      │  │ Metrics     │  │ MetricsRegistry │  │
│  │ Checker     │  │ Collectors  │  │ (Prometheus)    │  │
│  └─────────────┘  └─────────────┘  └─────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                     HTTP Endpoints                          │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐            │
│  │ /health   │  │ /health   │  │ /metrics  │            │
│  │ /live     │  │ /ready    │  │           │            │
│  └───────────┘  └───────────┘  └───────────┘            │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 依赖添加

```toml
# Cargo.toml
[dependencies]
sqlrustgo-common = { path = "crates/common" }
sqlrustgo-server = { path = "crates/server" }
```

---

## 2. 健康检查端点

### 2.1 端点列表

| 端点 | 方法 | 用途 |
|------|------|------|
| `/health/live` | GET | 存活探针 (Liveness Probe) |
| `/health/ready` | GET | 就绪探针 (Readiness Probe) |
| `/health` | GET | 综合健康检查 |

### 2.2 存活探针 (/health/live)

用于检查服务是否存活。

**请求:**
```bash
curl http://localhost:5432/health/live
```

**响应:**
```json
{
  "status": "alive",
  "version": "1.3.0"
}
```

### 2.3 就绪探针 (/health/ready)

用于检查服务是否就绪（所有组件正常）。

**请求:**
```bash
curl http://localhost:5432/health/ready
```

**响应:**
```json
{
  "status": "ready",
  "checks": {
    "storage": {
      "status": "healthy",
      "latency_ms": 5
    },
    "memory": {
      "status": "healthy",
      "usage_percent": 45.2
    },
    "connections": {
      "status": "healthy",
      "active": 10,
      "max": 100
    }
  }
}
```

### 2.4 综合健康 (/health)

返回完整的健康报告，包含指标。

**请求:**
```bash
curl http://localhost:5432/health
```

**响应:**
```json
{
  "status": "healthy",
  "timestamp": "2026-03-15T10:00:00Z",
  "uptime_seconds": 3600,
  "components": {
    "storage": {
      "status": "healthy"
    },
    "executor": {
      "status": "healthy"
    },
    "network": {
      "status": "healthy"
    }
  },
  "metrics": {
    "queries_total": 1000,
    "queries_failed": 5,
    "avg_query_ms": 25.5
  }
}
```

### 2.5 使用 HealthChecker

```rust
use sqlrustgo_server::health::{HealthChecker, HealthStatus};

let checker = HealthChecker::new();

// 检查存活
let live_status = checker.check_live();
assert_eq!(live_status.status, "alive");

// 检查就绪
let ready_status = checker.check_ready();
assert_eq!(ready_status.status, "ready");
```

---

## 3. 指标端点

### 3.1 指标端点 (/metrics)

返回 Prometheus 格式的指标数据。

**请求:**
```bash
curl http://localhost:5432/metrics
```

**响应:**
```prometheus
# TYPE sqlrustgo_queries_total counter
sqlrustgo_queries_total 1000

# TYPE sqlrustgo_connections_active gauge
sqlrustgo_connections_active 10

# TYPE sqlrustgo_query_duration_seconds histogram
sqlrustgo_query_duration_seconds_bucket{le="0.001"} 500
sqlrustgo_query_duration_seconds_bucket{le="0.01"} 900
sqlrustgo_query_duration_seconds_bucket{le="0.1"} 990
sqlrustgo_query_duration_seconds_bucket{le="1"} 1000
```

### 3.2 内置指标

| 指标名称 | 类型 | 描述 |
|----------|------|------|
| `sqlrustgo_queries_total` | Counter | 总查询数 |
| `sqlrustgo_queries_failed_total` | Counter | 失败查询数 |
| `sqlrustgo_query_duration_seconds` | Histogram | 查询耗时 |
| `sqlrustgo_rows_processed_total` | Counter | 处理行数 |
| `sqlrustgo_connections_active` | Gauge | 活跃连接数 |
| `sqlrustgo_connections_total` | Counter | 总连接数 |
| `sqlrustgo_bytes_sent_total` | Counter | 发送字节数 |
| `sqlrustgo_bytes_received_total` | Counter | 接收字节数 |
| `sqlrustgo_buffer_pool_hits_total` | Counter | 缓存命中数 |
| `sqlrustgo_buffer_pool_misses_total` | Counter | 缓存未命中数 |

---

## 4. Metrics Trait

### 4.1 Trait 定义

```rust
use sqlrustgo_common::metrics::{Metrics, MetricValue};

pub trait Metrics {
    fn record_query(&mut self, query_type: &str, duration_ms: u64);
    fn record_error(&mut self);
    fn record_error_with_type(&mut self, error_type: &str);
    fn record_bytes_read(&mut self, bytes: u64);
    fn record_bytes_written(&mut self, bytes: u64);
    fn record_cache_hit(&mut self);
    fn record_cache_miss(&mut self);
    fn get_metric(&self, name: &str) -> Option<MetricValue>;
    fn get_metric_names(&self) -> Vec<String>;
    fn reset(&mut self);
}
```

### 4.2 MetricValue 枚举

```rust
pub enum MetricValue {
    Counter(u64),      // 计数器
    Gauge(f64),        // 仪表盘
    Histogram(Vec<u64>), // 直方图
    Timing(u64),      // 计时器
}
```

---

## 5. 自定义指标

### 5.1 实现自定义 Metrics

```rust
use sqlrustgo_common::metrics::{Metrics, MetricValue};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct MyCustomMetrics {
    requests_total: AtomicU64,
    latency_ms: AtomicU64,
}

impl MyCustomMetrics {
    pub fn new() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            latency_ms: AtomicU64::new(0),
        }
    }

    pub fn record_request(&self, latency: u64) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.latency_ms.store(latency, Ordering::Relaxed);
    }
}

impl Metrics for MyCustomMetrics {
    fn record_query(&mut self, _query_type: &str, duration_ms: u64) {
        self.record_request(duration_ms);
    }

    fn record_error(&mut self) { /* ... */ }
    fn record_error_with_type(&mut self, _error_type: &str) { /* ... */ }
    fn record_bytes_read(&mut self, _bytes: u64) { /* ... */ }
    fn record_bytes_written(&mut self, _bytes: u64) { /* ... */ }
    fn record_cache_hit(&mut self) { /* ... */ }
    fn record_cache_miss(&mut self) { /* ... */ }

    fn get_metric(&self, name: &str) -> Option<MetricValue> {
        match name {
            "requests_total" => Some(MetricValue::Counter(self.requests_total.load(Ordering::Relaxed))),
            "latency_ms" => Some(MetricValue::Gauge(self.latency_ms.load(Ordering::Relaxed) as f64)),
            _ => None,
        }
    }

    fn get_metric_names(&self) -> Vec<String> {
        vec!["requests_total".to_string(), "latency_ms".to_string()]
    }

    fn reset(&mut self) {
        self.requests_total.store(0, Ordering::Relaxed);
        self.latency_ms.store(0, Ordering::Relaxed);
    }
}
```

### 5.2 注册到 MetricsRegistry

```rust
use sqlrustgo_server::metrics_endpoint::MetricsRegistry;
use std::sync::{Arc, RwLock};

let mut registry = MetricsRegistry::new();
let custom_metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(MyCustomMetrics::new()));
registry.register_metrics(custom_metrics);
```

---

## 6. 集成测试

### 6.1 健康检查端点测试

```rust
// tests/health_endpoint_test.rs
#[cfg(test)]
mod tests {
    use reqwest;
    use tokio;

    #[tokio::test]
    async fn test_health_live() {
        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:5432/health/live")
            .send()
            .await;

        assert!(response.is_ok());
        let body = response.unwrap().json::<serde_json::Value>().await.unwrap();
        assert_eq!(body["status"], "alive");
    }

    #[tokio::test]
    async fn test_health_ready() {
        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:5432/health/ready")
            .send()
            .await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_health_comprehensive() {
        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:5432/health")
            .send()
            .await;

        assert!(response.is_ok());
        let body = response.unwrap().json::<serde_json::Value>().await.unwrap();
        assert!(body.get("components").is_some());
    }
}
```

### 6.2 指标端点测试

```rust
// tests/metrics_endpoint_test.rs
#[cfg(test)]
mod tests {
    use reqwest;
    use tokio;

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:5432/metrics")
            .send()
            .await;

        assert!(response.is_ok());
        let body = response.unwrap().text().await.unwrap();

        // 验证 Prometheus 格式
        assert!(body.contains("# TYPE"));
        assert!(body.contains("sqlrustgo_"));
    }
}
```

---

## 7. 检查清单

### 7.1 功能实现检查

- [ ] HealthChecker 实现
- [ ] /health/live 端点返回 200 + {"status": "alive"}
- [ ] /health/ready 端点返回 200 + 所有组件状态
- [ ] /health 端点返回完整健康报告
- [ ] Metrics trait 定义
- [ ] BufferPoolMetrics 实现
- [ ] ExecutorMetrics 实现
- [ ] NetworkMetrics 实现
- [ ] MetricsRegistry 实现
- [ ] /metrics 端点返回 Prometheus 格式

### 7.2 测试检查

- [ ] Metrics 单元测试 (每个 Metrics 实现 ≥ 3 个测试)
- [ ] HealthChecker 单元测试 (≥ 3 个测试)
- [ ] /health/live HTTP 集成测试
- [ ] /health/ready HTTP 集成测试
- [ ] /health HTTP 集成测试
- [ ] /metrics HTTP 集成测试
- [ ] Grafana Dashboard JSON 验证测试
- [ ] Prometheus Alerts YAML 验证测试

### 7.3 文档检查

- [ ] Health Check 规范文档 (HEALTH_CHECK_SPECIFICATION.md)
- [ ] Grafana Dashboard 导入说明 (docs/monitoring/README.md)
- [ ] Prometheus Alert 规则说明 (PROMETHEUS_ALERTS.md)
- [ ] 可观测性 API 开发指南 (本文档)
- [ ] 端点调用示例代码
- [ ] 自定义指标实现示例

### 7.4 代码质量检查

- [ ] `cargo build --workspace` 通过
- [ ] `cargo test --workspace` 通过
- [ ] `cargo clippy --workspace` 零警告
- [ ] `cargo fmt` 格式化通过
- [ ] 单元测试覆盖率 ≥ 80%

### 7.5 验证命令

```bash
# 构建验证
cargo build --workspace

# 测试验证
cargo test --workspace

# 代码检查
cargo clippy --workspace -- -D warnings
cargo fmt --check --all

# 覆盖率
cargo tarpaulin --output-dir ./target/coverage

# 文档验证
cargo test --test monitoring_test
```

---

## 附录

### A. 相关文件

| 文件 | 描述 |
|------|------|
| `crates/common/src/metrics.rs` | Metrics trait 定义 |
| `crates/common/src/network_metrics.rs` | NetworkMetrics 实现 |
| `crates/common/src/buffer_pool_metrics.rs` | BufferPoolMetrics 实现 |
| `crates/server/src/health.rs` | HealthChecker 实现 |
| `crates/server/src/metrics_endpoint.rs` | MetricsRegistry 实现 |
| `docs/monitoring/grafana-dashboard.json` | Grafana 仪表盘 |
| `docs/monitoring/prometheus-alerts.yml` | Prometheus 告警规则 |

### B. 参考资料

- [Prometheus 数据模型](https://prometheus.io/docs/concepts/data_model/)
- [Prometheus 告警规则](https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/)
- [Grafana Dashboard 导入](https://grafana.com/docs/grafana/latest/dashboards/export-import/)

---

*文档版本: 1.0*
*最后更新: 2026-03-15*
