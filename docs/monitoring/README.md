# Grafana Dashboard for SQLRustGo

This directory contains Grafana dashboard templates for monitoring SQLRustGo database metrics.

## Files

- `grafana-dashboard.json` - Grafana dashboard template

## Importing the Dashboard

### Option 1: Grafana UI

1. Open Grafana in your browser
2. Navigate to **Dashboards** → **Import**
3. Click **Upload JSON file** and select `grafana-dashboard.json`
4. Configure the datasource (Prometheus)
5. Click **Import**

### Option 2: Grafana HTTP API

```bash
curl -X POST https://grafana.example.com/api/dashboards/db \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $GRAFANA_API_KEY" \
  -d @docs/monitoring/grafana-dashboard.json
```

## Dashboard Panels

### Overview Section
- **Query Rate (QPS)** - Queries per second
- **Avg Query Latency** - Average query execution time
- **Cache Hit Ratio** - Buffer pool hit ratio

### Query Performance
- **Queries by Type** - Breakdown by SELECT/INSERT/UPDATE/DELETE
- **Query Latency Percentiles** - p50, p95, p99 latency
- **Error Count** - Total error count over time

### Storage Metrics
- **Storage I/O** - Bytes read/written per second

## Required Metrics

The dashboard expects the following Prometheus metrics:

| Metric Name | Type | Description |
|-------------|------|-------------|
| `sqlrustgo_queries_total` | Counter | Total query count |
| `sqlrustgo_queries_by_type_total` | Counter | Queries by type |
| `sqlrustgo_query_duration_seconds` | Histogram | Query duration |
| `sqlrustgo_cache_hits` | Counter | Cache hit count |
| `sqlrustgo_cache_misses` | Counter | Cache miss count |
| `sqlrustgo_storage_bytes_read_total` | Counter | Bytes read |
| `sqlrustgo_storage_bytes_written_total` | Counter | Bytes written |
| `sqlrustgo_errors_total` | Counter | Error count |

## Customization

Edit `grafana-dashboard.json` to customize:
- Panel layouts
- Thresholds
- Color schemes
- Refresh intervals
- Time ranges

## Dependencies

- Grafana 8.0+
- Prometheus data source
- M-004 /metrics endpoint implemented
