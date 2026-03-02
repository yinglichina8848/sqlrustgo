# SQLRustGo 测试覆盖率提升计划

> 版本：v1.0
> 日期：2026-03-02
> 目标：将测试覆盖率从 82.86% 提升到 90%+

---

## 一、当前覆盖率状态

| 模块 | 行覆盖率 | 状态 | 说明 |
|------|----------|------|------|
| auth/mod.rs | 98.92% | ✅ 优秀 | - |
| executor/mod.rs | 95.70% | ✅ 优秀 | - |
| lexer/*.rs | 94.85% | ✅ 优秀 | - |
| network/mod.rs | 95.84% | ✅ 优秀 | - |
| storage/*.rs | 96-100% | ✅ 优秀 | - |
| transaction/*.rs | 97%+ | ✅ 优秀 | - |
| types/*.rs | 86-100% | ⚠️ 良好 | value.rs 需提升 |
| parser/mod.rs | 88.18% | ⚠️ 良好 | 需提升 |
| **planner/*.rs** | **0%** | ❌ 缺失 | **需添加测试** |
| main.rs | 0% | ❌ 缺失 | 入口文件 |
| **总计** | **82.86%** | ⚠️ 需提升 | 目标 90%+ |

---

## 二、问题分析

### 2.1 主要问题

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          覆盖率问题分析                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   planner 模块 (0% 覆盖) - 最大问题                                         │
│   ├── logical_plan.rs: 71 行未覆盖                                          │
│   ├── physical_plan.rs: 504 行未覆盖                                        │
│   ├── executor.rs: 50 行未覆盖                                              │
│   └── mod.rs: 118 行未覆盖                                                  │
│                                                                              │
│   其他需提升模块                                                             │
│   ├── parser/mod.rs: 78 行未覆盖 (88.18%)                                   │
│   ├── types/value.rs: 12 行未覆盖 (86.52%)                                  │
│   └── main.rs: 83 行未覆盖 (0%)                                             │
│                                                                              │
│   影响: planner 模块 0% 覆盖拉低整体覆盖率约 10%                             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 覆盖率计算

```
当前: 6335 行总数, 1086 行未覆盖 = 82.86%
目标: 90% 覆盖 = 最多 633 行未覆盖
需要: 减少 453 行未覆盖代码

主要目标: planner 模块 (743 行)
如果 planner 达到 80% 覆盖: 减少 ~594 行未覆盖
整体覆盖率将提升到: ~92%
```

---

## 三、提升计划

### Phase 1: planner 模块测试 (优先级最高)

| 任务 | 文件 | 目标覆盖率 | 预估时间 |
|------|------|------------|----------|
| COV-001 | planner/mod.rs | 80% | 2h |
| COV-002 | planner/logical_plan.rs | 85% | 3h |
| COV-003 | planner/physical_plan.rs | 80% | 4h |
| COV-004 | planner/executor.rs | 85% | 2h |

### Phase 2: 其他模块提升

| 任务 | 文件 | 当前 | 目标 | 预估时间 |
|------|------|------|------|----------|
| COV-005 | parser/mod.rs | 88% | 95% | 2h |
| COV-006 | types/value.rs | 86% | 95% | 1h |

### Phase 3: CI 集成

| 任务 | 说明 | 预估时间 |
|------|------|----------|
| COV-007 | 添加覆盖率 CI 检查 | 1h |
| COV-008 | 设置覆盖率门槛 (90%) | 0.5h |
| COV-009 | PR 覆盖率报告 | 0.5h |

---

## 四、测试实现

### 4.1 planner/mod.rs 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_planner_module_exists() {
        // 测试模块导出
    }
    
    #[test]
    fn test_logical_plan_creation() {
        // 测试逻辑计划创建
    }
    
    #[test]
    fn test_physical_plan_creation() {
        // 测试物理计划创建
    }
}
```

### 4.2 planner/logical_plan.rs 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_logical_plan_projection() {
        let plan = LogicalPlan::Projection {
            input: Box::new(LogicalPlan::TableScan {
                table_name: "users".to_string(),
                projection: None,
            }),
            expr: vec![Expr::Column("id".to_string())],
        };
        assert!(matches!(plan, LogicalPlan::Projection { .. }));
    }
    
    #[test]
    fn test_logical_plan_filter() {
        // 测试 Filter 节点
    }
    
    #[test]
    fn test_logical_plan_join() {
        // 测试 Join 节点
    }
    
    #[test]
    fn test_logical_plan_aggregate() {
        // 测试 Aggregate 节点
    }
    
    #[test]
    fn test_logical_plan_display() {
        // 测试 Display trait
    }
}
```

### 4.3 planner/physical_plan.rs 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_physical_plan_trait() {
        // 测试 PhysicalPlan trait
    }
    
    #[test]
    fn test_seq_scan_exec() {
        // 测试顺序扫描
    }
    
    #[test]
    fn test_projection_exec() {
        // 测试投影
    }
    
    #[test]
    fn test_filter_exec() {
        // 测试过滤
    }
    
    #[test]
    fn test_hash_join_exec() {
        // 测试 HashJoin
    }
    
    #[test]
    fn test_aggregate_exec() {
        // 测试聚合
    }
    
    #[test]
    fn test_sort_exec() {
        // 测试排序
    }
    
    #[test]
    fn test_limit_exec() {
        // 测试 Limit
    }
}
```

### 4.4 planner/executor.rs 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_execution_engine_trait() {
        // 测试 ExecutionEngine trait
    }
    
    #[test]
    fn test_engine_registry() {
        // 测试引擎注册
    }
    
    #[test]
    fn test_default_executor() {
        // 测试默认执行器
    }
}
```

---

## 五、CI 集成

### 5.1 GitHub Actions 配置

```yaml
# .github/workflows/coverage.yml
name: Coverage

on:
  push:
    branches: [develop-v1.1.0]
  pull_request:
    branches: [develop-v1.1.0]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      
      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov
      
      - name: Generate coverage
        run: cargo llvm-cov --all-features --lcov --output-path lcov.info
      
      - name: Check coverage threshold
        run: |
          COVERAGE=$(cargo llvm-cov --all-features --summary-only 2>&1 | grep TOTAL | awk '{print $4}' | sed 's/%//')
          echo "Coverage: $COVERAGE%"
          if (( $(echo "$COVERAGE < 90" | bc -l) )); then
            echo "Coverage below 90% threshold"
            exit 1
          fi
      
      - name: Upload to codecov
        uses: codecov/codecov-action@v4
        with:
          files: lcov.info
```

### 5.2 PR 覆盖率检查

```yaml
# 在 PR 中显示覆盖率变化
- name: Report coverage
  run: |
    echo "## Coverage Report" >> $GITHUB_STEP_SUMMARY
    cargo llvm-cov --all-features --summary-only 2>&1 | tail -5 >> $GITHUB_STEP_SUMMARY
```

---

## 六、任务分解

| ID | 任务 | 优先级 | 预估时间 |
|----|------|--------|----------|
| COV-001 | planner/mod.rs 测试 | P0 | 2h |
| COV-002 | planner/logical_plan.rs 测试 | P0 | 3h |
| COV-003 | planner/physical_plan.rs 测试 | P0 | 4h |
| COV-004 | planner/executor.rs 测试 | P0 | 2h |
| COV-005 | parser/mod.rs 测试补充 | P1 | 2h |
| COV-006 | types/value.rs 测试补充 | P1 | 1h |
| COV-007 | 覆盖率 CI 集成 | P1 | 1h |
| COV-008 | 设置覆盖率门槛 | P2 | 0.5h |
| COV-009 | PR 覆盖率报告 | P2 | 0.5h |

**总计**: ~16 小时

---

## 七、验收标准

- [ ] 整体覆盖率 >= 90%
- [ ] planner 模块覆盖率 >= 80%
- [ ] parser 模块覆盖率 >= 95%
- [ ] CI 覆盖率检查通过
- [ ] PR 覆盖率报告正常

---

## 八、覆盖率目标

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          覆盖率目标                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   当前: 82.86%                                                              │
│   目标: 90.00%                                                              │
│   差距: 7.14% (约 453 行)                                                   │
│                                                                              │
│   预期结果:                                                                  │
│   ├── planner 模块: 0% → 80% (+743 行覆盖)                                  │
│   ├── parser 模块: 88% → 95% (+46 行覆盖)                                   │
│   ├── types 模块: 86% → 95% (+8 行覆盖)                                     │
│   └── 整体: 82.86% → 92%+                                                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

*本文档由 TRAE (GLM-5.0) 创建*
