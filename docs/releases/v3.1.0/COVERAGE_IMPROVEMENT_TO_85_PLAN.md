# v3.1.0 覆盖率提升至 85% 全面计划

> **版本**: 1.0
> **日期**: 2026-05-13
> **分支**: feature/coverage-improvement
> **目标**: 将整体覆盖率从 81.48% 提升至 ≥85%
> **状态**: 🟡 规划中

---

## 一、现状分析

### 1.1 当前覆盖率数据 (p1-coverage worktree)

| 指标 | 当前值 | 85% 目标 | 差距 |
|------|--------|---------|------|
| **Regions** | 81.48% | 85% | -3.52% |
| **Functions** | 89.45% | 90% | -0.55% |
| **Lines** | 79.76% | 85% | -5.24% |

### 1.2 模块级覆盖率详情

| Package | Regions | Functions | Lines | 85% 目标 | 差距 |
|---------|---------|-----------|-------|---------|------|
| optimizer | 81.48% | 89.45% | 79.76% | 85% | -5.24% |
| rag | 97.85% | 94.38% | 96.97% | 85% | ✅ 超额 |
| sql-cli | 78.86% | 90.48% | 80.74% | 85% | -4.26% |

### 1.3 覆盖率差距根因分析

基于 `executor-whitebox-test-analysis.md` 和 `COVERAGE_GAP_ANALYSIS.md`：

| 根因类别 | 影响模块 | 具体缺口 | 优先级 |
|---------|---------|---------|--------|
| **G1: Volcano Iterator 路径覆盖不足** | executor | HashJoin/Filter/Aggregate 复杂路径 | 🔴 P0 |
| **G2: NULL 边界处理缺失** | executor, optimizer | NULL 比较/聚合/Join | 🔴 P0 |
| **G3: 聚合函数覆盖不全** | executor | SUM/AVG/MIN/MAX 多字段 GROUP BY | 🔴 P0 |
| **G4: Multi-table DML 执行链路缺失** | executor | UPDATE/DELETE multi-table | 🟡 P1 |
| **G5: Window 函数执行未实现** | executor | LEAD/LAG/NTILE 等 | 🔴 P0 |
| **G6: MERGE 语句执行缺失** | executor | 0% 覆盖 | 🔴 P0 |
| **G7: CBO/CostModel 测试不足** | optimizer | CostModel 代价计算 | 🟡 P1 |
| **G8: 执行链路回归测试未完成** | sql-cli | execution_chain_regression_test | 🟡 P1 |

---

## 二、执行链路覆盖缺口

### 2.1 Critical Gap (0% 覆盖)

| 语句/特性 | 当前状态 | 实现要求 | 优先级 |
|----------|---------|---------|--------|
| **MERGE** | Parser 存在，Executor 疑似 stub | 完整实现 + 测试 | 🔴 P0 |
| **NTILE** | Parser 存在，Executor 未实现 | 完整实现 + 测试 | 🔴 P0 |
| **LEAD/LAG** | Parser 存在，Executor 未实现 | 完整实现 + 测试 | 🔴 P0 |
| **FIRST_VALUE/LAST_VALUE** | Parser 存在，Executor 未实现 | 完整实现 + 测试 | 🔴 P0 |
| **NTH_VALUE** | Parser 存在，Executor 未实现 | 完整实现 + 测试 | 🔴 P0 |

### 2.2 High Priority Gap (<70% 覆盖)

#### DML 语句

| 语句 | 当前覆盖 | 目标 | 缺口测试 |
|------|---------|------|---------|
| UPDATE (multi-table) | 50% | 85% | 多表关联更新 |
| DELETE (multi-table) | 45% | 85% | 多表关联删除 |
| INSERT...SELECT | 65% | 85% | INSERT SELECT 完整路径 |

#### DDL 语句

| 语句 | 当前覆盖 | 目标 | 缺口测试 |
|------|---------|------|---------|
| TRUNCATE | 40% | 85% | 清空表执行链路 |
| RENAME TABLE | 30% | 85% | 表重命名执行链路 |
| CREATE VIEW | 50% | 85% | 视图创建执行链路 |
| DROP VIEW | 50% | 85% | 视图删除执行链路 |
| ALTER TABLE DROP | 65% | 85% | 列删除执行链路 |
| ALTER TABLE MODIFY | 60% | 85% | 列修改执行链路 |

#### 集合运算

| 语句 | 当前覆盖 | 目标 | 缺口测试 |
|------|---------|------|---------|
| INTERSECT | 45% | 85% | 交集执行 |
| EXCEPT | 40% | 85% | 差集执行 |
| MINUS | 40% | 85% | 差集执行（别名） |

---

## 三、执行链路重构设计

### 3.1 执行 Pipeline 统一重构 (Critical)

基于 `EXECUTION_PIPELINE_REFACTORING.md`，当前问题：

```
execute_select:
├── has_join = true  → execute_select_with_join (直接返回，绕过 AGG/HAVING) ❌
└── has_join = false → scan → WHERE → GROUP BY → AGG → HAVING ✅
```

**目标架构**:

```
execute_select:
└── FROM/JOIN (返回 rows)
    └── 统一 pipeline:
        ├── WHERE      (eval_predicate)
        ├── GROUP BY   (build_groups)
        ├── AGGREGATE (compute_aggregates)
        ├── HAVING     (eval_predicate)
        └── PROJECTION (evaluate_expression)
```

**重构收益**:
- ✅ 单一执行模型，无语义分叉
- ✅ JOIN 可复用 WHERE/AGG/HAVING
- ✅ 易于扩展（Window / Subquery）
- ✅ 预期覆盖提升: +8%

### 3.2 Executor 白盒测试补全

基于 `executor-whitebox-test-analysis.md` 的 Top 15 高优先级补测清单：

#### P0 必须补测 (预计 +5%)

| 排名 | 操作符 | 测试场景 | 测试文件 |
|------|--------|---------|---------|
| 1 | IndexScan | Eq (=) | `tests/index_scan_test.rs` |
| 2 | IndexScan | Gt/Lt (> / <) | `tests/index_scan_test.rs` |
| 3 | Aggregate | SUM | `tests/aggregate_test.rs` |
| 4 | Aggregate | AVG | `tests/aggregate_test.rs` |
| 5 | Aggregate | 多字段 GROUP BY | `tests/aggregate_test.rs` |
| 6 | HashJoin | Left Join | `tests/hash_join_test.rs` |
| 7 | HashJoin | Cross Join | `tests/hash_join_test.rs` |
| 8 | Delete | 全表删除 | `tests/delete_test.rs` |

#### P1 应该补测 (预计 +3%)

| 排名 | 操作符 | 测试场景 | 测试文件 |
|------|--------|---------|---------|
| 9 | Filter | NULL 谓词 | `tests/filter_test.rs` |
| 10 | IndexScan | GtEq/LtEq | `tests/index_scan_test.rs` |
| 11 | SeqScan | Storage 错误 | `tests/seq_scan_test.rs` |
| 12 | Aggregate | MAX/MIN | `tests/aggregate_test.rs` |
| 13 | HashJoin | 无匹配行 | `tests/hash_join_test.rs` |
| 14 | Filter | 空 children | `tests/filter_test.rs` |

### 3.3 窗口函数实现计划

| 窗口函数 | Parser | Executor | 测试覆盖 | 优先级 |
|---------|--------|----------|---------|--------|
| ROW_NUMBER | ✅ | ✅ | ~70% | 继续补测 |
| RANK | ✅ | ✅ | ~70% | 继续补测 |
| DENSE_RANK | ✅ | ✅ | ~70% | 继续补测 |
| NTILE | ✅ | ❌ | 0% | 🔴 P0 |
| LEAD | ✅ | ❌ | 0% | 🔴 P0 |
| LAG | ✅ | ❌ | 0% | 🔴 P0 |
| FIRST_VALUE | ✅ | ❌ | 0% | 🔴 P0 |
| LAST_VALUE | ✅ | ❌ | 0% | 🔴 P0 |
| NTH_VALUE | ✅ | ❌ | 0% | 🔴 P0 |

---

## 四、覆盖率提升任务分解

### 4.1 Phase 1: 核心执行链路补测 (Week 1-2)

**目标**: 提升 +3% 覆盖率

| 任务 | 测试文件 | 预期覆盖 |
|------|---------|---------|
| T1.1: IndexScan Eq/Gt/Lt 测试 | `tests/index_scan_predicate_test.rs` | +1% |
| T1.2: Aggregate SUM/AVG 测试 | `tests/aggregate_functions_test.rs` | +1% |
| T1.3: HashJoin Left/Cross 测试 | `tests/hash_join_types_test.rs` | +0.5% |
| T1.4: Delete 全表删除测试 | `tests/delete_execution_test.rs` | +0.5% |

### 4.2 Phase 2: 执行链路回归测试 (Week 3-4)

**目标**: 提升 +2% 覆盖率

| 任务 | 测试文件 | 预期覆盖 |
|------|---------|---------|
| T2.1: execution_chain_regression_test 完善 | `tests/execution_chain_regression_test.rs` | +0.5% |
| T2.2: multi-table UPDATE 测试 | `tests/multi_table_update_test.rs` | +0.5% |
| T2.3: multi-table DELETE 测试 | `tests/multi_table_delete_test.rs` | +0.5% |
| T2.4: TRUNCATE/RENAME TABLE 测试 | `tests/ddl_execution_test.rs` | +0.5% |

### 4.3 Phase 3: Window 函数与 MERGE (Week 5-6)

**目标**: 提升 +2% 覆盖率

| 任务 | 状态 | 优先级 |
|------|------|--------|
| T3.1: NTILE 实现 + 测试 | 待实现 | 🔴 P0 |
| T3.2: LEAD/LAG 实现 + 测试 | 待实现 | 🔴 P0 |
| T3.3: FIRST/LAST/NTH_VALUE 实现 + 测试 | 待实现 | 🔴 P0 |
| T3.4: MERGE 语句实现 + 测试 | 待实现 | 🔴 P0 |

### 4.4 Phase 4: 执行 Pipeline 重构 (Week 7-8)

**目标**: 提升 +8% 覆盖率（架构改进）

基于 `EXECUTION_PIPELINE_REFACTORING.md`：

| 阶段 | 任务 | 预期覆盖 |
|------|------|---------|
| P4.1 | 提取 `execute_from_with_join`，返回 `(rows, schema)` | 验证 +2% |
| P4.2 | 在 `execute_select` 中调用 `execute_from_with_join`，然后走 pipeline | 验证 +2% |
| P4.3 | 删除原 `execute_select_with_join` 中的 pipeline 逻辑 | 验证 +2% |
| P4.4 | 添加 `apply_*` 辅助函数 | 验证 +2% |

**重构必须通过的测试**:

| 测试 | SQL | 验证点 |
|------|-----|--------|
| `test_join_where_agg` | `SELECT COUNT(t2.id) FROM t1 LEFT JOIN t2 ON t1.id = t2.id WHERE t2.id IS NOT NULL` | JOIN + WHERE + AGG |
| `test_join_group_having` | `SELECT t1.id, COUNT(t2.id) FROM t1 LEFT JOIN t2 ON t1.id = t2.id GROUP BY t1.id HAVING COUNT(t2.id) > 0` | JOIN + GROUP BY + AGG + HAVING |
| `test_agg_all_null` | `SELECT SUM(col) FROM t` (all NULL) | 全 NULL aggregate |

---

## 五、测试框架增强

### 5.1 覆盖率收集机制

```bash
# 使用 cargo llvm-cov (不是 tarpaulin)
cargo llvm-cov --all-features --lcov --output-path lcov.info
cargo llvm-cov --html --output-dir target/llvm-cov-report
```

### 5.2 覆盖率门禁脚本

```bash
# scripts/gate/check_coverage_v310.sh
# 按包检查覆盖率，未达标阻断

for crate in optimizer sql-cli rag; do
    threshold=85
    actual=$(cargo llvm-cov -p sqlrustgo-$crate --summary-only | grep TOTAL | awk '{print $NF}' | tr -d '%')
    if (( $(echo "$actual < $threshold" | bc -l) )); then
        echo "FAIL: sqlrustgo-$crate $actual% < $threshold%"
        exit 1
    fi
done
```

### 5.3 覆盖率趋势追踪

| 日期 | Total | optimizer | sql-cli | rag | 备注 |
|------|-------|-----------|---------|-----|------|
| 2026-05-13 | 81.48% | 79.76% | 80.74% | 96.97% | 基线 |
| 2026-05-20 | - | - | - | - | Week 1-2 目标 |
| 2026-05-27 | - | - | - | - | Week 3-4 目标 |
| 2026-06-03 | - | - | - | - | Week 5-6 目标 |
| 2026-06-10 | ≥85% | ≥85% | ≥85% | ≥85% | 最终目标 |

---

## 六、关键测试场景清单

### 6.1 DML 测试 (P0)

```rust
// INSERT 测试场景
- 简单 INSERT ✅ 已有
- INSERT SELECT ❌ 需补测
- INSERT ON DUPLICATE KEY UPDATE ✅ 已有
- 批量 INSERT ❌ 需补测

// UPDATE 测试场景
- 简单 UPDATE ✅ 已有
- UPDATE with JOIN ❌ 需补测
- UPDATE multi-table ❌ 需补测
- 批量 UPDATE ✅ 已有

// DELETE 测试场景
- 简单 DELETE ✅ 已有
- DELETE with JOIN ❌ 需补测
- DELETE multi-table ❌ 需补测
- TRUNCATE ❌ 需补测
```

### 6.2 聚合函数测试 (P0)

```rust
// COUNT ✅ 已有
// SUM ❌ 需补测
// AVG ❌ 需补测
// MIN ❌ 需补测
// MAX ❌ 需补测
// 多字段 GROUP BY ❌ 需补测
```

### 6.3 窗口函数测试 (P0)

```rust
// ROW_NUMBER ✅ 已有部分
// RANK ✅ 已有部分
// DENSE_RANK ✅ 已有部分
// NTILE ❌ 未实现
// LEAD ❌ 未实现
// LAG ❌ 未实现
// FIRST_VALUE ❌ 未实现
// LAST_VALUE ❌ 未实现
// NTH_VALUE ❌ 未实现
```

### 6.4 边界条件测试 (P1)

```rust
// NULL 语义
- NULL = NULL ❌ 需补测
- NULL > 10 ❌ 需补测
- NULL IN (1,2,3) ❌ 需补测
- COUNT(NULL) ❌ 需补测
- SUM(NULL) ❌ 需补测

// 类型边界
- i64::MAX + 1 ❌ 需补测
- 浮点精度 ❌ 需补测
```

---

## 七、风险与缓解

### 7.1 已知风险

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| Window 函数实现复杂 | 可能延期 | 优先实现 LEAD/LAG，延后 NTILE |
| 执行 Pipeline 重构影响现有功能 | 可能引入回归 | 每个阶段后运行完整测试套件 |
| 磁盘空间不足 | 覆盖率测试失败 | 分模块测试，及时清理中间文件 |

### 7.2 依赖关系

```
T1.1-T1.4 (Phase 1) ──┬── 无依赖，可并行
                        │
T2.1-T2.4 (Phase 2) ──┼── 依赖 Phase 1 完成的测试框架
                        │
T3.1-T3.4 (Phase 3) ──┼── 依赖 Window 函数实现
                        │
T4.1-T4.4 (Phase 4) ──┴── 依赖执行链路重构完成
```

---

## 八、验收标准

### 8.1 覆盖率目标

| 指标 | 当前 | Phase 1 | Phase 2 | Phase 3 | Phase 4 | 最终目标 |
|------|------|---------|---------|---------|---------|---------|
| Total Lines | 79.76% | 82% | 83% | 84% | 86% | **≥85%** |
| optimizer | 79.76% | 81% | 82% | 83% | 85% | **≥85%** |
| sql-cli | 80.74% | 82% | 83% | 84% | 86% | **≥85%** |
| rag | 96.97% | 97% | 97% | 97% | 97% | **≥85%** ✅ |

### 8.2 测试通过标准

- [ ] `cargo test --package sqlrustgo-optimizer --lib` 全部通过
- [ ] `cargo test --package sqlrustgo-sql-cli --lib` 全部通过
- [ ] `cargo test --package sqlrustgo-rag --lib` 全部通过
- [ ] `cargo llvm-cov --package sqlrustgo-optimizer --summary-only` Lines ≥ 85%
- [ ] `cargo llvm-cov --package sqlrustgo-sql-cli --summary-only` Lines ≥ 85%

---

## 九、后续行动

### 立即行动 (本周)

1. [ ] 运行 `cargo llvm-cov` 获取当前覆盖率基线
2. [ ] 创建 Issue 追踪 Phase 1 任务
3. [ ] 开始 IndexScan Predicate 测试编写

### 短期目标 (Week 1-4)

1. [ ] 完成 Phase 1: 核心执行链路补测
2. [ ] 完成 Phase 2: 执行链路回归测试
3. [ ] 验证覆盖率 ≥83%

### 中期目标 (Week 5-8)

1. [ ] 完成 Phase 3: Window 函数与 MERGE
2. [ ] 完成 Phase 4: 执行 Pipeline 重构
3. [ ] 验证覆盖率 ≥85%

---

## 十、相关文档索引

| 文档 | 路径 | 用途 |
|------|------|------|
| Executor 白盒测试分析 | `docs/analysis/executor-whitebox-test-analysis.md` | 执行器测试补测清单 |
| 执行 Pipeline 重构方案 | `docs/architecture/EXECUTION_PIPELINE_REFACTORING.md` | 执行模型重构设计 |
| v3.1.0 覆盖缺口分析 | `docs/releases/v3.1.0/oo/coverage/COVERAGE_GAP_ANALYSIS.md` | 缺口扫描结果 |
| SQL 执行链路矩阵 | `docs/releases/v3.0.0/oo/SQL_EXECUTION_MATRIX.md` | 语句覆盖矩阵 |
| 测试体系重构计划 | `docs/plans/2026-06-03-brainstorming-test-system-refactor.md` | 测试框架增强 |

---

*本文档由 claude agent 创建*
*覆盖率基线日期: 2026-05-13*
*目标达成日期: 2026-06-10*
