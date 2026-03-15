# SQLRustGo Trae Agent 配置

> **重要提示**：执行文档操作前必须先阅读 `.trae/rules/dev_rules.md`

---

## 执行前必读

在执行任何文档操作之前，**必须先读取** `.trae/rules/dev_rules.md`

```bash
# 读取规范
cat .trae/rules/dev_rules.md
```

---

## 核心原则

```
🚫 禁止: 用 Write 完全覆盖文档
✅ 正确: 用 SearchReplace 局部修改
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

**配置文件版本**: 1.0
**创建日期**: 2026-03-16
