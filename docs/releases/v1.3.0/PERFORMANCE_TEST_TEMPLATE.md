# 性能测试模板

> **版本**: v1.3.0  
> **目标**: 建立性能基准和回归测试  
> **工具**: Criterion

---

## 1. 性能测试配置

### 1.1 Cargo.toml 配置

```toml
[[bench]]
name = "executor_bench"
harness = false

[[bench]]
name = "planner_bench"
harness = false

[[bench]]
name = "optimizer_bench"
harness = false
```

### 1.2 基准测试结构

```rust
// benches/executor_bench.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use sqlrustgo_executor::*;

// 共享测试数据
fn setup_test_data(row_count: usize) -> TestData {
    // 生成测试数据
}

fn benchmark_table_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("table_scan");
    
    for size in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let data = setup_test_data(size);
            b.iter(|| {
                // 执行表扫描
            });
        });
    }
}

criterion_group!(benches, benchmark_table_scan);
criterion_main!(benches);
```

---

## 2. 测试场景模板

### 2.1 Executor 性能测试

| 测试场景 | 数据量 | 目标 | 优先级 |
|----------|--------|------|--------|
| SeqScan 全表扫描 | 1K ~ 1M 行 | <100ms/100K行 | P0 |
| SeqScan + Filter | 1K ~ 100K 行 | <50ms/100K行 | P0 |
| Projection | 1K ~ 100K 行 | <20ms/100K行 | P1 |
| Hash Join | 10K x 10K | <500ms | P0 |
| 聚合 (COUNT) | 1K ~ 100K 行 | <100ms | P1 |
| 聚合 (GROUP BY) | 1K ~ 100K 行 | <200ms | P1 |

### 2.2 Planner 性能测试

| 测试场景 | 复杂度 | 目标 | 优先级 |
|----------|--------|------|--------|
| 简单 SELECT | 1 表 | <10ms | P0 |
| 多表 JOIN | 2-3 表 | <50ms | P0 |
| 复杂 WHERE | 5+ 条件 | <20ms | P1 |
| 子查询 | 嵌套 2 层 | <100ms | P1 |

### 2.3 Optimizer 性能测试

| 测试场景 | 规则数 | 目标 | 优先级 |
|----------|--------|------|--------|
| Predicate Pushdown | 3 规则 | <10ms | P0 |
| Constant Folding | 5 规则 | <5ms | P1 |
| Join Reordering | 4 表 | <100ms | P1 |

---

## 3. 测试数据生成器

### 3.1 Schema 定义

```rust
// 测试数据生成器
pub struct TestDataGenerator {
    pub schema: Schema,
}

impl TestDataGenerator {
    pub fn new() -> Self {
        Self {
            schema: Schema::new(vec![
                Field::new("id".to_string(), DataType::Integer),
                Field::new("name".to_string(), DataType::Text),
                Field::new("age".to_string(), DataType::Integer),
                Field::new("salary".to_string(), DataType::Float),
                Field::new("created_at".to_string(), DataType::Timestamp),
            ]),
        }
    }

    pub fn generate_rows(&self, count: usize) -> Vec<Record> {
        (0..count)
            .map(|i| vec![
                Value::Integer(i as i64),
                Value::Text(format!("user_{}", i)),
                Value::Integer((i % 100) as i64),
                Value::Float((i as f64) * 1.5),
                Value::Timestamp(/* ... */),
            ])
            .collect()
    }

    pub fn generate_with_joins(&self, size_a: usize, size_b: usize) -> (Vec<Record>, Vec<Record>) {
        // 生成可关联的测试数据
    }
}
```

### 3.2 测试数据配置

```rust
// 测试数据规模
#[derive(Clone, Copy)]
pub enum DataSize {
    Tiny,    // 100 行
    Small,   // 1,000 行
    Medium,  // 10,000 行
    Large,   // 100,000 行
    Huge,    // 1,000,000 行
}

impl DataSize {
    pub fn row_count(&self) -> usize {
        match self {
            Self::Tiny => 100,
            Self::Small => 1_000,
            Self::Medium => 10_000,
            Self::Large => 100_000,
            Self::Huge => 1_000_000,
        }
    }
}
```

---

## 4. 基准测试模板

### 4.1 表扫描基准

```rust
// benches/table_scan_bench.rs

fn bench_seq_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("executor/seq_scan");
    
    let sizes = [100, 1000, 10000, 100000];
    
    for size in sizes.iter() {
        let data = generate_test_data(*size);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| {
                b.iter(|| {
                    // 执行 SeqScan
                    let mut executor = SeqScanExec::new(
                        "test_table".to_string(),
                        data.schema.clone(),
                    );
                    executor.execute()
                });
            }
        );
    }
}
```

### 4.2 Hash Join 基准

```rust
fn bench_hash_join(c: &mut Criterion) {
    let mut group = c.benchmark_group("executor/hash_join");
    
    let sizes = [(100, 100), (1000, 1000), (10000, 10000)];
    
    for (size_a, size_b) in sizes.iter() {
        let (data_a, data_b) = generate_join_data(*size_a, *size_b);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}x{}", size_a, size_b)),
            &(size_a, size_b),
            |b, _| {
                b.iter(|| {
                    // 执行 Hash Join
                    let mut join = HashJoinExec::new(
                        data_a.clone(),
                        data_b.clone(),
                        JoinType::Inner,
                        "id".to_string(),
                    );
                    join.execute()
                });
            }
        );
    }
}
```

### 4.3 聚合查询基准

```rust
fn bench_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("executor/aggregation");
    
    let sizes = [1000, 10000, 100000];
    let agg_funs = ["COUNT", "SUM", "AVG", "MIN", "MAX"];
    
    for size in sizes.iter() {
        for func in agg_funs.iter() {
            let data = generate_test_data(*size);
            
            group.bench_with_input(
                BenchmarkId::new(func, size),
                &(func, size),
                |b, _| {
                    b.iter(|| {
                        // 执行聚合
                        execute_aggregate(&data, func)
                    });
                }
            );
        }
    }
}
```

---

## 5. 性能回归检测

### 5.1 阈值配置

```yaml
# .github/workflows/performance.yml
- name: Performance Regression
  run: |
    # 运行基准测试
    cargo bench --no-save
    
    # 比较结果
    cargo bench --save-baseline current
    
    # 与上次比较
    if [ -f "previous_baseline" ]; then
      cargo bench --compare previous_baseline
    fi
```

### 5.2 性能阈值表

| 操作 | 阈值 (ms) | 超过则 |
|------|-----------|--------|
| SeqScan 1K 行 | 10 | 警告 |
| SeqScan 100K 行 | 100 | 警告 |
| Hash Join 10K x 10K | 500 | 警告 |
| 聚合 10K 行 | 50 | 警告 |
| Planner 优化 | 100 | 警告 |

---

## 6. 内存使用测试

### 6.1 内存基准

```rust
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");
    
    let sizes = [1000, 10000, 100000];
    
    for size in sizes.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| {
                b.iter(|| {
                    // 测量内存使用
                    let before = memory_usage();
                    let data = generate_test_data(*size);
                    let after = memory_usage();
                    after - before
                });
            }
        );
    }
}

fn memory_usage() -> usize {
    // 获取当前内存使用
    std::mem::size_of::<TestData>() // 或使用 os_info
}
```

### 6.2 内存阈值

| 场景 | 最大内存 |
|------|----------|
| 1K 行数据 | 10 MB |
| 100K 行数据 | 100 MB |
| 1M 行数据 | 1 GB |

---

## 7. 报告生成

### 7.1 HTML 报告

```bash
# 生成 HTML 报告
cargo bench --output-format html

# 查看报告
open target/criterion/report/index.html
```

### 7.2 CI 性能报告

```yaml
- name: Generate Performance Report
  run: |
    cargo bench --message-format=json > bench_results.json
    
- name: Upload Results
  uses: actions/upload-artifact@v4
  with:
    name: benchmark-results
    path: target/criterion/
```

---

## 8. 测试脚本

### 8.1 快速性能检查

```bash
#!/bin/bash
# scripts/quick_bench.sh

echo "========================================="
echo "快速性能测试"
echo "========================================="

# 小规模测试 (快速)
echo "运行小规模测试..."
cargo bench --small

# 中等规模测试
echo "运行中等规模测试..."
cargo bench --medium

# 检查结果
echo ""
echo "性能检查结果:"
cat target/criterion/summary.txt
```

### 8.2 完整性能测试

```bash
#!/bin/bash
# scripts/full_bench.sh

set -e

echo "========================================="
echo "完整性能测试"
echo "========================================="

# 清理之前的基准
cargo bench --clean

# 预热
cargo bench --warmup

# 完整测试
cargo bench

# 生成报告
cargo bench --output-format html

# 上传到跟踪系统
curl -X POST "$PERFORMANCE_API" -d @target/criterion/report/data.json

echo "✅ 性能测试完成"
```

---

## 9. 相关文档

- [v1.2.0 性能报告](../v1.2.0/PERFORMANCE_REPORT.md)
- [测试计划](./TEST_PLAN.md)
- [覆盖率模板](./COVERAGE_TEST_TEMPLATE.md)

---

**模板版本**: 1.0  
**最后更新**: 2026-03-15
