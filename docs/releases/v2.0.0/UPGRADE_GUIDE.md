# SQLRustGo v2.0.0 升级指南

**从**: v1.x → **到**: v2.0.0

---

## 升级前检查

### 1. 数据备份

```bash
# 备份数据目录
cp -r /path/to/data /path/to/data.backup

# 备份 WAL 日志
cp -r /path/to/wal /path/to/wal.backup

# 备份配置文件
cp /path/to/config.toml /path/to/config.toml.backup
```

### 2. 版本检查

```bash
# 检查当前版本
cargo run --bin sqlrustgo -- --version

# 检查依赖兼容性
cargo update --dry-run
```

---

## 升级步骤

### 步骤 1: 更新依赖

```bash
# 更新 Cargo.toml 中的版本
sed -i '' 's/sqlrustgo = "1\.x"/sqlrustgo = "2.0"/g' Cargo.toml

# 更新所有依赖
cargo update
```

### 步骤 2: 重新编译

```bash
# 清理旧构建
cargo clean

# 完整编译
cargo build --release --all-features
```

### 步骤 3: 迁移配置

#### 新增配置项

```toml
# config.toml (v2.0)

[storage]
type = "row"              # 或 "columnar" 启用列式存储
buffer_pool_size = 8192   # 页缓存大小 (默认 8192)

[transaction]
distributed = false        # 启用 2PC 分布式事务
coordinator_address = "" # 分布式协调器地址

[parquet]
enabled = false            # 启用 Parquet 支持
chunk_size = 8388608      # 8MB 分块大小

[executor]
parallel_enabled = true    # 启用并行执行
max_threads = 8           # 最大并行线程数
```

#### 旧配置迁移映射

| v1.x 配置 | v2.0 配置 | 说明 |
|-----------|-----------|------|
| `[server]` | `[server]` | 保持不变 |
| `[storage]` | `[storage]` | 新增 `type` 字段 |
| `[log]` | `[log]` | 保持不变 |

### 步骤 4: 启动迁移

```bash
# 首次启动会自动执行 Catalog 迁移
cargo run --release --bin sqlrustgo

# 检查日志确认迁移成功
grep "migration completed" /path/to/logs/sqlrustgo.log
```

---

## 新功能配置

### 启用列式存储

```toml
[storage]
type = "columnar"

[parquet]
enabled = true
```

### 启用并行查询

```toml
[executor]
parallel_enabled = true
max_threads = 16
```

### 启用分布式事务

```toml
[transaction]
distributed = true
coordinator_address = "grpc://localhost:50051"
```

---

## 破坏性变更

### 1. API 变更

| 模块 | 旧 API | 新 API |
|------|--------|--------|
| Storage | `StorageEngine` | `StorageEngine` + `ColumnarStorage` |
| Transaction | `TransactionManager` | `TransactionManager` + `DistributedTransactionManager` |
| Executor | `Executor` | `Executor` + `ParallelExecutor` |

### 2. 数据格式变更

- **WAL 格式**: v2.0 使用新版 WAL 格式，与 v1.x 不兼容
- **Catalog**: 自动迁移，无需手动操作
- **Parquet**: 新增格式，v1.x 无法读取

### 3. 配置格式变更

- 新增 `[parquet]` 配置节
- 新增 `[executor]` 配置节
- `buffer_pool` 重命名为 `buffer_pool_size`

---

## 回滚指南

### 如果升级失败

```bash
# 1. 停止 v2.0 服务
pkill sqlrustgo

# 2. 恢复数据目录
rm -rf /path/to/data
cp -r /path/to/data.backup /path/to/data

# 3. 恢复 WAL
rm -rf /path/to/wal
cp -r /path/to/wal.backup /path/to/wal

# 4. 恢复配置文件
cp /path/to/config.toml.backup /path/to/config.toml

# 5. 恢复 v1.x 二进制
# (需要预先保存的 v1.x 二进制)
```

---

## 兼容性矩阵

| 功能 | v1.x | v2.0 | 说明 |
|------|------|------|------|
| 客户端协议 | v1.x | v2.0 | 兼容 MySQL 协议 |
| SQL 方言 | SQL-92 | SQL-92+ | 新增 COPY, 窗口函数 |
| 数据文件 | v1 格式 | v2 格式 | 不兼容 |
| WAL 日志 | v1 格式 | v2 格式 | 不兼容 |

---

## 常见问题

### Q: 编译失败，提示缺少依赖

```bash
# 确保使用最新 Rust 版本
rustup update

# 重新下载依赖
cargo update
```

### Q: 启动失败，Catalog 迁移错误

```bash
# 检查备份是否存在
ls -la /path/to/data.backup

# 手动执行迁移 (仅在专业人员指导下)
cargo run --bin sqlrustgo -- migrate --force
```

### Q: 性能下降

- 检查是否启用了列式存储 (`type = "columnar"`)
- 调整 `buffer_pool_size` 大小
- 启用并行执行 (`parallel_enabled = true`)

---

## 技术支持

- **Issues**: https://github.com/minzuuniversity/sqlrustgo/issues
- **Discussion**: https://github.com/minzuuniversity/sqlrustgo/discussions

---

*最后更新: 2026-03-29*
