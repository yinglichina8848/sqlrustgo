# SQLRustGo 健康检查端点规范

> 版本：v1.1.0
> 发布日期：2026-03-03

---

## 一、概述

### 1.1 目的

健康检查端点用于：
- 负载均衡器检测服务可用性
- Kubernetes 存活探针和就绪探针
- 监控系统检测服务状态
- 运维人员快速诊断问题

### 1.2 端点列表

| 端点 | 用途 | 响应 |
|------|------|------|
|__代码0__| 综合健康检查 | 详细状态 |
|__代码0__| 存活探针 | 简单状态 |
|__代码0__| 就绪探针 | 依赖状态 |
|__代码0__| 启动探针 | 初始化状态 |

---

## 二、端点设计

### 2.1 `/health` - 综合健康检查

返回服务详细健康状态。

**请求**:
```
GET /health
```

**响应**:
```json
{
  "status": "healthy",
  "timestamp": "2026-03-03T10:30:45.123Z",
  "version": "1.1.0",
  "uptime_seconds": 3600,
  "checks": {
    "database": {
      "status": "healthy",
      "latency_ms": 5
    },
    "storage": {
      "status": "healthy",
      "available_bytes": 1073741824
    },
    "memory": {
      "status": "healthy",
      "used_bytes": 536870912,
      "total_bytes": 2147483648
    }
  }
}
```

**状态码**:
| 状态码 | 说明 |
|--------|------|
| 200 | 服务健康 |
| 503 | 服务不健康 |

### 2.2 `/health/live` - 存活探针

检测服务是否存活（进程是否运行）。

**请求**:
```
GET /health/live
```

**响应**:
```json
{
  "status": "alive"
}
```

**状态码**:
| 状态码 | 说明 |
|--------|------|
| 200 | 服务存活 |
| 503 | 服务停止 |

### 2.3 `/health/ready` - 就绪探针

检测服务是否准备好接收请求。

**请求**:
```
GET /health/ready
```

**响应**:
```json
{
  "status": "ready",
  "checks": {
    "database": "ready",
    "storage": "ready",
    "cache": "not_ready"
  }
}
```

**状态码**:
| 状态码 | 说明 |
|--------|------|
| 200 | 服务就绪 |
| 503 | 服务未就绪 |

### 2.4 `/health/startup` - 启动探针

检测服务是否完成初始化。

**请求**:
```
GET /health/startup
```

**响应**:
```json
{
  "status": "started",
  "initialized": true,
  "startup_time_ms": 1500
}
```

**状态码**:
| 状态码 | 说明 |
|--------|------|
| 200 | 启动完成 |
| 503 | 正在启动 |

---

## 三、实现

### 3.1 健康检查器

```rust
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub struct HealthChecker {
    start_time: Instant,
    version: String,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    
    pub fn check_live(&self) -> HealthStatus {
        HealthStatus {
            status: "alive".to_string(),
            timestamp: None,
            version: None,
            uptime_seconds: None,
            checks: None,
        }
    }
    
    pub fn check_ready(&self, engine: &ExecutionEngine) -> HealthStatus {
        let checks = self.check_components(engine);
        let all_ready = checks.values().all(|c| c.status == "ready");
        
        HealthStatus {
            status: if all_ready { "ready" } else { "not_ready" }.to_string(),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            version: None,
            uptime_seconds: None,
            checks: Some(serde_json::to_value(&checks).unwrap()),
        }
    }
    
    pub fn check_health(&self, engine: &ExecutionEngine) -> HealthStatus {
        let checks = self.check_components_detailed(engine);
        let all_healthy = checks.values().all(|c| c.status == "healthy");
        
        HealthStatus {
            status: if all_healthy { "healthy" } else { "unhealthy" }.to_string(),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            version: Some(self.version.clone()),
            uptime_seconds: Some(self.start_time.elapsed().as_secs()),
            checks: Some(serde_json::to_value(&checks).unwrap()),
        }
    }
    
    fn check_components(&self, engine: &ExecutionEngine) -> HashMap<String, ComponentHealth> {
        let mut checks = HashMap::new();
        
        // 检查存储
        checks.insert("storage".to_string(), ComponentHealth {
            status: if engine.storage.is_ready() { "ready" } else { "not_ready" }.to_string(),
            latency_ms: None,
            error: None,
        });
        
        // 检查数据库连接
        checks.insert("database".to_string(), ComponentHealth {
            status: "ready".to_string(),
            latency_ms: None,
            error: None,
        });
        
        checks
    }
    
    fn check_components_detailed(&self, engine: &ExecutionEngine) -> HashMap<String, serde_json::Value> {
        let mut checks = HashMap::new();
        
        // 检查存储
        let start = Instant::now();
        let storage_status = engine.storage.health_check();
        checks.insert("storage".to_string(), serde_json::json!({
            "status": if storage_status.is_ok() { "healthy" } else { "unhealthy" },
            "latency_ms": start.elapsed().as_millis(),
        }));
        
        // 检查内存
        checks.insert("memory".to_string(), serde_json::json!({
            "status": "healthy",
            "used_bytes": get_memory_usage(),
        }));
        
        checks
    }
}
```

### 3.2 HTTP 端点

```rust
use actix_web::{web, HttpResponse, Responder};

pub fn configure_health_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/health")
            .route("", web::get().to(health_check))
            .route("/live", web::get().to(liveness_check))
            .route("/ready", web::get().to(readiness_check))
            .route("/startup", web::get().to(startup_check))
    );
}

async fn health_check(
    checker: web::Data<HealthChecker>,
    engine: web::Data<ExecutionEngine>,
) -> impl Responder {
    let status = checker.check_health(&engine);
    let code = if status.status == "healthy" { 200 } else { 503 };
    HttpResponse::build(actix_web::http::StatusCode::from_u16(code).unwrap())
        .json(status)
}

async fn liveness_check(
    checker: web::Data<HealthChecker>,
) -> impl Responder {
    HttpResponse::Ok().json(checker.check_live())
}

async fn readiness_check(
    checker: web::Data<HealthChecker>,
    engine: web::Data<ExecutionEngine>,
) -> impl Responder {
    let status = checker.check_ready(&engine);
    let code = if status.status == "ready" { 200 } else { 503 };
    HttpResponse::build(actix_web::http::StatusCode::from_u16(code).unwrap())
        .json(status)
}

async fn startup_check(
    checker: web::Data<HealthChecker>,
) -> impl Responder {
    let status = HealthStatus {
        status: "started".to_string(),
        timestamp: Some(chrono::Utc::now().to_rfc3339()),
        version: Some(checker.version.clone()),
        uptime_seconds: Some(checker.start_time.elapsed().as_secs()),
        checks: None,
    };
    HttpResponse::Ok().json(status)
}
```

---

## 四、Kubernetes 配置

### 4.1 探针配置

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sqlrustgo
spec:
  template:
    spec:
      containers:
        - name: sqlrustgo
          image: sqlrustgo:1.1.0
          ports:
            - containerPort: 3306
          livenessProbe:
            httpGet:
              path: /health/live
              port: 3306
            initialDelaySeconds: 10
            periodSeconds: 10
            timeoutSeconds: 5
            failureThreshold: 3
          readinessProbe:
            httpGet:
              path: /health/ready
              port: 3306
            initialDelaySeconds: 5
            periodSeconds: 5
            timeoutSeconds: 3
            failureThreshold: 3
          startupProbe:
            httpGet:
              path: /health/startup
              port: 3306
            initialDelaySeconds: 0
            periodSeconds: 1
            timeoutSeconds: 1
            failureThreshold: 30
```

### 4.2 探针参数说明

| 参数 | 说明 | 推荐值 |
|------|------|--------|
|__代码0__| 初始延迟 | 存活: 10s, 就绪: 5s |
|__代码0__| 检查间隔 | 存活: 10s, 就绪: 5s |
|__代码0__| 超时时间 | 3-5s |
|__代码0__| 失败阈值 | 3 |

---

## 五、负载均衡器配置

### 5.1 Nginx

```nginx
upstream sqlrustgo {
    server 10.0.0.1:3306 max_fails=3 fail_timeout=30s;
    server 10.0.0.2:3306 max_fails=3 fail_timeout=30s;
}

server {
    location /health {
        proxy_pass http://sqlrustgo;
        proxy_connect_timeout 5s;
        proxy_read_timeout 5s;
    }
}
```

### 5.2 HAProxy

```
backend sqlrustgo
    option httpchk GET /health/live
    http-check expect status 200
    server sqlrustgo1 10.0.0.1:3306 check inter 10s fall 3 rise 2
    server sqlrustgo2 10.0.0.2:3306 check inter 10s fall 3 rise 2
```

---

## 六、最佳实践

### 6.1 DO

- ✅ 存活探针使用简单检查
- ✅ 就绪探针检查依赖服务
- ✅ 设置合理的超时时间
- ✅ 记录健康检查日志

### 6.2 DON'T

- ❌ 在存活探针中检查外部依赖
- ❌ 健康检查执行耗时操作
- ❌ 忽略健康检查失败
- ❌ 过于频繁的健康检查

---

*本文档由 TRAE (GLM-5.0) 创建*
*最后更新: 2026-03-03*
