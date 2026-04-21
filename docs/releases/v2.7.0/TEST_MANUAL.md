# SQLRustGo v2.7.0 测试手册

**版本**: v2.7.0 (Enterprise Resilience GA)
**发布日期**: 2026-04-22

---

## 一、测试环境准备

### 1.1 环境要求

| 组件 | 要求 |
|------|------|
| Rust | 1.70+ |
| 内存 | 16GB+ |
| 磁盘 | 20GB+ SSD |
| 操作系统 | macOS/Linux |

### 1.2 环境搭建

```bash
# 1. 克隆代码
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 2. 切换到 v2.7.0
git checkout v2.7.0

# 3. 编译项目
cargo build --release

# 4. 验证安装
cargo test --version
```

### 1.3 测试数据准备

```bash
# 创建测试数据库
./target/release/sqlrustgo --database test_db

# 初始化 TPC-H 测试数据 (SF1)
cargo run --bin tpch_data -- --sf 1 --output ./test_data/tpch_sf1

# 初始化 GMP Top10 测试数据
cargo run --bin gmp_test_data -- --output ./test_data/gmp_top10

# 初始化向量搜索测试数据
cargo run --bin vector_test_data -- --output ./test_data/vector_search
```

---

## 二、测试类别

### 2.1 L0 冒烟测试（5-10 分钟）

**目的**: 快速判断分支可用性

```bash
# 编译检查
cargo build --all-features
cargo fmt --check
cargo clippy --all-features -- -D warnings

# 核心路径冒烟
cargo test --test sql_corpus_quick
cargo test --test wal_basic_test
```

### 2.2 L1 模块测试（20-40 分钟）

**目的**: 核心 crate 行为正确

```bash
# 运行所有模块单元测试
cargo test --lib --workspace

# 关键模块覆盖率检查
cargo test -p sqlrustgo-parser --lib
cargo test -p sqlrustgo-executor --lib
cargo test -p sqlrustgo-storage --lib
cargo test -p sqlrustgo-transaction --lib
```

### 2.3 L2 集成测试（40-90 分钟）

**目的**: 跨模块、跨引擎、跨协议验证

```bash
# 全链路集成测试
cargo test --test '*'

# 特定集成测试
cargo test --test sql_full_path_test
cargo test --test transaction_wal_test
cargo test --test mysql_protocol_test
```

### 2.4 L3 深度验证（夜间/周跑）

**目的**: 发布级稳定性与性能

```bash
# TPC-H SF1
cargo test --test tpch_sf1_benchmark

# Sysbench OLTP
cargo test --test sysbench_oltp

# 72h 长稳测试
cargo test --test stability_72h

# 崩溃恢复测试
cargo test --test crash_recovery_test

# 备份恢复测试
cargo test --test backup_restore_test
```

---

## 三、SQL 合规性测试

### 3.1 SQL-92 合规性测试

```bash
# 运行 SQL-92 测试
cargo test --test sql92_compliance_test

# 运行特定特性测试
cargo test --test sql92_compliance_test case_expression
cargo test --test sql92_compliance_test subquery
cargo test --test sql92_compliance_test set_operation
```

### 3.2 MySQL 5.7 兼容性测试

```bash
# 运行 MySQL 兼容测试
cargo test --test mysql_compatibility_test

# 运行特定兼容性测试
cargo test --test mysql_compatibility_test dml
cargo test --test mysql_compatibility_test ddl
```

### 3.3 SQL Corpus 测试

```bash
# 运行 SQL 语法回归测试
cargo test --test sql_corpus

# 输出结果统计
cargo test --test sql_corpus -- --nocapture
```

---

## 四、功能测试程序

### 4.1 WAL 崩溃恢复测试 (T-01)

**测试目标**: 验证 Write-Ahead Logging 崩溃恢复机制

**测试步骤**:

1. 创建测试数据库并插入数据
```bash
./target/release/sqlrustgo --database wal_test
```

2. 执行事务操作
```sql
CREATE TABLE orders (id INTEGER PRIMARY KEY, amount DECIMAL);
INSERT INTO orders VALUES (1, 100.00);
INSERT INTO orders VALUES (2, 200.00);
BEGIN;
UPDATE orders SET amount = 150.00 WHERE id = 1;
COMMIT;
```

3. 模拟崩溃 (`kill -9`)
```bash
# 获取进程 PID
ps aux | grep sqlrustgo
kill -9 <PID>
```

4. 重启服务并验证数据完整性
```bash
./target/release/sqlrustgo --database wal_test
SELECT * FROM orders;
-- 验证数据已正确恢复
```

**通过标准**: 崩溃前后数据一致，无数据丢失

### 4.2 外键稳定性测试 (T-02)

**测试目标**: 验证外键约束稳定性增强

**测试步骤**:

```bash
# 运行 FK 稳定性测试
cargo test --test fk_stability_test

# 测试级联删除
cargo test --test fk_cascade_test

# 测试循环依赖
cargo test --test fk_circular_deps_test
```

**通过标准**: FK 约束正确执行，级联操作无死锁

### 4.3 备份恢复测试 (T-03)

**测试目标**: 验证完整备份恢复机制

**测试步骤**:

1. 执行全量备份
```bash
./target/release/sqlrustgo backup --all --output ./backup_$(date +%Y%m%d)
```

2. 模拟数据变更
```sql
INSERT INTO orders VALUES (3, 300.00);
UPDATE orders SET amount = 250.00 WHERE id = 2;
```

3. 执行恢复
```bash
./target/release/sqlrustgo restore --input ./backup_20260422
```

4. 验证数据完整性
```sql
SELECT * FROM orders;
-- 验证恢复到备份点状态
```

**通过标准**: 恢复后数据与备份点一致

### 4.4 QMD Bridge 测试 (T-04)

**测试目标**: 验证 Query Metadata Bridge 功能

**测试步骤**:

```bash
# 运行 QMD Bridge 测试
cargo test --test qmd_bridge_test

# 验证元数据查询
cargo test --test qmd_metadata_test
```

### 4.5 统一搜索 API 测试 (T-05)

**测试目标**: 验证 Unified Search API

**测试步骤**:

```bash
# 向量搜索测试
cargo test --test vector_search_test

# HNSW 索引测试
cargo test --test hnsw_test

# 混合排序测试
cargo test --test hybrid_rerank_test
```

### 4.6 审计证据链测试 (T-08)

**测试目标**: 验证完整审计追踪功能

**测试步骤**:

1. 启用审计日志
```bash
./target/release/sqlrustgo --audit-enabled --audit-output ./audit_logs
```

2. 执行操作
```sql
CREATE TABLE audit_test (id INTEGER, data TEXT);
INSERT INTO audit_test VALUES (1, 'test');
UPDATE audit_test SET data = 'updated' WHERE id = 1;
DELETE FROM audit_test WHERE id = 1;
```

3. 验证审计日志
```bash
cat ./audit_logs/audit_*.log
# 验证操作已记录，包含时间戳、用户、操作类型
```

**通过标准**: 所有操作有完整审计记录

---

## 五、性能测试程序

### 5.1 TPC-H 基准测试

```bash
# TPC-H SF1 全量查询
cargo test --test tpch_sf1_benchmark

# 对比 v2.6.0 性能
cargo test --test tpch_sf1_benchmark -- --compare-baseline v2.6.0
```

**目标**: 核心查询响应时间 <= v2.6.0 的 85%

### 5.2 Sysbench OLTP 测试

```bash
# 点查 QPS
cargo test --test sysbench_oltp -- oltp_point_select

# 只读 QPS
cargo test --test sysbench_oltp -- oltp_read_only

# 读写混合 TPS
cargo test --test sysbench_oltp -- oltp_read_write
```

**目标**: QPS >= 4000, TPS >= 400

### 5.3 SIMD 性能测试

```bash
# SIMD 聚合测试
cargo test --test simd_aggregation_test

# SIMD 过滤测试
cargo test --test simd_filter_test
```

---

## 六、稳定性测试程序

### 6.1 72 小时长稳测试

```bash
# 启动长稳测试
cargo test --test stability_72h -- --duration 72h

# 监控资源使用
./scripts/monitor_resources.sh &
```

**监控指标**:
- 内存泄漏检测
- CPU 使用率
- 磁盘 I/O
- 查询延迟 P95/P99

### 6.2 压力测试

```bash
# 并发连接测试
cargo test --test concurrency_test -- --threads 100

# 混合负载测试
cargo test --test mixed_workload_test
```

---

## 七、故障排查

### 7.1 测试失败排查

```bash
# 1. 查看详细错误
cargo test -- --nocapture

# 2. 运行单个测试
cargo test test_name -- --nocapture --test-threads=1

# 3. 清理并重新测试
cargo clean
cargo test
```

### 7.2 性能测试问题

```bash
# 1. 检查系统资源
top
free -h

# 2. 运行 debug 版本对比
cargo test --test tpch_sf1_benchmark -- --debug
```

### 7.3 WAL 相关问题

```bash
# 检查 WAL 日志
cat ./data/*/wal/*.log

# 验证 WAL 完整性
./target/release/sqlrustgo check --wal-integrity
```

---

## 八、测试数据设置

### 8.1 TPC-H 测试数据

```bash
# 生成 SF1 测试数据
cargo run --bin tpch_data -- --sf 1 --output ./test_data/tpch_sf1

# 生成 SF10 测试数据 (夜间)
cargo run --bin tpch_data -- --sf 10 --output ./test_data/tpch_sf10
```

### 8.2 GMP Top10 测试数据

```bash
# 生成 GMP 审核场景测试数据
cargo run --bin gmp_test_data -- --output ./test_data/gmp_top10
```

### 8.3 向量搜索测试数据

```bash
# 生成向量测试数据
cargo run --bin vector_test_data -- --dimensions 1536 --count 10000 --output ./test_data/vector_search
```

---

## 九、相关文档

| 文档 | 说明 |
|------|------|
| [TEST_PLAN.md](./TEST_PLAN.md) | 全面测试计划 |
| [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md) | 升级指南 |
| [RELEASE_NOTES.md](./RELEASE_NOTES.md) | 版本发布说明 |
| [PERFORMANCE_TARGETS.md](./PERFORMANCE_TARGETS.md) | 性能目标 |

---

*测试手册 v2.7.0*
*最后更新: 2026-04-22*
*Enterprise Resilience GA 版本*
