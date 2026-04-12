# PPTX Mermaid Fix Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把两份指定 Marp Markdown 的 Mermaid 图渲染成 PNG 并嵌入 PPTX，更新 PPTX 输出。

**Architecture:** 使用 mermaid-cli + Chrome 渲染图片，生成临时 rendered.md（Mermaid 块替换为图片链接），再用 Marp 生成 PPTX 覆盖原文件。

**Tech Stack:** Python 3、@mermaid-js/mermaid-cli、@marp-team/marp-cli、Google Chrome headless

---

### Task 1: 生成渲染脚本与图片

**Files:**
- Create: `/tmp/render_mermaid_md.py`
- Create: `docs/tutorials/教学计划/PPT/第4讲-UML规范与图例-扩展版.images/`
- Create: `docs/tutorials/教学实践/学生操作手册/week-04-OOA到OOP的实践讲解.images/`

**Step 1: 写渲染脚本**

```python
# /tmp/render_mermaid_md.py
# 读取 Markdown，提取 ```mermaid 块，渲染为 PNG，并生成 *.rendered.md
```

**Step 2: 执行渲染（扩展版）**

Run: `python3 /tmp/render_mermaid_md.py \
"/Users/liying/workspace/yinglichina/sqlrustgo/docs/tutorials/教学计划/PPT/第4讲-UML规范与图例-扩展版.md" \
--out-dir \
"/Users/liying/workspace/yinglichina/sqlrustgo/docs/tutorials/教学计划/PPT/第4讲-UML规范与图例-扩展版.images"`

Expected: 生成 `第4讲-UML规范与图例-扩展版.rendered.md` 和若干 PNG。

**Step 3: 执行渲染（实践讲解）**

Run: `python3 /tmp/render_mermaid_md.py \
"/Users/liying/workspace/yinglichina/sqlrustgo/docs/tutorials/教学实践/学生操作手册/week-04-OOA到OOP的实践讲解.md" \
--out-dir \
"/Users/liying/workspace/yinglichina/sqlrustgo/docs/tutorials/教学实践/学生操作手册/week-04-OOA到OOP的实践讲解.images"`

Expected: 生成 `week-04-OOA到OOP的实践讲解.rendered.md` 和若干 PNG。

**Step 4: 生成 PPTX（扩展版）**

Run: `npx --yes @marp-team/marp-cli --allow-local-files \
"/Users/liying/workspace/yinglichina/sqlrustgo/docs/tutorials/教学计划/PPT/第4讲-UML规范与图例-扩展版.rendered.md" \
--pptx --output \
"/Users/liying/workspace/yinglichina/sqlrustgo/docs/tutorials/教学计划/PPT/第4讲-UML规范与图例-扩展版.pptx"`

Expected: 覆盖更新 PPTX。

**Step 5: 生成 PPTX（实践讲解）**

Run: `npx --yes @marp-team/marp-cli --allow-local-files \
"/Users/liying/workspace/yinglichina/sqlrustgo/docs/tutorials/教学实践/学生操作手册/week-04-OOA到OOP的实践讲解.rendered.md" \
--pptx --output \
"/Users/liying/workspace/yinglichina/sqlrustgo/docs/tutorials/教学实践/学生操作手册/week-04-OOA到OOP的实践讲解.pptx"`

Expected: 覆盖更新 PPTX。

**Step 6: Commit**

```bash
git add \
  docs/tutorials/教学计划/PPT/第4讲-UML规范与图例-扩展版.pptx \
  docs/tutorials/教学实践/学生操作手册/week-04-OOA到OOP的实践讲解.pptx

git commit -m "docs: fix mermaid rendering in pptx"
```
