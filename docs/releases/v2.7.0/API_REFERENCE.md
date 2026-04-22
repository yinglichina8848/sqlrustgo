# SQLRustGo REST API 参考

> **版本**: v2.7.0
> **更新日期**: 2026-04-22

---

## 1. 概述

SQLRustGo 提供 HTTP REST API 接口，支持健康检查、监控指标查询等基础功能。

### 基础信息

| 项目 | 值 |
|------|-----|
| 默认端口 | 8080 |
| 默认地址 | http://127.0.0.1:8080 |
| 数据格式 | JSON |
| 字符编码 | UTF-8 |

---

## 2. 健康检查接口

### 2.1 Liveness Probe

检查服务是否存活。

**端点**: `GET /health` 或 `GET /health/live`

**请求示例**:
```bash
curl http://127.0.0.1:8080/health
```

**响应示例**:
```json
{
    "status": "healthy"
}
```

**响应码**:
- `200 OK` - 服务正常

---

### 2.2 Readiness Probe

检查服务是否就绪。

**端点**: `GET /ready` 或 `GET /health/ready`

**请求示例**:
```bash
curl http://127.0.0.1:8080/ready
```

**响应示例**:
```json
{
    "status": "ready",
    "version": "1.4.0"
}
```

**响应码**:
- `200 OK` - 服务就绪

---

## 3. 监控指标接口

### 3.1 Prometheus Metrics

获取 Prometheus 格式的监控指标。

**端点**: `GET /metrics`

**请求示例**:
```bash
curl http://127.0.0.1:8080/metrics
```

**响应示例**:
```text
# HELP sqlrustgo_queries_total Total number of queries executed
# TYPE sqlrustgo_queries_total counter
sqlrustgo_queries_total{type="select"} 1234
sqlrustgo_queries_total{type="insert"} 567
sqlrustgo_queries_total{type="update"} 89
sqlrustgo_queries_total{type="delete"} 12

# HELP sqlrustgo_query_duration_seconds Query execution duration
# TYPE sqlrustgo_query_duration_seconds histogram
sqlrustgo_query_duration_seconds_bucket{le="0.01"} 1000
sqlrustgo_query_duration_seconds_bucket{le="0.1"} 1500
sqlrustgo_query_duration_seconds_bucket{le="1.0"} 1800
sqlrustgo_query_duration_seconds_sum 45.6
sqlrustgo_query_duration_seconds_count 2002
```

**响应头**:
```
Content-Type: text/plain; version=0.0.4
```

---

## 4. 错误响应

### 4.1 错误格式

所有错误响应都遵循以下格式:

```json
{
    "error": "Error Type",
    "message": "Detailed error message"
}
```

### 4.2 错误码

| HTTP 状态码 | 说明 |
|-------------|------|
| 200 | 成功 |
| 400 | 请求格式错误 |
| 404 | 端点不存在 |
| 500 | 服务器内部错误 |

### 4.3 错误示例

**404 Not Found**:
```json
{
    "error": "Not Found",
    "message": "Path '/api/v1/query' not found"
}
```

**400 Bad Request**:
```json
{
    "error": "Bad Request"
}
```

---

## 5. 使用场景

### 5.1 Kubernetes 健康检查配置

```yaml
livenessProbe:
  httpGet:
    path: /health/live
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /health/ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10
```

### 5.2 Prometheus 抓取配置

```yaml
scrape_configs:
  - job_name: 'sqlrustgo'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
```

### 5.3 Grafana 仪表盘

使用以下指标创建 Grafana 仪表盘:

- `sqlrustgo_queries_total` - 查询总数
- `rate(sqlrustgo_queries_total[5m])` - QPS
- `sqlrustgo_query_duration_seconds_bucket` - 查询延迟分布
- `histogram_quantile(0.95, sqlrustgo_query_duration_seconds_bucket)` - P95 延迟

---

## 6. 完整使用示例

### 6.1 启动服务器

```bash
# 终端1: 启动 REST API 服务器
cargo run --release --bin sqlrustgo-server
```

### 6.2 健康检查

```bash
# 检查服务状态
curl http://127.0.0.1:8080/health
# 输出: {"status": "healthy"}

curl http://127.0.0.1:8080/ready
# 输出: {"status":"ready","version":"1.4.0"}
```

### 6.3 获取监控指标

```bash
# 获取 Prometheus 指标
curl http://127.0.0.1:8080/metrics
```

### 6.4 Kubernetes 部署清单

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: sqlrustgo
spec:
  containers:
    - name: sqlrustgo
      image: minzuuniversity/sqlrustgo:v2.7.0
      ports:
        - containerPort: 3306
          name: mysql
        - containerPort: 8080
          name: http
      livenessProbe:
        httpGet:
          path: /health/live
          port: 8080
        initialDelaySeconds: 30
        periodSeconds: 10
      readinessProbe:
        httpGet:
          path: /health/ready
          port: 8080
        initialDelaySeconds: 5
        periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: sqlrustgo-service
spec:
  selector:
    app: sqlrustgo
  ports:
    - name: mysql
      port: 3306
      targetPort: 3306
    - name: http
      port: 8080
      targetPort: 8080
```

---

## 7. 限制与注意事项

### 7.1 当前限制

| 限制项 | 值 | 说明 |
|--------|-----|------|
| 请求超时 | 无 | 当前版本无请求超时限制 |
| 最大请求体 | 无 | 当前版本无请求体大小限制 |
| 并发限制 | 无 | 当前版本无并发连接数限制 |

### 7.2 安全建议

1. **网络隔离**: REST API 默认监听 127.0.0.1，生产环境建议配置防火墙
2. **TLS 加密**: 生产环境建议使用反向代理 (如 Nginx) 添加 TLS
3. **认证授权**: 当前版本无内置认证，建议通过 API Gateway 提供

---

## 8. 相关文档

- [客户端连接指南](./CLIENT_CONNECTION.md)
- [部署指南](./DEPLOYMENT_GUIDE.md)
- [快速开始](./QUICK_START.md)

---

*本文档由 SQLRustGo Team 维护*
