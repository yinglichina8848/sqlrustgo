# v3.0.0 GA Gate 全面审查报告

> **版本**: 1.0
> **日期**: 2026-05-08
> **审查依据**: gate_spec_v300.md §六（G-Gate）+ check_ga_v300.sh 脚本对比
> **目的**: 建立 GA 阶段差距遗留问题清单，延续到 v3.1.0

---

## 一、GA Gate 门禁项对比（规范 vs 实现）

### 1.1 规范定义 vs 脚本实现

| 规范项 | 规范要求 | 脚本实现 | 差距描述 |
|--------|----------|----------|----------|
| G1 Build | `cargo build --release --workspace` 无错误 | GA-1 | ✅ 一致 |
| G2 Test | `cargo test --all-features` 100% 通过 | GA-2 | ✅ 一致 |
| G3 Clippy | 零警告 | GA-4 | ✅ 一致 |
| G4 Format | 无格式错误 | GA-5 | ✅ 一致 |
| G5 Coverage | ≥85% | GA-6 | ✅ 一致 |
| G6 Security | 无漏洞 | GA-7 | ⚠️ 规范声明"无漏洞"，但 `cargo audit` R-05 未修复 |
| G7 Point Select QPS | ≥10,000 QPS | **未实现** | ❌ 差距：脚本无 QPS 实际测量 |
| G8 UPDATE QPS | ≥5,000 QPS | **未实现** | ❌ 差距：脚本无 QPS 实际测量 |
| G9 DELETE QPS | ≥2,000 QPS | **未实现** | ❌ 差距：脚本无 QPS 实际测量 |
| G10 TPC-H SF=1 | 22/22 无 OOM | GA-9 | ✅ 一致 |
| G11 SQL Corpus | ≥98% | GA-14 | ⚠️ 规范要求≥98%，脚本检查≥95% |
| G12 (规范有，脚本无) | B-S 稳定性测试 | **未实现** | ❌ 差距：B-S1~B-S5 未纳入 GA |
| G13 (规范有，脚本无) | MySQL Protocol Test | **未实现** | ❌ 差距：mysql:5.7 握手测试未实现 |

### 1.2 脚本有但规范无

| 脚本项 | 规范状态 | 说明 |
|--------|----------|------|
| GA-3 | 规范无此检查项 | 集成测试脚本存在但未在 gate_spec_v300.md 定义 |
| GA-8 | 规范无此检查项 | 文档链接检查，gate_spec 中仅在 R7 定义 |
| GA-10 | 规范无此检查项 | 性能回归检查 |
| GA-11 | 规范无此检查项 | 形式化证明 ≥10 个文件 |
| GA-12 | 规范无此检查项 | Sysbench gate |
| GA-13 | 规范无此检查项 | 发布文档完整性 |
| GA-15 | 规范无此检查项 | 版本一致性 |

---

## 二、具体差距分析

### 2.1 G6 Security — R-05 未修复

**问题**: `cargo audit` 返回 R-05（advisory #21327，semver vulnerability），脚本用 `|| true` 静默忽略。

**规范要求**: gate_spec_v300.md G6 定义为"无漏洞"，但实际存在已知漏洞未修复。

**当前状态**: `cargo audit || true` 始终 PASS，掩盖了真实漏洞。

**修复方案**:
```bash
# 修改 check_ga_v300.sh GA-7 部分
cargo audit 2>/dev/null  # 移除 || true
# 或要求 cargo audit 输出中不包含 R-05 关键字
```

**遗留问题编号**: GA-GAP-01

---

### 2.2 G7/G8/G9 — 性能 QPS 未测量

**问题**: gate_spec_v300.md 定义了三个 QPS 门禁（G7: Point Select ≥10K, G8: UPDATE ≥5K, G9: DELETE ≥2K），但 `check_ga_v300.sh` 中 GA-10 只是调用 `check_regression.sh` 检查回归，没有实际运行 `cargo bench` 并解析 QPS 数值。

**规范要求**: Point Select QPS ≥10,000, UPDATE QPS ≥5,000, DELETE QPS ≥2,000。

**当前状态**: 
- DEVELOPMENT_PLAN.md 报告 Point SELECT QPS = 7,312（低于 10K 目标）
- UPDATE QPS = 42,427 ✅
- DELETE QPS = 62,352 ✅

**修复方案**: GA 门禁脚本需要包含实际 QPS 测量逻辑：
```bash
# 解析 cargo bench 输出
POINT_QPS=$(cargo bench -- point_select 2>&1 | grep -oE '[0-9]+\.[0-9]+ ops/s' | ...)
UPDATE_QPS=$(cargo bench -- update_simple 2>&1 | ...)
DELETE_QPS=$(cargo bench -- delete_simple 2>&1 | ...)
```

**遗留问题编号**: GA-GAP-02

---

### 2.3 G11 SQL Corpus — 阈值不一致

**问题**: gate_spec_v300.md G11 定义"≥98%"，但 check_ga_v300.sh GA-14 检查"≥95%"。

**影响**: 当前 test_sql_corpus_all = 94.1%，如果按规范 ≥98% 判定则 FAIL。

**修复方案**: 统一为 ≥98%（规范值），或修改规范为 ≥95% 并记录偏差。

**遗留问题编号**: GA-GAP-03

---

### 2.4 B-S 稳定性测试未纳入 GA

**问题**: gate_spec_v300.md §六 G-Gate 入口条件要求"所有已知问题已关闭"，BETA_GATE_MASTER_CONTROL.md 定义了 B-S1~B-S5 稳定性测试，但 check_ga_v300.sh 没有包含这些检查。

**当前 B-S 状态**（根据 BETA_GATE_MASTER_CONTROL.md）:
- B-S1: concurrency_stress_test — 需验证
- B-S2: crash_recovery_test — 需验证
- B-S3: long_run_stability_test — 需验证
- B-S4: wal_integration_test — 需验证
- B-S5: network_tcp_smoke_test — 需验证

**修复方案**: GA 门禁脚本应包含 B-S1~B-S5 检查，或明确豁免。

**遗留问题编号**: GA-GAP-04

---

### 2.5 G13 MySQL Protocol Test 未实现

**问题**: gate_spec_v300.md §五 R12 定义了"MySQL Protocol Test: mysql:5.7 容器握手测试"，但 check_ga_v300.sh 没有包含此检查。

**修复方案**: 在 GA 门禁中实现：
```bash
docker run --rm mysql:5.7 mysql -h <host> -P <port> -u root -e "SELECT 1"
```

**遗留问题编号**: GA-GAP-05

---

### 2.6 GA-3 Integration tests — 脚本未验证退出码

**问题**: check_ga_v300.sh GA-3 调用 `bash scripts/test/run_integration.sh --quick`，但 `check()` 函数只判断退出码，没有输出"Integration tests" PASS/FAIL 的具体原因。

**当前状态**: 脚本存在，但未验证是否真正通过。

**遗留问题编号**: GA-GAP-06

---

### 2.7 GA-11 Formal proofs — 数量不足

**问题**: gate_spec_v300.md 定义"≥10 个形式化证明文件"，但只检查 .json 文件数量，不验证证明本身的有效性。

**当前状态**: 14 个 .json 文件（满足 ≥10），但：
- PROOF-011-type-safety.json — 格式可能有问题
- PROOF-013-btree-invariants.json — 需验证
- 多个 .dfy (Dafny) 和 .tla (TLA+) 文件未纳入计数

**修复方案**: 扩展检查范围到 .dfy、.tla、.formalog、.formulog 文件。

**遗留问题编号**: GA-GAP-07

---

### 2.8 GA-13 Release Documentation — 缺失文档

**问题**: check_ga_v300.sh 检查以下文档存在性，但有 3 个不存在：

| 缺失文档 | 状态 |
|----------|------|
| `docs/releases/v3.0.0/INSTALL.md` | ❌ 不存在 |
| `docs/releases/v3.0.0/DEPLOYMENT_GUIDE.md` | ❌ 不存在 |
| `docs/releases/v3.0.0/QUICK_START.md` | ❌ 不存在 |

**规范要求**: gate_spec_v300.md G13（通过 R7 子项定义）要求发布文档完整。

**修复方案**: 创建这三个缺失文档，或在 gate_spec_v300.md 中移除对它们的强制要求。

**遗留问题编号**: GA-GAP-08

---

### 2.9 文档与脚本不一致 — GA-8 未在规范定义

**问题**: gate_spec_v300.md §六 G-Gate 检查清单（G1~G11）没有包含 GA-8（文档链接检查），但 check_ga_v300.sh 实现了 GA-8。

**规范 SSOT 声明**: gate_spec_v300.md 是唯一权威来源，check_ga_v300.sh 中定义的检查项不应独立于规范存在。

**修复方案**: 
- 选项 A：将 GA-8 添加到 gate_spec_v300.md G-Gate 检查清单
- 选项 B：从 check_ga_v300.sh 移除 GA-8（因为 R7 已覆盖）

**遗留问题编号**: GA-GAP-09

---

## 三、GA-Gate 差距遗留问题总清单

| 编号 | 类别 | 差距描述 | 规范引用 | 当前状态 | 修复优先级 |
|------|------|----------|----------|----------|------------|
| GA-GAP-01 | Security | R-05 (semver) 未修复，`cargo audit || true` 掩盖漏洞 | G6 | cargo audit 有漏洞未修复 | P1 |
| GA-GAP-02 | Performance | G7/G8/G9 QPS 门禁未实际测量，Point SELECT 7,312 < 10,000 | G7/G8/G9 | 脚本无 QPS 测量 | P0 |
| GA-GAP-03 | SQL Compat | SQL Corpus 规范≥98%，脚本≥95%，不一致 | G11 | 94.1% 同时不满足两者 | P0 |
| GA-GAP-04 | Stability | B-S1~B-S5 稳定性测试未纳入 GA Gate | G12(隐含) | 未在 GA 中检查 | P1 |
| GA-GAP-05 | Protocol | MySQL Protocol Test 未实现 | G13(R12) | 缺失 | P2 |
| GA-GAP-06 | Integration | run_integration.sh 未验证退出码 | GA-3 | 脚本存在但无验证 | P2 |
| GA-GAP-07 | Proof | Formal proofs 仅检查 .json，未覆盖 .dfy/.tla | GA-11 | 14 个 .json，计数正确 | P2 |
| GA-GAP-08 | Documentation | INSTALL/DEPLOYMENT_GUIDE/QUICK_START.md 缺失 | GA-13 | 3 个文档不存在 | P1 |
| GA-GAP-09 | Governance | GA-8 未在 gate_spec_v300.md 定义 | GA-8 | 脚本有但规范无 | P3 |

---

## 四、v3.1.0 必需修复项（来自 GA 审查）

> 基于 GA_GATE_AUDIT.md §三建立

### 4.1 P0 — 阻塞 GA

| 遗留编号 | 任务 | 验收条件 |
|----------|------|----------|
| GA-GAP-02 | 实现 G7/G8/G9 QPS 实际测量 | `cargo bench -- point_select` 输出 ≥10,000 ops/s |
| GA-GAP-03 | 统一 SQL Corpus 阈值为 ≥98% | `test_sql_corpus_all` ≥98%，当前 94.1% |

### 4.2 P1 — 阻塞 RC

| 遗留编号 | 任务 | 验收条件 |
|----------|------|----------|
| GA-GAP-01 | 修复 R-05 semver 漏洞或申请豁免 | `cargo audit` 输出不包含 R-05，或 Architect 批准豁免 |
| GA-GAP-08 | 创建缺失的 3 个文档 | INSTALL.md、DEPLOYMENT_GUIDE.md、QUICK_START.md 存在且内容完整 |
| GA-GAP-04 | 将 B-S1~B-S5 稳定性测试纳入 GA Gate | check_ga_v300.sh 包含 B-S1~B-S5 检查 |

### 4.3 P2 — 建议完成

| 遗留编号 | 任务 | 验收条件 |
|----------|------|----------|
| GA-GAP-05 | 实现 MySQL Protocol Test | `docker run --rm mysql:5.7 mysql -h <host> -e "SELECT 1"` 成功 |
| GA-GAP-06 | 修复 run_integration.sh 退出码验证 | `bash scripts/test/run_integration.sh --quick` 退出码为 0 |
| GA-GAP-07 | 扩展 formal proofs 检查到 .dfy/.tla | check_ga_v300.sh GA-11 计数 ≥10 个文件（所有格式） |

### 4.4 P3 — 规范对齐

| 遗留编号 | 任务 | 验收条件 |
|----------|------|----------|
| GA-GAP-09 | 将 GA-8 添加到 gate_spec_v300.md 或从脚本移除 | gate_spec_v300.md 与 check_ga_v300.sh GA-8 定义一致 |

---

## 五、check_ga_v300.sh 增强需求

### 5.1 必需添加的检查

```bash
# GA-GAP-02: 实际 QPS 测量
# 解析 cargo bench 输出，验证 Point Select ≥10K, UPDATE ≥5K, DELETE ≥2K

# GA-GAP-04: B-S 稳定性测试
# 添加 B-S1~B-S5 检查（与 check_beta_v300.sh 一致）

# GA-GAP-01: 移除 || true，真实检查 cargo audit
```

### 5.2 必需修复的检查

```bash
# GA-GAP-03: SQL Corpus 阈值改为 ≥98%
CORPUS_PCT >= 98

# GA-GAP-08: 文档检查（已有 GA-13，验证是否通过）
```

### 5.3 增强输出格式

与 check_beta_v300.sh 一致的失败输出：

```bash
if [ $BLOCKERS -gt 0 ]; then
    echo ""
    echo "=== 未通过项详情 ==="
    for reason in "${FAIL_REASONS[@]}"; do
        echo "  - $reason"
    done
    echo ""
    echo "=== 建议行动 ==="
    echo "  1. 为每个 BLOCKER 创建 Gitea Issue（milestone: v3.0.0-ga）"
    echo "  2. 在 docs/releases/v3.0.0/GA_GATE_AUDIT.md 中登记失败项"
    echo "  3. 修复后重新运行 check_ga_v300.sh"
    echo "  4. 如当前版本无法修复，将任务延续到 v3.1.0"
fi
```

---

## 六、gate_spec_v300.md 修正建议

### 6.1 G-Gate 检查清单修正

将 G7/G8/G9 从"命令存在"改为"实际测量"：

```
|| G7 | Point Select QPS | `cargo bench -- point_select` | ≥10,000 ops/s | 实际解析 bench 输出 ||
|| G8 | UPDATE QPS | `cargo bench -- update_simple` | ≥5,000 ops/s | 实际解析 bench 输出 ||
|| G9 | DELETE QPS | `cargo bench -- delete_simple` | ≥2,000 ops/s | 实际解析 bench 输出 ||
```

### 6.2 G11 阈值修正

```
|| G11 | SQL Corpus | `cargo test -p sqlrustgo-sql-corpus` | ≥98% | 与 check_ga_v300.sh GA-14 一致 ||
```

### 6.3 新增 G12/G13

```
|| G12 | B-S 稳定性测试 | B-S1~B-S5 全部 PASS | check_beta_v300.sh B-S 项 ||
|| G13 | MySQL Protocol | mysql:5.7 容器握手 | docker run --rm mysql:5.7 mysql ... ||
```

---

## 七、追踪机制

```
GA 门禁检查 FAIL
    ↓
识别 GA-GAP-XX 编号
    ↓
在 GA_GATE_AUDIT.md §三登记
    ↓
在 v3.1.0 DEVELOPMENT_PLAN.md §6 建立延续映射
    ↓
创建 Issue（milestone: v3.1.0-beta/rc）
    ↓
修复完成 → PR 合并 → 验证门禁 PASS
    ↓
更新 GA_GATE_AUDIT.md 状态为 CLOSED
```

---

*本文档由 hermes agent 创建，基于 gate_spec_v300.md §六 vs check_ga_v300.sh 对比分析。*
*最后更新: 2026-05-08*
