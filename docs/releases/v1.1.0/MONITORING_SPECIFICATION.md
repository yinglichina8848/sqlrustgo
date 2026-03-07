# SQLRustGo 性能指标监控规范

> 版本：v1.1.0
> 发布日期：2026-03-03

---

## 一、监控架构

### 1.1 架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          监控架构                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────┐     ┌──────────────┐     ┌──────────────┐               │
│   │  SQLRustGo   │────▶│  Prometheus  │────▶│   Grafana    │               │
│   │   Server     │     │   Server     │     │  Dashboard   │               │
│   └──────────────┘     └──────────────┘     └──────────────┘               │
│          │                    │                                             │
│          │                    ▼                                             │
│          │             ┌──────────────┐                                    │
│          │             │   Alert      │                                    │
│          │             │   Manager    │                                    │
│          │             └──────────────┘                                    │
│          ▼                                                                   │
│   ┌──────────────┐                                                          │
│   │  /metrics    │                                                          │
│   │  endpoint    │                                                          │
│   └──────────────┘                                                          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 技术栈

| 组件 | 用途 | 版本 |
|------|------|------|
|普罗米修斯| 指标收集和存储 | ≥2.40 |
|格拉法纳| 可视化仪表盘 | ≥10.0 |
|警报管理器| 告警管理 | ≥0.25 |

---

## 二、核心指标

### 2.1 请求指标

| 指标名称 | 类型 | 说明 | 标签 |
|----------|------|------|------|
|__代码0__|柜台| 请求总数 |__代码0__，__代码1__|
|__代码0__|直方图| 请求延迟 |__代码0__|
|__代码0__| Gauge | 活跃连接数 | - |
|__代码0__|柜台| 查询总数 | `type` |

### 2.2 执行器指标

| 指标名称 | 类型 | 说明 | 标签 |
|----------|------|------|------|
|__代码0__|直方图| 查询执行时间 |__代码0__|
|__代码0__|柜台| 处理行数 | `table` |
|__代码0__|柜台| 索引命中次数 |__代码0__，__代码1__|
|__代码0__|柜台| 索引未命中次数 | `table` |

### 2.3 存储指标

| 指标名称 | 类型 | 说明 | 标签 |
|----------|------|------|------|
|__代码0__|柜台| 读取字节数 | `table` |
|__代码0__|柜台| 写入字节数 | `table` |
|__代码0__| Gauge | 表行数 | `table` |
|__代码0__| Gauge | 表大小 | `table` |

### 2.4 网络指标

| 指标名称 | 类型 | 说明 | 标签 |
|----------|------|------|------|
|__代码0__|柜台| 连接总数 |__代码0__|
|__代码0__|直方图| 连接持续时间 | - |
|__代码0__|柜台| 接收字节数 | - |
|__代码0__|柜台| 发送字节数 | - |

---

## 三、指标实现

### 3.1 依赖配置

```toml
# Cargo.toml
[dependencies]
prometheus = "0.13"
lazy_static = "1.4"
```

### 3.2 指标定义

```rust
use prometheus::{Counter, CounterVec, Histogram, HistogramVec, Gauge, GaugeVec, Registry};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    
    pub static ref REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "sqlrustgo_requests_total",
        "Total number of requests",
        &["method", "status"]
    ).unwrap();
    
    pub static ref REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "sqlrustgo_request_duration_seconds",
        "Request duration in seconds",
        &["method"],
        vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
    ).unwrap();
    
    pub static ref ACTIVE_CONNECTIONS: Gauge = register_gauge!(
        "sqlrustgo_active_connections",
        "Number of active connections"
    ).unwrap();
    
    pub static ref QUERY_DURATION: HistogramVec = register_histogram_vec!(
        "sqlrustgo_query_duration_seconds",
        "Query execution duration in seconds",
        &["query_type"],
        vec![0.0001, 0.001, 0.01, 0.1, 1.0]
    ).unwrap();
}
```

### 3.3 指标使用

```rust
// 记录请求
pub async fn handle_query(query: &str) -> SqlResult<QueryResult> {
    let timer = REQUEST_DURATION.with_label_values(&["query"]).start_timer();
    
    let result = execute_query(query);
    
    match &result {
        Ok(_) => REQUESTS_TOTAL.with_label_values(&["query", "success"]).inc(),
        Err(_) => REQUESTS_TOTAL.with_label_values(&["query", "error"]).inc(),
    }
    
    timer.observe_duration();
    result
}

// 连接管理
pub fn on_connect() {
    ACTIVE_CONNECTIONS.inc();
}

pub fn on_disconnect() {
    ACTIVE_CONNECTIONS.dec();
}
```

### 3.4 Metrics 端点

```rust
use actix_web::{web, HttpResponse, Responder};
use prometheus::Encoder;

pub async fn metrics() -> impl Responder {
    let encoder = prometheus::TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer)
}
```

---

## 四、Grafana 仪表盘

### 4.1 仪表盘配置

```json
{
  "dashboard": {
    "title": "SQLRustGo Dashboard",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(sqlrustgo_requests_total[5m])",
            "legendFormat": "{{method}} - {{status}}"
          }
        ]
      },
      {
        "title": "Request Latency",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, rate(sqlrustgo_request_duration_seconds_bucket[5m]))",
            "legendFormat": "p99"
          },
          {
            "expr": "histogram_quantile(0.95, rate(sqlrustgo_request_duration_seconds_bucket[5m]))",
            "legendFormat": "p95"
          }
        ]
      },
      {
        "title": "Active Connections",
        "type": "gauge",
        "targets": [
          {
            "expr": "sqlrustgo_active_connections"
          }
        ]
      }
    ]
  }
}
```

### 4.2 关键面板

| 面板名称 | 指标 | 说明 |
|----------|------|------|
| 请求速率 |__代码0__| 每秒请求数 |
| 请求延迟 P99 |__代码0__| 99% 请求延迟 |
| 活跃连接数 |__代码0__| 当前连接数 |
| 查询执行时间 |__代码0__| 查询延迟分布 |
| 索引命中率 |__代码0__| 索引效率 |

---

## 五、告警规则

### 5.1 Prometheus 告警配置

```yaml
groups:
  - name: sqlrustgo
    rules:
      - alert: HighErrorRate
        expr: |
          rate(sqlrustgo_requests_total{status="error"}[5m]) 
          / rate(sqlrustgo_requests_total[5m]) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }}"
      
      - alert: HighLatency
        expr: |
          histogram_quantile(0.99, 
            rate(sqlrustgo_request_duration_seconds_bucket[5m])
          ) > 1.0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High request latency"
          description: "P99 latency is {{ $value }}s"
      
      - alert: TooManyConnections
        expr: sqlrustgo_active_connections > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Too many active connections"
          description: "Active connections: {{ $value }}"
      
      - alert: LowIndexHitRate
        expr: |
          rate(sqlrustgo_index_hits_total[5m]) 
          / (rate(sqlrustgo_index_hits_total[5m]) + rate(sqlrustgo_index_misses_total[5m])) < 0.5
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Low index hit rate"
          description: "Index hit rate is {{ $value | humanizePercentage }}"
```

---

## 六、性能基准

### 6.1 目标指标

| 指标 | 目标值 | 说明 |
|------|--------|------|
| P99 延迟 | < 100ms | 99% 请求延迟 |
| 吞吐量 | > 10000 QPS | 每秒查询数 |
| 错误率 | < 0.1% | 请求错误率 |
| 连接数 | < 1000 | 最大连接数 |

### 6.2 基准测试

```bash
# 使用 wrk 进行基准测试
wrk -t4 -c100 -d30s http://localhost:3306/query

# 使用 sysbench 进行数据库基准测试
sysbench oltp_read_write run --tables=10 --table-size=10000
```

---

## 七、监控最佳实践

### 7.1 DO

- ✅ 使用 Histogram 记录延迟分布
- ✅ 为指标添加有意义的标签
- ✅ 设置合理的告警阈值
- ✅ 定期审查仪表盘

### 7.2 DON'T

- ❌ 使用 Counter 记录可减小的值
- ❌ 过度使用标签（基数爆炸）
- ❌ 忽略告警
- ❌ 在生产环境使用 TRACE 级别日志

---

*本文档由 TRAE (GLM-5.0) 创建*
*最后更新: 2026-03-03*
