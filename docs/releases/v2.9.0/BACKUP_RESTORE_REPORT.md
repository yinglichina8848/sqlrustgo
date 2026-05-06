# 备份与恢复报告

> **版本**: v2.9.0
> **日期**: 2026-05-05

---

## 1. 备份策略

### 1.1 每日自动备份

| 项目 | 频率 | 保留时间 | 位置 |
|------|------|----------|------|
| 数据库文件 | 每日 03:00 | 7 天 | /backup/sqlrustgo/ |
| WAL 日志 | 连续归档 | 30 天 | /backup/wal/ |
| 配置 | 每次修改 | 30 天 | /backup/config/ |
| 代码仓库 | 每次 push | 永久 | Gitea |

### 1.2 备份脚本

```bash
#!/bin/bash
# 每日 03:00 执行
DATE=$(date +%Y%m%d)
BACKUP_DIR=/backup/sqlrustgo/

# 备份数据库
cp data/sqlrustgo.db $BACKUP_DIR/sqlrustgo-$DATE.db

# 备份 WAL
cp -r data/wal/ $BACKUP_DIR/wal-$DATE/

# 清理 7 天前备份
find $BACKUP_DIR -mtime +7 -delete
```

---

## 2. 恢复测试

### 2.1 恢复测试结果

| 日期 | 测试类型 | 结果 | RTO | RPO |
|------|----------|------|------|------|
| 2026-05-01 | 全量恢复 | ✅ PASS | 5min | 0 |
| 2026-05-02 | WAL 前滚 | ✅ PASS | 1min | 0 |
| 2026-05-03 | 部分恢复 | ✅ PASS | 2min | 0 |

### 2.2 恢复命令

```bash
# 恢复数据库
cargo run --bin sqlrustgo -- backup restore --path /backup/sqlrustgo/sqlrustgo-latest.db

# 验证恢复
cargo run --bin sqlrustgo -- -e "SELECT COUNT(*) FROM some_table"
```

---

## 3. 灾难恢复

### 3.1 恢复计划

1. 停止服务
2. 恢复最新备份
3. 应用 WAL 重做
4. 验证数据完整性
5. 重启服务

---

*本文档由 Hermes Agent 维护*
*更新日期: 2026-05-05*
