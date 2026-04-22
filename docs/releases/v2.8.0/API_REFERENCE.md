# SQLRustGo REST API Reference

> Version: `v2.8.0`
> Last Updated: 2026-04-23

---

## Overview

SQLRustGo provides HTTP REST API for health checks, metrics, and query execution.

## Base Information

| Item | Value |
|------|-------|
| Default Port | 8080 |
| Default Address | http://127.0.0.1:8080 |
| Data Format | JSON |
| Character Encoding | UTF-8 |

---

## Health Check Endpoints

### Liveness Probe

Check if service is alive.

**Endpoint**: `GET /health` or `GET /health/live`

**Request**:
```bash
curl http://127.0.0.1:8080/health
```

**Response**:
```json
{
    "status": "healthy"
}
```

**Status Codes**:
- `200 OK` - Service is healthy

---

### Readiness Probe

Check if service is ready.

**Endpoint**: `GET /ready` or `GET /health/ready`

**Request**:
```bash
curl http://127.0.0.1:8080/ready
```

**Response**:
```json
{
    "status": "ready",
    "version": "2.8.0"
}
```

---

## Query Endpoints

### Execute SQL

Execute a SQL query.

**Endpoint**: `POST /query`

**Request**:
```bash
curl -X POST http://127.0.0.1:8080/query \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users WHERE id = 1"}'
```

**Response**:
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

---

### Execute with Parameters

Execute with prepared statement parameters.

**Endpoint**: `POST /query/params`

**Request**:
```bash
curl -X POST http://127.0.0.1:8080/query/params \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users WHERE id = ?", "params": [1]}'
```

---

## Metrics Endpoint

### Prometheus Metrics

Get Prometheus-format metrics.

**Endpoint**: `GET /metrics`

**Request**:
```bash
curl http://127.0.0.1:8080/metrics
```

**Response**:
```text
# HELP sqlrustgo_queries_total Total number of queries executed
# TYPE sqlrustgo_queries_total counter
sqlrustgo_queries_total{type="select"} 1234

# HELP sqlrustgo_query_duration_seconds Query execution duration
# TYPE sqlrustgo_query_duration_seconds histogram
sqlrustgo_query_duration_seconds_bucket{le="0.01"} 1000
```

---

## Table Management

### List Tables

**Endpoint**: `GET /tables`

**Response**:
```json
{
    "tables": ["users", "orders", "products"]
}
```

### Table Schema

**Endpoint**: `GET /tables/{table_name}`

**Response**:
```json
{
    "name": "users",
    "columns": [
        {"name": "id", "type": "INTEGER", "nullable": false},
        {"name": "name", "type": "TEXT", "nullable": true}
    ],
    "primary_key": ["id"]
}
```

---

## Transaction Management

### Begin Transaction

**Endpoint**: `POST /transaction/begin`

**Response**:
```json
{
    "transaction_id": "tx_123456"
}
```

### Commit Transaction

**Endpoint**: `POST /transaction/{transaction_id}/commit`

### Rollback Transaction

**Endpoint**: `POST /transaction/{transaction_id}/rollback`

---

## Error Responses

All errors follow this format:

```json
{
    "error": {
        "code": 1146,
        "type": "TableNotFound",
        "message": "Table 'users' doesn't exist"
    }
}
```

### Common Error Codes

| Code | Type | Description |
|------|------|-------------|
| 1000 | ParseError | SQL syntax error |
| 1006 | TableNotFound | Table does not exist |
| 1007 | ColumnNotFound | Column does not exist |
| 1005 | ConstraintViolation | Constraint check failed |
| 1008 | DuplicateKey | Duplicate key violation |

---

## MySQL Wire Protocol

SQLRustGo also supports MySQL wire protocol on port 3306:

```bash
mysql -h 127.0.0.1 -P 3306 -u root -p
```

---

## Related Documentation

- [Error Messages Reference](./ERROR_MESSAGES.md)
- [Security Hardening Guide](./SECURITY_HARDENING.md)
- [Client Connection Guide](./CLIENT_CONNECTION.md)