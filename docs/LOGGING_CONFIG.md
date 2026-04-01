# 日志配置指南

## 概述

SQLRustGo v2.1.0+ 支持日志轮转、级别过滤、JSON 格式等高级日志功能。

## 环境变量配置

| 变量 | 默认值 | 描述 |
|------|--------|------|
| `SQLRUSTGO_LOG_DIR` | `./logs` | 日志文件目录 |
| `SQLRUSTGO_LOG_LEVEL` | `INFO` | 日志级别 (ERROR/WARN/INFO/DEBUG) |
| `SQLRUSTGO_LOG_JSON` | (off) | 启用 JSON 结构化日志 |
| `SQLRUSTGO_LOG_MAX_SIZE` | `10485760` | 单个日志文件最大字节数 (默认 10MB) |
| `SQLRUSTGO_LOG_MAX_FILES` | `5` | 保留的日志文件数量 |

## 日志级别

| 级别 | 描述 | 包含消息 |
|------|------|----------|
| `ERROR` | 错误 | ERROR only |
| `WARN` | 警告 | ERROR, WARN |
| `INFO` | 信息 | ERROR, WARN, INFO |
| `DEBUG` | 调试 | ERROR, WARN, INFO, DEBUG |

## 使用示例

### 1. 基本配置

```bash
export SQLRUSTGO_LOG_DIR=/var/log/sqlrustgo
export SQLRUSTGO_LOG_LEVEL=INFO
```

### 2. JSON 结构化日志 (用于日志收集系统)

```bash
export SQLRUSTGO_LOG_DIR=/var/log/sqlrustgo
export SQLRUSTGO_LOG_LEVEL=INFO
export SQLRUSTGO_LOG_JSON=1
```

### 3. 生产环境配置

```bash
export SQLRUSTGO_LOG_DIR=/var/log/sqlrustgo
export SQLRUSTGO_LOG_LEVEL=WARN
export SQLRUSTGO_LOG_MAX_SIZE=52428800   # 50MB
export SQLRUSTGO_LOG_MAX_FILES=10
```

### 4. 开发环境配置

```bash
export SQLRUSTGO_LOG_DIR=./logs
export SQLRUSTGO_LOG_LEVEL=DEBUG
export SQLRUSTGO_LOG_JSON=1
```

## 日志格式

### 文本格式 (默认)

```
[2026-04-01 10:30:45.123] [INFO] [main.rs:42] Application started
[2026-04-01 10:30:45.456] [ERROR] [db.rs:100] Connection failed: timeout
```

### JSON 格式

```json
{"timestamp":"2026-04-01T10:30:45.123+08:00","level":"INFO","module":"sqlrustgo_server","file":"main.rs","line":42,"message":"Application started"}
{"timestamp":"2026-04-01T10:30:45.456+08:00","level":"ERROR","module":"sqlrustgo_storage","file":"db.rs","line":100,"message":"Connection failed: timeout"}
```

## 日志轮转

当日志文件达到 `SQLRUSTGO_LOG_MAX_SIZE` 指定的大小后：

1. 当前日志文件被压缩为 `.gz` 文件
2. 创建新的日志文件继续写入
3. 如果日志文件数量超过 `SQLRUSTGO_LOG_MAX_FILES`，最旧的 `.gz` 文件会被删除

### 示例

```bash
# 配置 10MB 文件大小，保留 5 个文件
export SQLRUSTGO_LOG_MAX_SIZE=10485760
export SQLRUSTGO_LOG_MAX_FILES=5
```

日志目录结构：
```
logs/
├── sqlrustgo_20260401_103045.log    # 当前日志
├── sqlrustgo_20260401_093045.log.gz # 压缩的旧日志
├── sqlrustgo_20260401_083045.log.gz
├── sqlrustgo_20260401_073045.log.gz
└── sqlrustgo_20260401_063045.log.gz
```

## 集成日志收集系统

### Fluentd 配置

```bash
export SQLRUSTGO_LOG_DIR=/var/log/sqlrustgo
export SQLRUSTGO_LOG_LEVEL=INFO
export SQLRUSTGO_LOG_JSON=1
```

```conf
<source>
  @type tail
  path /var/log/sqlrustgo/*.log
  pos_file /var/log/fluentd/sqlrustgo.pos
  tag sqlrustgo.logs
  <parse>
    @type json
  </parse>
</source>
```

### Loki/Promtail 配置

```yaml
scrape_configs:
  - job_name: sqlrustgo
    static_configs:
      - targets:
          - localhost
        labels:
          job: sqlrustgo
          __path__: /var/log/sqlrustgo/*.log
```

## 故障排除

### 日志目录不存在

如果 `SQLRUSTGO_LOG_DIR` 指定的目录不存在，系统会自动创建。

### 磁盘空间不足

当日志文件数量达到 `SQLRUSTGO_LOG_MAX_FILES` 限制后，最旧的文件会自动删除。

### 权限问题

确保运行 SQLRustGo 的用户对日志目录有写入权限：

```bash
sudo mkdir -p /var/log/sqlrustgo
sudo chown sqlrustgo:sqlrustgo /var/log/sqlrustgo
```

## 相关文档

- [DEPLOYMENT_GUIDE.md](./v1.9.0/DEPLOYMENT_GUIDE.md) - 部署指南
- [issue #1022](./ISSUE_1022_LOG_ROTATION.md) - 功能设计文档
