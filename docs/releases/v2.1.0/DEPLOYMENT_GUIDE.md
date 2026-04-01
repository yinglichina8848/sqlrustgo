# SQLRustGo v2.1.0 部署指南

**版本**: v2.1.0
**更新日期**: 2026-04-02

---

## 一、系统要求

### 1.1 硬件要求

| 组件 | 最低配置 | 推荐配置 |
|------|----------|----------|
| CPU | 2 核 | 8 核 |
| 内存 | 4 GB | 16 GB |
| 磁盘 | 10 GB | 100 GB SSD |
| 网络 | 100 Mbps | 1 Gbps |

### 1.2 软件要求

| 软件 | 版本要求 |
|------|----------|
| Rust | 1.70+ |
| OpenSSL | 1.1+ |
| Linux/macOS | 最新稳定版 |

---

## 二、部署步骤

### 2.1 从源码编译

```bash
# 克隆仓库
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 切换到 v2.1.0
git checkout develop/v2.1.0

# 编译 release 版本
cargo build --release

# 安装
cargo install --path . --release
```

### 2.2 配置

```bash
# 创建配置目录
sudo mkdir -p /etc/sqlrustgo
sudo mkdir -p /var/log/sqlrustgo
sudo mkdir -p /data/db
sudo mkdir -p /data/wal
sudo mkdir -p /data/backup

# 复制配置模板
sudo cp etc/sqlrustgo.toml /etc/sqlrustgo/
sudo cp etc/logging.toml /etc/sqlrustgo/
```

### 2.3 配置文件

```toml
# /etc/sqlrustgo/sqlrustgo.toml

[server]
host = "0.0.0.0"
port = 5432
workers = 4

[storage]
type = "memory"
data_dir = "/data/db"

[observability]
prometheus_enabled = true
metrics_port = 9090
slow_query_threshold_ms = 1000

[firewall]
enabled = true
alert_on_suspicious = true

[backup]
dir = "/data/backup"
wal_dir = "/data/wal"
```

---

## 三、系统服务

### 3.1 systemd 服务 (Linux)

```ini
# /etc/systemd/system/sqlrustgo.service

[Unit]
Description=SQLRustGo Database Server
After=network.target

[Service]
Type=simple
User=sqlrustgo
Group=sqlrustgo
ExecStart=/usr/local/bin/sqlrustgo-server --config /etc/sqlrustgo/sqlrustgo.toml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
# 启用服务
sudo systemctl enable sqlrustgo
sudo systemctl start sqlrustgo

# 检查状态
sudo systemctl status sqlrustgo
```

### 3.2 日志配置

```toml
# /etc/sqlrustgo/logging.toml

[logging]
level = "info"
format = "json"

[logging.file]
enabled = true
path = "/var/log/sqlrustgo/server.log"
max_size_mb = 100
max_backups = 10

[logging.slow_query]
enabled = true
path = "/var/log/sqlrustgo/slow_query.log"
threshold_ms = 1000

[logging.firewall]
enabled = true
path = "/var/log/sqlrustgo/firewall_alerts.log"
```

---

## 四、备份部署

### 4.1 每日全量备份

```bash
#!/bin/bash
# /etc/cron.daily/sqlrustgo-backup

BACKUP_DIR=/data/backup
DATA_DIR=/data/db
WAL_DIR=/data/wal
LOG_FILE=/var/log/sqlrustgo/backup.log

date >> $LOG_FILE
sqlrustgo-tools physical-backup backup \
    --dir $BACKUP_DIR \
    --data-dir $DATA_DIR \
    --wal-dir $WAL_DIR \
    --compression gzip \
    >> $LOG_FILE 2>&1

# 保留策略：最近7个
sqlrustgo-tools physical-backup prune \
    --dir $BACKUP_DIR \
    --keep 7 \
    --force \
    >> $LOG_FILE 2>&1
```

### 4.2 增量备份 (每小时)

```bash
#!/bin/bash
# /etc/cron.hourly/sqlrustgo-incremental-backup

BACKUP_DIR=/data/backup/incremental
DATA_DIR=/data/db
WAL_DIR=/data/wal

sqlrustgo-tools physical-backup backup \
    --dir $BACKUP_DIR \
    --data-dir $DATA_DIR \
    --wal-dir $WAL_DIR \
    --incremental
```

---

## 五、监控部署

### 5.1 Prometheus 配置

```yaml
# /etc/prometheus/prometheus.yml

scrape_configs:
  - job_name: 'sqlrustgo'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### 5.2 Grafana Dashboard

导入 `docs/monitoring/grafana-dashboard.json` 到 Grafana。

---

## 六、高可用部署

### 6.1 主从复制

```toml
# 主库配置
[replication]
enabled = true
role = "master"
binlog_dir = "/data/binlog"

# 从库配置
[replication]
enabled = true
role = "slave"
master_host = "master.example.com"
master_port = 5432
```

### 6.2 故障转移

```bash
# 配置 keepalived
# /etc/keepalived/keepalived.conf

vrrp_instance VI_1 {
    state BACKUP
    interface eth0
    virtual_router_id 51
    priority 100
    advert_int 1
    virtual_ipaddress {
        192.168.1.100
    }
}
```

---

## 七、验证部署

### 7.1 健康检查

```bash
# 检查服务状态
curl http://localhost:8080/health

# 检查 Prometheus 指标
curl http://localhost:9090/metrics

# 运行测试
cargo test --test regression_test
```

### 7.2 性能验证

```bash
# 运行 TPC-H 基准测试
cargo bench --bench tpch_benchmark

# 验证 QPS ≥ 1000
```

---

## 八、故障排查

### 8.1 服务启动失败

```bash
# 查看日志
journalctl -u sqlrustgo -n 100

# 检查配置
sqlrustgo-server --config /etc/sqlrustgo/sqlrustgo.toml --dry-run
```

### 8.2 性能问题

```bash
# 查看慢查询
tail -f /var/log/sqlrustgo/slow_query.log

# 查看 Prometheus 指标
curl http://localhost:9090/metrics | grep sqlrustgo
```

---

*部署指南 v2.1.0*
