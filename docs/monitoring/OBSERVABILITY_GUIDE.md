# SQLRustGo 可观测性 API 开发指南

> **版本**: 1.1
> **创建日期**: 2026-03-15
> **更新时间**: 2026-03-15
> **状态**: ✅ 已完成 (M-003, M-004)

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

### 3.3 集成 actix-web

在 Cargo.toml 中添加依赖：

```toml
[dependencies]
actix-web = "4"
actix-rt = "2"
sqlrustgo-server = { path = "crates/server" }
```

### 3.4 配置 Metrics Endpoint

```rust
use actix_web::{web, App, HttpServer, Responder};
use sqlrustgo_server::metrics_endpoint::{configure_metrics_scope, MetricsRegistry};
use sqlrustgo_common::metrics::DefaultMetrics;
use std::sync::{Arc, RwLock};

async fn metrics() -> impl Responder {
    // 获取全局 MetricsRegistry
    let registry = get_global_registry();
    let output = registry.read().unwrap().to_prometheus_format();
    
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(output)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/metrics", web::get().to(metrics))
    })
    .bind("127.0.0.1:5432")?
    .run()
    .await
}
```

### 3.5 MetricsRegistry API

```rust
use sqlrustgo_server::metrics_endpoint::MetricsRegistry;

// 创建新的注册表
let mut registry = MetricsRegistry::new();

// 注册 Metrics collector
let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
registry.register_metrics(metrics);

// 注册自定义指标
registry.register_custom_metric("build_info".to_string(), "version=\"1.4.0\"".to_string());

// 导出 Prometheus 格式
let output = registry.to_prometheus_format();
println!("{}", output);
```

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

### 6.3 MetricsRegistry 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_common::metrics::DefaultMetrics;

    #[test]
    fn test_metrics_registry_creation() {
        let registry = MetricsRegistry::new();
        assert!(registry.to_prometheus_format().is_empty());
    }

    #[test]
    fn test_metrics_registry_with_default_metrics() {
        let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics.write().unwrap();
            m.record_query("SELECT", 100);
        }

        let mut registry = MetricsRegistry::new();
        registry.register_metrics(metrics);
        let output = registry.to_prometheus_format();

        assert!(output.contains("sqlrustgo_queries"));
    }

    #[actix_web::test]
    async fn test_metrics_endpoint_handler() {
        let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics.write().unwrap();
            m.record_query("SELECT", 100);
        }

        let mut registry = MetricsRegistry::new();
        registry.register_metrics(metrics);
        
        let registry = Arc::new(RwLock::new(registry));
        
        let app = actix_web::test::init_service(
            actix_web::App::new()
                .app_data(web::Data::new(registry))
                .configure(configure_metrics_scope)
        ).await;

        let req = actix_web::test::TestRequest::get()
            .uri("/metrics")
            .to_request();

        let resp = actix_web::test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = actix_web::test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("sqlrustgo"));
    }
}
```

---

## 7. 检查清单

### 7.1 功能实现检查

- [x] HealthChecker 实现
- [x] /health/live 端点返回 200 + {"status": "alive"}
- [x] /health/ready 端点返回 200 + 所有组件状态
- [x] /health 端点返回完整健康报告
- [x] Metrics trait 定义
- [x] BufferPoolMetrics 实现
- [x] ExecutorMetrics 实现
- [x] NetworkMetrics 实现
- [x] MetricsRegistry 实现
- [x] /metrics 端点返回 Prometheus 格式
- [x] actix-web 集成 (M-004)
- [x] configure_metrics_scope 配置函数 (M-004)
- [x] metrics_handler HTTP handler (M-004)

### 7.2 测试检查

- [x] Metrics 单元测试 (每个 Metrics 实现 ≥ 3 个测试)
- [x] HealthChecker 单元测试 (≥ 3 个测试)
- [x] /health/live HTTP 集成测试
- [x] /health/ready HTTP 集成测试
- [x] /health HTTP 集成测试
- [x] /metrics HTTP 集成测试
- [x] MetricsRegistry 单元测试
- [x] metrics_handler 集成测试 (actix-web)
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
