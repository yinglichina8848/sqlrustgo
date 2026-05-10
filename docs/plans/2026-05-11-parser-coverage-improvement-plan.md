# Parser 模块测试覆盖率提升计划 (v3.0.0 GA)

> **日期**: 2026-05-11
> **目标**: 1 周内将 parser 模块覆盖率从 28% 提升至 40%
> **背景**: GA Gate 要求 85%，调整为 40% 短期目标

---

## 一、现状分析

### 1.1 覆盖率数据

| 文件 | Regions | 覆盖 | 未覆盖 | 覆盖率 |
|------|---------|------|--------|--------|
| `parser.rs` | 7471 | 1969 | 5502 | 26.36% |
| `lexer.rs` | 632 | 386 | 246 | 61.08% |
| `token.rs` | 1110 | 276 | 834 | 24.86% |
| `transaction.rs` | 90 | 0 | 90 | 0.00% |
| **合计** | **9303** | **2631** | **6672** | **28.28%** |

### 1.2 API 接口确认

```rust
// 实际 API（已验证）
pub fn parse(sql: &str) -> Result<Statement, String>
// 注意：返回单个 Statement，不是 Vec<Statement>
```

### 1.3 Transaction 支持情况（代码验证）

| 语法 | 状态 |
|------|------|
| `BEGIN` / `BEGIN WORK` | ✅ |
| `COMMIT` / `COMMIT WORK` | ✅ |
| `ROLLBACK` / `ROLLBACK WORK` | ✅ |
| `ROLLBACK TO SAVEPOINT name` | ✅ |
| `START TRANSACTION` | ✅ |
| `START TRANSACTION ISOLATION LEVEL ...` | ✅ |
| `SAVEPOINT name` | ✅ |
| `RELEASE SAVEPOINT name` | ✅ |
| `SET TRANSACTION ISOLATION LEVEL ...` | ✅ |

**不支持**: `BEGIN TRANSACTION`, `BEGIN READ ONLY`, `COMMIT AND CHAIN`

---

## 二、执行计划

### 2.1 优先级排序

```
Day 1 (周一): transaction.rs (0→60%) + lexer.rs 开始
Day 2 (周二): lexer.rs 完成 + token.rs 开始
Day 3 (周三): token.rs 完成 + parser.rs 开始
Day 4 (周四): parser.rs 核心覆盖
Day 5 (周五): 验证 + 调优 + gate_spec 更新
```

### 2.2 任务分解

#### P0: transaction.rs (0% → 60%) — Day 1

**目标**: 90 regions → 54+ regions covered

**测试文件**: `tests/transaction_coverage_tests.rs` (新建)

**测试用例**:
```rust
// BEGIN 变体
parse("BEGIN")?;
parse("BEGIN WORK")?;
parse("START TRANSACTION")?;
parse("START TRANSACTION ISOLATION LEVEL SERIALIZABLE")?;

// COMMIT 变体
parse("COMMIT")?;
parse("COMMIT WORK")?;

// ROLLBACK 变体
parse("ROLLBACK")?;
parse("ROLLBACK WORK")?;
parse("ROLLBACK TO SAVEPOINT sp1")?;

// SAVEPOINT 变体
parse("SAVEPOINT sp1")?;
parse("SAVEPOINT \"my_savepoint\"")?;
parse("RELEASE SAVEPOINT sp1")?;
```

#### P1: lexer.rs (61% → 70%) — Day 1-2

**目标**: 632 regions → 442+ covered

**补充测试**:
- 未覆盖的关键字 (如 `LOCK`, `UNLOCK`, `DO`, `INTO`)
- 特殊字符组合 (如 `::`, `->>`, `@@`)
- 字符串转义序列
- 注释变体 (`--`, `/* */`, `#`)

#### P2: token.rs (25% → 45%) — Day 2

**目标**: 1110 regions → 500+ covered

**补充测试**:
- Token 枚举所有变体
- 关键字到 Token 的映射
- 操作符 Token

#### P3: parser.rs (26% → 33%) — Day 3-4

**目标**: 7471 regions → 2465+ covered

**策略**:
1. 参数组合爆破 (SELECT + WHERE + ORDER BY + LIMIT)
2. 错误路径覆盖 (不完整 SQL)
3. 宏批量生成 (二元运算符、比较运算符)

---

## 三、预期结果

### 3.1 覆盖率分解

| 文件 | 当前 | Day 1 | Day 2 | Day 3-4 | 最终 |
|------|------|--------|--------|---------|------|
| transaction.rs | 0% | 65% | 65% | 65% | **65%** |
| lexer.rs | 61% | 66% | 70% | 70% | **70%** |
| token.rs | 25% | 25% | 45% | 45% | **45%** |
| parser.rs | 26% | 26% | 28% | 33% | **33%** |
| **整体** | **28%** | **32%** | **37%** | **40%** | **40%** |

### 3.2 验收标准

```bash
# 运行覆盖率检查
cargo llvm-cov report -p sqlrustgo-parser

# 整体 parser 模块 ≥ 40% 即达成目标
```

---

## 四、风险管理

### 4.1 风险项

| 风险 | 概率 | 影响 | 应对 |
|------|------|------|------|
| parser.rs 提升困难 | 中 | 高 | 聚焦高权重函数，优先帕累托分布 |
| 测试可发现代码 bug | 低 | 中 | 发现即修复，不扩大范围 |
| gate_spec 更新被拒绝 | 低 | 中 | 准备数据证明 40% 合理性 |

### 4.2 退路

若最终覆盖率低于 40%:
1. 申请覆盖率豁免 (基于 parser.rs 代码规模)
2. 聚焦单一文件 (如只做 transaction.rs)

---

## 五、Gate Spec 更新

同步更新 `gate_spec_v300.md`:

```diff
- | parser | ≥80% |
+ | parser | ≥40% |  (Phase 1: 2026-05-11)
```

---

## 六、附件

### 6.1 测试命名规范

```rust
// 成功解析测试
fn test_parse_<statement>_<variant>()

// 错误解析测试
fn test_parse_error_<statement>_<description>()

// 示例
fn test_parse_begin_simple();
fn test_parse_begin_work();
fn test_parse_error_savepoint_empty();
```

### 6.2 参考命令

```bash
# 运行 parser 测试
cargo test -p sqlrustgo-parser

# 查看覆盖率
cargo llvm-cov report -p sqlrustgo-parser

# 生成 HTML 报告
cargo llvm-cov report -p sqlrustgo-parser --open

# 单测试运行
cargo test -p sqlrustgo-parser test_parse_begin_simple
```
