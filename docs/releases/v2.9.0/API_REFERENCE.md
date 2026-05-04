# SQLRustGo REST API Reference

> **版本**: v2.9.0
> **更新日期**: 2026-05-04

---

## 概述

SQLRustGo 提供 HTTP REST API 用于健康检查、指标和查询执行。

## 基本信息

| 项目 | 值 |
|------|-----|
| 默认端口 | 8080 |
| 默认地址 | http://127.0.0.1:8080 |
| 数据格式 | JSON |
| 字符编码 | UTF-8 |

---

## 健康检查端点

### 存活探针

检查服务是否存活。

**端点**: `GET /health`

**请求**:
```bash
curl http://127.0.0.1:8080/health
```

**响应**:
```json
{
    "status": "healthy"
}
```

**状态码**:
- `200 OK` - 服务健康

---

### 就绪探针

检查服务是否就绪。

**端点**: `GET /ready`

**请求**:
```bash
curl http://127.0.0.1:8080/ready
```

**响应**:
```json
{
    "status": "ready",
    "version": "2.9.0"
}
```

---

## 查询端点

### 执行 SQL

执行 SQL 查询。

**端点**: `POST /query`

**请求**:
```bash
curl -X POST http://127.0.0.1:8080/query \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users WHERE id = 1"}'
```

**响应**:
```json
{
    "columns": ["id", "name", "email"],
    "rows": [
        [1, "John", "john@example.com"]
    ],
    "row_count": 1,
    "duration_ms": 5
}
```

**错误响应**:
```json
{
    "error": "Table 'users' does not exist",
    "code": "ER_NO_SUCH_TABLE"
}
```

---

## 指标端点

### Prometheus 指标

获取 Prometheus 格式的指标。

**端点**: `GET /metrics`

**请求**:
```bash
curl http://127.0.0.1:8080/metrics
```

**响应**:
```text
# HELP sqlrustgo_queries_total Total number of queries executed
# TYPE sqlrustgo_queries_total counter
sqlrustgo_queries_total{type="select"} 1234

# HELP sqlrustgo_query_duration_seconds Query execution duration
# TYPE sqlrustgo_query_duration_seconds histogram
sqlrustgo_query_duration_seconds_bucket{le="0.01"} 1000
```

---

## MySQL Wire Protocol

### TCP 连接

除了 REST API，SQLRustGo 还支持标准的 MySQL Wire Protocol：

```bash
# 使用 mysql 客户端连接
mysql -h 127.0.0.1 -P 3306 -u root
```

详细连接方式请参考 [CLIENT_CONNECTION.md](./CLIENT_CONNECTION.md)。

---

## 错误码

| 错误码 | 说明 |
|--------|------|
| `ER_NO_SUCH_TABLE` | 表不存在 |
| `ER_DUP_ENTRY` | 重复键冲突 |
| `ER_PARSE_ERROR` | SQL 语法错误 |
| `ER_NO_DB` | 未选择数据库 |
| `ER_ACCESS_DENIED` | 访问被拒绝 |

---

## 相关文档

- [快速开始](./QUICK_START.md)
- [客户端连接指南](./CLIENT_CONNECTION.md)
- [安全报告](./SECURITY_REPORT.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-05-04*