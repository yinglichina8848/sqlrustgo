# SQLRustGo v1.3.0 测试规划

> **版本**: v1.3.0  
> **代号**: Architecture Stabilization  
> **目标**: 测试覆盖率 ≥65%  
> **日期**: 2026-03-15

---

## 1. 当前状态

### 1.1 开发完成情况

| 类别 | 任务 | 状态 |
|------|------|------|
| E-xxx | Executor (7个) | ✅ 已完成 |
| T-001 | Planner 测试框架 | ✅ 已完成 |
| M-xxx | Metrics (2个) | ⏳ 待开始 |
| H-xxx | Health (2个) | ⏳ 待开始 |
| T-002 | Planner 覆盖率 | ⏳ 待开始 |
| T-003 | Optimizer 测试 | ⏳ 待开始 |

### 1.2 覆盖率现状

| 模块 | 当前覆盖率 | 目标覆盖率 | 差距 |
|------|-----------|------------|------|
| executor | 14% | 60% | 46% |
| planner | <20% | 60% | >40% |
| optimizer | <20% | 40% | >20% |
| **整体** | **14%** | **65%** | **51%** |

---

## 2. 测试需求分析

### 2.1 新增功能测试

| 模块 | 新增功能 | 测试优先级 | 预计测试数 |
|------|----------|------------|------------|
| **Planner** | 逻辑计划/物理计划转换 | P0 | 30 |
| **Optimizer** | 优化规则 | P1 | 20 |
| **Executor** | 算子实现 | P0 | 50 |
| **Metrics** | 指标收集 | P1 | 15 |
| **Health** | 健康检查 | P1 | 10 |
| **Storage** | 缓冲池指标 | P1 | 15 |

### 2.2 覆盖率目标分解

| 模块 | 当前 | 目标 | 需要新增测试 |
|------|------|------|-------------|
| executor | 14% | 60% | ~50 个 |
| planner | 15% | 60% | ~40 个 |
| optimizer | 15% | 40% | ~20 个 |
| common | 0% | 50% | ~10 个 |
| server | 0% | 40% | ~10 个 |

---

## 3. 测试策略

### 3.1 单元测试模板

```rust
// 模块级测试示例
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_function() {
        // Arrange - 准备测试数据
        let input = setup_test_data();
        
        // Act - 执行测试
        let result = function_under_test(input);
        
        // Assert - 验证结果
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_handling() {
        let input = invalid_test_data();
        let result = function_under_test(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_edge_case_empty() {
        let result = function_under_test(vec![]);
        // 边界情况处理
    }

    #[test]
    fn test_edge_case_large() {
        let result = function_under_test(large_test_data());
        // 大数据量处理
    }
}
```

### 3.2 Mock 基础设施

```rust
// Mock Storage 示例
pub struct MockStorage {
    tables: HashMap<String, Vec<Record>>,
}

impl StorageEngine for MockStorage {
    fn scan(&self, table: &str) -> SqlResult<Vec<Record>> {
        Ok(self.tables.get(table).cloned().unwrap_or_default())
    }
    // ... 其他方法
}

// Mock Executor Context
pub struct MockExecutorContext {
    pub storage: MockStorage,
    pub metrics: MockMetrics,
}
```

### 3.3 集成测试

```rust
// tests/integration_test.rs

#[test]
fn test_planner_to_executor_pipeline() {
    // Parser → Planner → Optimizer → Executor
}

#[test]
fn test_query_with_join() {
    // 多表 JOIN 查询
}

#[test]
fn test_health_endpoint() {
    // 健康检查端点测试
}
```

### 3.4 性能测试

```rust
// benches/benchmark.rs

fn benchmark_table_scan(c: &mut Criterion) {
    // 表扫描性能
}

fn benchmark_hash_join(c: &mut Criterion) {
    // Hash Join 性能
}

fn benchmark_aggregation(c: &mut Criterion) {
    // 聚合查询性能
}
```

---

## 4. 测试任务分解

### 4.1 Executor 测试 (E-007)

| 测试项 | 说明 | 测试数 |
|--------|------|--------|
| VolcanoExecutor trait | 核心接口测试 | 10 |
| SeqScanExec | 表扫描测试 | 10 |
| ProjectionExec | 投影算子测试 | 8 |
| FilterExec | 过滤算子测试 | 8 |
| HashJoinExec | Hash Join 测试 | 10 |
| 错误处理 | 异常情况测试 | 4 |

### 4.2 Planner 测试 (T-002)

| 测试项 | 说明 | 测试数 |
|--------|------|--------|
| LogicalPlan | 逻辑计划构建 | 8 |
| PhysicalPlan | 物理计划转换 | 10 |
| DefaultPlanner | 计划器测试 | 10 |
| Optimizer | 优化器测试 | 12 |

### 4.3 Optimizer 测试 (T-003)

| 测试项 | 说明 | 测试数 |
|--------|------|--------|
| PredicatePushdown | 谓词下推 | 5 |
| ProjectionPruning | 投影裁剪 | 5 |
| ConstantFolding | 常量折叠 | 5 |
| CostModel | 成本模型 | 5 |

### 4.4 Metrics 测试 (M-xxx)

| 测试项 | 说明 | 测试数 |
|--------|------|--------|
| Metrics trait | 指标接口 | 5 |
| BufferPoolMetrics | 缓冲池指标 | 10 |

### 4.5 Health 测试 (H-xxx)

| 测试项 | 说明 | 测试数 |
|--------|------|--------|
| /health/live | 存活探针 | 5 |
| /health/ready | 就绪探针 | 5 |

---

## 5. 覆盖率工具

### 5.1 安装与运行

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 运行覆盖率
cargo tarpaulin --workspace \
  --all-features \
  --out Html \
  --output-dir ./coverage

# 查看报告
open coverage/tarpaulin.html
```

### 5.2 CI 集成

```yaml
# .github/workflows/coverage.yml
- name: Test Coverage
  run: |
    cargo tarpaulin --workspace \
      --all-features \
      --out Xml \
      --output-dir ./coverage/cobertura
    # 上传到 Codecov 或显示在 PR 中
```

---

## 6. 测试里程碑

### Phase 1: 环境准备 (第1天)
- [ ] 配置覆盖率工具
- [ ] 验证基准覆盖率
- [ ] 创建 Mock 基础设施

### Phase 2: Executor 测试 (第2-4天)
- [ ] VolcanoExecutor trait 测试 (10)
- [ ] 算子测试 (30)
- [ ] 错误处理测试 (10)

### Phase 3: Planner/Optimizer 测试 (第5-7天)
- [ ] Planner 测试 (28)
- [ ] Optimizer 测试 (20)

### Phase 4: Metrics/Health 测试 (第8-9天)
- [ ] Metrics 测试 (15)
- [ ] Health 测试 (10)

### Phase 5: 覆盖率提升 (第10天)
- [ ] 分析覆盖率报告
- [ ] 补充缺失测试
- [ ] 达到目标覆盖率

---

## 7. 验收标准

### 7.1 功能测试
- [ ] cargo test --workspace 通过
- [ ] 所有新增测试通过
- [ ] cargo clippy 无警告

### 7.2 覆盖率
- [ ] 整体覆盖率 ≥65%
- [ ] executor 覆盖率 ≥60%
- [ ] planner 覆盖率 ≥60%
- [ ] optimizer 覆盖率 ≥40%

### 7.3 性能
- [ ] 表扫描 <50ms (1000行)
- [ ] Hash Join <500ms (10000行)
- [ ] 聚合查询 <200ms (10000行)

---

## 8. 相关文档

- [v1.3.0 开发计划](./DEVELOPMENT_PLAN.md)
- [v1.3.0 版本计划](./VERSION_PLAN.md)
- [v1.3.0 任务矩阵](./TASK_MATRIX.md)
- [v1.2.0 测试计划](../v1.2.0/TEST_PLAN.md)

---

**最后更新**: 2026-03-15
