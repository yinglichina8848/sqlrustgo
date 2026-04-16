# SQLRustGo 性能测试框架设计

> 版本：v1.0
> 日期：2026-03-02
> 目标：建立完整的性能测试和基准测试体系

---

## 一、当前状态

| 项目 | 状态 | 说明 |
|------|------|------|
|Criterion 依赖| ✅ 已添加 |0.8版本|
| 基准测试文件 | ❌ 不存在 |无 `benches/` 目录|
| 性能指标 | ❌ 无 | 无基准数据 |
| CI 集成 | ❌ 无 | 无性能回归检测 |

---

## 二、性能测试框架设计

### 2.1 目录结构

```
benches/
├── README.md                    # 基准测试说明
├── lexer_bench.rs               # 词法分析器基准
├── parser_bench.rs              # 语法分析器基准
├── executor_bench.rs            # 执行器基准
├── storage_bench.rs             # 存储引擎基准
├── network_bench.rs             # 网络层基准
├── planner_bench.rs             # 规划器基准
└── integration_bench.rs         # 端到端基准
```

### 2.2 基准测试分类

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          性能测试分类                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   微基准 (Micro-benchmarks)                                                 │
│   ├── Lexer: 词法分析速度                                                   │
│   ├── Parser: 语法分析速度                                                  │
│   ├── Executor: 单算子执行速度                                              │
│   └── Storage: B+树操作速度                                                 │
│                                                                              │
│   宏基准 (Macro-benchmarks)                                                 │
│   ├── 端到端查询: SELECT/INSERT/UPDATE/DELETE                               │
│   ├── 并发查询: 多线程执行                                                  │
│   └── 网络吞吐: Client-Server 通信                                          │
│                                                                              │
│   负载测试 (Load Testing)                                                   │
│   ├── 大数据集: 10万/100万/1000万行                                         │
│   ├── 高并发: 10/100/1000 连接                                              │
│   └── 长时间运行: 内存泄漏检测                                              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 三、基准测试实现

### 3.1 Lexer 基准测试

```rust
// benches/lexer_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sqlrustgo::lexer::Lexer;

fn bench_lexer_simple(c: &mut Criterion) {
    let sql = "SELECT id, name FROM users WHERE id = 1";
    c.bench_function("lexer_simple", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(sql));
            lexer.tokenize()
        })
    });
}

fn bench_lexer_complex(c: &mut Criterion) {
    let sql = r#"
        SELECT u.id, u.name, o.order_id, o.total
        FROM users u
        JOIN orders o ON u.id = o.user_id
        WHERE u.status = 'active' AND o.total > 100
        ORDER BY o.total DESC
        LIMIT 100
    "#;
    c.bench_function("lexer_complex", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(sql));
            lexer.tokenize()
        })
    });
}

criterion_group!(benches, bench_lexer_simple, bench_lexer_complex);
criterion_main!(benches);
```

### 3.2 Parser 基准测试

```rust
// benches/parser_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sqlrustgo::parser::Parser;
use sqlrustgo::lexer::Lexer;

fn bench_parser_select(c: &mut Criterion) {
    let sql = "SELECT id, name FROM users WHERE id = 1";
    c.bench_function("parser_select", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(sql));
            let tokens = lexer.tokenize().unwrap();
            let mut parser = Parser::new(tokens);
            parser.parse()
        })
    });
}

fn bench_parser_join(c: &mut Criterion) {
    let sql = r#"
        SELECT u.id, u.name, o.order_id
        FROM users u
        JOIN orders o ON u.id = o.user_id
        WHERE u.status = 'active'
    "#;
    c.bench_function("parser_join", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(sql));
            let tokens = lexer.tokenize().unwrap();
            let mut parser = Parser::new(tokens);
            parser.parse()
        })
    });
}

criterion_group!(benches, bench_parser_select, bench_parser_join);
criterion_main!(benches);
```

### 3.3 Executor 基准测试

```rust
// benches/executor_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sqlrustgo::executor::Executor;
use sqlrustgo::storage::StorageEngine;

fn bench_executor_select(c: &mut Criterion) {
    let mut group = c.benchmark_group("executor_select");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(format!("rows_{}", size), size, |b, &size| {
            // Setup: 创建测试数据
            let storage = StorageEngine::new_in_memory();
            storage.create_table("users", &["id", "name"]);
            for i in 0..size {
                storage.insert("users", vec![Value::Integer(i), Value::Text(format!("user_{}", i))]);
            }
            
            let mut executor = Executor::new(storage);
            let sql = "SELECT * FROM users";
            
            b.iter(|| {
                executor.execute(black_box(sql))
            });
        });
    }
    group.finish();
}

fn bench_executor_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("executor_insert");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(format!("batch_{}", size), size, |b, &size| {
            let storage = StorageEngine::new_in_memory();
            storage.create_table("users", &["id", "name"]);
            let mut executor = Executor::new(storage);
            
            b.iter(|| {
                for i in 0..*size {
                    let sql = format!("INSERT INTO users VALUES ({}, 'user_{}')", i, i);
                    executor.execute(black_box(&sql));
                }
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_executor_select, bench_executor_insert);
criterion_main!(benches);
```

### 3.4 Storage 基准测试

```rust
// benches/storage_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sqlrustgo::storage::{BPlusTree, StorageEngine};

fn bench_bptree_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("bptree_insert");
    
    for order in [32, 64, 128].iter() {
        group.bench_with_input(format!("order_{}", order), order, |b, &order| {
            let mut tree = BPlusTree::new(order);
            let mut i = 0i64;
            
            b.iter(|| {
                tree.insert(black_box(i), black_box(format!("value_{}", i)));
                i += 1;
            });
        });
    }
    group.finish();
}

fn bench_bptree_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("bptree_search");
    
    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(format!("size_{}", size), size, |b, &size| {
            let mut tree = BPlusTree::new(64);
            for i in 0..size {
                tree.insert(i, format!("value_{}", i));
            }
            
            let mut i = 0i64;
            b.iter(|| {
                tree.search(black_box(i % size));
                i += 1;
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_bptree_insert, bench_bptree_search);
criterion_main!(benches);
```

### 3.5 Network 基准测试

```rust
// benches/network_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

fn bench_network_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("network_query", |b| {
        b.to_async(&rt).iter(|| async {
            // 启动服务器
            // 连接客户端
            // 执行查询
            // 返回结果
        });
    });
}

fn bench_connection_pool(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("connection_pool_acquire_release", |b| {
        b.to_async(&rt).iter(|| async {
            // 从连接池获取连接
            // 使用连接
            // 释放连接
        });
    });
}

criterion_group!(benches, bench_network_query, bench_connection_pool);
criterion_main!(benches);
```

---

## 四、性能目标

### 4.1 目标指标

| 操作 | 目标 | 说明 |
|------|------|------|
| Lexer | < 1μs | 简单 SQL 词法分析 |
|解析器| < 10μs | 简单 SQL 语法分析 |
|SELECT (1K 行)| < 1ms | 全表扫描 |
|SELECT (10K 行)| < 10ms | 全表扫描 |
|INSERT (1 行)| < 100μs | 单行插入 |
|INSERT (1K 行)| < 100ms | 批量插入 |
| B+Tree 插入 | < 1μs | 单键插入 |
| B+Tree 查询 | < 100ns | 单键查询 |
| 网络延迟 | < 1ms | 单次查询往返 |

### 4.2 吞吐量目标

| 场景 | 目标 |
|------|------|
| QPS (简单查询) | > 10,000 |
| QPS (复杂查询) | > 1,000 |
| 并发连接 | > 1,000 |
| 数据量 | > 100万行 |

---

## 五、CI 集成

### 5.1 GitHub Actions 配置

```yaml
# .github/workflows/benchmark.yml
name: Benchmark

on:
  push:
    branches: [develop-v1.1.0]
  pull_request:
    branches: [develop-v1.1.0]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Run benchmarks
        run: cargo bench -- --save-baseline main
      
      - name: Compare with baseline
        run: |
          cargo bench -- --baseline main > bench_results.txt
          # 检查性能回归
          python scripts/check_benchmark.py bench_results.txt
      
      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results
          path: bench_results.txt
```

### 5.2 性能回归检测

```python
# scripts/check_benchmark.py
import sys
import re

def check_regression(file_path, threshold=0.1):
    """检查性能回归，阈值 10%"""
    with open(file_path) as f:
        content = f.read()
    
    # 解析基准测试结果
    # 如果性能下降超过阈值，返回错误
    # 否则返回成功
    
    return 0

if __name__ == "__main__":
    sys.exit(check_regression(sys.argv[1]))
```

---

## 六、任务分解

| ID | 任务 | 预估时间 | 优先级 |
|----|------|----------|--------|
| B-001 | 创建 benches/ 目录结构 | 1h | P0 |
| B-002 | 实现 lexer_bench.rs | 2h | P0 |
| B-003 | 实现 parser_bench.rs | 2h | P0 |
| B-004 | 实现 executor_bench.rs | 4h | P1 |
| B-005 | 实现 storage_bench.rs | 4h | P1 |
| B-006 | 实现 network_bench.rs | 4h | P2 |
| B-007 | 实现 planner_bench.rs | 2h | P1 |
| B-008 | 实现 integration_bench.rs | 4h | P2 |
| B-009 | CI 集成 | 2h | P1 |
| B-010 | 性能回归检测脚本 | 2h | P2 |

**总计**: ~27 小时

---

## 七、验收标准

- [ ] 所有基准测试可运行 (`cargo bench`)
- [ ] 生成性能报告
- [ ] CI 集成完成
- [ ] 性能回归检测生效
- [ ] 文档完整

---

*本文档由 TRAE (GLM-5.0) 创建*
