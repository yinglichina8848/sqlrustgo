# SQLRustGo v1.3.0 健康检查说明

> **版本**: v1.3.0
> **日期**: 2026-03-15

---

## 概述

v1.3.0 引入了健康检查端点，用于支持 Kubernetes/容器编排平台的存活探针和就绪探针。

---

## 端点说明

### GET /health/live - 存活探针

用于判断容器是否存活。返回简单状态，表示进程是否在运行。

**响应示例:**

```json
{
  "status": "healthy"
}
```

**用途:**
- Kubernetes livenessProbe
- 判断进程是否崩溃需要重启

---

### GET /health/ready - 就绪探针

用于判断服务是否准备好接收请求。检查各组件健康状态。

**响应示例:**

```json
{
  "status": "healthy",
  "version": "1.3.0",
  "timestamp": 1710489600000,
  "components": [
    {
      "name": "system",
      "status": "healthy",
      "message": "System is operational",
      "latency_ms": 0
    }
  ]
}
```

**组件状态说明:**

| 状态 | 含义 |
|------|------|
| healthy | 组件正常运行 |
| degraded | 组件部分降级 |
| unhealthy | 组件不可用 |

**用途:**
- Kubernetes readinessProbe
- 负载均衡器判断是否路由流量
- 滚动更新时等待服务就绪

---

## 使用示例

### curl 命令验证

```bash
# 检查存活探针
curl http://localhost:3306/health/live

# 检查就绪探针
curl http://localhost:3306/health/ready
```

### Kubernetes 配置示例

```yaml
livenessProbe:
  httpGet:
    path: /health/live
    port: 3306
  initialDelaySeconds: 5
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /health/ready
    port: 3306
  initialDelaySeconds: 5
  periodSeconds: 10
```

---

## 扩展健康检查

可以通过实现 `HealthComponent` trait 来添加自定义组件检查：

```rust
use sqlrustgo_server::health::{HealthComponent, ComponentHealth, HealthStatus};

pub struct StorageHealthComponent;

impl HealthComponent for StorageHealthComponent {
    fn name(&self) -> &str {
        "storage"
    }

    fn check(&self) -> ComponentHealth {
        ComponentHealth::new("storage", HealthStatus::Healthy)
            .with_message("Storage is operational")
    }
}
```

---

## 相关文档

- [RELEASE_NOTES.md](./RELEASE_NOTES.md)
- [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md)
