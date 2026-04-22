# v2.8.0 测试计划

> 版本: `v2.8.0`
> 日期: 2026-04-23
> 状态: Alpha (规划中)

---

## 一、测试策略

### 1.1 测试分层

```
┌─────────────────────────────────────────────────────────────┐
│                     E2E 测试 (10%)                          │
│            完整查询流程、分布式场景、用户故事                   │
├─────────────────────────────────────────────────────────────┤
│                  集成测试 (30%)                              │
│         模块间协作、分区表、复制、负载均衡                      │
├─────────────────────────────────────────────────────────────┤
│                  单元测试 (60%)                              │
│        解析器、执行器、存储、事务、网络各模块                   │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 测试覆盖率目标

| 模块 | 覆盖率目标 | 关键路径 |
|------|------------|----------|
| parser | ≥ 85% | SELECT/INSERT/UPDATE/DELETE 解析 |
| executor | ≥ 80% | JOIN、聚合、排序 |
| storage | ≥ 85% | B+Tree、BufferPool、MemoryStorage |
| transaction | ≥ 80% | WAL、MVCC |
| network | ≥ 75% | MySQL 协议、连接处理 |
| **整体** | **≥ 80%** | - |

---

## 二、单元测试

### 2.1 解析器测试 (parser)

| 测试组 | 测试用例 | 状态 |
|--------|----------|------|
| **SELECT** | 基本查询、JOIN、聚合、子查询 | ⏳ |
| **INSERT** | 单行、批量、REPLACE INTO | ⏳ |
| **UPDATE** | 单表、多表、别名 | ⏳ |
| **DELETE** | 单表、多表、级联 | ⏳ |
| **DDL** | CREATE/DROP TABLE、INDEX、VIEW | ⏳ |
| **窗口函数** | ROW_NUMBER、RANK、DENSE_RANK | ⏳ |
| **分区表** | RANGE/LIST/HASH/KEY 分区解析 | ⏳ |
| **FULL OUTER JOIN** | 语法解析验证 | ⏳ |
| **TRUNCATE** | TRUNCATE TABLE 语法 | ⏳ |

### 2.2 执行器测试 (executor)

| 测试组 | 测试用例 | 状态 |
|--------|----------|------|
| **JOIN** | INNER、LEFT、RIGHT、FULL OUTER | ⏳ |
| **聚合** | SUM/AVG/COUNT/MIN/MAX | ⏳ |
| **排序** | ORDER BY ASC/DESC | ⏳ |
| **分组** | GROUP BY、HAVING | ⏳ |
| **子查询** | 标量子查询、表子查询 | ⏳ |
| **窗口函数** | 排名窗口函数、聚合窗口函数 | ⏳ |
| **并行化** | Hash Join 多线程 | ⏳ |

### 2.3 存储测试 (storage)

| 测试组 | 测试用例 | 状态 |
|--------|----------|------|
| **B+Tree** | 插入、查找、删除、范围查询 | ⏳ |
| **BufferPool** | 页面置换、刷新、脏页 | ⏳ |
| **MemoryStorage** | 基础 CRUD、事务 | ⏳ |
| **ColumnarStorage** | 列式压缩、解压、聚合 | ⏳ |
| **分区表** | 分区裁剪、多分区查询 | ⏳ |

### 2.4 事务测试 (transaction)

| 测试组 | 测试用例 | 状态 |
|--------|----------|------|
| **WAL** | 日志写入、恢复、重放 | ⏳ |
| **MVCC** | 快照隔离、版本链 | ⏳ |
| **并发控制** | 死锁检测、两阶段锁 | ⏳ |
| **主从复制** | GTID、半同步、数据一致性 | ⏳ |

### 2.5 网络测试 (network)

| 测试组 | 测试用例 | 状态 |
|--------|----------|------|
| **MySQL 协议** | Handshake、认证、命令处理 | ⏳ |
| **连接管理** | 连接池、超时、心跳 | ⏳ |
| **负载均衡** | 轮询、最少连接、健康检查 | ⏳ |
| **故障转移** | 主节点切换、读写分离 | ⏳ |

---

## 三、集成测试

### 3.1 功能集成

| 测试场景 | 输入 | 预期输出 | 状态 |
|----------|------|----------|------|
| **T-11: FULL OUTER JOIN** | `SELECT * FROM a FULL OUTER JOIN b ON a.id = b.id` | 完整结果集 | ⏳ |
| **T-12: TRUNCATE** | `TRUNCATE TABLE t` | 表清空 | ⏳ |
| **T-12: REPLACE** | `REPLACE INTO t VALUES (1, 'a')` | 插入或替换 | ⏳ |
| **T-13: 窗口函数** | `SELECT RANK() OVER (PARTITION BY d ORDER BY s)` | 正确排名 | ⏳ |
| **T-23: 分区表** | `SELECT * FROM t PARTITION (p1) WHERE id < 100` | 分区裁剪 | ⏳ |
| **T-24: 主从复制** | Master 写入 → Slave 读取 | 数据一致 | ⏳ |

### 3.2 分布式集成

| 测试场景 | 测试方法 | 验证指标 | 状态 |
|----------|----------|----------|------|
| **故障转移** | kill master 节点 | 30s 内恢复 | ⏳ |
| **负载均衡** | 100 并发请求 | 每节点 33-34 次 | ⏳ |
| **读写分离** | 写 master，读 slave | 路由正确 | ⏳ |
| **GTID 复制** | 批量写入后检查 | 数据一致 | ⏳ |

详见 [DISTRIBUTED_TEST_DESIGN.md](./DISTRIBUTED_TEST_DESIGN.md)

### 3.3 SQL 回归测试

```bash
# 测试命令
./scripts/run_sql_corpus.sh

# 测试文件位置
./data/sql-corpus/*.sql

# 测试范围
- TPC-H 查询 (SF=1)
- DML 语句
- DDL 语句
- 边界条件
```

---

## 四、性能测试

### 4.1 基准测试

| 基准 | 工具 | 目标 | 状态 |
|------|------|------|------|
| **TPC-H SF=1** | go-tpc | Q1 < 2s | ⏳ |
| **Sysbench OLTP** | sysbench | 1000 QPS | ⏳ |
| **连接基准** | mysqlslap | 500 并发 | ⏳ |

### 4.2 SIMD 性能测试

| 测试 | 目标 | 验证方式 |
|------|------|----------|
| 向量化加速比 | ≥ 2x | 对比 SIMD 开/关 |
| CPU 利用率 | < 80% | top 监控 |
| 内存带宽 | < 50 GB/s | perf stat |

### 4.3 Hash Join 并行化测试

| 测试 | 目标 | 验证方式 |
|------|------|----------|
| 线程数 | ≥ 2 | CPU 核心利用 |
| 加速比 | ≥ 1.5x | 性能对比 |
| 内存使用 | < 8GB | 内存监控 |

---

## 五、安全测试

### 5.1 列级权限测试

| 测试场景 | 输入 | 预期 | 状态 |
|----------|------|------|------|
| SELECT 限制 | 无权限列的 SELECT | 错误或脱敏 | ⏳ |
| INSERT 限制 | 禁止列的 INSERT | 拒绝写入 | ⏳ |
| UPDATE 限制 | 禁止列的 UPDATE | 拒绝修改 | ⏳ |
| 权限继承 | 表权限与列权限 | 正确叠加 | ⏳ |

### 5.2 SQL 注入测试

| 测试用例 | 输入 | 预期 | 状态 |
|----------|------|------|------|
| 字符串注入 | `' OR '1'='1` | 参数化处理 | ⏳ |
| UNION 注入 | `UNION SELECT * FROM users` | 拒绝 | ⏳ |
| 注释注入 | `-- DROP TABLE` | 拒绝 | ⏳ |
| 存储过程 | `EXEC sp_executesql` | 拒绝 | ⏳ |

### 5.3 审计日志测试

| 测试 | 验证点 | 状态 |
|------|--------|------|
| 关键操作记录 | INSERT/UPDATE/DELETE | ⏳ |
| 日志格式 | JSON 可解析 | ⏳ |
| 性能影响 | QPS 下降 < 5% | ⏳ |

---

## 六、测试执行

### 6.1 每日构建测试

```bash
# CI 自动执行
cargo test --all-features
cargo clippy --all-features -- -D warnings
```

### 6.2 发布前测试

```bash
# Alpha/Beta/RC/GA 前执行
./scripts/run_sql_corpus.sh
./scripts/gate/check_docs_links.sh
./scripts/gate/check_coverage.sh
./scripts/gate/check_security.sh
```

### 6.3 测试报告

| 里程碑 | 覆盖率要求 | 关键指标 |
|--------|------------|----------|
| Alpha | ≥ 70% | 核心路径通过 |
| Beta | ≥ 75% | 分布式基础通过 |
| RC | ≥ 80% | 所有测试通过 |
| GA | ≥ 80% | 门禁全部通过 |

---

## 七、测试工具清单

| 工具 | 用途 | 安装 |
|------|------|------|
| cargo test | 单元测试 | 内置 |
| cargo tarpaulin | 覆盖率 | `cargo install cargo-tarpaulin` |
| sqlpp | SQL 语法验证 | 内置 |
| mysql | 客户端 | apt install mysql-client |
| wrk | HTTP 压测 | apt install wrk |
| sysbench | OLTP 基准 | apt install sysbench |
| go-tpc | TPC-H 基准 | go install |

---

## 八、测试用例映射

### T-11 FULL OUTER JOIN

| 用例 | SQL | 预期 |
|------|-----|------|
| 基本 FULL JOIN | `SELECT * FROM a FULL OUTER JOIN b ON a.id = b.id` | 左右表数据完整 |
| 左表独有 | `SELECT * FROM a FULL OUTER JOIN b ON a.id = b.id WHERE b.id IS NULL` | 左表独有数据 |
| 右表独有 | `SELECT * FROM a FULL OUTER JOIN b ON a.id = b.id WHERE a.id IS NULL` | 右表独有数据 |

### T-12 TRUNCATE/REPLACE

| 用例 | SQL | 预期 |
|------|-----|------|
| TRUNCATE 清空 | `TRUNCATE TABLE t; SELECT COUNT(*) FROM t;` | 0 |
| REPLACE 插入 | `REPLACE INTO t VALUES (1, 'a'); SELECT * FROM t;` | 1 row |
| REPLACE 替换 | `REPLACE INTO t VALUES (1, 'b'); SELECT * FROM t WHERE id=1;` | v='b' |

### T-23 分区表

| 用例 | SQL | 预期 |
|------|-----|------|
| RANGE 分区 | `CREATE TABLE t (id INT) PARTITION BY RANGE(id) (PARTITION p0 VALUES LESS THAN (100))` | 分区创建 |
| 分区裁剪 | `EXPLAIN SELECT * FROM t WHERE id < 50` | 仅扫描 p0 |

### T-24 主从复制

| 用例 | 操作 | 预期 |
|------|------|------|
| 同步验证 | Master: INSERT; Slave: SELECT | 数据一致 |
| GTID 验证 | `SHOW MASTER STATUS; SHOW SLAVE STATUS;` | GTID 递增 |

---

*本文档由 SQLRustGo Team 维护*
*最后更新: 2026-04-23*