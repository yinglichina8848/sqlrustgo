# Metis 模型升级到 DeepSeek V4 Pro

## TL;DR
> **Quick Summary**: 将 Metis agent 使用的模型从 `deepseek/deepseek-v4-flash` 升级为 `deepseek/deepseek-v4-pro`，与 Prometheus、Oracle、Momus 保持一致。
> 
> **Deliverables**: 修改 1 行配置文件
> 
> **Estimated Effort**: Quick (1 分钟)

---

## Context

当前 `oh-my-opencode.json` 中 Metis 配置：
```json
"Metis": {
  "model": "deepseek/deepseek-v4-flash"    // ← 需改为 v4-pro
},
```

其他规划/审查 agent（Prometheus、Oracle、Momus）均已使用 `deepseek/deepseek-v4-pro`。

---

## 改动

- [ ] 修改 `.config/opencode/oh-my-opencode.json` 第 37 行
  - 文件: `/Users/liying/.config/opencode/oh-my-opencode.json`
  - 旧: `"model": "deepseek/deepseek-v4-flash"`
  - 新: `"model": "deepseek/deepseek-v4-pro"`

---

## 验证

重启 OpenCode 后，再次调用 Metis agent（如通过 `/` 命令或 `task(subagent_type="metis")`）确认正常运行。
