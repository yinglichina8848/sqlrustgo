# Parser 模块测试覆盖率提升指南

> **目标**: 将 parser 模块覆盖率从当前 **52.79%** 提升至 **85%+**
> **版本**: v3.0.0
> **更新**: 2026-05-10

---

## 1. 当前覆盖率分析

### 1.1 模块级覆盖率

| 模块 | Regions | Missed | 覆盖率 | Functions | Missed | 覆盖率 | Lines | Missed | 覆盖率 |
|------|---------|--------|--------|-----------|--------|--------|-------|--------|--------|
| **parser.rs** | 6,925 | 3,269 | **52.79%** | 190 | 54 | 71.58% | 3,932 | 1,805 | **54.09%** |
| lexer.rs | 643 | 93 | 85.54% | 25 | 4 | 84.00% | 357 | 35 | 90.20% |
| token.rs | 1,094 | 388 | 64.53% | 17 | 0 | 100.00% | 540 | 131 | 75.74% |
| transaction.rs | 90 | 6 | 93.33% | 9 | 0 | 100.00% | 98 | 6 | 93.88% |
| **TOTAL** | 8,752 | 3,756 | 57.08% | 241 | 58 | 75.93% | 4,927 | 1,977 | 59.87% |

### 1.2 parser.rs 未覆盖核心函数清单

基于 llvm-cov 报告，以下是未覆盖的关键函数及其优先级：

| 优先级 | Parser 函数 | 触发 SQL 模板 | 未覆盖行数 |
|--------|------------|--------------|------------|
| **高** | `parse_set_transaction` | `BEGIN ISOLATION LEVEL <level>` | ~45 |
| **高** | `parse_create_role` | `CREATE ROLE name [WITH PARENT parent]` | ~30 |
| **高** | `parse_drop_role` | `DROP ROLE name` | ~10 |
| **中** | `parse_grant_role` | `GRANT role_name TO user` | ~20 |
| **中** | `parse_revoke_role` | `REVOKE role_name FROM user` | ~20 |
| **中** | `parse_truncate` | `TRUNCATE TABLE name` | ~25 |
| **中** | `parse_position_expression` | `POSITION(expr IN expr)` | ~15 |
| **中** | `parse_if_expression` | `IF(cond, true_val, false_val)` | ~25 |
| **低** | `evaluate_binary_op` | (常量折叠内部函数) | ~50 |
| **低** | `fold_constants` | (表达式优化) | ~80 |

### 1.3 覆盖盲区根本原因

**已修复**: `lexer.rs` 缺失 `ROLE`/`ROLES` 关键字 → 已添加

**仍需修复**: 无

---

## 2. 覆盖率驱动测试设计方法论

本指南结合 GitNexus 执行流分析与 llvm-cov 报告，实现"分析→定位→设计→验证"闭环。

### 2.1 核心理念

```
不要面向行号补测试，要面向业务场景补测试。
用 GitNexus 看清"谁调用谁"，让每个测试覆盖更多真实路径。
中枢节点优先，参数化批量跟进。
```

### 2.2 工作流

```
┌─────────────────────────────────────────────────────────────────┐
│  1. 探索 (Explore)                                               │
│     └── gitnexus flows --entry parse_statement --depth 3        │
│     └── 识别中枢节点与共享路径                                    │
├─────────────────────────────────────────────────────────────────┤
│  2. 清单生成 (Inventory)                                         │
│     └── cargo llvm-cov --text --output-dir /tmp/coverage        │
│     └── 合并流信息 → 优先级表                                    │
├─────────────────────────────────────────────────────────────────┤
│  3. 影响评估 (Impact)                                            │
│     └── gitnexus impact --function parse_xxx                    │
│     └── 确保修改安全                                             │
├─────────────────────────────────────────────────────────────────┤
│  4. 编写测试 (Implement)                                         │
│     └── 组合测试 → 错误路径 → 参数化批量                          │
├─────────────────────────────────────────────────────────────────┤
│  5. 验证 (Verify)                                               │
│     └── cargo llvm-cov 对比增量                                  │
│     └── 记录归档到 Flow 快照                                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. 未覆盖场景聚合清单

### 3.1 高优先级场景 (中枢路径)

#### 场景 1: SET TRANSACTION 隔离级别

**触发 SQL**:
```sql
BEGIN ISOLATION LEVEL READ UNCOMMITTED;
BEGIN ISOLATION LEVEL READ COMMITTED;
BEGIN ISOLATION LEVEL REPEATABLE READ;
BEGIN ISOLATION LEVEL SERIALIZABLE;
BEGIN READ ONLY;
BEGIN READ WRITE;
BEGIN;
```

**覆盖函数**:
- `parse_begin` (lines ~950-1012)
- `parse_isolation_level_value` (lines ~1026-1050)

**组合测试设计**:
```rust
#[test]
fn test_parse_begin_all_isolation_levels() {
    let isolation_levels = vec![
        "READ UNCOMMITTED",
        "READ COMMITTED",
        "REPEATABLE READ",
        "SERIALIZABLE",
    ];
    for level in isolation_levels {
        let sql = format!("BEGIN ISOLATION LEVEL {}", level);
        let result = parse(&sql);
        assert!(result.is_ok(), "Failed: {}", sql);
    }
}
```

#### 场景 2: CREATE/DROP ROLE 层级

**触发 SQL**:
```sql
CREATE ROLE admin;
CREATE ROLE admin WITH PARENT super_admin;
DROP ROLE admin;
```

**覆盖函数**:
- `parse_create_role` (lines ~1086-1115)
- `parse_drop_role` (lines ~3881-3889)

### 3.2 中优先级场景

#### 场景 3: GRANT/REVOKE Role

**触发 SQL**:
```sql
GRANT admin TO user1;
REVOKE admin FROM user1;
```

**覆盖函数**:
- `parse_grant_role` (lines ~4359-4377)
- `parse_revoke_role` (lines ~4391-4406)

#### 场景 4: 表达式函数

**触发 SQL**:
```sql
SELECT POSITION('test' IN 'test_string');
SELECT IF(1 > 0, 'yes', 'no');
SELECT IF(age > 18, 'adult', 'minor') FROM users;
```

**覆盖函数**:
- `parse_position_expression` (lines ~3062-3075)
- `parse_if_expression` (lines ~3092-3120)

### 3.3 低优先级场景 (常量折叠优化器)

#### 场景 5: fold_constants 常量折叠

**触发 SQL** (需要特殊表达式解析路径):
```sql
SELECT 1 + 2;
SELECT TRUE AND FALSE;
SELECT FALSE OR TRUE;
SELECT NOT TRUE;
```

**覆盖函数**:
- `fold_constants` (lines ~629-750)
- `evaluate_binary_op` (lines ~784-850)

---

## 4. GitNexus 执行流分析

### 4.1 识别中枢节点

```bash
npx gitnexus clusters --min-size 5
```

**已知中枢节点**:
- `parse_expression` - 所有表达式解析入口
- `parse_statement` - 所有语句解析入口
- `parse_identifier` - 共享标识符解析
- `parse_literal` - 共享字面量解析

### 4.2 一石多鸟路径设计

**单一测试覆盖多个中枢**:
```sql
-- 场景: 用户管理完整流程
BEGIN ISOLATION LEVEL SERIALIZABLE;
CREATE ROLE admin WITH PARENT super_admin;
GRANT admin TO user1;
SET ROLE admin;
DROP ROLE old_admin;
COMMIT;
```

**覆盖**:
| 函数 | 覆盖分支 |
|------|---------|
| `parse_begin` | SERIALIZABLE 隔离级别 |
| `parse_create_role` | WITH PARENT 语法 |
| `parse_grant_role` | 完整路径 |
| `parse_set_role` | STRING 字面量角色名 |
| `parse_drop_role` | IDENTIFIER 角色名 |
| `parse_commit` | COMMIT 路径 |

### 4.3 影响分析

修改解析器前必查:
```bash
npx gitnexus impact --function parse_where_clause --direction upstream
```

---

## 5. 批量测试生成策略

### 5.1 参数化测试模式

使用 Rust 宏批量生成:
```rust
macro_rules! test_isolation_level {
    ($($name:ident: $level:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let sql = format!("BEGIN ISOLATION LEVEL {}", $level);
                assert!(parse(&sql).is_ok());
            }
        )*
    }
}

test_isolation_level! {
    test_begin_read_uncommitted: "READ UNCOMMITTED",
    test_begin_read_committed: "READ COMMITTED",
    test_begin_repeatable_read: "REPEATABLE READ",
    test_begin_serializable: "SERIALIZABLE",
}
```

### 5.2 自动化回归验证

```bash
#!/bin/bash
# verify_coverage.sh

# 保存基线
cargo llvm-cov --text -p sqlrustgo-parser --output-dir /tmp/before

# 运行测试
cargo test -p sqlrustgo-parser

# 保存当前
cargo llvm-cov --text -p sqlrustgo-parser --output-dir /tmp/after

# 对比
diff /tmp/before/text/index.txt /tmp/after/text/index.txt
```

---

## 6. 测试覆盖验证

### 6.1 运行覆盖率报告

```bash
cargo llvm-cov --text --output-dir /tmp/coverage -p sqlrustgo-parser
```

### 6.2 查看未覆盖行

```bash
cat /tmp/coverage/text/coverage/.../parser.rs.txt | grep "|0|" | head -50
```

### 6.3 覆盖率目标检查

| 阶段 | 目标覆盖率 | 关键里程碑 |
|------|-----------|-----------|
| Phase 1 | 60% | 修复 ROLE 关键字 + 基础 ROLE 测试 |
| Phase 2 | 70% | 隔离级别 + 错误路径 |
| Phase 3 | 80% | 表达式函数 + 组合测试 |
| Phase 4 | 85%+ | 常量折叠 + 参数化批量 |

---

## 7. 已完成的改进

### 7.1 lexer.rs 关键字修复

```rust
// crates/parser/src/lexer.rs (lines 243-244)
"ROLE" => Token::Role,
"ROLES" => Token::Roles,
```

### 7.2 新增测试用例 (11个)

| 测试函数 | 覆盖函数 |
|---------|---------|
| `test_parse_set_role` | `parse_set_role` |
| `test_parse_set_role_with_string` | `parse_set_role` |
| `test_parse_create_role` | `parse_create_role` |
| `test_parse_create_role_with_parent` | `parse_create_role` |
| `test_parse_drop_role` | `parse_drop_role` |
| `test_parse_drop_role_with_string` | `parse_drop_role` |
| `test_parse_grant_role` | `parse_grant_role` |
| `test_parse_revoke_role` | `parse_revoke_role` |
| `test_parse_position_expression` | `parse_position_expression` |
| `test_parse_if_expression` | `parse_if_expression` |
| `test_parse_if_expression_in_where` | `parse_if_expression` |

### 7.3 验证状态

```bash
cargo test -p sqlrustgo-parser    # ✅ 113 passed
cargo clippy -p sqlrustgo-parser # ✅ zero warnings
cargo fmt --check -p sqlrustgo-parser # ✅ clean
```

---

## 8. 下一步行动计划

### 立即执行

1. [ ] **隔离级别批量测试**: 添加 4 个 `BEGIN ISOLATION LEVEL` 测试
2. [ ] **WITH PARENT 语法测试**: 验证角色继承路径
3. [ ] **TRUNCATE TABLE 测试**: 覆盖 `parse_truncate` 函数

### 短期计划

4. [ ] **表达式函数批量**: POSITION, IF, CASE 变体
5. [ ] **错误路径测试**: 无效 SQL 的错误处理
6. [ ] **组合场景测试**: 一石多鸟路径

### 中期计划

7. [ ] **常量折叠测试**: 1+2, TRUE AND FALSE 等
8. [ ] **参数化宏重构**: 减少重复测试代码
9. [ ] **覆盖率回归脚本**: 自动化验证

---

## 9. 参考资源

- **llvm-cov 报告**: `/tmp/parser_coverage_v2/text/`
- **GitNexus 索引**: `npx gitnexus analyze`
- **parser 源码**: `crates/parser/src/parser.rs`
- **覆盖率测试**: `crates/parser/tests/parser_coverage_tests.rs`

---

## 附录 A: 覆盖率报告解读

### llvm-cov 输出格式

```
Filename    Regions    Missed Regions    Cover   Functions  Missed Functions  Executed    Lines     Missed Lines     Cover
--------------------------------------------------------------------------------------------------------------------------------------
parser.rs   6925       3269            52.79%     190       54              71.58%    3932      1805            54.09%
```

### 关键指标

| 指标 | 含义 |
|------|------|
| Regions | 代码区域（分支/条件/循环）总数 |
| Regions Cover | 被执行过的区域百分比 |
| Functions | 函数总数 |
| Functions Executed | 被调用过的函数百分比 |
| Lines | 非空非注释行总数 |
| Lines Cover | 被执行过的行百分比 |

### 目标值

- **Regions 85%+**: 几乎所有分支都被覆盖
- **Functions 90%+**: 所有函数都被测试调用
- **Lines 85%+**: 绝大多数代码行都被执行
