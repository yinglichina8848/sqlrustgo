# 测试覆盖率模板

> **版本**: v1.3.0  
> **目标**: 覆盖率 ≥65%  
> **模板**: 可复用的测试覆盖率配置

---

## 1. 覆盖率配置

### 1.1 Cargo.toml 配置

```toml
[dev-dependencies]
tarpaulin = "0.27"

# 可选: 输出格式
[package.metadata.tarpaulin]
# 包含测试
include-tests = true
# 忽略panic
ignore-panics = true
# 忽略函数 bodies
ignore-fn-bodies = true
# 排除特定文件
exclude = [
    "benches/",
    "tests/",
    ".*test.*",
    ".*benchmark.*",
]
# 覆盖阈值
fail-on-coverage = "65"
```

### 1.2 CI 配置

```yaml
# .github/workflows/coverage.yml
name: Coverage

on:
  push:
    branches: [main, develop/*]
  pull_request:
    branches: [main]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        run: rustup install stable
        
      - name: Install cargo-tarpaulin
        run: cargo install cargo-tarpaulin
        
      - name: Run coverage
        run: |
          cargo tarpaulin --workspace \
            --all-features \
            --out Html \
            --out Xml \
            --output-dir ./coverage
            
      - name: Upload coverage report
        uses: actions/upload-artifact@v4
        with:
          name: coverage-report
          path: coverage/
          
      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/cobertura.xml
          fail_ci_if_error: false
```

---

## 2. 模块级覆盖率配置

### 2.1 Executor 模块

```bash
# Executor 单独覆盖率
cargo tarpaulin \
  --package sqlrustgo-executor \
  --all-features \
  --out Html \
  --output-dir coverage/executor

# 目标: ≥60%
```

### 2.2 Planner 模块

```bash
# Planner 单独覆盖率
cargo tarpaulin \
  --package sqlrustgo-planner \
  --all-features \
  --out Html \
  --output-dir coverage/planner

# 目标: ≥60%
```

### 2.3 Optimizer 模块

```bash
# Optimizer 单独覆盖率
cargo tarpaulin \
  --package sqlrustgo-optimizer \
  --all-features \
  --out Html \
  --output-dir coverage/optimizer

# 目标: ≥40%
```

---

## 3. 测试报告模板

### 3.1 覆盖率报告格式

```
========================================
测试覆盖率报告
========================================
版本: v1.3.0
日期: {DATE}
基准: {COMMIT_HASH}

模块统计:
----------------------------------------
| 模块       | 覆盖率 | 目标  | 状态 |
|------------|--------|-------|------|
| executor   | XX%    | 60%   | XX   |
| planner    | XX%    | 60%   | XX   |
| optimizer  | XX%    | 40%   | XX   |
| storage    | XX%    | 50%   | XX   |
| common     | XX%    | 50%   | XX   |
| server     | XX%    | 40%   | XX   |
----------------------------------------
| 整体       | XX%    | 65%   | XX   |
========================================
```

### 3.2 未覆盖代码分析模板

```bash
# 生成未覆盖代码列表
cargo tarpaulin \
  --workspace \
  --all-features \
  --output-dir coverage \
  2>&1 | grep -E "uncovered|missed" > coverage/uncovered.txt
```

**未覆盖代码分析:**

| 文件 | 函数 | 行号 | 原因 | 优先级 |
|------|------|------|------|--------|
| executor.rs | execute() | 100 | 空实现 | P0 |
| planner.rs | optimize() | 50 | 复杂逻辑 | P1 |
| ... | ... | ... | ... | ... |

---

## 4. 覆盖率提升指南

### 4.1 快速分析

```bash
# 1. 生成基准报告
cargo tarpaulin --workspace --all-features --output-dir coverage/baseline

# 2. 查看未覆盖文件
cat coverage/baseline/tarpaulin-report.txt | grep -E "0%"

# 3. 聚焦低覆盖模块
cat coverage/baseline/tarpaulin-report.txt | grep -E "executor|planner|optimizer"
```

### 4.2 提升策略

#### 优先级 1: 核心路径
- [ ] Executor execute() 方法
- [ ] Planner create_physical_plan()
- [ ] Optimizer optimize()

#### 优先级 2: 错误处理
- [ ] Result/Error 处理分支
- [ ] Option unwrap 保护
- [ ] 边界条件

#### 优先级 3: 复杂逻辑
- [ ] Hash Join 算法
- [ ] 成本计算
- [ ] 统计信息收集

### 4.3 测试补充模板

```rust
// 未覆盖代码: executor.rs:100
#[test]
fn test_execute_returns_records() {
    // Arrange
    let executor = create_test_executor();
    let context = MockContext::new();
    
    // Act
    let result = executor.execute(&context);
    
    // Assert
    assert!(result.is_ok());
    let records = result.unwrap();
    assert!(!records.is_empty());
}

// 未覆盖代码: planner.rs:50
#[test]
fn test_optimizer_handles_complex_predicate() {
    // 测试复杂谓词优化
}
```

---

## 5. 覆盖率检查脚本

### 5.1 本地检查脚本

```bash
#!/bin/bash
# scripts/check_coverage.sh

set -e

echo "========================================="
echo "检查测试覆盖率"
echo "========================================="

# 运行覆盖率
cargo tarpaulin --workspace \
  --all-features \
  --output-dir ./coverage \
  --ignore-panics \
  --timeout 600

# 提取覆盖率
COVERAGE=$(cat coverage/tarpaulin-report.txt | grep "Total" | awk '{print $1}' | tr -d '%')
TARGET=65

echo ""
echo "当前覆盖率: ${COVERAGE}%"
echo "目标覆盖率: ${TARGET}%"

if [ "$COVERAGE" -lt "$TARGET" ]; then
    echo "❌ 覆盖率未达标!"
    exit 1
else
    echo "✅ 覆盖率达标!"
fi
```

### 5.2 模块检查脚本

```bash
#!/bin/bash
# scripts/check_module_coverage.sh

MODULE=$1
TARGET=$2

echo "检查 $MODULE 模块覆盖率..."

cargo tarpaulin \
  --package "sqlrustgo-$MODULE" \
  --all-features \
  --output-dir "coverage/$MODULE"

COVERAGE=$(cat "coverage/$MODULE/tarpaulin-report.txt" | grep "Total" | awk '{print $1}' | tr -d '%')

echo "$MODULE: ${COVERAGE}% (目标: ${TARGET}%)"

if [ "$COVERAGE" -lt "$TARGET" ]; then
    exit 1
fi
```

---

## 6. 覆盖率阈值配置

### 6.1 PR 检查配置

```yaml
# .github/workflows/pr-coverage.yml
- name: Check Coverage
  run: |
    # 检查整体覆盖率
    OVERALL=$(cat coverage/tarpaulin.txt | grep "Total" | awk '{print $1}')
    
    # 检查模块覆盖率
    EXEC=$(cat coverage/executor.txt | grep "Total" | awk '{print $1}')
    PLANNER=$(cat coverage/planner.txt | grep "Total" | awk '{print $1}')
    
    # 设置阈值
    if [ "$OVERALL" -lt 65 ]; then exit 1; fi
    if [ "$EXEC" -lt 60 ]; then exit 1; fi
    if [ "$PLANNER" -lt 60 ]; then exit 1; fi
```

### 6.2 阈值配置表

| 检查项 | 阈值 | 严重性 |
|--------|------|--------|
| 整体覆盖率 | 65% | 🔴 阻断 |
| executor | 60% | 🔴 阻断 |
| planner | 60% | 🔴 阻断 |
| optimizer | 40% | 🟡 警告 |
| storage | 50% | 🟡 警告 |
| 新增代码覆盖率 | 80% | 🟢 推荐 |

---

## 7. 覆盖率趋势追踪

### 7.1 趋势图表

| 日期 | 整体 | executor | planner | optimizer |
|------|------|----------|---------|-----------|
| 2026-03-14 | 14% | 14% | <20% | <20% |
| 2026-03-15 | - | - | - | - |
| 2026-03-20 | - | - | - | - |
| 2026-03-25 | - | - | - | - |
| 2026-04-01 | 65% | 60% | 60% | 40% |

### 7.2 自动化追踪

```yaml
# GitHub Action: Coverage Trend
- name: Record Coverage
  uses: dorny/coverage-added@v1
  with:
    files: coverage/cobertura.xml
```

---

## 8. 相关文档

- [覆盖率提升计划](../coverage_improvement_plan.md)
- [v1.2.0 测试计划](../v1.2.0/TEST_PLAN.md)
- [性能测试模板](./PERFORMANCE_TEST_TEMPLATE.md)

---

**模板版本**: 1.0  
**最后更新**: 2026-03-15
