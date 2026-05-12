# v3.1.0 覆盖率分析报告与改进计划

> **日期**: 2026-05-13  
> **分支**: develop/v3.1.0  
> **当前覆盖率**: 41.16% (需要 ≥50% Alpha / ≥75% Beta)

---

## 一、当前覆盖率状况

### 1.1 整体覆盖率

| 指标 | 当前值 | Alpha 要求 | Gap |
|------|--------|-----------|-----|
| **Line Coverage** | 41.16% | 50% | -8.84% |
| **Function Coverage** | 42.75% | 50% | -7.25% |
| **Region Coverage** | 41.16% | — | — |

### 1.2 核心模块覆盖率 (已知数据)

| 模块 | Line Coverage | Function Coverage | 优先级 |
|------|--------------|------------------|--------|
| `executor` (execution_engine.rs) | **41.21%** | **43.07%** | 🔴 高 |
| `storage` | 已知未测量 | — | 🟡 中 |
| `parser` | 已知未测量 | — | 🟡 中 |
| `planner` | 已知未测量 | — | 🟡 中 |
| `optimizer` | 已知未测量 | — | 🟡 中 |

### 1.3 未覆盖代码分布 (execution_engine.rs 详细)

```
Total lines:      3,658
Lines missed:     2,179 (59.57% uncovered)
Functions:        274 total
Functions missed: 156 (56.93% uncovered)
```

---

## 二、覆盖率瓶颈分析

### 2.1 核心瓶颈识别

#### 🔴 高影响未覆盖区域

| 区域 | 行数 | 覆盖率 | 说明 |
|------|------|--------|------|
| `execute_insert` | ~300行 | <20% | INSERT 执行路径 |
| `execute_update` | ~200行 | <20% | UPDATE 执行路径 |
| `execute_delete` | ~200行 | <20% | DELETE 执行路径 |
| `execute_merge` | ~250行 | <20% | MERGE 执行路径 |
| `transaction` 相关 | ~400行 | <30% | 事务管理 |

#### 🟡 中等影响未覆盖区域

| 区域 | 行数 | 覆盖率 | 说明 |
|------|------|--------|------|
| `execute_select` 复杂路径 | ~500行 | 40% | 复杂 SELECT |
| Window 函数执行 | ~300行 | 45% | 窗口函数 |
| JOIN 执行 | ~400行 | 50% | JOIN 算法 |

### 2.2 覆盖率低的主要原因

1. **事务边界测试缺失**: MVCC/WAL 路径未充分测试
2. **DML 路径测试不足**: INSERT/UPDATE/DELETE 覆盖率低
3. **错误路径未测试**: 各种错误情况未覆盖
4. **集成测试不足**: 端到端场景覆盖少
5. **CBO/优化器测试不足**: 代价模型路径未充分测试

---

## 三、覆盖率提升策略

### 3.1 策略一：增量测试覆盖 (快速见效)

**目标**: 在 2 周内提升 5-8%

| 阶段 | 任务 | 预期提升 | 工作量 |
|------|------|---------|--------|
| 1.1 | 补充 DML 单元测试 | +3% | 中 |
| 1.2 | 补充错误路径测试 | +2% | 低 |
| 1.3 | 补充事务边界测试 | +2% | 中 |
| 1.4 | 补充 CBO 代价模型测试 | +1% | 低 |

### 3.2 策略二：模块化覆盖率攻坚

**目标**: 在 4 周内提升至 50%+

| 模块 | 目标覆盖率 | 关键测试 |
|------|-----------|---------|
| `executor` | 55% | DML, 事务, 错误处理 |
| `storage` | 60% | 页面管理, B+树, WAL |
| `planner` | 50% | 计划生成, 计划缓存 |
| `optimizer` | 45% | CBO, 规则优化 |
| `parser` | 70% | 全面语法覆盖 |

### 3.3 策略三：集成测试增强

**目标**: 在 6 周内提升至 60%+

| 测试类型 | 覆盖目标 | 关键场景 |
|---------|---------|---------|
| TPC-H 完整测试 | SF=1, SF=10 | 全量查询 |
| 并发测试 | 10 并发 | 事务隔离 |
| 故障恢复测试 | OOM, Crash | WAL 恢复 |
| SQL 兼容性测试 | 200+ SQL | MySQL 兼容性 |

---

## 四、具体改进计划 (8 周)

### Phase 1: 基础覆盖 (Week 1-2)

| # | 任务 | Owner | 预期覆盖 | PR |
|---|------|-------|---------|-----|
| P1.1 | INSERT 执行路径单元测试 | TBD | +2% | — |
| P1.2 | UPDATE/DELETE 路径测试 | TBD | +1.5% | — |
| P1.3 | 错误处理路径测试 | TBD | +1% | — |
| P1.4 | Null/边界值测试 | TBD | +0.5% | — |

### Phase 2: 核心模块覆盖 (Week 3-4)

| # | 任务 | Owner | 预期覆盖 | PR |
|---|------|-------|---------|-----|
| P2.1 | executor 事务测试 | TBD | +3% | — |
| P2.2 | storage B+树测试 | TBD | +2% | — |
| P2.3 | planner 计划生成测试 | TBD | +1.5% | — |
| P2.4 | optimizer CBO 测试 | TBD | +1% | — |

### Phase 3: 集成测试 (Week 5-6)

| # | 任务 | Owner | 预期覆盖 | PR |
|---|------|-------|---------|-----|
| P3.1 | TPC-H SF=1 全量测试 | TBD | +2% | — |
| P3.2 | 并发事务测试 | TBD | +2% | — |
| P3.3 | 故障恢复测试 | TBD | +1% | — |
| P3.4 | SQL 兼容性扩展 | TBD | +1% | — |

### Phase 4: 优化与验证 (Week 7-8)

| # | 任务 | Owner | 预期覆盖 | PR |
|---|------|-------|---------|-----|
| P4.1 | 覆盖率缺口扫描与补充 | TBD | +1% | — |
| P4.2 | 慢路径覆盖优化 | TBD | +1% | — |
| P4.3 | Alpha 门禁验证 | TBD | — | — |
| P4.4 | Beta 门禁验证 | TBD | — | — |

---

## 五、关键测试场景清单

### 5.1 DML 测试 (高优先级)

```rust
// INSERT 测试场景
- 简单 INSERT
- INSERT SELECT
- INSERT ON DUPLICATE KEY UPDATE ✅ 已覆盖
- INSERT IGNORE
- REPLACE INTO
- 批量 INSERT
- 边界值 (NULL, 空表, 大表)

// UPDATE 测试场景
- 简单 UPDATE
- UPDATE with JOIN
- UPDATE with WHERE
- 批量 UPDATE
- 自引用 UPDATE

// DELETE 测试场景
- 简单 DELETE
- DELETE with JOIN
- TRUNCATE
- 批量 DELETE
```

### 5.2 事务测试 (高优先级)

```rust
// 事务隔离测试
- READ COMMITTED
- READ UNCOMMITTED (如支持)
- 事务回滚
- 嵌套事务
- 长事务

// 并发测试
- 写写冲突
- 读写冲突
- 死锁检测
```

### 5.3 错误处理测试 (中优先级)

```rust
// 错误场景
- 主键冲突
- 外键约束违反
- NOT NULL 约束违反
- 类型不匹配
- 除零错误
- 空指针处理
```

---

## 六、工具与方法

### 6.1 覆盖率工具

| 工具 | 用途 |
|------|------|
| `cargo llvm-cov` | 主覆盖率工具 |
| `--openhtml` | 生成 HTML 报告 |
| `--lcov` | 生成 LCOV 格式 (CI 集成) |
| `--codecov` | Codecov 集成 |

### 6.2 覆盖率阈值

| 阶段 | 阈值 | 说明 |
|------|------|------|
| Alpha | 50% | 基础门槛 |
| Beta | 75% | 质量门槛 |
| RC | 80% | 发布门槛 |
| GA | 85% | 稳定门槛 |

### 6.3 覆盖率目标分解

```
当前: 41.16%
  ↓ +5% (Phase 1: 基础覆盖) → 46%
  ↓ +7% (Phase 2: 核心模块) → 53% ← Alpha 门槛达成
  ↓ +7% (Phase 3: 集成测试) → 60%
  ↓ +5% (Phase 4: 优化) → 65% ← Beta 门槛接近
```

---

## 七、结论与建议

### 7.1 现状

- 当前覆盖率 **41.16%**，距离 Alpha 门槛 **50%** 差 **8.84%**
- 核心瓶颈在 `executor` 模块的 DML 执行路径

### 7.2 建议行动

1. **立即**: 补充 INSERT/UPDATE/DELETE 基础测试 (目标 +5%)
2. **短期**: 完成 Phase 1-2，达到 Alpha 门槛 50%
3. **中期**: 完成 Phase 3-4，向 Beta 门槛 75% 迈进

### 7.3 风险

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 测试代码膨胀 | 维护成本增加 | 自动化测试生成 |
| 测试运行时间 | CI 变慢 | 并行化测试 |
| 测试脆弱性 | Flaky tests | 稳定性测试框架 |

---

## 八、附录：覆盖率测量命令

```bash
# 快速覆盖率检查
cargo llvm-cov report

# 生成 HTML 报告
cargo llvm-cov report --html --output-dir /tmp/cov_html
open /tmp/cov_html/index.html

# 工作区覆盖率
cargo llvm-cov --workspace report

# 特定包覆盖率
cargo llvm-cov --package sqlrustgo-executor report

# CI 集成 (LCOV 格式)
cargo llvm-cov --lcov --output-path lcov.info
```
