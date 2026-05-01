# SQLRustGo 基准测试

## 目录

- [PERFORMANCE_COMPARISON_REPORT.md](./PERFORMANCE_COMPARISON_REPORT.md) - 完整性能对比报告
- [WORK_LOG.md](./WORK_LOG.md) - 详细工作记录和测试命令
- [results_summary.csv](./results_summary.csv) - 测试数据 CSV

## 测试结果

### Point Select (32 线程)

| 数据库 | TPS | P99 延迟 |
|--------|-----|---------|
| PostgreSQL 16 | 285,128 | 0.13ms |
| MySQL 8.0 | 224,931 | 0.16ms |
| SQLite 3.45 | 13,617 | 2.51ms |
| SQLRustGo | ❌ 无法测试 | — |

### OLTP Read Only (16 线程)

| 数据库 | TPS | P99 延迟 |
|--------|-----|---------|
| PostgreSQL 16 | 5,814 | 3.49ms |
| MySQL 8.0 | 4,873 | 3.49ms |
| SQLite 3.45 | 2,306 | 13.70ms |
| SQLRustGo | ❌ 无法测试 | — |

### OLTP Write Only (16 线程)

| 数据库 | TPS | P99 延迟 |
|--------|-----|---------|
| MySQL 8.0 | 1,531 | 25.12ms |
| SQLite 3.45 | 338 | 738.74ms |
| PostgreSQL 16 | 215 | 1,013.60ms |
| SQLRustGo | ❌ 无法测试 | — |

### OLTP Read Write Mixed (16 线程)

| 数据库 | TPS | P99 延迟 |
|--------|-----|---------|
| MySQL 8.0 | 1,131 | 21.10ms |
| PostgreSQL 16 | 1,093 | 17.63ms |
| SQLite 3.45 | 797 | 328.85ms |
| SQLRustGo | ❌ 无法测试 | — |

### High Concurrency (64 线程, 120 秒)

| 数据库 | TPS | P99 延迟 |
|--------|-----|---------|
| MySQL 8.0 | 2,587 | 36.53ms |
| SQLite 3.45 | 859 | 1,239.94ms |
| PostgreSQL 16 | 36 ❌ | 12,163ms |
| SQLRustGo | ❌ 无法测试 | — |

## 关键发现

1. **PostgreSQL 高并发崩溃**: 64 线程下 TPS 暴跌 97% (1,093→36)
2. **MySQL Group Commit 优势**: 写入 TPS 是 PostgreSQL 的 7 倍
3. **SQLRustGo 无法测试**: Server 启动后立即退出，无法完成任何测试

## Issue

- [#1679](https://github.com/minzuuniversity/sqlrustgo/issues/1679) - SQLRustGo Sysbench 基准测试整改
