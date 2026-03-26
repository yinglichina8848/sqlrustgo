# SQLRustGo 回归测试框架与系统设计

> **版本**: v1.9.0
> **日期**: 2026-03-26

---

## 1. 当前测试状况分析

### 1.1 测试统计

| 类别 | 数量 | 说明 |
|------|------|------|
| 单元测试 | 12 | 核心模块单元测试 |
| 集成测试 | 14 | 模块间集成测试 |
| 异常测试 | 15 | 边界场景、错误处理 |
| 压力测试 | 4 | 并发、长时间运行 |
| E2E 测试 | 3 | 端到端场景 |
| CI 测试 | 3 | CI 专用测试 |
| **总计** | **51** | |

### 1.2 测试目录结构

```
tests/
├── unit/                 # 单元测试
│   ├── bplus_tree_test.rs
│   ├── buffer_pool_test.rs
│   ├── optimizer_rules_test.rs
│   ├── parser_token_test.rs
│   └── ...
├── integration/         # 集成测试
│   ├── executor_test.rs
│   ├── planner_test.rs
│   ├── storage_integration_test.rs
│   └── ...
├── anomaly/            # 异常测试
│   ├── catalog_consistency_test.rs
│   ├── transaction_isolation_test.rs
│   ├── boundary_test.rs
│   └── ...
├── stress/             # 压力测试
│   ├── stress_test.rs
│   ├── crash_recovery_test.rs
│   ├── concurrency_stress_test.rs
│   └── production_scenario_test.rs
├── e2e/                # 端到端测试
│   ├── e2e_query_test.rs
│   ├── observability_test.rs
│   └── monitoring_test.rs
└── ci/                 # CI 专用测试
    ├── ci_test.rs
    └── buffer_pool_test.rs
```

### 1.3 覆盖模块

| 模块 | 单元测试 | 集成测试 | 压力测试 | E2E |
|------|----------|----------|----------|-----|
| Parser | ✅ | ✅ | - | - |
| Planner | ✅ | ✅ | - | - |
| Optimizer | ✅ | ✅ | - | - |
| Executor | ✅ | ✅ | ✅ | ✅ |
| Storage | ✅ | ✅ | ✅ | ✅ |
| Transaction | ✅ | ✅ | ✅ | - |
| Server | - | ✅ | - | ✅ |

### 1.4 现有能力

| 能力 | 状态 | 说明 |
|------|------|------|
| 单元测试 | ✅ 完善 | 各模块独立测试 |
| 集成测试 | ✅ 完善 | 模块协作测试 |
| 异常测试 | ✅ 完善 | 边界场景覆盖 |
| 压力测试 | ✅ 基础 | 并发测试 |
| 回归测试 | ⚠️ 缺失 | 需要建立 |
| 性能基准 | ⚠️ 缺失 | 需要完善 |
| 混沌测试 | ⚠️ 缺失 | 需要建立 |
| 模糊测试 | ⚠️ 缺失 | 需要完善 |

---

## 2. 回归测试框架设计

### 2.1 框架架构

```uml
@startuml

package "Regression Test Framework" {
  
  [Test Runner] as TR
  [Test Registry] as REG
  [Test Database] as TD
  [Result Collector] as RC
  [Report Generator] as RG
  
  package "Test Categories" {
    [Unit Tests] as UT
    [Integration Tests] as IT
    [Anomaly Tests] as AT
    [Stress Tests] as ST
    [E2E Tests] as E2E
  }
  
  package "CI/CD Integration" {
    [GitHub Actions] as GA
    [Coverage Report] as CR
    [Performance Report] as PR
  }
  
  TR --> REG
  TR --> TD
  TR --> UT
  TR --> IT
  TR --> AT
  TR --> ST
  TR --> E2E
  TR --> RC
  RC --> RG
  RG --> GA
  GA --> CR
  GA --> PR
}

@enduml
```

### 2.2 核心组件

```rust
// 回归测试运行器
pub struct RegressionTestRunner {
    config: TestConfig,
    registry: TestRegistry,
    database: TestDatabase,
    collector: ResultCollector,
}

impl RegressionTestRunner {
    // 运行所有回归测试
    pub fn run_all(&self) -> RegressionReport;
    
    // 运行特定类别
    pub fn run_category(&self, category: TestCategory) -> CategoryReport;
    
    // 运行特定模块
    pub fn run_module(&self, module: &str) -> ModuleReport;
    
    // 运行增量测试 (仅运行修改影响的测试)
    pub fn run_incremental(&self, changed_files: &[PathBuf]) -> IncrementalReport;
    
    // 生成报告
    pub fn generate_report(&self) -> TestReport;
}
```

### 2.3 测试注册表

```rust
// 测试元数据
pub struct TestMetadata {
    pub id: String,
    pub name: String,
    pub category: TestCategory,
    pub module: String,
    pub tags: Vec<String>,
    pub priority: TestPriority,
    pub timeout: Duration,
    pub flaky: bool,
}

// 测试注册表
pub struct TestRegistry {
    tests: HashMap<String, TestMetadata>,
}

impl TestRegistry {
    // 注册测试
    pub fn register(&mut self, metadata: TestMetadata);
    
    // 按类别查询
    pub fn query_by_category(&self, category: TestCategory) -> Vec<&TestMetadata>;
    
    // 按模块查询
    pub fn query_by_module(&self, module: &str) -> Vec<&TestMetadata>;
    
    // 按标签查询
    pub fn query_by_tag(&self, tag: &str) -> Vec<&TestMetadata>;
}
```

---

## 3. 增量回归测试

### 3.1 增量测试策略

```uml
@startuml

actor Developer

Developer -> GitHub: Push Code

GitHub -> CI: Trigger Pipeline

CI -> IncrementalRunner: Run Incremental Tests

IncrementalRunner -> Git: Get Changed Files

Git --> IncrementalRunner: Files Changed

IncrementalRunner -> TestRegistry: Query Affected Tests

TestRegistry --> IncrementalRunner: Test List

IncrementalRunner -> Executor: Run Tests

Executor --> IncrementalRunner: Results

IncrementalRunner -> Report: Generate Report

Report --> Developer: Test Results

@enduml
```

### 3.2 影响分析算法

```rust
// 模块依赖图
pub struct DependencyGraph {
    edges: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    // 分析变更影响
    pub fn analyze_impact(&self, changed_files: &[PathBuf]) -> Vec<String> {
        let mut affected_modules = Vec::new();
        
        for file in changed_files {
            let module = self.file_to_module(file);
            affected_modules.extend(self.get_affected_modules(&module));
        }
        
        affected_modules
    }
    
    // 获取受影响的测试
    pub fn get_affected_tests(&self, modules: &[String]) -> Vec<TestMetadata> {
        self.tests
            .iter()
            .filter(|t| modules.contains(&t.module))
            .cloned()
            .collect()
    }
}
```

---

## 4. 测试数据管理

### 4.1 测试数据库

```rust
// 测试数据管理器
pub struct TestDataManager {
    // 预设数据集
    datasets: HashMap<String, Dataset>,
    
    // 数据生成器
    generators: HashMap<String, DataGenerator>,
}

impl TestDataManager {
    // 加载预设数据
    pub fn load_dataset(&self, name: &str) -> Dataset;
    
    // 生成测试数据
    pub fn generate(&self, config: DataConfig) -> Dataset;
    
    // 清理测试数据
    pub fn cleanup(&self);
}
```

### 4.2 数据集定义

```rust
// 预设数据集
pub struct Dataset {
    pub name: String,
    pub schema: Schema,
    pub rows: Vec<Row>,
    pub description: &str,
}

// 常用数据集
pub struct StandardDatasets {
    pub users: Dataset,          // 用户表 (1000行)
    pub orders: Dataset,        // 订单表 (10000行)
    pub products: Dataset,      // 产品表 (500行)
    pub large_table: Dataset,    // 大表 (100000行)
    pub empty_table: Dataset,   // 空表
}
```

---

## 5. 测试优先级与分类

### 5.1 测试优先级

| 优先级 | 说明 | 运行频率 | 超时 |
|--------|------|----------|------|
| P0 | 核心功能回归 | 每次提交 | 30s |
| P1 | 重要功能回归 | 每次提交 | 60s |
| P2 | 次要功能回归 | 每日 | 120s |
| P3 | 边界/压力测试 | 每周 | 300s |
| P4 | 长期稳定性 | 发布前 | 3600s |

### 5.2 测试分类标签

```rust
pub struct TestTags {
    // 功能标签
    pub static: Vec<&'static str>,  // "parser", "planner", "optimizer"
    
    // 场景标签
    pub scenario: Vec<&'static str>, // "select", "insert", "join", "transaction"
    
    // 特性标签
    pub feature: Vec<&'static str>,  // "index", "mvcc", "wal"
    
    // 性能标签
    pub performance: Vec<&'static str>, // "slow", "memory", "concurrent"
}
```

---

## 6. 测试结果收集与分析

### 6.1 结果收集器

```rust
pub struct ResultCollector {
    results: Vec<TestResult>,
    start_time: Instant,
}

pub struct TestResult {
    pub test_id: String,
    pub status: TestStatus,  // Pass, Fail, Skip, Panic
    pub duration: Duration,
    pub memory_used: Option<u64>,
    pub output: String,
    pub stack_trace: Option<String>,
}

impl ResultCollector {
    pub fn record(&mut self, result: TestResult);
    pub fn summary(&self) -> TestSummary;
    pub fn failures(&self) -> Vec<&TestResult>;
    pub fn statistics(&self) -> TestStatistics;
}
```

### 6.2 统计分析

```rust
pub struct TestStatistics {
    pub total: u64,
    pub passed: u64,
    pub failed: u64,
    pub skipped: u64,
    pub flaky: u64,
    pub pass_rate: f64,
    pub avg_duration: Duration,
    pub slowest_tests: Vec<(String, Duration)>,
    pub most_flaky_tests: Vec<(String, f64)>,  // name, failure rate
}
```

---

## 7. 测试报告系统

### 7.1 报告生成器

```rust
pub struct ReportGenerator {
    template: ReportTemplate,
}

impl ReportGenerator {
    // 生成 HTML 报告
    pub fn generate_html(&self, results: &TestSummary) -> String;
    
    // 生成 JSON 报告 (供 CI 使用)
    pub fn generate_json(&self, results: &TestSummary) -> String;
    
    // 生成 Markdown 报告
    pub fn generate_markdown(&self, results: &TestSummary) -> String;
    
    // 生成性能趋势图
    pub fn generate_performance_trend(&self, history: &[TestSummary]) -> Chart;
}
```

### 7.2 报告内容

```markdown
# 回归测试报告

## 执行摘要
- 总测试数: 500
- 通过: 495 (99.0%)
- 失败: 3 (0.6%)
- 跳过: 2 (0.4%)
- 执行时间: 5m 30s

## 失败测试
| 测试名 | 模块 | 错误 | 持续时间 |
|--------|------|------|----------|
| test_join_100k | executor | OutOfMemory | 120s |

## 性能趋势
![Performance Trend](trend.png)

## 建议
- 建议关注 executor 模块的内存使用
- 建议增加索引相关测试覆盖率
```

---

## 8. CI/CD 集成

### 8.1 GitHub Actions 工作流

```yaml
name: Regression Tests

on:
  push:
    branches: [main, develop/**]
  pull_request:
    branches: [main, develop/**]

jobs:
  regression:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run Unit Tests
        run: cargo test --lib --workspace
        
      - name: Run Integration Tests
        run: cargo test --test integration --test anomaly
        
      - name: Run Stress Tests (nightly)
        if: github.event_name == 'schedule'
        run: cargo test --test stress
        
      - name: Upload Coverage
        uses: codecov/codecov-action@v3
        
      - name: Generate Report
        uses: dorny/test-reporter@v1
```

### 8.2 测试分组

```bash
#!/bin/bash
# run_regression.sh

# P0: 快速回归 (5分钟内)
cargo test --lib --workspace -- --test-threads=4

# P1: 标准回归 (15分钟内)  
cargo test --test unit --test integration

# P2: 完整回归 (30分钟内)
cargo test --test unit --test integration --test anomaly

# P3: 压力回归 (60分钟内)
cargo test --test stress

# P4: 夜间回归
cargo test --workspace
```

---

## 9. 监控与告警

### 9.1 测试监控指标

| 指标 | 阈值 | 动作 |
|------|------|------|
| 通过率 | <95% | 告警 |
| 执行时间 | >1.5x 基线 | 告警 |
| 内存使用 | >1.2x 基线 | 告警 |
| 失败测试数 | >5 | 阻塞合并 |
| 跳过率 | >20% | 警告 |

### 9.2 趋势分析

```rust
pub struct TrendAnalyzer {
    history: Vec<TestSummary>,
}

impl TrendAnalyzer {
    // 检测性能退化
    pub fn detect_regression(&self, test_id: &str) -> Option<RegressionAlert>;
    
    // 检测 flaky 测试
    pub fn detect_flaky(&self, test_id: &str) -> Option<FlakyAlert>;
    
    // 生成趋势建议
    pub fn suggest_improvements(&self) -> Vec<Suggestion>;
}
```

---

## 10. 实现路线图

### 10.1 第一阶段：基础框架 (1周)

- [ ] 建立测试注册表
- [ ] 实现测试运行器
- [ ] 集成现有测试
- [ ] 基础报告生成

### 10.2 第二阶段：增量测试 (1周)

- [ ] 实现代码影响分析
- [ ] 建立模块依赖图
- [ ] 实现增量测试选择
- [ ] CI 集成

### 10.3 第三阶段：高级功能 (2周)

- [ ] 测试数据管理
- [ ] 性能趋势分析
- [ ] Flaky 测试检测
- [ ] 自动分类

### 10.4 第四阶段：混沌与探索 (2周)

- [ ] 混沌测试框架
- [ ] 故障注入
- [ ] 随机测试生成
- [ ] 长期稳定性监控

---

## 11. 与现有代码的对应

### 11.1 可复用的现有组件

| 现有组件 | 可用于 |
|----------|----------|
| `crates/sqlancer` | 模糊测试 |
| `crates/transaction-stress` | 压力测试 |
| `tests/stress/*` | 压力测试场景 |
| `tests/anomaly/*` | 异常测试场景 |
| `tests/e2e/*` | E2E 测试场景 |

### 11.2 需要新增的组件

| 新组件 | 职责 |
|--------|------|
| `test-framework/` | 回归测试框架核心 |
| `test-registry/` | 测试注册与管理 |
| `test-report/` | 报告生成 |

---

## 12. 总结

### 12.1 当前优势

- ✅ 56 个测试文件覆盖主要模块
- ✅ 单元/集成/异常/压力测试分类清晰
- ✅ 已有 sqlancer 和 transaction-stress 框架

### 12.2 不足

- ⚠️ 缺少统一的回归测试运行器
- ⚠️ 缺少增量测试能力
- ⚠️ 缺少测试结果分析系统
- ⚠️ 缺少性能趋势监控

### 12.3 预期收益

- 每次提交快速反馈 (<10分钟)
- 精准定位问题模块
- 防止性能退化
- 提高测试效率

---

**文档版本历史**

| 版本 | 日期 | 变更 |
|------|------|------|
| 1.0 | 2026-03-26 | 初始版本 |

**状态**: ✅ 规划完成
