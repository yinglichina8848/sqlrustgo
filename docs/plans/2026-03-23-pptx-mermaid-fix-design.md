# PPTX Mermaid Fix Design

**Goal**
将指定两份 Marp Markdown 的 Mermaid 图渲染为 PNG，并插入到 PPTX 中，保证 PPTX 中图正确显示；保留原始 .md 不变。

**范围**
- 输入：
  - docs/tutorials/教学计划/PPT/第4讲-UML规范与图例-扩展版.md
  - docs/tutorials/教学实践/学生操作手册/week-04-OOA到OOP的实践讲解.md
- 输出：
  - 覆盖更新同名 PPTX（插入 PNG 图）
  - 复用现有 .images 目录
  - 生成临时 *.rendered.md（仅用于构建）

**方案**
1. 解析 Markdown 中 ```mermaid 代码块，渲染为 PNG（mermaid-cli + Chrome）。
2. 生成临时 rendered.md：将 Mermaid 代码块替换为图片链接（绝对路径）。
3. 用 Marp 以 rendered.md 生成 PPTX，覆盖原 PPTX。

**错误处理**
- Mermaid 不支持的语法类型直接报错，输出具体 block 位置，停止生成。

**验证**
- 生成的 PPTX 时间戳更新；必要时抽查 1-2 页确认图像显示。
