# 覆盖率提升计划

> 从执行链路分析出发，制定覆盖率提升策略

## 1. 覆盖率现状分析

### 1.1 当前覆盖率数据

根据 `COVERAGE_REPORT.md`:

| 模块 | 当前覆盖率 | 目标覆盖率 | 差距 |
|------|------------|------------|------|
| executor | ~70% | 85% | 15% |
| storage | ~70% | 85% | 15% |
| parser | ~85% | 90% | 5% |
| transaction | ~80% | 85% | 5% |
| optimizer | ~60% | 80% | 20% |
| planner | ~65% | 80% | 15% |

### 1.2 覆盖率低的原因分析

```
┌─────────────────────────────────────────────────────────────┐
│                    覆盖率低的原因                             │
├─────────────────────────────────────────────────────────────┤
│  1. 执行链路不清晰                                          │
│     - 缺少完整的算法时序图                                  │
│     - 缺少状态机转换图                                      │
│     - 缺少活动图描述                                       │
│                                                              │
│  2. 边界条件不明确                                         │
│     - NULL 处理路径未覆盖                                   │
│     - 异常情况处理未覆盖                                    │
│     - 类型转换边界未覆盖                                    │
│                                                              │
│  3. 集成路径缺失                                           │
│     - 模块间接口调用未覆盖                                  │
│     - 错误传播路径未覆盖                                    │
│     - 资源清理路径未覆盖                                    │
└─────────────────────────────────────────────────────────────┘
```

## 2. 执行链路分析

### 2.1 SELECT 完整执行链路

```
SELECT * FROM t1 JOIN t2 ON t1.id = t2.id WHERE t1.a > 10 ORDER BY t1.b LIMIT 100
    │
    ▼
┌─────────────────┐
│     Parser      │ → SQL → AST
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Planner      │ → Logical Plan
│  ┌───────────┐  │
│  │  Binder   │  │ → 语义分析，绑定表/列
│  └───────────┘  │
│  ┌───────────┐  │
│  │ Resolver  │  │ → 类型解析，函数解析
│  └───────────┘  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Optimizer     │ → Physical Plan (CBO)
│  ┌───────────┐  │
│  │ CostModel │  │ → 代价估算
│  └───────────┘  │
│  ┌───────────┐  │
│  │JoinOrder  │  │ → Join 次序优化
│  └───────────┘  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Executor      │ → Volcano Iterator Model
│  ┌───────────┐  │
│  │  HashJoin │  │ → Hash Join 执行
│  └───────────┘  │
│  ┌───────────┐  │
│  │  Filter   │  │ → t1.a > 10 过滤
│  └───────────┘  │
│  ┌───────────┐  │
│  │   Sort    │  │ → ORDER BY t1.b
│  └───────────┘  │
│  ┌───────────┐  │
│  │   Limit   │  │ → LIMIT 100
│  └───────────┘  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Storage      │
│  ┌───────────┐  │
│  │ B+Tree    │  │ → 索引查找
│  └───────────┘  │
│  ┌───────────┐  │
│  │BufferPool │  │ → 页面缓冲
│  └───────────┘  │
│  ┌───────────┐  │
│  │ FileStore │  │ → 磁盘读写
│  └───────────┘  │
└─────────────────┘
```

### 2.2 链路关键节点覆盖率要求

| 节点 | 路径数 | 已覆盖 | 需覆盖 |
|------|--------|--------|--------|
| Parser::parse_select | 50+ | 35 | 15 |
| Planner::plan_join | 20+ | 12 | 8 |
| Optimizer::optimize | 30+ | 18 | 12 |
| Executor::hash_join | 15+ | 10 | 5 |
| B+Tree::insert | 25+ | 18 | 7 |
| B+Tree::delete | 20+ | 12 | 8 |
| Transaction::commit | 10+ | 8 | 2 |

## 3. 覆盖率提升计划

### 3.1 阶段一: 链路梳理 (Week 1-2)

**目标**: 补全缺失的执行链路文档

- [ ] 完成 CBO 执行链路文档
- [ ] 完成 B+Tree 操作链路文档
- [ ] 完成 Transaction 提交链路文档
- [ ] 完成 Executor 核心算子链路文档

**产出**: 完整链路图 (时序图、状态图、活动图)

### 3.2 阶段二: 边界覆盖 (Week 3-4)

**目标**: 覆盖边界条件和异常路径

- [ ] NULL 值处理路径测试
- [ ] 类型转换边界测试
- [ ] 资源清理路径测试
- [ ] 错误传播路径测试

**测试用例模板**:
```rust
#[test]
fn test_null_handling_in_join() {
    // NULL = NULL 语义测试
    // NULL IN (1,2,3) 语义测试
    // NULL > 10 语义测试
}

#[test]
fn test_type_conversion_boundary() {
    // i64::MAX + 1 溢出测试
    // 浮点精度边界测试
    // 字符串编码边界测试
}
```

### 3.3 阶段三: 集成覆盖 (Week 5-6)

**目标**: 覆盖模块间调用路径

- [ ] Parser → Planner 接口测试
- [ ] Planner → Optimizer 接口测试
- [ ] Optimizer → Executor 接口测试
- [ ] Executor → Storage 接口测试

**测试策略**:
```rust
#[test]
fn test_parser_planner_integration() {
    let sql = "SELECT * FROM t WHERE a = 1";
    let ast = parse(sql).unwrap();
    let plan = planner.create_logical_plan(ast).unwrap();
    assert!(plan.is_valid());
}

#[test]
fn test_optimizer_executor_integration() {
    let plan = create_test_plan();
    let optimized = optimizer.optimize(plan).unwrap();
    let results = executor.execute(optimized).unwrap();
    assert!(verify_results(results));
}
```

### 3.4 阶段四: 性能覆盖 (Week 7-8)

**目标**: 性能关键路径覆盖

- [ ] B+Tree 大数据量测试
- [ ] Join 大数据量测试
- [ ] 并发压力测试
- [ ] 内存限制测试

## 4. 覆盖率提升技术

### 4.1 基于链路的测试生成

```rust
// 从执行链路自动生成测试用例
struct ExecutionPath {
    nodes: Vec<Node>,
    conditions: Vec<Condition>,
    expected_results: Vec<Result>,
}

fn generate_test_cases(path: &ExecutionPath) -> Vec<TestCase> {
    // 1. 覆盖所有分支
    let branch_coverage = generate_branch_tests(path);

    // 2. 覆盖边界值
    let boundary_coverage = generate_boundary_tests(path);

    // 3. 覆盖错误路径
    let error_coverage = generate_error_tests(path);

    merge(branch_coverage, boundary_coverage, error_coverage)
}
```

### 4.2 差异化覆盖率策略

| 区域 | 覆盖率目标 | 测试策略 |
|------|------------|----------|
| 核心算法 | 95%+ | 白盒测试 + 边界值 |
| 边界处理 | 90%+ | 等价类划分 + 边界值 |
| 错误处理 | 80%+ | 异常注入测试 |
| 性能关键 | 90%+ | 压力测试 |
| 辅助功能 | 70%+ | 基本功能测试 |

### 4.3 自动化测试生成

```bash
# 使用 cargo-llvm-cov 生成覆盖率报告
cargo llvm-cov --html --open

# 生成差异化覆盖率
cargo test --all-features
cargo llvm-cov --diff

# 跟踪覆盖率变化
cargo llvm-cov --prev-session ./cov baseline
```

## 5. 验证清单

### 5.1 覆盖率验证

```bash
# 检查覆盖率
cargo test --all-features
cargo llvm-cov report --show-missing

# 覆盖率目标
- executor: >= 85%
- storage: >= 85%
- optimizer: >= 80%
- planner: >= 80%
```

### 5.2 链路验证

- [ ] 每条链路有对应的测试用例
- [ ] 每个边界条件有测试覆盖
- [ ] 每个错误路径有测试覆盖

### 5.3 质量验证

- [ ] 新增测试全部通过
- [ ] 回归测试全部通过
- [ ] 性能测试达标
