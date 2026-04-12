# M-004 /metrics 端点测试与使用文档

> **功能**: /metrics 端点 - HTTP 暴露 Prometheus 指标
> **版本**: v1.4.0
> **创建日期**: 2026-03-15
> **状态**: ✅ 已完成

---

## 一、概述

M-004 实现了 `/metrics` HTTP 端点，用于暴露 Prometheus 格式的指标数据。该端点基于 actix-web 框架，支持 OpenMetrics 规范。

### 1.1 架构

```
┌─────────────────────────────────────────┐
│           SQLRustGo Server              │
├─────────────────────────────────────────┤
│  ┌─────────────────────────────────┐    │
│  │    MetricsRegistry              │    │
│  │  - metrics_collectors          │    │
│  │  - custom_metrics              │    │
│  └─────────────────────────────────┘    │
│                   │                      │
│                   ▼                      │
│  ┌─────────────────────────────────┐    │
│  │    /metrics endpoint            │    │
│  │  - metrics_handler (async)     │    │
│  │  - configure_metrics_scope     │    │
│  └─────────────────────────────────┘    │
├─────────────────────────────────────────┤
│  Response:                             │
│  Content-Type: text/plain;              │
│    version=0.0.4; charset=utf-8        │
└─────────────────────────────────────────┘
```

---

## 二、实现文件

| 文件路径 | 描述 |
|----------|------|
| `crates/server/Cargo.toml` | 添加 actix-web 依赖 |
| `crates/server/src/metrics_endpoint.rs` | 核心实现 |
| `crates/server/src/lib.rs` | 模块导出 |

---

## 三、API 使用

### 3.1 MetricsRegistry

```rust
use sqlrustgo_server::metrics_endpoint::MetricsRegistry;
use sqlrustgo_common::metrics::{Metrics, DefaultMetrics};
use std::sync::{Arc, RwLock};

// 创建注册表
let mut registry = MetricsRegistry::new();

// 注册指标收集器
let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
registry.register_metrics(metrics);

// 注册自定义指标
registry.register_custom_metric("build_info".to_string(), "version=\"1.4.0\"".to_string());

// 导出 Prometheus 格式
let output = registry.to_prometheus_format();
```

### 3.2 配置 actix-web

```rust
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use sqlrustgo_server::metrics_endpoint::{configure_metrics_scope, MetricsRegistry};
use std::sync::{Arc, RwLock};

async fn metrics_handler(data: web::Data<Arc<RwLock<MetricsRegistry>>>) -> impl Responder {
    let registry = data.read().unwrap();
    let output = registry.to_prometheus_format();
    
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(output)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let registry = Arc::new(RwLock::new(MetricsRegistry::new()));
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(registry.clone()))
            .configure(configure_metrics_scope)
    })
    .bind("127.0.0.1:5432")?
    .run()
    .await
}
```

---

## 四、测试说明

### 4.1 单元测试

#### test_metrics_registry_creation

测试 MetricsRegistry 创建功能。

```rust
#[test]
fn test_metrics_registry_creation() {
    let registry = MetricsRegistry::new();
    assert!(registry.to_prometheus_format().is_empty());
}
```

**预期结果**: 通过 - 空注册表输出空字符串

#### test_metrics_registry_with_default_metrics

测试注册指标收集器并导出功能。

```rust
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
```

**预期结果**: 通过 - 输出包含 `sqlrustgo_queries`

### 4.2 集成测试

#### test_metrics_endpoint_handler

测试 HTTP 端点功能。

```rust
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
```

**预期结果**: 
- HTTP 状态码 200
- 响应包含 "sqlrustgo"

---

## 五、运行测试

### 5.1 运行 server crate 测试

```bash
cargo test --package sqlrustgo-server
```

**预期输出**:
```
running 15 tests
test health::tests::test_health_checker_creation ... ok
test health::tests::test_component_health ... ok
...
test metrics_endpoint::tests::test_metrics_endpoint_handler ... ok

test result: ok. 15 passed; 0 failed; 0 ignored
```

### 5.2 运行所有测试

```bash
cargo test --workspace
```

### 5.3 运行 clippy 检查

```bash
cargo clippy --package sqlrustgo-server
```

**预期结果**: 无警告

---

## 六、端点使用示例

### 6.1 手动测试

```bash
# 启动服务后，执行:
curl http://localhost:5432/metrics
```

**预期响应** (Prometheus 格式):
```
# TYPE sqlrustgo_queries_total counter
sqlrustgo_queries_total 1

# TYPE sqlrustgo_query_duration_seconds gauge
sqlrustgo_query_duration_seconds 100
```

### 6.2 Prometheus 配置

在 Prometheus 配置中添加:

```yaml
scrape_configs:
  - job_name: 'sqlrustgo'
    static_configs:
      - targets: ['localhost:5432']
```

### 6.3 Grafana 集成

1. 添加 Prometheus 数据源
2. 查询指标: `sqlrustgo_queries_total`
3. 创建 Dashboard 可视化

---

## 七、依赖版本

| 依赖 | 版本 |
|------|------|
| actix-web | 4 |
| actix-rt | 2 |
| sqlrustgo-common | 1.3.0 |
| sqlrustgo-server | 1.3.0 |

---

## 八、变更日志

| 日期 | 变更 | 描述 |
|------|------|------|
| 2026-03-15 | 初始实现 | 添加 actix-web 集成和 /metrics 端点 |
| 2026-03-15 | 添加测试 | 添加单元测试和集成测试 |
| 2026-03-15 | 文档更新 | 更新 OBSERVABILITY_GUIDE.md |
| 2026-03-15 | 优化 v1 | 性能优化: String::with_capacity 预分配内存, write! 替代 format! |
| 2026-03-15 | 优化 v2 | 功能增强: 添加 HELP 注释支持, Clone trait, register_help API |
| 2026-03-15 | 测试增强 | 新增 3 个单元测试: help_text, clone, custom_metrics |

---

## 九、检查清单

- [x] actix-web 依赖添加
- [x] MetricsRegistry 实现
- [x] metrics_handler 实现
- [x] Prometheus 格式化输出
- [x] 单元测试 (5 个)
- [x] 集成测试 (1 个)
- [x] clippy 检查通过
- [x] 文档更新
- [x] 性能优化 (String::with_capacity, write!)
- [x] HELP 注释支持
- [x] Clone trait 支持

---

*文档版本: 1.0*
*最后更新: 2026-03-15*
