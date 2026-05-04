# SQLRustGo v2.9.0 部署指南

> **版本**: v2.9.0
> **代号**: Enterprise Resilience
> **最后更新**: 2026-05-05

---

## 概述

本文档提供 SQLRustGo v2.9.0 的部署指南，包括环境要求、配置选项和部署步骤。v2.9.0 是"企业级韧性"版本，聚焦分布式架构完成和生产就绪特性。

---

## 一、环境要求

### 1.1 系统要求

| 要求 | 最低版本 | 推荐版本 |
|------|----------|----------|
| Rust | 1.85+ | 1.94.1 |
| Cargo | 最新版 | 1.85.0 |
| 操作系统 | macOS / Linux | Ubuntu 22.04 / macOS 14 |
| 内存 | 8GB | 16GB+ |
| 磁盘 | 10GB | 50GB+ NVMe SSD |

### 1.2 运行时依赖

| 依赖 | 版本 | 说明 |
|------|------|------|
| OpenSSL | 1.1+ | TLS 支持 |
| CMake | 3.10+ | 编译依赖 |
| LLVM | 15+ | 覆盖率工具 |

---

## 二、从源码编译

### 2.1 克隆代码

```bash
git clone git@192.168.0.252:openclaw/sqlrustgo.git
cd sqlrustgo
git checkout v2.9.0   # 或 tag
```

### 2.2 编译

```bash
# Debug 构建
cargo build --all

# Release 构建（生产环境）
cargo build --all --release

# 仅构建核心二进制
cargo build --release --bin sqlrustgo
```

### 2.3 验证构建

```bash
cargo run --release --bin sqlrustgo -- --version
```

---

## 三、配置

### 3.1 配置文件

默认配置路径: `config/default.toml`

```toml
[database]
path = "data/sqlrustgo.db"
max_connections = 256

[performance]
max_memory_mb = 8192
buffer_pool_size_mb = 2048

[wal]
enabled = true
checkpoint_interval = 500

[security]
audit_log_enabled = true
audit_log_path = "logs/audit/"
```

### 3.2 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| SQLRUSTGO_DB_PATH | data/sqlrustgo.db | 数据库路径 |
| SQLRUSTGO_LOG_LEVEL | info | 日志级别 |
| SQLRUSTGO_MAX_CONNECTIONS | 256 | 最大连接数 |

---

## 四、部署模式

### 4.1 单机部署

```bash
# 创建数据目录
mkdir -p data logs

# 运行（前台）
cargo run --release --bin sqlrustgo

# 运行（后台）
nohup cargo run --release --bin sqlrustgo > logs/sqlrustgo.log 2>&1 &

# 验证运行
curl http://localhost:5432/status
```

### 4.2 Nomad 部署

```bash
# 提交 Nomad job
nomad run runner.nomad

# 查看 job 状态
nomad job status sqlrustgo-runner
```

Job 定义文件: `nomad/runner.nomad`

### 4.3 Docker 部署

```bash
# 构建镜像
docker build -t sqlrustgo:v2.9.0 .

# 运行容器
docker run -d \
  --name sqlrustgo \
  -p 5432:5432 \
  -v /data/sqlrustgo:/data \
  sqlrustgo:v2.9.0
```

---

## 五、数据目录结构

```
data/
├── sqlrustgo.db          # 主数据库文件
├── wal/
│   └── 0000000000.wal   # WAL 日志
└── backup/               # 备份目录
```

---

## 六、运维命令

### 6.1 健康检查

```bash
curl http://localhost:5432/health
```

### 6.2 备份

```bash
# 在线备份
cargo run --bin sqlrustgo -- backup create --path /backup/

# 检查备份
cargo run --bin sqlrustgo -- backup list
```

### 6.3 恢复

```bash
cargo run --bin sqlrustgo -- backup restore --path /backup/v2.9.0-latest.db
```

---

## 七、监控

### 7.1 日志

日志路径: `logs/sqlrustgo.log`

日志级别通过 `RUST_LOG` 环境变量设置:
```bash
RUST_LOG=debug cargo run --bin sqlrustgo
```

### 7.2 性能指标

```bash
# 查看运行时统计
curl http://localhost:5432/metrics
```

输出指标包括:
- 活跃连接数
- 查询 QPS
- 缓存命中率
- 事务数

---

## 八、升级

详见 [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md)

---

## 九、故障排查

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| 启动失败 | 端口被占用 | 检查 5432 端口占用 |
| 连接超时 | 防火墙阻止 | 开放 5432 端口 |
| 性能下降 | 缓存未命中 | 增加 buffer_pool_size |
| 数据损坏 | 磁盘空间不足 | 清理磁盘并恢复备份 |

---

## 十、相关文档

- [QUICK_START.md](./QUICK_START.md)
- [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md)
- [PERFORMANCE_TARGETS.md](./PERFORMANCE_TARGETS.md)

---

*本文档由 Hermes Agent 维护*
*更新日期: 2026-05-05*
