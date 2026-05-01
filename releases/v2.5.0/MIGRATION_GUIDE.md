# SQLRustGo v2.5.0 从 v2.1.0/v2.4.0 升级指南

**版本**: v2.5.0 (里程碑版本)
**发布日期**: 2026-04-16
**适用范围**: 从 v2.1.0、v2.4.0 升级

---

## 一、升级概述

v2.5.0 是 SQLRustGo 的里程碑版本，包含以下重大变更:

- **MVCC 事务**: 快照隔离，WAL 崩溃恢复
- **向量化执行**: SIMD 加速，并行查询
- **图引擎**: Cypher 查询，BFS/DFS 遍历
- **向量索引**: HNSW/IVF/IVFPQ + SIMD
- **统一查询**: SQL + 向量 + 图 融合

### 1.1 兼容性矩阵

| 组件 | v2.1.0 → v2.5.0 | v2.4.0 → v2.5.0 |
|------|------------------|------------------|
| SQL 语法 | ✅ 兼容 | ✅ 兼容 |
| 配置文件 | ⚠️ 需添加新配置 | ⚠️ 需添加新配置 |
| 存储格式 | ✅ 兼容 | ✅ 兼容 |
| API 接口 | ✅ 兼容 | ✅ 兼容 |
| WAL 格式 | ✅ 兼容 | ✅ 兼容 |

### 1.2 破坏性变更

| 变更项 | 影响 | 迁移操作 |
|--------|------|----------|
| 表达式系统重构 | 需要更新引用 | 见 3.1 节 |
| 向量索引格式 | 新建索引不兼容旧格式 | 需要重建索引 |
| 配置格式变更 | 需要添加新配置项 | 见 3.2 节 |

---

## 二、升级前准备

### 2.1 备份数据

```bash
# 1. 备份数据库
sqlrustgo-tools mysqldump --database mydb --out /backup/mydb.sql

# 2. 备份 WAL 文件 (如使用 WAL 模式)
cp -r /var/lib/sqlrustgo/wal /backup/wal.bak

# 3. 备份配置文件
cp /etc/sqlrustgo.toml /backup/sqlrustgo.toml.bak

# 4. 备份向量索引 (如使用)
cp -r /var/lib/sqlrustgo/vector_index /backup/vector_index.bak
```

### 2.2 检查系统要求

```bash
# 检查 Rust 版本 (需要 2021 edition 或更高)
rustc --version
# 输出应为: rustc 1.XX.0 或更高

# 检查操作系统
uname -a
```

### 2.3 停止服务

```bash
# 停止 SQLRustGo 服务
systemctl stop sqlrustgo

# 或直接停止进程
pkill -f sqlrustgo-server

# 确认进程已停止
ps aux | grep sqlrustgo
```

---

## 三、升级步骤

### 3.1 更新代码

```bash
# 1. 拉取最新代码
git fetch origin

# 2. 切换到 v2.5.0 分支
git checkout develop/v2.5.0

# 3. 更新子模块 (如有)
git submodule update --init --recursive

# 4. 编译新版本
cargo build --release

# 5. 验证版本
./target/release/sqlrustgo --version
# 输出: sqlrustgo 2.5.0
```

### 3.2 更新配置文件

v2.5.0 新增以下配置项:

```toml
# /etc/sqlrustgo.toml

# === v2.5.0 新增配置 ===

# MVCC 配置
[mvcc]
enabled = true
snapshot_isolation = true
gc_interval_seconds = 300

# WAL 配置
[wal]
enabled = true
wal_dir = "/var/lib/sqlrustgo/wal"
pitr_enabled = true
archive_enabled = true
archive_retention_days = 7

# 向量索引配置
[vector_index]
default_type = "hnsw"  # hnsw, ivf, ivfpq, flat
hnsw_m = 16
hnsw_ef_construction = 200
hnsw_ef_search = 100

# 图引擎配置
[graph]
enabled = true
graph_dir = "/var/lib/sqlrustgo/graph"
cypher_enabled = true

# 统一查询配置
[unified_query]
enabled = true
fusion_mode = "rrf"  # rrf, weighted
score_normalization = true

# 性能配置
[performance]
simd_enabled = true
parallel_execution = true
max_thread_count = 0  # 0 表示自动检测
```

### 3.3 数据迁移

#### 3.3.1 向量索引重建

如果从旧版本升级，需要重建向量索引:

```bash
# 1. 导出向量数据
sqlrustgo-tools export-vectors --database mydb --out /backup/vectors.json

# 2. 删除旧索引
rm -rf /var/lib/sqlrustgo/vector_index

# 3. 创建新索引
sqlrustgo-server --init-vector-index --type hnsw

# 4. 导入向量数据
sqlrustgo-tools import-vectors --database mydb --in /backup/vectors.json
```

#### 3.3.2 WAL 格式升级

v2.5.0 使用新版 WAL 格式，自动兼容旧格式:

```bash
# 1. 检查 WAL 状态
sqlrustgo-tools wal-status --dir /var/lib/sqlrustgo/wal

# 2. 如需转换，运行
sqlrustgo-tools wal-upgrade --dir /var/lib/sqlrustgo/wal
```

### 3.4 启动服务

```bash
# 1. 启动服务
sqlrustgo-server --config /etc/sqlrustgo.toml

# 2. 检查服务状态
curl http://localhost:8080/health

# 3. 检查版本
sqlrustgo --version
```

---

## 四、验证升级

### 4.1 功能验证

```bash
# 1. 运行回归测试
cargo test --test regression_test

# 2. 运行 MVCC 测试
cargo test --test mvcc_snapshot_isolation_test

# 3. 运行 TPC-H 测试
cargo test --test tpch_sf1_benchmark

# 4. 运行 OLTP 工作负载测试
cargo test --test oltp_workload_test

# 5. 验证 Cypher (图引擎)
cargo test --test graph_tests

# 6. 验证向量搜索
cargo test --test vector_search_test
```

### 4.2 性能验证

```bash
# 1. 运行基准测试
cargo bench --bench tpch_benchmark

# 2. 运行向量搜索基准
cargo bench --bench vector_benchmark

# 3. 运行图查询基准
cargo bench --bench graph_benchmark
```

### 4.3 验证检查清单

| 检查项 | 命令 | 预期结果 |
|--------|------|----------|
| 版本 | `sqlrustgo --version` | 2.5.0 |
| 健康检查 | `curl localhost:8080/health` | {"status":"ok"} |
| MVCC | `cargo test mvcc_*` | 全部通过 |
| TPC-H | `cargo test tpch_*` | 全部通过 |
| 向量搜索 | `cargo test vector_*` | 全部通过 |
| 图查询 | `cargo test graph_*` | 全部通过 |

---

## 五、回滚方案

### 5.1 回滚步骤

如果升级后出现问题，可按以下步骤回滚:

```bash
# 1. 停止服务
systemctl stop sqlrustgo

# 2. 恢复代码
git checkout v2.4.0  # 或 v2.1.0

# 3. 重新编译
cargo build --release

# 4. 恢复数据 (如有必要)
sqlrustgo-tools mysqldump --database mydb --in /backup/mydb.sql

# 5. 恢复配置
cp /backup/sqlrustgo.toml.bak /etc/sqlrustgo.toml

# 6. 恢复 WAL (如有必要)
cp -r /backup/wal.bak /var/lib/sqlrustgo/wal

# 7. 启动服务
systemctl start sqlrustgo
```

### 5.2 数据兼容性说明

| 数据类型 | 兼容性 | 说明 |
|----------|--------|------|
| 用户数据 | ✅ | 完全兼容 |
| 索引数据 | ⚠️ | 向量索引需重建 |
| WAL 日志 | ✅ | 自动向前兼容 |
| 配置 | ⚠️ | 需手动添加新配置项 |

### 5.3 回滚检查清单

| 检查项 | 命令 | 预期结果 |
|--------|------|----------|
| 版本 | `sqlrustgo --version` | 旧版本号 |
| 数据完整性 | `SELECT COUNT(*) FROM ...` | 与备份一致 |
| 索引可用 | `SHOW INDEX` | 正常 |

---

## 六、主要变更详解

### 6.1 MVCC 事务

v2.5.0 支持 MVCC 快照隔离:

```sql
-- 设置事务隔离级别
SET TRANSACTION ISOLATION LEVEL SNAPSHOT;

-- 开始事务
BEGIN;

-- 读取数据 (读取的是事务开始时的快照)
SELECT * FROM users WHERE id = 1;

-- 提交事务
COMMIT;
```

### 6.2 Cypher 图查询

v2.5.0 支持 Cypher 查询:

```sql
-- 图查询示例
CYPHER {
    MATCH (a:Person)-[:KNOWS]->(b:Person)
    WHERE a.name = 'Alice'
    RETURN b.name
}
```

### 6.3 统一查询

v2.5.0 支持 SQL + 向量 + 图融合查询:

```sql
-- 统一查询示例
SELECT name, score
FROM (
    SELECT name, 0.5 AS score FROM users WHERE age > 25
    UNION ALL
    SELECT name, similarity AS score FROM vector_search('embedding', '[0.1, 0.2]')
)
ORDER BY score DESC
LIMIT 10;
```

---

## 七、常见问题

### Q1: 升级后编译失败？

**A**: 确保 Rust 工具链是最新版本:

```bash
rustup update
cargo clean
cargo build --release
```

### Q2: 向量索引搜索变慢？

**A**: 可能是索引类型变更导致，请重建索引:

```bash
sqlrustgo-tools rebuild-vector-index --type hnsw --m 16 --ef 100
```

### Q3: MVCC 事务回滚失败？

**A**: 检查 WAL 文件是否完整:

```bash
sqlrustgo-tools wal-check --dir /var/lib/sqlrustgo/wal
```

### Q4: 测试失败怎么办？

**A**: 查看详细输出:

```bash
cargo test -- --nocapture
```

### Q5: 性能下降？

**A**: 运行基准测试对比:

```bash
cargo bench --bench tpch_benchmark -- --test
```

---

## 八、联系与支持

- **GitHub Issues**: https://github.com/minzuuniversity/sqlrustgo/issues
- **文档**: https://github.com/minzuuniversity/sqlrustgo/docs/releases/v2.5.0/
- **讨论组**: https://github.com/minzuuniversity/sqlrustgo/discussions

---

## 九、附录

### A. 完整配置示例

```toml
# /etc/sqlrustgo.toml
[server]
host = "0.0.0.0"
port = 5432
max_connections = 100

[storage]
data_dir = "/var/lib/sqlrustgo/data"
wal_dir = "/var/lib/sqlrustgo/wal"

[mvcc]
enabled = true
snapshot_isolation = true
gc_interval_seconds = 300

[wal]
enabled = true
pitr_enabled = true
archive_enabled = true
archive_retention_days = 7

[vector_index]
default_type = "hnsw"
hnsw_m = 16
hnsw_ef_construction = 200
hnsw_ef_search = 100

[graph]
enabled = true
cypher_enabled = true

[unified_query]
enabled = true
fusion_mode = "rrf"

[performance]
simd_enabled = true
parallel_execution = true
max_thread_count = 0

[logging]
level = "info"
path = "/var/log/sqlrustgo/"
```

### B. 升级检查脚本

```bash
#!/bin/bash
# upgrade_check.sh

echo "=== SQLRustGo v2.5.0 升级检查 ==="

echo "1. 检查版本..."
./target/release/sqlrustgo --version

echo "2. 检查健康状态..."
curl -s localhost:8080/health

echo "3. 运行回归测试..."
cargo test --test regression_test

echo "4. 检查 MVCC..."
cargo test mvcc_snapshot_isolation

echo "5. 检查向量索引..."
cargo test vector_search

echo "=== 升级检查完成 ==="
```

---

*升级指南 v2.5.0*
*最后更新: 2026-04-16*
