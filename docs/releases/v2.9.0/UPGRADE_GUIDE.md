# v2.9.0 升级指南

> **版本**: v2.9.0
> **适用**: 从 v2.8.0 升级
> **日期**: 2026-05-05
> **阶段**: GA (v2.9.0)

---

## 概述

本指南帮助用户从 v2.8.0 升级到 v2.9.0。v2.9.0 是"企业级韧性"版本，聚焦分布式架构完成和生产就绪特性。

**当前状态**: v2.9.0 已正式发布 (GA)。v2.8.0 到 v2.9.0 的大多数 SQL 语句无需修改即可运行。

---

## 重大变更

### 破坏性变更

| 变更 | 影响 | 迁移建议 |
|------|------|----------|
| TPC-H 基准函数签名变化 | 旧版 bench-cli 参数可能不兼容 | 使用 `cargo run --bin bench-cli -- tpch bench` |
| SHOW INDEX FROM table 行为调整 | 输出列顺序变化 | 脚本需适配列位置 |
| WAL 检查点间隔默认值调整 | 日志大小略有增加 | 可通过 `wal.checkpoint_interval` 配置还原 |

### 新增 SQL 语法

| 语法 | 示例 | 说明 |
|------|------|------|
| CTE (WITH) | `WITH cte AS (SELECT * FROM t) SELECT * FROM cte` | 通用表表达式 |
| 窗口函数 | `SELECT row_number() OVER (PARTITION BY a ORDER BY b)` | 分析函数 |
| CASE 表达式 | `SELECT CASE WHEN a > 1 THEN 'big' ELSE 'small' END` | 条件表达式 |
| JSON 操作 | `SELECT json_extract(data, '$.key')` | JSON 路径查询 |

### API 变更

| 旧 API | 新 API | 说明 |
|--------|--------|------|
| `ExecutionEngine::new()` | `ExecutionEngine::new().with_config()` | 推荐使用配置构建器 |
| `Connection::query_iter()` | `Connection::execute()` | 统一接口 |
| `Catalog::get_table_names()` | `Catalog::table_names()` | 返回类型变化：Vec→Iter |

---

## 升级步骤

### 1. 备份数据

```bash
# 备份数据库文件
cp -r data/ data_backup_v2.8.0/

# 备份配置文件
cp config/default.toml config/default.toml.bak
```

### 2. 更新 Cargo 依赖

```bash
cargo update
cargo build --all
```

### 3. 运行迁移脚本

```bash
# 检查兼容性
cargo run --bin sqlrustgo -- --check-migration

# 执行迁移（必要时）
cargo run --bin migrate --from 2.8.0 --to 2.9.0
```

### 4. 验证升级

```bash
cargo test --all-features
cargo run --bin bench-cli -- tpch bench --queries Q1,Q3,Q6 --sf 0.1
```

---

## 配置变更

### 新增配置项

```toml
[performance]
max_connections = 256          # 新增：最大连接数
statement_timeout = 30000       # 新增：查询超时(ms)

[security]
audit_log_enabled = true        # 新增：审计日志
audit_log_path = "logs/audit/"  # 新增：审计日志路径
```

### 已移除配置项

| 旧配置 | 替代方案 |
|--------|----------|
| `[vector].hnsw_enable` | `[vector].index_type = "hnsw"` |

---

## 已知问题

| 问题 | 严重程度 | 解决方式 |
|------|----------|----------|
| sysbench QPS 约 2000，低于 10K 目标 | 低 | v2.10.0 优化 |
| TPC-H SF=1 部分查询超时 | 低 | SF=0.1 正常运行 |
| TPC-H 9/22 查询可运行 | 中 | v2.10.0 目标 18/22 |

---

## 回滚步骤

如遇问题，回滚步骤：

```bash
# 停止服务
pkill sqlrustgo

# 恢复二进制
git checkout v2.8.0
cargo build --release

# 恢复数据目录
rm -rf data/
mv data_backup_v2.8.0 data/

# 重启服务
cargo run --release --bin sqlrustgo
```

---

## 相关文档

- [CHANGELOG.md](./CHANGELOG.md)
- [PERFORMANCE_TARGETS.md](./PERFORMANCE_TARGETS.md)
- [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md)

---

*本文档由 Hermes Agent 维护*
*更新日期: 2026-05-05*
