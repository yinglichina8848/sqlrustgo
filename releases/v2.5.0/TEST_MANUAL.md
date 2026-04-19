# SQLRustGo v2.5.0 测试手册

**版本**: v2.5.0 (里程碑版本)
**发布日期**: 2026-04-16

---

## 一、测试环境准备

### 1.1 环境要求

| 组件 | 要求 |
|------|------|
| Rust | 1.70+ |
| 内存 | 16GB+ |
| 磁盘 | 20GB+ SSD |
| 操作系统 | Linux/macOS |

### 1.2 环境搭建

```bash
# 1. 克隆代码
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 2. 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. 编译项目
cargo build --release

# 4. 验证安装
cargo test --version
```

---

## 二、测试类型与命令

### 2.1 单元测试

运行单个 crate 的单元测试:

```bash
# 运行 parser 单元测试
cargo test -p sqlrustgo-parser --lib

# 运行 storage 单元测试
cargo test -p sqlrustgo-storage --lib

# 运行 executor 单元测试
cargo test -p sqlrustgo-executor --lib

# 运行所有单元测试
cargo test --lib --workspace
```

运行特定测试:

```bash
# 运行指定测试函数
cargo test -p sqlrustgo-parser test_parse_select

# 运行匹配的测试
cargo test test_parse_

# 显示测试输出
cargo test -- --nocapture
```

### 2.2 集成测试

```bash
# 运行回归测试
cargo test --test regression_test

# 运行外键测试
cargo test --test foreign_key_test

# 运行预处理语句测试
cargo test --test prepared_statement_test

# 运行子查询测试
cargo test --test subquery_test

# 运行 MVCC 快照隔离测试
cargo test --test mvcc_snapshot_isolation_test

# 运行窗口函数测试
cargo test --test window_function_test
```

### 2.3 性能测试

#### TPC-H 测试

```bash
# TPC-H SF=1
cargo test --test tpch_sf1_benchmark

# TPC-H SF=10
cargo test --test tpch_sf10_benchmark

# 查看详细输出
cargo test --test tpch_sf1_benchmark -- --nocapture
```

#### OLTP 工作负载测试

```bash
# 运行所有 OLTP 测试
cargo test --test oltp_workload_test

# 运行特定工作负载
cargo test --test oltp_workload_test oltp_index_scan
cargo test --test oltp_workload_test oltp_insert
cargo test --test oltp_workload_test oltp_delete
```

#### 向量搜索测试

```bash
# 向量搜索测试
cargo test --test vector_search_test

# HNSW 索引测试
cargo test --test hnsw_test

# IVFPQ 索引测试
cargo test --test ivfpq_test
```

#### 图查询测试

```bash
# 图遍历测试
cargo test --test graph_tests

# Cypher 查询测试
cargo test --test cypher_test
```

### 2.4 压力测试

```bash
# 崩溃恢复测试
cargo test --test crash_recovery_test

# 长时间运行测试
cargo test --test long_run_stability_test

# 并发测试
cargo test --test mvcc_concurrency_test
```

### 2.5 覆盖率测试

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --workspace --out html

# 生成 XML 报告 (Cobertura)
cargo tarpaulin --workspace --out xml

# 查看报告
open tarpaulin-report.html
```

---

## 三、功能测试指南

### 3.1 MVCC 事务测试

```bash
# 1. 启动 SQLRustGo
cargo run --release &

# 2. 连接数据库
sqlrustgo --database test

# 3. 运行 MVCC 测试
# 测试快照隔离
CREATE TABLE t1 (id INT PRIMARY KEY, value TEXT);
BEGIN ISOLATION LEVEL SNAPSHOT;
INSERT INTO t1 VALUES (1, 'a');
COMMIT;

# 4. 验证测试
SELECT * FROM t1; -- 应该看到已提交的数据
```

### 3.2 外键约束测试

```bash
# 测试外键约束
CREATE TABLE parent (id INT PRIMARY KEY);
CREATE TABLE child (id INT, parent_id INT REFERENCES parent(id));

-- 正常插入
INSERT INTO parent VALUES (1);
INSERT INTO child VALUES (1, 1);

-- 违反外键 (应该失败)
INSERT INTO child VALUES (2, 99);
```

### 3.3 预处理语句测试

```bash
# 准备语句
PREPARE stmt AS SELECT * FROM t1 WHERE id = $1;

# 执行
EXECUTE stmt(1);

# 释放
DEALLOCATE stmt;
```

### 3.4 子查询测试

```sql
-- EXISTS 子查询
SELECT * FROM t1 WHERE EXISTS (SELECT 1 FROM t2 WHERE t2.id = t1.id);

-- IN 子查询
SELECT * FROM t1 WHERE id IN (SELECT id FROM t2);

-- ANY/ALL
SELECT * FROM t1 WHERE id > ANY (SELECT id FROM t2);
```

### 3.5 Cypher 图查询测试

```sql
-- 创建图
CREATE GRAPH my_graph (name VARCHAR);

-- 插入节点
CYPHER { CREATE (a:Person {name: 'Alice'}) }

-- 插入边
CYPHER { MATCH (a:Person {name: 'Alice'}) CREATE (a)-[:KNOWS]->(b:Person {name: 'Bob'}) }

-- 查询
CYPHER { MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name }
```

### 3.6 向量搜索测试

```sql
-- 创建向量表
CREATE TABLE vectors (id INT, embedding FLOAT[128]);

-- 插入向量
INSERT INTO vectors VALUES (1, '[0.1, 0.2, ...]');

-- 搜索
SELECT * FROM vectors ORDER BY vector_distance(embedding, '[0.1, 0.2, ...]') LIMIT 10;
```

---

## 四、测试用例详解

### 4.1 外键约束测试用例

```bash
# 测试 CASCADE
cargo test --test foreign_key_test fk_cascade -- --nocapture

# 测试 RESTRICT
cargo test --test foreign_key_test fk_restrict -- --nocapture

# 测试 SET NULL
cargo test --test foreign_key_test fk_set_null -- --nocapture
```

### 4.2 MVCC 快照隔离测试用例

```bash
# 测试读不阻塞写
cargo test --test mvcc_snapshot_isolation_test read_no_block_write -- --nocapture

# 测试写不阻塞读
cargo test --test mvcc_snapshot_isolation_test write_no_block_read -- --nocapture

# 测试快照一致性
cargo test --test mvcc_snapshot_isolation_test snapshot_consistency -- --nocapture
```

### 4.3 预处理语句测试用例

```bash
# 测试参数绑定
cargo test --test prepared_statement_test prepare_with_params -- --nocapture

# 测试多次执行
cargo test --test prepared_statement_test execute_multiple -- --nocapture
```

### 4.4 性能测试用例

```bash
# TPC-H Q1 性能测试
cargo test --test tpch_sf1_benchmark tpch_q1 -- --nocapture

# OLTP 点查性能测试
cargo test --test oltp_workload_test oltp_point_select -- --nocapture

# 向量搜索延迟测试
cargo test --test vector_search_test search_latency -- --nocapture
```

---

## 五、故障排查

### 5.1 测试失败排查

```bash
# 1. 查看详细错误
cargo test -- --nocapture

# 2. 运行单个测试
cargo test test_name -- --nocapture --test-threads=1

# 3. 检查测试数据
ls -la data/

# 4. 清理并重新测试
cargo clean
cargo test
```

### 5.2 性能测试问题

```bash
# 1. 检查系统资源
top
free -h
iostat -x 1

# 2. 运行 debug 版本对比
cargo test --test tpch_sf1_benchmark

# 3. 检查日志
tail -f logs/sqlrustgo.log
```

### 5.3 编译问题

```bash
# 1. 更新依赖
cargo update

# 2. 清理并重编译
cargo clean
cargo build

# 3. 检查 Rust 版本
rustc --version
rustup update
```

---

## 六、最佳实践

### 6.1 测试编写规范

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name_given_when_then() {
        // Given: 设置测试数据
        let input = create_test_data();

        // When: 执行操作
        let result = process(input);

        // Then: 验证结果
        assert_eq!(result, expected_value);
    }

    #[test]
    fn test_error_case() {
        let invalid_input = create_invalid_data();
        let result = process(invalid_input);
        assert!(result.is_err());
    }
}
```

### 6.2 测试数据管理

```bash
# 准备测试数据
./scripts/prepare_test_data.sh

# 清理测试数据
./scripts/cleanup_test_data.sh

# 重置测试环境
./scripts/reset_test_env.sh
```

### 6.3 CI 集成建议

```bash
# 本地运行完整测试套件
./scripts/run_full_tests.sh

# 本地运行快速测试
./scripts/run_quick_tests.sh

# 本地运行覆盖率检查
./scripts/check_coverage.sh
```

---

## 七、测试脚本

### 7.1 快速测试脚本

创建 `scripts/run_quick_tests.sh`:

```bash
#!/bin/bash
set -e

echo "=== Running Quick Tests ==="

# 单元测试
echo "Running unit tests..."
cargo test --lib --workspace

# 文档测试
echo "Running doc tests..."
cargo test --doc

# 快速集成测试
echo "Running quick integration tests..."
cargo test --test regression_test

echo "=== Quick Tests Complete ==="
```

### 7.2 完整测试脚本

创建 `scripts/run_full_tests.sh`:

```bash
#!/bin/bash
set -e

echo "=== Running Full Test Suite ==="

# 清理
cargo clean

# 编译
cargo build --release

# 单元测试
echo "Running unit tests..."
cargo test --lib --workspace

# 集成测试
echo "Running integration tests..."
cargo test --test '*'

# 性能测试
echo "Running performance tests..."
cargo bench --workspace

# 覆盖率
echo "Running coverage..."
cargo tarpaulin --workspace --out html

echo "=== Full Test Suite Complete ==="
```

### 7.3 测试数据准备脚本

创建 `scripts/prepare_test_data.sh`:

```bash
#!/bin/bash

echo "=== Preparing Test Data ==="

# 准备 TPC-H 数据
mkdir -p data/tpch-sf01
./target/release/sqlrustgo-tools generate-tpch --sf 1 --output data/tpch-sf01

# 准备向量测试数据
mkdir -p data/vectors
./target/release/sqlrustgo-tools generate-vectors --count 10000 --dim 128 --output data/vectors

echo "=== Test Data Ready ==="
```

---

## 八、附录

### A. 测试命令速查表

| 测试类型 | 命令 |
|----------|------|
| 单元测试 | `cargo test --lib` |
| 集成测试 | `cargo test --test <name>` |
| 性能测试 | `cargo bench --bench <name>` |
| 覆盖率 | `cargo tarpaulin --workspace` |
| 文档测试 | `cargo test --doc` |
| 全量测试 | `cargo test --workspace` |

### B. 测试数据路径

| 数据类型 | 路径 |
|----------|------|
| TPC-H SF=1 | `data/tpch-sf01/` |
| TPC-H SF=10 | `data/tpch-sf10/` |
| 向量测试 | `data/vectors/` |
| 图测试 | `data/graph/` |

### C. 测试日志位置

| 日志类型 | 路径 |
|----------|------|
| 测试日志 | `logs/test.log` |
| 性能日志 | `logs/benchmark.log` |
| 错误日志 | `logs/error.log` |

---

*测试手册 v2.5.0*
*最后更新: 2026-04-16*
