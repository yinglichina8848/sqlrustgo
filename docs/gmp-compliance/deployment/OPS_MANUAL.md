# 运维手册

**版本**: v3.1.0  
**日期**: 2026-05-14  
**状态**: Beta 完成

---

## 一、日常运维

### 1.1 健康检查

```bash
#!/bin/bash
# 健康检查脚本

# 检查进程
pgrep -f sqlrustgo || echo "CRITICAL: Process not running"

# 检查端口
netstat -an | grep 3306 || echo "CRITICAL: Port 3306 not listening"

# 检查磁盘空间
df -h / | awk '{print $5}' | grep -v Use | cut -d'%' -f1 | \
  while read pct; do
    if [ "$pct" -gt 90 ]; then
      echo "CRITICAL: Disk usage $pct%"
    fi
  done

# 检查审计链完整性
./sqlrustgo audit verify --chain || echo "WARNING: Hash chain verification failed"

# 检查 WAL
./sqlrustgo db status --wal
```

### 1.2 监控指标

| 指标 | 阈值 | 告警级别 |
|------|------|----------|
| CPU 使用率 | > 80% | Warning |
| 内存使用 | > 85% | Warning |
| 磁盘使用 | > 90% | Critical |
| 连接数 | > 1000 | Warning |
| 审计链断裂 | 1 | Critical |
| WAL 错误 | 1 | Critical |

---

## 二、审计日志管理

### 2.1 审计日志备份

```bash
#!/bin/bash
# 审计日志备份脚本

BACKUP_DIR="/var/backup/audit"
DATE=$(date +%Y%m%d)

# 本地备份
cp -r /var/lib/sqlrustgo/audit "$BACKUP_DIR/audit-$DATE"

# 压缩
tar -czf "$BACKUP_DIR/audit-$DATE.tar.gz" "$BACKUP_DIR/audit-$DATE"

# 上传 S3
aws s3 cp "$BACKUP_DIR/audit-$DATE.tar.gz" \
  s3://audit-backup-prod/audit-$DATE.tar.gz

# 删除本地备份保留最近 7 天
find "$BACKUP_DIR" -mtime +7 -delete
```

### 2.2 审计日志归档策略

| 级别 | 保留期 | 存储 |
|------|--------|------|
| 热数据 | 90 天 | 本地 SSD |
| 温数据 | 1 年 | S3 Standard |
| 冷数据 | 7 年 | S3 Glacier |

### 2.3 审计恢复

```bash
# 从备份恢复审计日志
./sqlrustgo audit restore --from s3://audit-backup/audit-20240101.tar.gz

# 验证恢复完整性
./sqlrustgo audit verify --full
```

---

## 三、异常恢复流程

### 3.1 崩溃恢复

```
1. 检测崩溃
   - 监控告警
   - 健康检查失败

2. 自动恢复
   - WAL 重放
   - Checkpoint 恢复
   - 连接重建

3. 验证完整性
   - ./sqlrustgo audit verify --chain
   - ./sqlrustgo db verify --consistency

4. 通知相关方
   - 发送告警
   - 生成恢复报告
```

### 3.2 数据损坏恢复

```bash
# 1. 检测损坏
./sqlrustgo db verify --page-crc

# 2. 停止写入
./sqlrustgo admin pause-writes

# 3. 从 Checkpoint 恢复
./sqlrustgo db restore --checkpoint latest

# 4. 重放 WAL
./sqlrustgo db replay-wal --from-checkpoint

# 5. 验证
./sqlrustgo audit verify --chain
```

### 3.3 审计链断裂处理

```bash
# 1. 检测断裂
./sqlrustgo audit verify --chain

# 2. 确定断裂点
./sqlrustgo audit find-break --start-time "2024-01-01"

# 3. 调查原因
# - 检查存储故障
# - 检查时间回拨
# - 检查恶意篡改

# 4. 报告并修复
./sqlrustgo audit repair --from <last_valid_index>
```

---

## 四、密钥轮换

### 4.1 签名密钥轮换流程

```bash
#!/bin/bash
# 密钥轮换脚本

# 1. 生成新密钥
openssl genrsa -out new_private.pem 2048
openssl rsa -in new_private.pem -pubout -out new_public.pem

# 2. 备份旧密钥 (保留 30 天)
mv audit_private.pem audit_private.pem.old
mv audit_public.pem audit_public.pem.old

# 3. 部署新密钥
mv new_private.pem audit_private.pem
mv new_public.pem audit_public.pem

# 4. 重新加载服务
./sqlrustgo admin reload-config

# 5. 验证
./sqlrustgo audit verify --chain
```

### 4.2 密钥轮换时间表

| 密钥类型 | 轮换周期 | 保留期 | 状态 |
|----------|----------|--------|------|
| 审计签名 | 90 天 | 2 年 | ⚠️ 需配置 |
| TLS 证书 | 1 年 | 2 年 | ⚠️ 需配置 |
| 存储加密 | 1 年 | N/A | ⚠️ v3.2.0 |

---

## 五、容量规划

### 5.1 存储计算

```
审计日志大小估算:
- 每条事件: ~1 KB
- 每秒事件: ~10 (典型负载)
- 每天事件: 10 * 60 * 60 * 24 = 864,000
- 每天存储: 864,000 KB ≈ 864 MB
- 每年存储: 864 MB * 365 ≈ 315 GB
- 7 年保留: 315 GB * 7 ≈ 2.2 TB
```

### 5.2 资源规划

| 组件 | 最低配置 | 推荐配置 |
|------|----------|----------|
| CPU | 4 核 | 8 核 |
| 内存 | 8 GB | 16 GB |
| 存储 (数据) | 100 GB | 500 GB |
| 存储 (审计) | 500 GB | 2 TB |
| 存储 (WAL) | 50 GB | 100 GB |

---

## 六、灾难恢复

### 6.1 RTO/RPO 目标

| 指标 | 目标 | 说明 |
|------|------|------|
| RTO | < 1 小时 | 恢复时间目标 |
| RPO | < 1 小时 | 恢复点目标 |

### 6.2 备份策略

```yaml
backup:
  frequency:
    full: daily
    incremental: hourly
  retention:
    daily: 7
    weekly: 4
    monthly: 12
    yearly: 7
  destinations:
    - type: s3
      bucket: s3://sqlrustgo-backup
      region: us-east-1
    - type: local
      path: /var/backup/sqlrustgo
```

### 6.3 恢复演练

```bash
# 每季度执行恢复演练
./sqlrustgo disaster-recovery drill \
  --backup s3://backup/latest.tar.gz \
  --verify-audit-chain \
  --verify-data-integrity
```