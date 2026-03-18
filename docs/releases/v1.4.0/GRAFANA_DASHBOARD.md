# M-005 Grafana Dashboard 使用文档

> **功能**: Grafana Dashboard 模板
> **版本**: v1.4.0
> **创建日期**: 2026-03-15
> **状态**: ✅ 已完成

---

## 一、概述

M-005 提供了 SQLRustGo 的 Grafana Dashboard 模板，用于可视化监控数据库运行状态。

### 1.1 Dashboard 文件

| 文件路径 | 描述 |
|----------|------|
| `docs/monitoring/grafana-dashboard.json` | Grafana Dashboard 导出文件 |

---

## 二、面板说明

Dashboard 包含 7 个监控面板：

| 面板名称 | 类型 | 指标 | 描述 |
|----------|------|------|------|
| Query Rate (QPS) | Graph | `rate(sqlrustgo_queries_total[5m])` | 每秒查询数 |
| Avg Query Latency | Graph | `rate(sqlrustgo_query_duration_seconds[5m])` | 平均查询延迟 |
| Cache Hit Ratio | Graph | `sqlrustgo_cache_hits / (sqlrustgo_cache_hits + sqlrustgo_cache_misses)` | 缓存命中率 |
| Queries by Type | Pie Chart | `sqlrustgo_queries_total by type` | 按类型统计查询 |
| Storage I/O | Graph | `sqlrustgo_bytes_read/written` | 存储读写 |
| Error Count | Graph | `sqlrustgo_queries_failed_total` | 错误计数 |
| Query Latency Percentiles | Graph | `histogram_quantile()` | 延迟百分位 |

---

## 三、导入步骤

### 3.1 Grafana 配置

1. 登录 Grafana
2. 添加 Prometheus 数据源:
   - URL: `http://localhost:9090`
   - Access: Browser

### 3.2 导入 Dashboard

```bash
# 方式1: Web 导入
1. Grafana -> Dashboards -> Import
2. 上传 grafana-dashboard.json
3. 选择 Prometheus 数据源
4. 点击 Import

# 方式2: API 导入
curl -X POST -H "Content-Type: application/json" \
  -d @docs/monitoring/grafana-dashboard.json \
  http://localhost:3000/api/dashboards/db
```

### 3.3 变量配置

Dashboard 支持以下变量：

| 变量 | 值 | 描述 |
|------|-----|------|
| `instance` | `localhost:5432` | 数据库实例 |

---

## 四、验证

### 4.1 验证指标

```promql
# 查询是否有数据
sqlrustgo_queries_total

# 检查查询速率
rate(sqlrustgo_queries_total[5m])
```

### 4.2 预期效果

- Query Rate: 显示 QPS 曲线
- Latency: 显示 p50/p95/p99 延迟
- Cache: 显示缓存命中率趋势
- Errors: 显示错误计数

---

## 五、依赖

- Prometheus 数据源
- /metrics 端点可用 (M-004)
- 指标收集启用

---

## 六、变更历史

| 日期 | 变更 |
|------|------|
| 2026-03-15 | v1.4.0 M-005 完成 |

---

## 七、检查清单

- [x] Grafana Dashboard JSON 有效
- [x] 7 个监控面板完整
- [x] 文档更新
- [x] 状态标记为完成

---

*文档版本: 1.0*
*最后更新: 2026-03-15*
