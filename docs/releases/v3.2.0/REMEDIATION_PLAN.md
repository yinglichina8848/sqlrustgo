# SQLRustGo v3.2.0 整改计划

> **版本**: v1.0
> **创建日期**: 2026-05-17
> **基于**: VERIFICATION_REPORT.md
> **状态**: 🔴 待执行

---

## 一、问题摘要

### 1.1 核心问题

| 问题类型 | 数量 | 说明 |
|----------|------|------|
| 缺失测试文件 | 12 个 | P0: 8个, P1: 4个 |
| 门禁脚本不匹配 | 9+ 处 | 引用不存在的测试文件 |
| 功能追踪虚报 | ~25 项 | Matrix 与实际不符 |

### 1.2 影响评估

| 影响项 | 级别 | 说明 |
|--------|------|------|
| RC/GA Gate 阻塞 | 🔴 严重 | 无法完成门禁 |
| 发布风险 | 🔴 严重 | 功能缺陷未被发现 |
| 文档可信度 | 🟡 中等 | 追踪矩阵不可信 |

---

## 二、整改阶段

### Phase 1: 门禁脚本修复 ⏱️ 预计 2h

#### 1.1 问题定位

```bash
# 检查问题引用
grep -n "audit_trail_test\|four_eyes_test" scripts/gate/check_ga_v320.sh
```

#### 1.2 修复任务

| 任务 | 文件 | 修复方案 |
|------|------|----------|
| G-S7 审计链验证 | check_ga_v320.sh | 改为 `gmp_audit_chain_verify_test` |
| four_eyes_test | check_ga_v320.sh | 改为 `gmp_electronic_signature_test` 或创建 |
| 其他缺失引用 | check_ga_v320.sh | 根据实际存在的测试文件修正 |

#### 1.3 验证

```bash
# 验证修复后脚本语法正确
bash -n scripts/gate/check_ga_v320.sh

# 验证引用的测试文件存在
grep -E "test --test" scripts/gate/check_ga_v320.sh | while read line; do
    filename=$(echo $line | grep -oE '[a-z_]+\.rs' | head -1)
    if [ -n "$filename" ] && [ ! -f "tests/${filename}" ]; then
        echo "MISSING: tests/${filename}"
    fi
done
```

---

### Phase 2: P0 测试文件补充 ⏱️ 预计 8h

#### 2.1 测试文件清单

| # | 测试文件 | 对应功能 | 关键验证点 |
|---|----------|----------|------------|
| 1 | gmp_immutable_record_test.rs | GMP-3 Immutable Record | INSERT 成功, UPDATE/DELETE 被拒绝 |
| 2 | gmp_correction_chain_test.rs | GMP-4 Correction Chain | 修正记录创建, 签名验证 |
| 3 | gmp_provenance_test.rs | GMP-5 Provenance | 数据血缘追踪 |
| 4 | gmp_timestamp_test.rs | GMP-6 Trusted Timestamp | RFC3161 时间戳验证 |
| 5 | gmp_hsm_test.rs | GMP-8 HSM Integration | 密钥操作, 签名验证 |
| 6 | gmp_workflow_test.rs | GMP-9 Workflow Engine | 状态转换, 事件触发 |
| 7 | audit_trail_test.rs | GMP-7 审计链验证 | 完整审计链验证 |
| 8 | cold_storage_test.rs | SQL-3 冷存储 | 热数据迁移, S3 操作 |

#### 2.2 模板结构

```rust
//! GMP-{功能号} Test Suite
//!
//! 对应功能: {功能名称}
//! 门禁项: {GA Gate ID}
//!
//! 测试覆盖:
//! - 基础功能验证
//! - 边界条件测试
//! - 错误处理测试

use sqlrustgo::{*};
use std::sync::{Arc, RwLock};

/// 基础功能测试
#[test]
fn test_gmp_{功能小写}_basic() {
    // TODO: 实现基础功能测试
}

/// 边界条件测试
#[test]
fn test_gmp_{功能小写}_edge_cases() {
    // TODO: 实现边界条件测试
}

/// 错误处理测试
#[test]
fn test_gmp_{功能小写}_error_handling() {
    // TODO: 实现错误处理测试
}
```

---

### Phase 3: P1 测试文件补充 ⏱️ 预计 6h

#### 3.1 测试文件清单

| # | 测试文件 | 对应功能 | 关键验证点 |
|---|----------|----------|------------|
| 1 | recursive_cte_test.rs | SQL-1 RECURSIVE CTE | 递归深度限制, 循环检测 |
| 2 | memory_footprint_test.rs | PERF-5 内存优化 | 内存使用监控 |
| 3 | four_eyes_test.rs | GMP-2 电子签名 | 双签审批流程 |
| 4 | ps_*.rs (一组) | SQL-2 Performance Schema | 性能指标收集 |

---

### Phase 4: 追踪矩阵修正 ⏱️ 预计 2h

#### 4.1 修正任务

1. **验证所有源码模块存在性**
2. **验证所有测试文件存在性**
3. **更新功能状态为实际状态**
4. **添加验证人/日期字段**

#### 4.2 修正模板

```markdown
| 功能ID | 功能名称 | 代码状态 | 测试文件 | 测试状态 | 门禁项 | 实际状态 |
|--------|----------|----------|----------|----------|--------|----------|
| GMP-3 | Immutable Record | ✅ 存在 | gmp_immutable_record_test.rs | ✅ 存在 | G-S7 | ✅ |
```

---

### Phase 5: RC/GA Gate 验证 ⏱️ 预计 4h (在大内存机器上)

#### 5.1 门禁执行

```bash
# RC Gate
bash scripts/gate/check_rc_v320.sh

# GA Gate
bash scripts/gate/check_ga_v320.sh
```

#### 5.2 通过标准

| Gate | 通过标准 |
|------|----------|
| RC | R1-R16 ≥ 80% PASS |
| GA | G1-G12 + G-QA + G-S ≥ 90% PASS |

---

## 三、任务分配

### 3.1 分支策略

```
develop/v3.2.0
├── fix/gate-script-mismatch      # Phase 1: 门禁脚本修复
├── fix/missing-p0-tests           # Phase 2: P0 测试补充
├── fix/missing-p1-tests           # Phase 3: P1 测试补充
└── docs/feature-tracking-fix      # Phase 4: 追踪矩阵修正
```

### 3.2 提交规范

```
类型(范围): 描述

feat(gmp): add immutable_record_test for GMP-3
fix(gate): correct audit_trail_test reference in check_ga_v320.sh
docs(matrix): update FEATURE_TRACKING_MATRIX with verified data
```

---

## 四、时间线

| 阶段 | 任务 | 预计时间 | 产出物 |
|------|------|----------|--------|
| Phase 1 | 门禁脚本修复 | 2h | PR 合并 |
| Phase 2 | P0 测试补充 | 8h | 8 个测试文件 + PR |
| Phase 3 | P1 测试补充 | 6h | 4 个测试文件 + PR |
| Phase 4 | 追踪矩阵修正 | 2h | 修正后的 Matrix + PR |
| Phase 5 | RC/GA Gate 验证 | 4h | Gate 报告 |

**总计**: ~22 小时 (分多次 session 完成)

---

## 五、验收标准

### 5.1 Phase 1 验收

```bash
# 所有门禁脚本引用存在
for test in $(grep -oE 'test --test [a-z_]+' scripts/gate/check_ga_v320.sh); do
    filename=$(echo $test | awk '{print $3}')
    [ -f "tests/${filename}.rs" ] || echo "MISSING: $filename"
done
# 预期: 无输出
```

### 5.2 Phase 2 验收

```bash
# P0 测试文件存在
for test in gmp_immutable_record_test gmp_correction_chain_test gmp_provenance_test \
            gmp_timestamp_test gmp_hsm_test gmp_workflow_test audit_trail_test cold_storage_test; do
    [ -f "tests/${test}.rs" ] || echo "MISSING: $test"
done
# 预期: 无输出
```

### 5.3 Phase 3 验收

```bash
# P1 测试文件存在
for test in recursive_cte_test memory_footprint_test four_eyes_test; do
    [ -f "tests/${test}.rs" ] || echo "MISSING: $test"
done
# 预期: 无输出
```

### 5.4 最终验收

```bash
# RC Gate 80%+ 通过
bash scripts/gate/check_rc_v320.sh 2>&1 | grep -E "PASS|FAIL"

# GA Gate 90%+ 通过 (在 z6g4 上执行)
bash scripts/gate/check_ga_v320.sh 2>&1 | grep -E "PASS|FAIL"
```

---

## 六、风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 测试编写遗漏关键场景 | 高 | 参考现有测试模式, 同行 review |
| 门禁脚本修改引入新问题 | 中 | 修改后运行 `bash -n` 语法检查 |
| 大内存机器不可用 | 低 | 预留 2 周缓冲期 |

---

## 七、相关文档

- [核验报告](./VERIFICATION_REPORT.md)
- [门禁规范](../../docs/governance/GATE_SPEC_MASTER.md)
- [测试计划](./TEST_PLAN.md)

---

*计划创建: 2026-05-17*
*维护人: hermes (250系统) → sqlrustgo agent*