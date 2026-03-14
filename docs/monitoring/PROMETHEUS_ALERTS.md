# Prometheus Alert Rules for SQLRustGo

This directory contains Prometheus alert rules for SQLRustGo monitoring.

## Files

- `prometheus-alerts.yml` - Prometheus alert rules

## Alert Rules

### Warning Alerts

| Alert | Description | Condition |
|-------|-------------|-----------|
| HighQueryLatency | High query latency | avg > 1s for 5m |
| HighErrorRate | High error rate | > 5% for 5m |
| LowBufferPoolHitRatio | Low cache hit ratio | < 80% for 10m |
| NoQueries | No queries detected | 0 queries for 5m |
| HighQueryDurationP99 | High p99 latency | p99 > 5s for 5m |

### Critical Alerts

| Alert | Description | Condition |
|-------|-------------|-----------|
| CriticalQueryLatency | Critical latency | avg > 5s for 2m |
| CriticalErrorRate | Critical error rate | > 10% for 2m |
| CriticalBufferPoolHitRatio | Critical cache hit | < 50% for 5m |

### Info Alerts

| Alert | Description | Condition |
|-------|-------------|-----------|
| HighQueryRate | High query rate | > 10k QPS for 10m |
| HighStorageRead | High read I/O | > 1 GB/s for 10m |

## Configuration

### Prometheus Configuration

Add to your `prometheus.yml`:

```yaml
rule_files:
  - "/path/to/prometheus-alerts.yml"
```

### Alertmanager Configuration

Add receivers for notifications:

```yaml
alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

route:
  group_by: ['alertname']
  receiver: 'default'
  routes:
    - match:
        severity: critical
      receiver: 'critical-alerts'
    - match:
        severity: warning
      receiver: 'warning-alerts'

receivers:
  - name: 'default'
  - name: 'critical-alerts'
  - name: 'warning-alerts'
```

## Testing Alerts

Test alerts with Prometheus web UI:
1. Navigate to /rules
2. Verify alert rules are loaded
3. Check /alerts for firing alerts

## Customization

Edit `prometheus-alerts.yml` to adjust:
- Thresholds
- Duration
- Labels
- Annotations

## Dependencies

- M-003 Prometheus Metrics Format
- M-004 /metrics HTTP Endpoint
- M-005 Grafana Dashboard
