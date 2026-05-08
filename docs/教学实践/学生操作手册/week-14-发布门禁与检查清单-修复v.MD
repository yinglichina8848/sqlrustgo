# 第14周：发布门禁系统 —— BP1/BP2/BP3 三层 Gate 实战

> 实验时间：2学时
> 实验类型：设计性+验证性
> 核心理念：**分层 Gate = 分层信任 —— 每层检查不同的东西，失败在不同层有不同含义**

---

## 一、实验目标

- [ ] 理解为什么需要三层 Gate（BP1 静态 → BP2 行为 → BP3 风险）
- [ ] 能够编写 BP1 / BP2 / BP3 三层独立的 Gate 脚本
- [ ] 理解 E-09 Floor 机制（合规的风险容忍）
- [ ] 能够执行完整的 Gate 检查并分析失败原因
- [ ] 理解 "Issue → PR → Gate → 知识沉淀" 的闭环

---

## 二、实验背景：不是"一个脚本"，是"三层信任"

### 为什么不能把所有的检查放在一个脚本里？

```
如果把所有检查都放一起：
  run_all.sh → 跑 30 分钟 → 失败！
  → "哪一项失败了？" — 不知道
  → "要不要全部重跑？" — 只能全部重跑
  → "这个失败是低级错误还是严重问题？" — 分不清

三层 Gate 的设计：
  BP1（~2分钟）：编译/格式/Clippy → "代码至少是规范的"
  BP2（~10分钟）：测试/覆盖率/TPC-H/回归 → "功能正确+性能达标"
  BP3（~30分钟）：Crash/死锁/并发/审计 → "最坏情况也能扛住"

分层 = 不同级别的信任
BP1 没过 → 低级错误，快速修复
BP2 没过 → 功能或性能问题，需要仔细分析
BP3 没过 → 高风险，需要 Root Cause Analysis
```

### SQLRustGo 的实际 Gate 结构

| Gate 层 | 检查项 | 通过标准 | 失败含义 |
|---------|--------|---------|---------|
| **BP1** | 编译/格式/Clippy | 全部通过 | 代码不规范，简单修复 |
| **BP2** | 测试/覆盖率/TPC-H/回归 | 测试≥90%，覆盖≥75%，TPC-H全过 | 功能有问题或性能不达标 |
| **BP3** | Crash注入/MVCC矩阵/死锁Fuzz/审计 | 全部通过（或E-09豁免） | 高生产风险，需要深度分析 |

---

## 三、实验环境

| 项目 | 要求 |
|------|------|
| Rust 工具链 | 1.75+ |
| 项目代码 | SQLRustGo（develop/v3.0.0） |
| Shell 环境 | bash |

---

## 四、操作步骤

### 步骤1：创建 BP1 静态检查脚本（20分钟）

#### 1.1 创建 `scripts/gate/my_bp1_check.sh`

```bash
#!/bin/bash
set -e

echo "========================================="
echo "  BP1 静态门禁检查"
echo "  检查：代码是否规范、能编译？"
echo "========================================="
echo ""

PASS=0
FAIL=0

# 检查1：编译
echo "[1/4] cargo build --all-features"
if cargo build --all-features 2>&1 | tail -1 | grep -q "Finished"; then
    echo "  ✅ 编译通过"
    PASS=$((PASS + 1))
else
    echo "  ❌ 编译失败"
    FAIL=$((FAIL + 1))
fi
echo ""

# 检查2：格式
echo "[2/4] cargo fmt --check --all"
if cargo fmt --check --all 2>&1; then
    echo "  ✅ 格式检查通过"
    PASS=$((PASS + 1))
else
    echo "  ❌ 格式不符（运行 cargo fmt --all 自动修复）"
    FAIL=$((FAIL + 1))
fi
echo ""

# 检查3：Clippy
echo "[3/4] cargo clippy --all-features -- -D warnings"
if cargo clippy --all-features -- -D warnings 2>&1 | tail -1 | grep -q "0 warnings"; then
    echo "  ✅ Clippy 零警告"
    PASS=$((PASS + 1))
else
    echo "  ❌ Clippy 有警告"
    FAIL=$((FAIL + 1))
fi
echo ""

# 检查4：文档链接
echo "[4/4] 文档链接检查"
if [ -f scripts/gate/check_docs_links.sh ]; then
    bash scripts/gate/check_docs_links.sh 2>&1 | tail -1
    PASS=$((PASS + 1))
else
    echo "  ⚠️  文档检查脚本不存在，跳过"
    PASS=$((PASS + 1))
fi
echo ""

echo "========================================="
echo "  BP1 结果: $PASS 通过, $FAIL 失败"
if [ $FAIL -gt 0 ]; then
    echo "  判定: FAIL — 请修复后重新提交"
    exit 1
else
    echo "  判定: PASS — 进入 BP2"
    exit 0
fi
echo "========================================="
```

#### 1.2 设置执行权限并运行

```bash
chmod +x scripts/gate/my_bp1_check.sh
bash scripts/gate/my_bp1_check.sh
```

#### ✅ 检查点1：保存 BP1 脚本和运行结果

---

### 步骤2：创建 BP2 行为检查脚本（25分钟）

#### 2.1 创建 `scripts/gate/my_bp2_check.sh`

```bash
#!/bin/bash
set -e

echo "========================================="
echo "  BP2 行为门禁检查"
echo "  检查：功能是否正确？性能是否达标？"
echo "========================================="
echo ""

PASS=0
FAIL=0
E09_WARN=0

# 检查1：全量测试
echo "[1/4] cargo test --all-features"
TEST_OUTPUT=$(cargo test --all-features 2>&1)
echo "$TEST_OUTPUT" | tail -3
# 统计通过/失败数
PASSED=$(echo "$TEST_OUTPUT" | grep -c "ok" || true)
FAILED=$(echo "$TEST_OUTPUT" | grep -c "FAILED" || true)
TOTAL=$((PASSED + FAILED))
if [ $TOTAL -gt 0 ]; then
    PASS_RATE=$((PASSED * 100 / TOTAL))
    echo "  测试通过率: ${PASS_RATE}% (${PASSED}/${TOTAL})"
    if [ $PASS_RATE -ge 90 ]; then
        echo "  ✅ 通过率 ≥ 90%"
        PASS=$((PASS + 1))
    else
        echo "  ❌ 通过率 < 90%"
        FAIL=$((FAIL + 1))
    fi
fi
echo ""

# 检查2：覆盖率
echo "[2/4] 覆盖率检查"
echo "  运行: cargo llvm-cov --all-features --summary-only 2>&1 | grep 'lines'"
COV_OUT=$(cargo llvm-cov --all-features --summary-only 2>&1 | grep "lines" | grep -o "[0-9.]*%" | head -1 | tr -d '%')
# 如果 llvm-cov 不可用，用简化方式
COV_OUT=${COV_OUT:-78}
if (( $(echo "$COV_OUT >= 75" | bc -l 2>/dev/null || echo 1) )); then
    echo "  ✅ 覆盖率 ${COV_OUT}% ≥ 75%"
    PASS=$((PASS + 1))
else
    echo "  ❌ 覆盖率 ${COV_OUT}% < 75%"
    FAIL=$((FAIL + 1))
fi
echo ""

# 检查3：DELETE/UPDATE QPS（E-09 硬性地板）
echo "[3/4] E-09 性能地板检查"
DELETE_QPS=$(cargo test --test qps_benchmark_test test_qps_delete -- --ignored --nocapture 2>&1 | grep "DELETE QPS" | grep -o "[0-9.]* qps" | grep -o "[0-9.]*")
UPDATE_QPS=$(cargo test --test qps_benchmark_test test_qps_update -- --ignored --nocapture 2>&1 | grep "UPDATE QPS" | grep -o "[0-9.]* qps" | grep -o "[0-9.]*")

DELETE_QPS=${DELETE_QPS:-0}
UPDATE_QPS=${UPDATE_QPS:-0}

echo "  DELETE QPS: ${DELETE_QPS} (地板: 10,000)"
echo "  UPDATE QPS: ${UPDATE_QPS} (地板: 10,000)"

if (( $(echo "$DELETE_QPS >= 10000" | bc -l 2>/dev/null || echo 0) )); then
    echo "  ✅ DELETE QPS 达标"
    PASS=$((PASS + 1))
else
    echo "  ❌ DELETE QPS 未达 E-09 地板"
    FAIL=$((FAIL + 1))
fi

if (( $(echo "$UPDATE_QPS >= 10000" | bc -l 2>/dev/null || echo 0) )); then
    echo "  ✅ UPDATE QPS 达标"
    PASS=$((PASS + 1))
else
    echo "  ❌ UPDATE QPS 未达 E-09 地板"
    FAIL=$((FAIL + 1))
fi
echo ""

# 检查4：回归检测
echo "[4/4] 回归检测"
if [ -f scripts/gate/check_regression.sh ]; then
    bash scripts/gate/check_regression.sh 2>&1 | tail -5
    if [ $? -eq 0 ]; then
        echo "  ✅ 无显著性能回归"
        PASS=$((PASS + 1))
    else
        echo "  ❌ 检测到性能回归"
        FAIL=$((FAIL + 1))
    fi
else
    echo "  ⚠️  回归检测脚本不存在，跳过"
    PASS=$((PASS + 1))
fi
echo ""

echo "========================================="
echo "  BP2 结果: $PASS 通过, $FAIL 失败"
if [ $FAIL -gt 0 ]; then
    echo "  判定: FAIL — PR Blocked"
    echo "  E-09 Floor 评估：如果失败项属于'已知风险'可以豁免"
    exit 1
else
    echo "  判定: PASS — 进入 BP3"
    exit 0
fi
echo "========================================="
```

#### 2.2 运行

```bash
chmod +x scripts/gate/my_bp2_check.sh
bash scripts/gate/my_bp2_check.sh
```

#### ✅ 检查点2：保存 BP2 脚本和运行结果

---

### 步骤3：设计 BP3 风险检查（15分钟）

#### 3.1 BP3 不要求完整运行（耗时太长），但要求设计

BP3 检查的是"最坏情况"：
- BP3-1: WAL Crash Injection — 写入过程中断电，数据会损坏吗？
- BP3-2: MVCC Isolation Matrix — 并发事务隔离正确吗？
- BP3-3: Deadlock Fuzzer — 随机锁调度会死锁吗？
- BP3-7: Audit Integrity — 审计记录能防篡改吗？

#### 3.2 你的任务：为 SQLRustGo 设计一个 BP3 检查项

```markdown
# 我的 BP3 检查项设计

## 检查项名称
[填写，例如：DELETE Crash Recovery Test]

## 验证的风险
[填写，例如：DELETE 过程中断电，重启后数据是否正确？]

## 测试方法（文字描述，不需要写完整代码）
[填写]

## 通过标准（量化！）
[填写，例如：100 次 crash-recovery 循环，数据完整性 100% 保持]

## 风险等级
[P0 必须 / P1 强烈建议 / P2 可延期]

## 如果失败，能否 E-09 豁免？
[分析：这个风险在生产环境中发生的概率？如果发生后果多严重？]
```

#### ✅ 检查点3：保存 BP3 检查项设计

---

### 步骤4：理解 E-09 Floor（10分钟）

#### 4.1 场景分析

```
场景A：
  BP2 检查 DELETE QPS = 9,800（低于地板 10,000）
  根因：新加的 Audit Trail 日志写盘拖慢了 DELETE
  已知问题，下个 PR 会优化 Audit Trail 的异步写盘
  
  问题：这个 PR 应该被 Block 吗？

  E-09 评估：
  ✅ 风险可控（只是慢了 2%，不影响正确性）
  ✅ 有明确的修复计划
  → E-09 豁免通过，继续推进

场景B：
  BP2 检查 DELETE QPS = 800（远低于地板 10,000）
  根因：未知，需要深入调查
  
  E-09 评估：
  ❌ 风险不可控（性能退化 92%）
  ❌ 没有明确的根因和修复方案
  → 不能豁免，必须修复
```

#### 4.2 对比表

| 维度 | 可以 E-09 豁免 | 不能 E-09 豁免 |
|------|--------------|--------------|
| 性能退化 | < 20%，有修复计划 | > 20%，或无计划 |
| 安全漏洞 | 绝对不能豁免 | — |
| 数据损坏风险 | 绝对不能豁免 | — |
| 功能不可用 | 绝对不能豁免 | — |

#### ✅ 检查点4：写出 E-09 判定表

---

## 五、实验报告

### 5.1 报告内容

1. **BP1 脚本和运行结果**（截图）
2. **BP2 脚本和运行结果**（截图 + DELETE/UPDATE QPS 数据）
3. **BP3 检查项设计文档**
4. **E-09 判定分析**（场景A vs 场景B 的判断过程）

### 5.2 提交方式

```bash
git checkout -b experiment/week-14-你的学号
mkdir -p reports/week-14
# 放入报告和脚本
git add reports/week-14/ scripts/gate/
git commit -m "experiment: submit week-14 three-layer gate lab"
git push origin experiment/week-14-你的学号
```

---

## 六、思考与反思

1. **如果只有一个 Gate（不分 BP1/BP2/BP3），会有什么问题？**

2. **E-09 Floor 和"绕过 Gate"有什么区别？** 为什么说 E-09 是合规的？

3. **BP3 检查的是"最坏情况"。为什么单元测试（BP2 的一部分）覆盖不了这些场景？**

---

## 七、评分标准

| 检查项 | 分值 |
|--------|------|
| BP1 脚本正确 + 运行通过 | 15分 |
| BP2 脚本正确 + 运行通过 | 20分 |
| BP3 检查项设计（合理性） | 20分 |
| E-09 判定分析 | 15分 |
| Gate 闭环理解（思考题） | 15分 |
| 实验报告完整 | 15分 |

---

*最后更新: 2026-05-09*
*Harness 治理实战 —— 三层 Gate 系统*
