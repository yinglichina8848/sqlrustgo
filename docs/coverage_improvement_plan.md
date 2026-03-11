# 测试覆盖率提升计划 (Test Coverage Improvement Plan)

## 1. 概述

### 1.1 目标
将测试覆盖率从当前 **66.20%** 提升至 **≥80%**

### 1.2 当前状态
- 总行数: 3021
- 覆盖行数: 2000
- 覆盖率: 66.20%

### 1.3 目标模块

| 模块 | 当前 | 目标 | 差距 |
|------|------|------|------|
| planner/physical_plan.rs | 23% | 80% | +57% |
| planner/optimizer.rs | 22% | 80% | +58% |
| optimizer/rules.rs | 42% | 80% | +38% |
| executor/local_executor.rs | 63% | 80% | +17% |
| parser/parser.rs | 72% | 80% | +8% |
| storage/file_storage.rs | 78% | 80% | +2% |
| storage/page.rs | 78% | 80% | +2% |

---

## 2. 执行流程

### 2.1 环境准备

```bash
# 确保代码最新
git fetch origin
git checkout develop/v1.2.0
git pull origin develop/v1.2.0

# 清理之前的覆盖率报告
rm -rf target/tarpaulin/
```

### 2.2 基准测试 (Baseline)

```bash
# 运行完整测试套件
cargo test --workspace

# 生成覆盖率报告
cargo tarpaulin --workspace \
  --output-dir target/tarpaulin/baseline \
  --ignore-panics \
  --timeout 300

# 提取基准数据
cat target/tarpaulin/baseline/tarpaulin-report.txt | grep -E "coverage|Tested/Total"
```

**预期输出示例:**
```
66.20% coverage, 2000/3021 lines covered
```

### 2.3 分模块测试策略

#### 模块 1: planner/physical_plan.rs (23% → 80%)

**未覆盖代码分析:**
```bash
# 查看未覆盖的行
cargo tarpaulin --workspace \
  --output-dir target/tarpaulin/physical_plan \
  --ignore-panics \
  --timeout 300 \
  2>&1 | grep "physical_plan.rs"
```

**需要测试的函数:**
1ScanExec::execute. `Seq()` - 空实现，需模拟数据
2. `ProjectionExec::evaluate_expr()` - 支持更多表达式类型
3. `FilterExec::evaluate_predicate()` - 完善谓词评估
4. `AggregateExec::compute_aggregate()` - 完善聚合计算
5. `HashJoinExec::execute()` - JOIN 执行逻辑

**测试命令:**
```bash
cargo test --package sqlrustgo-planner physical_plan --nocapture
```

#### 模块 2: planner/optimizer.rs (22% → 80%)

**需要测试的函数:**
1. `Optimizer` trait 实现
2. `RuleSet` 相关方法
3. `CostModel` 接口

**测试命令:**
```bash
cargo test --package sqlrustgo-planner optimizer --nocapture
```

#### 模块 3: optimizer/rules.rs (42% → 80%)

**需要测试的函数:**
1. `PredicatePushdown::apply()` - 谓词下推逻辑
2. `ProjectionPruning::apply()` - 投影裁剪逻辑
3. `ConstantFolding::apply()` - 常量折叠逻辑
4. `ExpressionSimplification` 规则
5. `JoinReordering` 规则

**测试命令:**
```bash
cargo test --package sqlrustgo-optimizer rules --nocapture
```

#### 模块 4: executor/local_executor.rs (63% → 80%)

**需要测试的函数:**
1. `LocalExecutor::execute()` - 各执行器分支
2. `LocalExecutor::execute_seq_scan()`
3. `LocalExecutor::execute_filter()`
4. `LocalExecutor::execute_projection()`
5. `LocalExecutor::execute_aggregate()`

**测试命令:**
```bash
cargo test --package sqlrustgo-executor --nocapture
```

#### 模块 5-7: storage 模块 (78% → 80%)

**测试命令:**
```bash
cargo test --package sqlrustgo-storage --nocapture
```

### 2.4 增量验证

每完成一个模块后，运行增量测试:

```bash
# 增量覆盖率测试
cargo tarpaulin --workspace \
  --output-dir target/tarpaulin/increment_$(date +%Y%m%d_%H%M%S) \
  --ignore-panics \
  --timeout 300

# 对比结果
diff target/tarpaulin/baseline/tarpaulin-report.txt \
     target/tarpaulin/increment_*/tarpaulin-report.txt
```

---

## 3. 测试用例设计指南

### 3.1 测试结构

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_name() {
        // Arrange - 准备测试数据
        let input = setup_test_data();
        
        // Act - 执行被测代码
        let result = function_under_test(input);
        
        // Assert - 验证结果
        assert_eq!(result, expected_value);
    }
}
```

### 3.2 物理计划测试模板

```rust
#[test]
fn test_projection_exec_with_expression() {
    // 测试表达式投影
    let schema = Schema::new(vec![
        Field::new("a".to_string(), DataType::Integer),
        Field::new("b".to_string(), DataType::Integer),
    ]);
    let child = SeqScanExec::new("test".to_string(), schema.clone());
    
    // 测试 a + b 表达式
    let expr = Expr::BinaryExpr {
        left: Box::new(Expr::column("a")),
        op: Operator::Plus,
        right: Box::new(Expr::column("b")),
    };
    
    let exec = ProjectionExec::new(Box::new(child), vec![expr], schema);
    let result = exec.execute().unwrap();
    
    assert!(!result.is_empty());
}
```

---

## 4. 报告模板

### 4.1 中间报告 (每次提交后)

```markdown
## 覆盖率进度报告 - YYYY-MM-DD

### 总体进度
| 指标 | 基准 | 当前 | 变化 |
|------|------|------|------|
| 总覆盖率 | 66.20% | XX.XX% | ±X.XX% |

### 模块进度
| 模块 | 基准 | 当前 | 目标 | 状态 |
|------|------|------|------|------|
| planner/physical_plan.rs | 23% | XX% | 80% | 进行中 |
| ... | ... | ... | ... | ... |

### 本次变更
- 新增测试: XX 个
- 覆盖行数变化: +XX / -XX

### 阻塞问题
1. [问题描述]
```

### 4.2 最终报告

```markdown
## 覆盖率测试最终报告

### 执行摘要
- 开始日期: YYYY-MM-DD
- 结束日期: YYYY-MM-DD
- 总耗时: X 天
- 目标: ≥80%

### 测试统计
- 新增测试用例: XX 个
- 修改测试用例: XX 个
- 总测试数: XXX 个

### 覆盖率变化

| 模块 | 基准 | 最终 | 变化 |
|------|------|------|------|
| 总计 | 66.20% | XX.XX% | +XX.XX% |

### 各模块详情

| 模块 | 基准 | 最终 | 目标 | 状态 |
|------|------|------|------|------|
| planner/physical_plan.rs | 23% | XX% | 80% | ✅/❌ |
| planner/optimizer.rs | 22% | XX% | 80% | ✅/❌ |
| optimizer/rules.rs | 42% | XX% | 80% | ✅/❌ |
| executor/local_executor.rs | 63% | XX% | 80% | ✅/❌ |
| parser/parser.rs | 72% | XX% | 80% | ✅/❌ |
| storage/file_storage.rs | 78% | XX% | 80% | ✅/❌ |
| storage/page.rs | 78% | XX% | 80% | ✅/❌ |

### 测试命令记录
```bash
# 完整测试
cargo test --workspace

# 覆盖率测试
cargo tarpaulin --workspace --output-dir target/tarpaulin --ignore-panics --timeout 300
```

### 结论
- [ ] 达到目标 (覆盖率 ≥80%)
- [ ] 未达到目标 (覆盖率 XX%)
```

---

## 5. 验证命令清单

### 5.1 基础验证

```bash
# 1. 编译检查
cargo build --workspace                    # Debug
cargo build --release --workspace          # Release
cargo build --all-features --workspace     # 全特性

# 2. 代码规范
cargo clippy -- -D warnings               # Clippy
cargo fmt --check                         # 格式化
cargo doc --no-deps                       # 文档

# 3. 测试验证
cargo test --workspace                    # 所有测试
```

### 5.2 覆盖率验证

```bash
# 生成覆盖率报告
cargo tarpaulin \
  --workspace \
  --output-dir target/tarpaulin/final \
  --ignore-panics \
  --timeout 300

# 提取总覆盖率
cat target/tarpaulin/final/tarpaulin-report.txt | grep "coverage"

# 提取各模块覆盖率
cat target/tarpaulin/final/tarpaulin-report.txt | grep "Tested/Total"
```

### 5.3 增量对比

```bash
# 对比基准和最终结果
echo "=== 覆盖率对比 ==="
echo "基准: 66.20% (2000/3021)"
echo "最终: $(cat target/tarpaulin/final/tarpaulin-report.txt | grep 'coverage' | awk '{print $1}')"
```

---

## 6. 时间估算

| 模块 | 当前 | 目标 | 预计工作量 |
|------|------|------|-----------|
| planner/physical_plan.rs | 23% | 80% | 4-6 小时 |
| planner/optimizer.rs | 22% | 80% | 4-6 小时 |
| optimizer/rules.rs | 42% | 80% | 2-3 小时 |
| executor/local_executor.rs | 63% | 80% | 1-2 小时 |
| parser/parser.rs | 72% | 80% | 1 小时 |
| storage 模块 | 78% | 80% | 1 小时 |

**总计: 约 13-19 小时**

---

## 7. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 测试数据准备复杂 | 高 | 使用 mock/桩数据 |
| 部分代码难以测试 | 中 | 集成测试补充 |
| 回归问题 | 中 | 频繁运行全量测试 |

---

## 8. 执行检查点

- [ ] 基准测试完成
- [ ] 模块 1 (physical_plan.rs) 达标
- [ ] 模块 2 (optimizer.rs) 达标
- [ ] 模块 3 (rules.rs) 达标
- [ ] 模块 4 (local_executor.rs) 达标
- [ ] 模块 5-7 (storage) 达标
- [ ] 最终覆盖率验证
- [ ] 所有测试通过
- [ ] 无编译警告
