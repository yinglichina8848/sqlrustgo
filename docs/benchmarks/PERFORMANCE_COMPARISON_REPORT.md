# SQLRustGo vs MySQL vs PostgreSQL vs SQLite — OLTP 基准测试对比报告

**测试日期**: 2026-04-20  
**测试环境**: Ubuntu 24.04 (Noble)  
**硬件**: Intel Xeon Gold 6138 @ 2.00GHz (80 核), 409 GB DDR4, NVMe SSD 1.9TB  
**测试工具**: sysbench 1.0.20 (MySQL/PostgreSQL), Python 3 (SQLite), sqlrustgo-bench (SQLRustGo)

---

## 1. 测试结果总表

### 1.1 Point Select (32 线程, 主键点查)

| 数据库 | TPS | QPS | P95 延迟 | P99 延迟 | 最大延迟 |
|--------|-----|-----|---------|---------|---------|
| **PostgreSQL 16** | **285,128** | 285,128 | 0.13ms | 0.13ms | 2.94ms |
| MySQL 8.0 | 224,931 | 224,931 | 0.16ms | - | 4.95ms |
| SQLite 3.45 | 13,617 | - | 2.02ms | 2.51ms | - |
| SQLRustGo | (无法测试) | - | - | - | - |

→ **PostgreSQL 最快** (比 MySQL 快 27%)  
→ SQLite 最慢 (比 PG 慢 21x)

### 1.2 OLTP Read Only (16 线程, 混合读)

| 数据库 | TPS | QPS | P95 延迟 | P99 延迟 | 最大延迟 |
|--------|-----|-----|---------|---------|---------|
| **PostgreSQL 16** | **5,814** | 93,027 | 3.49ms | - | 6.67ms |
| MySQL 8.0 | 4,873 | 77,967 | 3.49ms | - | 5.35ms |
| SQLite 3.45 | 2,306 | - | 11.27ms | 13.70ms | - |
| SQLRustGo | (无法测试) | - | - | - | - |

→ **PostgreSQL 最快** (比 MySQL 快 19%)  
→ SQLite 范围扫描较慢 (比 PG 慢 2.5x)

### 1.3 OLTP Write Only (16 线程, 纯写入)

| 数据库 | TPS | QPS | P95 延迟 | P99 延迟 | 最大延迟 |
|--------|-----|-----|---------|---------|---------|
| **MySQL 8.0** | **1,531** | 9,188 | 18.61ms | - | 60.47ms |
| PostgreSQL 16 | 215 | 1,344 | 1,013.60ms | - | 2,027.32ms |
| SQLite 3.45 | 338 | - | 187.86ms | 738.74ms | - |
| SQLRustGo | (无法测试) | - | - | - | - |

→ **MySQL 最快** (比 PostgreSQL 快 7x) — WAL group commit 优势  
→ PostgreSQL 最慢 (WAL 同步写入瓶颈)  
→ SQLite 中等 (受单文件锁限制)

### 1.4 OLTP Read Write Mixed (16 线程, 混合读写)

| 数据库 | TPS | QPS | P95 延迟 | P99 延迟 | 最大延迟 |
|--------|-----|-----|---------|---------|---------|
| **MySQL 8.0** | **1,131** | 22,614 | 21.89ms | - | 103.06ms |
| PostgreSQL 16 | 1,093 | 22,690 | 17.63ms | - | 1,107.30ms |
| SQLite 3.45 | 797 | - | 86.07ms | 328.85ms | - |
| SQLRustGo | (无法测试) | - | - | - | - |

→ **MySQL 最快** (比 PostgreSQL 快 3%)  
→ 三者接近，差距主要在写入延迟

### 1.5 High Concurrency (64 线程, 120 秒)

| 数据库 | TPS | QPS | P95 延迟 | P99 延迟 | 最大延迟 |
|--------|-----|-----|---------|---------|---------|
| **MySQL 8.0** | **2,587** | 51,765 | 52.89ms | - | 1,085ms |
| SQLite 3.45 | 859 | - | 428.89ms | 1,239.94ms | - |
| PostgreSQL 16 | 36 | 849 | 12,163.09ms | - | 26,287ms |
| SQLRustGo | (无法测试) | - | - | - | - |

→ **MySQL 碾压级优势** (比 PG 快 71x)  
→ PostgreSQL **灾难性退化** (16→64 线程 TPS 暴跌 97%)  
→ MySQL **反而更好** (16→64 线程 TPS 提升 129%)

---

## 2. 性能排名

### TPS 排名 (越高越好)

| 测试场景 | 第1名 | 第2名 | 第3名 | 第4名 |
|---------|-------|-------|-------|-------|
| Point Select | PostgreSQL **285K** | MySQL 225K | SQLite 14K | SQLRustGo N/A |
| Read Only | PostgreSQL **5.8K** | MySQL 4.9K | SQLite 2.3K | SQLRustGo N/A |
| Write Only | MySQL **1.5K** | SQLite 338 | PostgreSQL 215 | SQLRustGo N/A |
| Read Write | MySQL **1.1K** | PostgreSQL 1.1K | SQLite 797 | SQLRustGo N/A |
| High Concurrency | MySQL **2.6K** | SQLite 859 | PostgreSQL 36 | SQLRustGo N/A |

### 延迟排名 (越低越好)

| 测试场景 | 第1名 | 第2名 | 第3名 | 第4名 |
|---------|-------|-------|-------|-------|
| Point Select | PostgreSQL **0.13ms** | MySQL 0.16ms | SQLite 2.02ms | SQLRustGo N/A |
| Read Only | MySQL **3.49ms** | PostgreSQL 3.49ms | SQLite 11.27ms | SQLRustGo N/A |
| Write Only | MySQL **18.61ms** | SQLite 187.86ms | PostgreSQL 1,013ms | SQLRustGo N/A |
| Read Write | PostgreSQL **17.63ms** | MySQL 21.89ms | SQLite 86.07ms | SQLRustGo N/A |

---

## 3. 关键分析

### 3.1 写入性能: MySQL >> SQLite >> PostgreSQL

```
MySQL:      1,531 TPS   (group commit, 异步刷盘)
SQLite:       338 TPS   (文件锁, 单 writer)
PostgreSQL:   215 TPS   (WAL 同步, MVCC 开销)
```

**根因**:
- MySQL: Group commit 批量提交，WAL 写入可并行
- PostgreSQL: 每次 COMMIT 必须 fsync() 同步刷盘，串行化
- SQLite: 数据库级文件锁，写入完全串行化

### 3.2 高并发扩展性: MySQL >>> SQLite > PostgreSQL

```
MySQL:    1,131 TPS (16t) → 2,587 TPS (64t) = +129% ✅
SQLite:     797 TPS (16t) →   859 TPS (64t) = +8%  ⚠️
PostgreSQL:1,093 TPS (16t) →    36 TPS (64t) = -97% ❌
```

**根因**:
- MySQL: 连接池 + 行级锁 + MVCC，读写不互阻塞
- PostgreSQL: WAL 串行写入 + lightweight lock 竞争，64 线程崩溃
- SQLite: 写锁串行化，但读不阻塞写所以略好于 PG

### 3.3 读取性能: PostgreSQL ≈ MySQL >> SQLite

```
PostgreSQL: 5,814 TPS (Read Only), 285K TPS (Point Select)
MySQL:      4,873 TPS (Read Only), 225K TPS (Point Select)
SQLite:     2,306 TPS (Read Only),  14K TPS (Point Select)
```

**根因**:
- PostgreSQL: MVCC 读不阻塞写，索引缓存高效
- MySQL: InnoDB MVCC，类似性能
- SQLite: 无服务器开销，但缺乏查询优化器，范围扫描全表

---

## 4. SQLRustGo 状态

### 4.1 无法完成测试的原因

1. **Server 无法启动**: `sqlrustgo` 二进制运行后立即退出，无错误信息
2. **sqlrustgo-server 编译失败**: 6 个编译错误 (类型推断失败)
3. **sqlrustgo-bench 连接失败**: server 未运行，返回 0 TPS

### 4.2 预期性能 (基于代码分析)

| 指标 | MySQL | PostgreSQL | SQLite | SQLRustGo 预期 |
|------|-------|-----------|--------|----------------|
| Point Select | 225K | 285K | 14K | ~10K-50K |
| Read Only | 4.9K | 5.8K | 2.3K | ~1K-3K |
| Write TPS | 1.5K | 215 | 338 | ~200-500 |
| High Concurrency | 2.6K | 36 | 859 | ~500-1K |

**预期 SQLRustGo 瓶颈**:
- 无 MVCC: 读写相互阻塞
- 无 CBO: 全表扫描为主
- 无连接池: 每次查询重新打开文件
- 无 group commit: WAL 逐个提交

---

## 5. 测试配置详情

### 5.1 测试参数
```
sysbench: 4 tables × 10,000 rows (MySQL/PostgreSQL)
SQLite:   4 tables × 10,000 rows (Python 多线程)
时间:     20 秒 (高并发 120 秒)
线程:     16 (标准) / 64 (高并发)
```

### 5.2 数据库配置

**MySQL 8.0.45**:
```ini
innodb_buffer_pool_size = 128M
innodb_log_file_size = 48M
innodb_flush_log_at_trx_commit = 1
```

**PostgreSQL 16.13**:
```ini
shared_buffers = 4GB
effective_cache_size = 12GB
max_connections = 500
synchronous_commit = on
full_page_writes = on
```

**SQLite 3.45**:
```python
# WAL mode disabled (default)
# journal_mode = DELETE
# synchronous = NORMAL
```

---

## 6. 结论与建议

### 6.1 选型建议

| 场景 | 推荐 | 备选 |
|------|------|------|
| **高并发写入** | MySQL | PostgreSQL (需调优 WAL) |
| **只读为主** | PostgreSQL | MySQL |
| **嵌入式/轻量** | SQLite | SQLRustGo (未来) |
| **OLTP 混合负载** | MySQL | PostgreSQL |
| **超高并发 (64t+)** | MySQL | 绝对避免 PostgreSQL |

### 6.2 PostgreSQL 优化建议 (针对写入和高并发)

```ini
# postgresql.conf
synchronous_commit = off        # 异步提交 (牺牲持久性)
wal_buffers = 64MB              # 增大 WAL 缓冲
commit_delay = 10               # group commit (微秒)
max_connections = 200           # 限制连接数减少锁竞争
```

### 6.3 SQLRustGo 开发优先级

1. **P0**: 修复 server 启动问题 (能跑起来才能测试)
2. **P0**: 实现 MVCC (解决高并发退化)
3. **P0**: 实现 group commit (解决写入瓶颈)
4. **P1**: 添加 CBO 优化器 (提升复杂查询性能)
5. **P1**: 添加连接池 (减少文件打开开销)

---

## 7. 原始数据

### 7.1 MySQL (sysbench)
```
Point Select:  224,931 TPS, P95=0.16ms, max=4.95ms
Read Only:       4,873 TPS, P95=3.49ms, max=5.35ms
Write Only:      1,531 TPS, P95=18.61ms, max=60.47ms
Read Write:      1,131 TPS, P95=21.89ms, max=103.06ms
High (64t):      2,587 TPS, P95=52.89ms, max=1085ms
```

### 7.2 PostgreSQL (sysbench)
```
Point Select:  285,128 TPS, P99=0.13ms, max=2.94ms
Read Only:       5,814 TPS, P99=3.49ms, max=6.67ms
Write Only:       215 TPS, P99=1013.60ms, max=2027.32ms
Read Write:     1,093 TPS, P99=17.63ms, max=1107.30ms
High (64t):       36 TPS, P99=12163.09ms, max=26287ms
```

### 7.3 SQLite (Python benchmark)
```
Point Select:   13,617 TPS, P95=2.02ms, P99=2.51ms
Read Only:       2,306 TPS, P95=11.27ms, P99=13.70ms
Write Only:        338 TPS, P95=187.86ms, P99=738.74ms
Read Write:        797 TPS, P95=86.07ms, P99=328.85ms
High (64t):        859 TPS, P95=428.89ms, P99=1239.94ms
```

---

*测试完成日期: 2026-04-20*
*sysbench 1.0.20 + Python 3 + sqlrustgo-bench 0.1.0*
