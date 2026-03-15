# SQLRustGo v1.3.0 测试手册

> 版本：v1.3.0
> 发布日期：2026-03-15
> 适用用户：开发人员、测试工程师

---

## 一，测试概览

### 1.1 测试目标

v1.3.0 版本测试覆盖以下关键领域：

| 模块 | 覆盖率目标 | 实际覆盖率 |
|------|-----------|-----------|
| 整体 | ≥65% | **81.26%** |
| Executor | ≥60% | 87%+ |
| Planner | ≥60% | 76% |
| Optimizer | ≥40% | 82% |

### 1.2 测试类型

| 类型 | 说明 | 位置 |
|------|------|------|
| 单元测试 | 各模块独立测试 | `crates/*/src/` |
| 集成测试 | 多模块协作测试 | `tests/` |
| 基准测试 | 性能测试 | `benches/` |

---

## 二，运行测试

### 2.1 快速开始

```bash
# 运行所有测试
cargo test --workspace

# 运行特定 crate 的测试
cargo test -p sqlrustgo-executor

# 运行特定测试
cargo test test_executor

# 查看测试覆盖率
cargo tarpaulin --workspace --all-features
```

### 2.2 测试命令参考

| 命令 | 说明 |
|------|------|
| `cargo test` | 运行所有测试 |
| `cargo test --workspace` | 运行工作区所有测试 |
| `cargo test --doc` | 运行文档测试 |
| `cargo test --no-run` | 编译测试但不运行 |
| `cargo test -- --nocapture` | 显示测试输出 |

### 2.3 覆盖率命令

```bash
# 使用 tarpaulin (推荐)
cargo tarpaulin --workspace --all-features --output-dir ./target/coverage

# 使用 llvm-cov
cargo llvm-cov --workspace --all-features --summary-only

# 查看 HTML 报告
open target/tarpaulin/tarpaulin.html
```

---

## 三，单元测试

### 3.1 Executor 模块测试

位置：`crates/executor/src/`

```bash
cargo test -p sqlrustgo-executor
```

测试覆盖：
- VolcanoExecutor trait
- TableScan 算子
- Projection 算子
- Filter 算子
- HashJoin 算子
- Aggregate 算子
- Limit 算子

### 3.2 Planner 模块测试

位置：`crates/planner/src/`

```bash
cargo test -p sqlrustgo-planner
```

测试覆盖：
- LogicalPlan 创建
- PhysicalPlan 转换
- 表达式求值

### 3.3 Optimizer 模块测试

位置：`crates/optimizer/src/`

```bash
cargo test -p sqlrustgo-optimizer
```

测试覆盖：
- 谓词下推 (PredicatePushdown)
- 投影剪枝 (ProjectionPruning)
- 常量折叠 (ConstantFolding)
- 表达式简化 (ExpressionSimplification)
- Join 重排序 (JoinReordering)

### 3.4 Storage 模块测试

位置：`crates/storage/src/`

```bash
cargo test -p sqlrustgo-storage
```

测试覆盖：
- Buffer Pool
- Page 管理
- B+ Tree 索引
- 文件存储

### 3.5 可观测性测试 (v1.3.0 新增)

位置：`tests/observability_test.rs`

```bash
cargo test --test observability_test
```

测试覆盖：
- Health Checker 单元测试
- Metrics 单元测试
- Network Metrics 单元测试
- Executor Metrics 单元测试

测试用例：

```rust
#[test]
fn test_health_checker_live() { /* ... */ }

#[test]
fn test_health_checker_ready() { /* ... */ }

#[test]
fn test_network_metrics_connection_lifecycle() { /* ... */ }

#[test]
fn test_executor_metrics_query_recording() { /* ... */ }

#[test]
fn test_metrics_registry_prometheus_format() { /* ... */ }
```

### 3.6 监控配置测试

位置：`tests/monitoring_test.rs`

```bash
cargo test --test monitoring_test
```

测试覆盖：
- Grafana Dashboard JSON 验证
- Prometheus Alert YAML 验证

---

## 四，集成测试

### 4.1 可观测性集成测试

位置：`tests/observability_test.rs`

```bash
cargo test --test observability_test
```

测试场景：

| 测试 | 说明 |
|------|------|
| `test_health_checker_component_status` | 健康检查组件状态 |
| `test_metrics_collection_flow` | 指标收集流程 |
| `test_prometheus_format_output` | Prometheus 格式输出 |

### 4.2 执行器集成测试

位置：`crates/executor/tests/`

```bash
cargo test --test integration_test
```

测试场景：
- 完整查询执行流程
- 多算子协作
- 错误处理

---

## 五，性能测试

### 5.1 基准测试

位置：`benches/`

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench --bench bench_v130

# 运行自定义基准测试
cargo bench --bench executor_bench
```

### 5.2 v1.3.0 基准测试

位置：`benches/bench_v130.rs`

#### TableScan 基准

```bash
cargo bench --bench bench_v130 -- tablescan
```

| 测试 | 数据量 |
|------|--------|
| tablescan/full_scan/100 | 100 行 |
| tablescan/full_scan/1000 | 1,000 行 |
| tablescan/full_scan/10000 | 10,000 行 |
| tablescan_projection | 列投影 |

#### Filter 基准

```bash
cargo bench --bench bench_v130 -- filter
```

| 测试 | 条件类型 |
|------|----------|
| filter/equality | 等值过滤 |
| filter_range | 范围过滤 |
| filter_and | AND 条件 |

#### HashJoin 基准

```bash
cargo bench --bench bench_v130 -- hashjoin
```

| 测试 | 连接类型 |
|------|----------|
| hashjoin_inner | 内连接 |
| hashjoin_left | 左连接 |
| hashjoin_cross | 交叉连接 |

### 5.3 性能目标验证

```bash
# 运行性能测试
cargo bench

# 比较结果
cargo bench --compare main
```

| 操作 | 目标 | 验证命令 |
|------|------|----------|
| INSERT 100k rows | < 2s | `cargo bench --bench bench_insert` |
| SELECT * (100k) | < 200ms | `cargo bench --bench bench_scan` |
| HashJoin | < 2s | `cargo bench --bench bench_v130 -- hashjoin_inner` |

---

## 六，测试开发

### 6.1 编写单元测试

位置：`crates/*/src/` 内的 `#[cfg(test)]` 模块

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        // 测试代码
        assert_eq!(expected, actual);
    }
}
```

### 6.2 编写集成测试

位置：`tests/` 目录

```rust
// tests/my_integration_test.rs
use sqlrustgo::*;

#[test]
fn test_integration() {
    // 集成测试代码
}
```

### 6.3 编写基准测试

位置：`benches/` 目录

```rust
// benches/my_bench.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_my_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // 基准测试代码
        });
    });
}

criterion_group!(benches, bench_my_function);
criterion_main!(benches);
```

### 6.4 测试最佳实践

1. **命名规范**
   - 单元测试: `test_module_function_scenario`
   - 集成测试: `test_integration_workflow`
   - 基准测试: `bench_operation_size`

2. **测试隔离**
   - 每个测试独立运行
   - 不依赖测试执行顺序
   - 清理测试数据

3. **断言清晰**
   - 使用有意义的断言消息
   - 包含预期值和实际值

4. **覆盖率**
   - 新功能必须包含测试
   - 目标: ≥80% 覆盖率

---

## 七，持续集成

### 7.1 GitHub Actions

项目配置了自动化 CI：

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Run tests
        run: cargo test --workspace
      - name: Run clippy
        run: cargo clippy --workspace -- -D warnings
      - name: Check format
        run: cargo fmt --all -- --check
```

### 7.2 本地 CI

```bash
# 运行完整检查
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
cargo tarpaulin --workspace --all-features
```

---

## 八，测试报告

### 8.1 生成报告

```bash
# 生成 HTML 报告
cargo tarpaulin --workspace --all-features --output-dir ./target/coverage --html

# 生成 XML 报告 (CI)
cargo tarpaulin --workspace --all-features --out Xml

# 生成 JSON 报告
cargo llvm-cov --workspace --all-features --json --output-path coverage.json
```

### 8.2 报告解读

| 指标 | 说明 | 目标 |
|------|------|------|
| 行覆盖率 | 代码行数覆盖率 | ≥65% |
| 分支覆盖率 | 条件分支覆盖率 | ≥50% |
| 函数覆盖率 | 函数调用覆盖率 | ≥70% |

---

## 九，测试检查清单

### 9.1 功能测试

- [ ] Executor 算子测试通过
- [ ] Planner 测试通过
- [ ] Optimizer 测试通过
- [ ] Storage 测试通过

### 9.2 可观测性测试 (v1.3.0)

- [ ] Health Checker 单元测试
- [ ] Metrics 单元测试
- [ ] Network Metrics 单元测试
- [ ] Executor Metrics 单元测试
- [ ] /health/live 端点测试
- [ ] /health/ready 端点测试
- [ ] /metrics 端点测试

### 9.3 性能测试

- [ ] TableScan 基准测试
- [ ] Filter 基准测试
- [ ] HashJoin 基准测试
- [ ] 性能目标验证

### 9.4 质量检查

- [ ] `cargo build --workspace` 通过
- [ ] `cargo test --workspace` 全部通过
- [ ] `cargo clippy --workspace` 零警告
- [ ] `cargo fmt --all` 格式化通过
- [ ] 覆盖率 ≥ 65%

---

## 十，相关文档

| 文档 | 说明 |
|------|------|
| `TEST_PLAN.md` | 测试计划 |
| `TEST_VERIFICATION_PLAN.md` | 测试验证计划 |
| `COVERAGE_TEST_TEMPLATE.md` | 覆盖率测试模板 |
| `PERFORMANCE_TEST_TEMPLATE.md` | 性能测试模板 |
| `OBSERVABILITY_GUIDE.md` | 可观测性开发指南 |

---

*文档版本: 1.0*
*最后更新: 2026-03-15*
