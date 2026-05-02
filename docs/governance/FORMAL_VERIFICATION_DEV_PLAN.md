# PROOF-011~014 形式化验证实施计划

> **Issue**: #117 Follow-up - E2E Formal Verification
> **Status**: Draft
> **Owner**: openclaw
> **Created**: 2026-05-02
> **Version**: 0.1.0

## 1. 当前状态

### 1.1 已有的证明文件

| Proof ID | 证明内容 | 语言 | 形式化文件 | 状态 |
|---------|----------|------|-------------|------|
| PROOF-011 | Type System Safety | Dafny | PROOF-011-type-safety.dfy | 文档存在，未验证 |
| PROOF-012 | WAL ACID Properties | TLA+ | PROOF-012-wal-acid.tla | 文档存在，未验证 |
| PROOF-013 | B+Tree Invariants | Dafny | PROOF-013-btree-invariants.dfy | 文档存在，未验证 |
| PROOF-014 | Query Equivalence | Formulog | PROOF-014-query-equivalence.formalog | 文档存在，未验证 |

### 1.2 缺失的流程

- [ ] 没有运行 Dafny/TLA+/Formulog 验证工具
- [ ] 没有 spec-to-code 映射文档
- [ ] 没有自动化 E2E 流程
- [ ] CI 未集成形式化验证

## 2. 开发计划

### 2.1 Phase 1: 工具安装与脚本开发 (Week 1)

**目标**: 安装验证工具并创建基础脚本

**交付物**:
- `scripts/verify/dafny-verify.sh` - 运行 Dafny 验证
- `scripts/verify/tla-check.sh` - 运行 TLA+ 模型检查
- `scripts/verify/formulog-check.sh` - 运行 Formulog 检查

**任务清单**:

```bash
# T1.1: 安装 Dafny
dotnet tool install -g Dafny

# T1.2: 安装 TLA+ Toolbox
docker pull tlatools/tlatools

# T1.3: 安装 Formulog
pip install formulog

# T1.4: 创建验证脚本
# - scripts/verify/dafny-verify.sh
# - scripts/verify/tla-check.sh  
# - scripts/verify/formulog-check.sh

# T1.5: 测试工具可用性
dafny --version
docker run --rm tlatools/tlatools tlcrun -version
formulog --version
```

### 2.2 Phase 2: PROOF-011 Type Safety 验证 (Week 2)

**目标**: 运行 Dafny 验证并更新证据

**交付物**: 
- 验证报告 (PROOF-011-type-safety.verify Output)
- 更新的 PROOF-011-type-safety.dfy
- PROOF-011.json 包含 formal_verification 字段

**任务清单**:

```bash
# T2.1: 审查 PROOF-011-type-safety.dfy
# - 检查 lemmas 和 theorems
# - 确保可验证

# T2.2: 运行 Dafny 验证
bash scripts/verify/dafny-verify.sh docs/proof/PROOF-011-type-safety.dfy

# T2.3: 修复验证错误 (如有)
# - 修改 .dfy 文件
# - 重新验证

# T2.4: 运行 Rust 测试
cargo test -p sqlrustgo-types

# T2.5: 更新 PROOF-011.json
# 添加 formal_verification 字段:
{
  "formal_verification": {
    "tool": "dafny",
    "version": "4.0.0",
    "command": "dafny verify PROOF-011-type-safety.dfy",
    "result": "passed",
    "verified_at": "2026-05-09"
  }
}
```

### 2.3 Phase 3: PROOF-012 WAL ACID 验证 (Week 2)

**目标**: 运行 TLA+ 模型检查

**交付物**:
- TLA+ 模型检查报告
- PROOF-012.json 更新

**任务清单**:

```bash
# T3.1: 审查 PROOF-012-wal-acid.tla
# - 检查 invariants
# - 确保 TLC 可运行

# T3.2: 运行 TLA+ 模型检查
bash scripts/verify/tla-check.sh SPEC.tla WAL_Recovery

# T3.3: 修复错误 (如有)
# - 修改 .tla 文件
# - 重新检查

# T3.4: 运行 Rust 测试
cargo test -p sqlrustgo-transaction

# T3.5: 更新 PROOF-012.json
```

### 2.4 Phase 4: PROOF-013 B-Tree 验证 (Week 3)

**目标**: 运行 Dafny 验证 B-Tree 不变量

**交付物**:
- 验证报告
- PROOF-013.json 更新

### 2.5 Phase 5: PROOF-014 Query Equivalence (Week 3)

**目标**: 运行 Formulog 检查查询等价性

**交付物**:
- Formulog 检查报告
- PROOF-014.json 更新

### 2.6 Phase 6: CI 集成 (Week 4)

**目标**: 将形式化验证集成到 CI

**交付物**:
- 更新的 `scripts/gate/check_proof.sh`
- Gitea Actions workflow

**任务清单**:

```bash
# T6.1: 更新 scripts/gate/check_proof.sh
# 添加:
# - 运行 dafny verify
# - 运行 tla-check
# - 运行 formulog-check
# - 收集验证报告

# T6.2: 创建 Gitea Actions workflow
# .github/workflows/formal-verification.yml

# T6.3: 测试 Gate R10
bash scripts/gate/check_proof.sh
```

### 2.7 Phase 7: 最终 Gate Pass (Week 4)

**目标**: 完成 R10 验证

**交付物**:
- R10 验证通过报告
- Issue #117 关闭

## 3. 测试设计

### 3.1 验证映射表

| Proof | 形式化验证 | Rust 测试 | 期望结果 |
|-------|-----------|-----------|----------|
| PROOF-011 | `dafny verify` | `cargo test -p sqlrustgo-types` | 两者通过 |
| PROOF-012 | `tlc` | `cargo test -p sqlrustgo-transaction` | 两者通过 |
| PROOF-013 | `dafny verify` | `cargo test -p sqlrustgo-storage` | 两者通过 |
| PROOF-014 | `formulog check` | `cargo test -p sqlrustgo-optimizer` | 两者通过 |

### 3.2 证据收集流程

```
formal_verification tool 运行
        ↓
   output → PROOF-XXX.verify Output
        ↓
   extract result (passed/failed)
        ↓
   update PROOF-XXX.json formal_verification
        ↓
   cargo test 运行
        ↓
   output → test_results
        ↓
   update PROOF-XXX.json evidence
        ↓
   Gate R10 check → final pass/fail
```

## 4. 风险与依赖

### 4.1 风险

| 风险 | 影响 | 缓解 |
|------|------|------|
| Dafny 验证失败 | 需要修改 .dfy | 预留 Week 2 Flex 时间 |
| TLA+ 状态爆炸 | 内存不足 | 简化模型 |
| 工具安装失败 | 流程阻塞 | 使用 Docker |

### 4.2 依赖

- dotnet 6.0+ (Dafny)
- Docker (TLA+)
- Python 3.8+ (Formulog)
- Rust toolchain (cargo test)

## 5. 里程碑与时间线

| 周 | 里程碑 | 关键交付物 |
|----|--------|-----------|
| W1 | M1: 工具安装 | 验证脚本 |
| W1 | M2: 文档 | E2E workflow.md |
| W2 | M3: PROOF-011 | Type safety verified |
| W2 | M4: PROOF-012 | WAL ACID verified |
| W3 | M5: PROOF-013 | B-Tree verified |
| W3 | M6: PROOF-014 | Query equivalence verified |
| W4 | M7: CI Integration | Gate script enhanced |
| W4 | M8: Gate Pass | R10 verified |

---

## 6. 后续步骤

1. **立即**: 创建 Issue (需要 Gitea API write:issue scope)
2. **T+1 week**: 完成 Phase 1 工具安装
3. **T+2 weeks**: 完成 Phase 2-3 验证
4. **T+3 weeks**: 完成 Phase 4-5 验证
5. **T+4 weeks**: 完成 Phase 6-7

---

> **References**:
> - [E2E Workflow](docs/governance/FORMAL_VERIFICATION_E2E.md)
> - [G-02 Proof Registry](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/128)
> - [Proof Files](docs/proof/)
> - [Gate Scripts](scripts/gate/check_proof.sh)