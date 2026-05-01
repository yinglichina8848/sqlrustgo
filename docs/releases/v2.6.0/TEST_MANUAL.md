# SQLRustGo v2.6.0 测试手册

**版本**: v2.6.0 (生产就绪版本)
**发布日期**: 2026-04-17

---

## 一、测试环境准备

### 1.1 环境要求

| 组件 | 要求 |
|------|------|
| Rust | 1.70+ |
| 内存 | 16GB+ |
| 磁盘 | 20GB+ SSD |

### 1.2 环境搭建

```bash
# 1. 克隆代码
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 2. 切换到 v2.6.0
git checkout v2.6.0

# 3. 编译项目
cargo build --release

# 4. 验证安装
cargo test --version
```

---

## 二、测试类型与命令

### 2.1 单元测试

```bash
# 运行所有单元测试
cargo test --lib --workspace

# 运行特定 crate 测试
cargo test -p sqlrustgo-parser --lib
cargo test -p sqlrustgo-executor --lib
```

### 2.2 集成测试

```bash
# 运行所有集成测试
cargo test --test '*'

# 运行特定测试
cargo test --test regression_test
cargo test --test sql_compliance_test
```

### 2.3 性能测试

```bash
# TPC-H 测试
cargo test --test tpch_sf1_benchmark

# OLTP 测试
cargo test --test oltp_workload_test
```

### 2.4 覆盖率测试

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --workspace --out html
```

---

## 三、SQL 合规性测试

### 3.1 运行 SQL-92 合规性测试

```bash
# 运行 SQL-92 测试
cargo test --test sql92_compliance_test

# 运行特定特性测试
cargo test --test sql92_compliance_test case_expression
cargo test --test sql92_compliance_test subquery
cargo test --test sql92_compliance_test set_operation
```

### 3.2 MySQL 兼容性测试

```bash
# 运行 MySQL 兼容测试
cargo test --test mysql_compatibility_test

# 运行特定兼容性测试
cargo test --test mysql_compatibility_test dml
cargo test --test mysql_compatibility_test ddl
```

---

## 四、功能测试指南

### 4.1 MVCC 事务测试

```bash
# 测试 MVCC 快照隔离
cargo test --test mvcc_snapshot_isolation_test

# 测试 SSI (Serializable Snapshot Isolation)
cargo test --test mvcc_snapshot_isolation_test ssi
```

### 4.2 向量搜索测试

```bash
# 向量搜索测试
cargo test --test vector_search_test

# HNSW 索引测试
cargo test --test hnsw_test

# IVFPQ 索引测试
cargo test --test ivfpq_test
```

### 4.3 SIMD 性能测试

```bash
# SIMD 聚合测试
cargo test --test simd_aggregation_test

# SIMD 过滤测试
cargo test --test simd_filter_test
```

---

## 五、故障排查

### 5.1 测试失败排查

```bash
# 1. 查看详细错误
cargo test -- --nocapture

# 2. 运行单个测试
cargo test test_name -- --nocapture --test-threads=1

# 3. 清理并重新测试
cargo clean
cargo test
```

### 5.2 性能测试问题

```bash
# 1. 检查系统资源
top
free -h

# 2. 运行 debug 版本对比
cargo test --test tpch_sf1_benchmark
```

---

## 六、相关文档

| 文档 | 说明 |
|------|------|
| [TEST_PLAN.md](./TEST_PLAN.md) | 测试计划 |
| [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md) | 升级指南 |

---

*测试手册 v2.6.0*
*最后更新: 2026-04-17*
