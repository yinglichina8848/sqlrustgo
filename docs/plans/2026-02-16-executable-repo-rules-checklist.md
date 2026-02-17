# SQLRustGo 可执行仓库规则清单（Evaluation -> Alpha）

> 日期：2026-02-16  
> 分支：feature/v1.0.0-evaluation  
> 目标：将“文档规则”升级为“可执行门禁”

---

## 1. 适用范围与核心原则

1. 本清单适用于 `v1.0.0` 当前阶段，覆盖 `evaluation -> alpha -> beta -> baseline -> main`。
2. 原则：
   - 先评估再升级：`evaluation` 不达标不得进入 `alpha`。
   - 未通过门禁不得合并：所有升级通过 PR 完成。
   - 规则可审计：每条门禁都能用命令或页面状态验证。

---

## 2. 分支职责与流向（唯一合法路径）

1. 功能/修复分支：`feature/v1.0.0-<topic>`、`fix/v1.0.0-<topic>`  
   只允许合并到 `feature/v1.0.0-evaluation`。
2. 评估分支：`feature/v1.0.0-evaluation`  
   只允许合并到 `feature/v1.0.0-alpha`。
3. Alpha 分支：`feature/v1.0.0-alpha`  
   只允许合并到 `feature/v1.0.0-beta`。
4. Beta 分支：`feature/v1.0.0-beta`  
   只允许合并到 `baseline`。
5. 发布分支：`baseline`  
   只允许合并到 `main`，并在 `main` 打 `v1.0.0` 标签。

---

## 3. 角色与评审规则

1. PR 作者不能审批自己的 PR。
2. 每个 PR 至少 1 个审批（建议 2 个：1 个技术、1 个测试/文档）。
3. 审批身份必须与作者身份不同（GitHub 账号或提交邮箱不同）。
4. 审查评论必须关闭（Conversation Resolved）后方可合并。

---

## 4. 门禁标准（硬门禁）

### 4.1 Evaluation -> Alpha 入口

1. `cargo test --all-features` 通过。
2. `cargo clippy --all-features -- -D warnings` 通过。
3. 覆盖率达到目标（当前目标：>= 90%）。
4. P0 缺陷清零；P1 缺陷有“修复或延期理由”。
5. 评估文档更新完成（7 维评估 + 综合改进计划 + 本清单）。

### 4.2 Alpha -> Beta 入口

1. 保持测试/Clippy 全绿。
2. API 文档、用户文档、CHANGELOG 完整。
3. 无新增 P0 缺陷。

### 4.3 Beta -> Baseline 入口

1. CI 全流程通过（构建、测试、格式、文档、覆盖率）。
2. 基准测试和安全扫描完成并记录结果。
3. 发布说明可追溯（变更范围、风险、回滚方案）。

---

## 5. 可执行检查命令

```bash
# 1) 质量门禁
cargo build --all-features
cargo test --all-features
cargo clippy --all-features -- -D warnings
cargo fmt --check --all

# 2) 文档与发布检查
cargo doc --no-deps

# 3) 可选增强（建议在 Beta 必须执行）
cargo bench
cargo audit
```

---

## 6. GitHub 配置清单（执行项）

1. 保护分支：`main`、`baseline`、`feature/v1.0.0-evaluation`、`feature/v1.0.0-alpha`、`feature/v1.0.0-beta`。
2. 保护规则：
   - Require pull request reviews（>=1）。
   - Dismiss stale reviews。
   - Require conversation resolution。
   - Disable force push / branch deletion。
3. CI 触发分支必须覆盖：`baseline`、`main`、`feature/*-evaluation`、`feature/*-alpha`、`feature/*-beta`。

---

## 7. PR 模板要求（执行项）

每个 PR 必须包含：

1. 变更目标和范围。
2. 关联 Issue（如 `Closes #1`）。
3. 风险评估与回滚方案。
4. 验证结果（命令与结论）。
5. 阶段门禁清单（打勾）。

---

## 8. 评估迭代机制（不达标继续迭代）

1. `evaluation` 不达标时：
   - 从 `evaluation` 切新修复分支。
   - 修复后 PR 合回 `evaluation`。
   - 重新执行门禁与评估。
2. 达标后才允许创建 `evaluation -> alpha` PR。
3. 禁止“带未解决 P0 问题晋级”。

---

## 9. 当前落地状态（2026-02-16）

1. 已满足：
   - `main`、`baseline` 已有保护。
2. 待落地：
   - `evaluation/alpha/beta` 分支保护。
   - CI 触发分支与文档声明对齐。
   - PR 模板标准化与门禁字段强制化。

---

## 10. 执行决议（建议）

1. 立即按本清单修正规则并公告到 Issue #1。
2. 从本周起以 `evaluation` 为唯一质量汇聚分支。
3. 仅当 Evaluation 入口标准全部通过，才发起 `-> alpha` PR。

