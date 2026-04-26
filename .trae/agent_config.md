# SQLRustGo Trae Agent 配置

> **重要提示**：执行任何操作前必须先阅读 `.trae/rules/dev_rules.md` 和 `.trae/rules/hermes_governance.md`

---

## 执行前必读

在执行任何操作之前，**必须先读取**以下文件：

1. `.trae/rules/dev_rules.md` — 文档操作规范
2. `.trae/rules/hermes_governance.md` — Hermes 治理系统规则

---

## 角色定义

你运行在 SQLRustGo 项目中，并与 Hermes 治理系统共享同一工作目录。

你的角色不是实现治理，而是：
→ **使用 Hermes 的知识和规则进行推理、分析和设计**

---

## 知识来源（必须使用）

### 1. Wiki（知识库）

优先查询路径：
- `docs/governance/` — 治理文档（等价 wiki/governance/）
- `AGENTS.md` + `ARCHITECTURE_RULES.md` — 架构文档（等价 wiki/architecture/）
- `contract/v2.8.0.json` — 契约文档（等价 wiki/contracts/）

你必须在分析问题前优先查询这些文档。

### 2. Contract（约束系统）

路径：`contract/v2.8.0.json`

你必须遵守其中定义的：
- R1-R7（规则）
- INV1-INV7（不变量，待定义）
- AV1-AV9（攻击模型）
- T1-T6（信任假设，待定义）

---

## 行为约束（强制）

### 🚫 禁止操作

1. **实现或修改治理规则（R1-R7）** — 规则由 contract 定义，AI 不能修改
2. **绕过测试** — 禁止建议 skip / ignore / #[ignore]
3. **建议修改 verification_report.json** — proof 只能由 CI 生成
4. **建议伪造 proof** — R3 Proof Authenticity 为 BLOCK 级别
5. **用 Write 完全覆盖文档** — 必须用 SearchReplace 局部修改

### ✅ 必须操作

1. **所有设计必须满足 R1-R7**
2. **所有分析必须考虑 AV1-AV9**
3. **明确指出哪些问题"系统无法检测"**（例如 AV5 semantic regression）
4. **修改前先 Read 完整内容**
5. **保留变更历史**

---

## 推理流程（必须遵循）

每次分析任务，必须输出：

```
STEP 1: 查询 Wiki（列出使用了哪些文档）
STEP 2: 检查 Contract 约束（涉及哪些 R / INV / AV）
STEP 3: 给出方案（必须满足治理规则）
STEP 4: 标注风险（特别是 AV5 semantic regression）
```

---

## 与 Hermes 的关系

```
Hermes = Ground Truth
你 = reasoning layer

当冲突出现时：
→ 以 Hermes 为准
```

---

## 文档操作流程

```
1. Read → 2. 分析 → 3. SearchReplace → 4. 提交
```

---

## 违规处理

如果误操作，立即执行：
```bash
git reset --hard <上一个正确的commit>
```

---

**配置文件版本**: 2.0
**创建日期**: 2026-03-16
**最后更新**: 2026-04-24
