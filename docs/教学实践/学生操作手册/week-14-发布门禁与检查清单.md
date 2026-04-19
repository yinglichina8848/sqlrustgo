# 第14周：发布门禁与检查清单

> 实验时间：2学时（🔧 修车技能第2周）
> 实验类型：设计性+验证性

---

## 一、实验目标

- [ ] 理解发布门禁概念
- [ ] 能够设计门禁检查
- [ ] 能够编写门禁脚本
- [ ] 能够执行RC版本验收

---

## 二、实验环境

| 项目 | 要求 |
|------|------|
| GitHub账号 | 需要仓库管理权限 |
| 项目代码 | SQLRustGo |

---

## 三、操作步骤

### 步骤1：设计门禁检查清单（20分钟）

#### 1.1 门禁检查清单模板

创建 `docs/releases/v2.6.0/RELEASE_GATE_CHECKLIST.md`：

```markdown
# v2.6.0 发布检查清单

## 代码门禁

| 检查项 | 命令 | 结果 | 日期 | 签名 |
|--------|------|------|------|------|
| 编译 | cargo build --release | ⬜ | - | |
| 测试 | cargo test --all-features | ⬜ | - | |
| Clippy | cargo clippy | ⬜ | - | |
| 格式化 | cargo fmt --check | ⬜ | - | |

## 质量门禁

| 检查项 | 目标 | 实际 | 结果 |
|--------|------|------|------|
| 测试覆盖率 | ≥80% | ? | ⬜ |
| 代码质量评级 | A | ? | ⬜ |

## 安全门禁

| 检查项 | 结果 | 日期 | 签名 |
|--------|------|------|------|
| cargo audit | ⬜ | - | |
| 安全扫描 | ⬜ | - | |

## 功能门禁

| 检查项 | 结果 | 备注 |
|--------|------|------|
| 集成测试 | ⬜ | |
| 回归测试 | ⬜ | |

## 性能门禁

| 检查项 | 目标 | 实际 | 结果 |
|--------|------|------|------|
| 查询响应时间 | <100ms | ? | ⬜ |
| 并发能力 | >1000 QPS | ? | ⬜ |

## 文档门禁

| 检查项 | 结果 |
|--------|------|
| README完善 | ⬜ |
| API文档100% | ⬜ |
| 用户手册 | ⬜ |

## 批准

- [ ] 技术负责人批准：__________
- [ ] 产品负责人批准：__________
- [ ] 发布经理批准：__________
```

#### ✅ 检查点1：保存门禁清单

---

### 步骤2：编写门禁脚本（30分钟）

#### 2.1 创建门禁脚本

创建 `scripts/run_gates.sh`：

```bash
#!/bin/bash
set -e

echo "=== Running Gate Checks ==="

# Code Gates
echo "[1/6] Building..."
cargo build --release

echo "[2/6] Running tests..."
cargo test --all-features

echo "[3/6] Running Clippy..."
cargo clippy --all-features -- -D warnings

echo "[4/6] Checking format..."
cargo fmt --check --all

# Quality Gates
echo "[5/6] Checking coverage..."
cargo tarpaulin --out Xml --packages parser,executor,storage

# Security Gates
echo "[6/6] Running security audit..."
cargo audit

echo "=== All Gates Passed ==="
```

#### 2.2 设置执行权限

```bash
chmod +x scripts/run_gates.sh
```

#### 2.3 运行门禁脚本

```bash
./scripts/run_gates.sh
```

#### ✅ 检查点2：记录门禁运行结果

---

### 步骤3：创建发布检查清单文件（15分钟）

#### 3.1 创建版本发布目录

```bash
mkdir -p docs/releases/v2.6.0
```

#### 3.2 填写检查清单

根据门禁脚本运行结果，填写 `RELEASE_GATE_CHECKLIST.md`。

#### 3.3 提交检查清单

```bash
git add docs/releases/v2.6.0/
git commit -m "docs: add v2.6.0 release gate checklist"
```

#### ✅ 检查点3：保存检查清单

---

### 步骤4：执行RC版本验收（15分钟）

#### 4.1 确保代码最新

```bash
git checkout develop/v2.6.0
git pull origin develop/v2.6.0
```

#### 4.2 运行完整门禁

```bash
./scripts/run_gates.sh
```

#### 4.3 记录验收结果

| 检查项 | 状态 | 备注 |
|--------|------|------|
| 编译 | ✅ | |
| 测试 | ✅ | |
| Clippy | ✅ | |
| 格式化 | ✅ | |
| 覆盖率 | ✅ | 82% |
| 安全扫描 | ✅ | |

#### ✅ 检查点4：记录RC验收结果

---

## 四、实验报告

### 4.1 报告内容

1. **门禁检查清单**
2. **门禁脚本**
3. **门禁执行结果**
4. **发现的问题**

### 4.2 提交方式

```bash
git checkout -b experiment/week-14-你的学号
mkdir -p reports/week-14
# 放入报告
git add reports/week-14/
git commit -m "experiment: submit week-14 report"
git push origin experiment/week-14-你的学号
```

---

## 五、思考与反思

### 🎯 本周能力阶段：Harness Engineering（规则约束工程）

**本周重点**：设计完整的规则约束体系

### 问题1：优化与改进
- 我设计的门禁规则，哪些是"必须"的？哪些是"最好"的？
- 如何让规则更清晰、可执行、可验证？

### 问题2：AI不可替代的能力
- AI能理解"这个功能虽然能work，但还不够好发布"这个判断吗？
- 人类的"发布决策"考虑了哪些AI无法理解的维度？

### 问题3：自我提升
- 我是否开始学会设计"完整的规则体系"？
- 我是否能区分"技术问题"和"业务决策"？

---

## 六、评分标准

| 检查项 | 分值 |
|--------|------|
| 门禁检查清单设计 | 20分 |
| 门禁脚本编写 | 25分 |
| 门禁执行结果 | 25分 |
| 问题记录 | 15分 |
| 实验报告完整 | 15分 |

---

*最后更新: 2026-04-19*
*🔧 修车技能第2周 - 发布门禁与检查清单*
