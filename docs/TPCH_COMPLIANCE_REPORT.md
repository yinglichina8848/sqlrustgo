# TPC-H Q1-Q22 合规性测试报告

**项目**: SQLRustGo  
**分支**: feat/tpch-compliance-v2  
**日期**: 2026-04-02  
**状态**: Parser 全部通过，Executor 存在差异

---

## 1. 测试概述

### 1.1 测试目标
- 验证 SQLRustGo 的 TPC-H Q1-Q22 查询解析能力
- 对比 SQLRustGo 与 SQLite 的查询结果一致性

### 1.2 测试环境
- **Parser**: sqlrustgo-parser
- **Executor**: SQLRustGo ExecutionEngine
- **Reference**: SQLite 3.x
- **测试数据**: 简化的 TPC-H 测试数据集

---

## 2. 测试结果摘要

### 2.1 Parser 解析测试

| 类别 | 结果 |
|------|------|
| Q1-Q22 解析 | ✅ **22/22 通过** |
| 聚合函数 (SUM, COUNT, AVG, MIN, MAX) | ✅ 通过 |
| COUNT(DISTINCT) | ✅ 通过 |
| CASE WHEN 表达式 | ✅ 通过 |
| EXTRACT 函数 | ✅ 通过 |
| SUBSTRING 函数 | ✅ 通过 |
| 算术表达式 (a + b, a / b) | ✅ 通过 |
| JOIN 语法 (INNER JOIN, 逗号连接) | ✅ 通过 |
| GROUP BY | ✅ 通过 |
| ORDER BY | ✅ 通过 |

### 2.2 Executor 执行测试

| Query | SQLite Rows | SQLRustGo Rows | 结果匹配 | SQLite Status | SQLRustGo Status |
|-------|-------------|----------------|----------|---------------|------------------|
| Q1 | 3 | 1 | NO | OK | OK |
| Q2 | 0 | 4 | NO | OK | OK |
| Q3 | 5 | 1 | NO | OK | OK |
| Q4 | 0 | 1 | NO | OK | OK |
| Q5 | 0 | 1 | NO | OK | OK |
| Q6 | 1 | 1 | - | OK | OK |
| Q7 | 7 | 1 | NO | OK | OK |
| Q8 | 0 | 1 | NO | **ERR** | OK |
| Q9 | 0 | 1 | NO | **ERR** | OK |
| Q10 | 3 | 1 | NO | OK | OK |
| Q11 | 0 | 1 | NO | OK | OK |
| Q12 | 0 | 1 | NO | OK | OK |
| Q13 | 5 | 1 | NO | OK | OK |
| Q14 | 1 | 7 | NO | OK | OK |
| Q15 | 0 | 1 | NO | OK | OK |
| Q16 | 0 | 1 | NO | OK | OK |
| Q17 | 1 | 1 | - | OK | OK |
| Q18 | 5 | 1 | NO | OK | OK |
| Q19 | 1 | 1 | - | OK | OK |
| Q20 | 0 | 3 | NO | OK | OK |
| Q21 | 0 | 1 | NO | OK | OK |
| Q22 | 0 | 1 | NO | **ERR** | OK |

**统计**:
- Parser 通过率: **22/22 (100%)**
- Executor 结果匹配: **0/22 (0%)**
- SQLite 语法错误: **3** (Q8, Q9, Q22 - SQLite 不支持标准 EXTRACT/SUBSTRING 语法)

---

## 3. 详细分析

### 3.1 Parser 问题修复 (已解决)

本次工作修复了以下 Parser 问题：

#### 3.1.1 CASE WHEN 解析 Bug
**问题**: `parse_case_when_expression` 函数中 ELSE token 被提前消费，导致第二次检查时永远为 false。

**修复**: 使用 `has_else` 标志位替代重复检查。

#### 3.1.2 DISTINCT 关键字缺失
**问题**: `DISTINCT` 关键字未在词法分析器中注册。

**修复**: 在 `lexer.rs` 中添加 `"DISTINCT" => Token::Distinct`。

#### 3.1.3 EXTRACT/SUBSTRING 别名处理
**问题**: 解析 EXTRACT/SUBSTRING 表达式后未处理 `AS` 别名。

**修复**: 在 `parse_select` 函数中添加别名解析逻辑。

#### 3.1.4 聚合函数算术表达式
**问题**: `SUM(...) / 7.0` 这类表达式中，第二个操作数无法被解析。

**修复**: 将 `parse_primary_expression()` 改为 `parse_expression()`。

#### 3.1.5 数字开头的表达式
**问题**: `100.00 * SUM(...)` 这类以数字开头的表达式未被处理。

**修复**: 在 SELECT 列解析中添加 `NumberLiteral` 分支。

### 3.2 Executor 差异分析

#### 3.2.1 行数差异
大部分查询的 SQLite 和 SQLRustGo 返回行数不同，可能原因：

1. **JOIN 语义差异**: 隐式连接 (逗号连接) 与显式 JOIN 的处理可能不同
2. **聚合实现差异**: GROUP BY 的实现可能存在差异
3. **数据加载**: 两个引擎的数据加载可能存在细微差异

#### 3.2.2 SQLite 语法限制
SQLite 不支持标准 SQL 的 EXTRACT 和 SUBSTRING 语法：

```sql
-- 标准语法 (SQLRustGo 支持)
SELECT EXTRACT(YEAR FROM o_orderdate)
SELECT SUBSTRING(c_phone FROM 1 FOR 2)

-- SQLite 语法 (不兼容)
SELECT strftime('%Y', o_orderdate)
SELECT substr(c_phone, 1, 2)
```

这导致 Q8, Q9, Q22 在 SQLite 中报语法错误。

---

## 4. 改进建议

### 4.1 高优先级

#### 4.1.1 Executor 行数差异调查
需要深入调查 SQLRustGo 与 SQLite 返回行数不同的根本原因。

**建议步骤**:
1. 使用完全相同的测试数据分别加载到两个引擎
2. 运行简单查询验证 JOIN 和 GROUP BY 行为
3. 对比每一步的执行结果

#### 4.1.2 统一语法支持
为 SQLite 不支持的语法添加兼容性处理。

### 4.2 中优先级

#### 4.2.1 测试数据标准化
- 使用 TPC-H 标准测试数据集 (SF=0.01 或更大)
- 确保 SQLite 和 SQLRustGo 使用完全相同的初始数据

#### 4.2.2 结果对比报告
- 详细输出每个查询的 SQLite 和 SQLRustGo 结果
- 标记具体的列值差异

### 4.3 低优先级

#### 4.3.1 性能基准测试
添加 TPC-H Q1-Q22 的执行时间对比。

#### 4.3.2 完整 TPC-H 覆盖
实现 Q2, Q9 等复杂查询的完整执行验证。

---

## 5. 结论

### 5.1 Parser (语法分析器)
**状态**: ✅ **完全通过**
- 22/22 TPC-H 查询全部成功解析
- 所有标准 SQL 特性 (聚合、CASE WHEN、EXTRACT、SUBSTRING 等) 均正常工作

### 5.2 Executor (执行引擎)
**状态**: ⚠️ **需要调查**
- 所有查询都能执行 (无运行时错误)
- 但结果与 SQLite 存在差异，需要进一步调查

### 5.3 后续工作

1. **调查 Executor 差异** (高优先级)
   - 验证数据加载是否一致
   - 对比 JOIN 和 GROUP BY 行为
   - 修复任何发现的执行 bug

2. **完善测试框架** (中优先级)
   - 使用标准 TPC-H 数据集
   - 添加详细的差异报告

3. **性能测试** (低优先级)
   - 添加执行时间对比

---

## 附录 A: 修改的文件

### Parser 修改
1. `crates/parser/src/lexer.rs` - 添加 DISTINCT 关键字
2. `crates/parser/src/parser.rs` - 修复 CASE WHEN、EXTRACT/SUBSTRING、聚合函数解析

### 测试修改
1. `tests/integration/tpch_compliance_test.rs` - 添加 Q2, Q8, Q9, Q14, Q16, Q22 测试，实现 SQLite 对比

---

## 附录 B: 测试命令

```bash
# 运行所有 TPC-H 测试
cargo test --test tpch_compliance_test

# 运行带输出的测试
cargo test --test tpch_compliance_test -- --nocapture

# 运行单个测试
cargo test --test tpch_compliance_test test_tpch_q14_case_when -- --nocapture
```
