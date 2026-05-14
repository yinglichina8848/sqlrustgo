# 第15周：版本发布与知识管理

> 实验时间：2学时
> 实验类型：设计性

---

## 一、实验目标

- [ ] 理解版本发布的完整闭环（Issue → PR → Gate → Release → 知识沉淀）
- [ ] 能够创建版本标签和编写 Release Notes
- [ ] 理解 GBrain SSOT 的三层知识架构
- [ ] 能够记录一条知识（Pattern / Episode）到 SSOT

---

## 二、操作步骤

### 步骤1：创建版本标签（15分钟）

```bash
git checkout develop/v3.0.0
git tag -a v3.0.0-beta.1 -m "Beta Phase 2 功能验证完成"
git push origin v3.0.0-beta.1
```

#### ✅ 检查点1：标签创建截图

---

### 步骤2：编写 Release Notes（25分钟）

参考模板：

```markdown
# SQLRustGo v3.0.0-beta.1

## 核心改进
- DELETE QPS: 206 → 64,896（315x 提升）
- UPDATE QPS: 950 → 43,121（45x 提升）

## Gate 验收
| Gate | 结果 |
|------|------|
| BP1 静态 | ✅ |
| BP2 集成 | ✅（TPC-H 22/22，覆盖率 78%） |
| BP3 风险 | E-09: R-05 豁免（cargo audit 已知漏洞） |

## 已知问题
- R-05: serde_json CVE-XXXX，上游未修复

## E-09 Floor 验证
- DELETE ≥ 10,000 ✅ (64,896)
- UPDATE ≥ 10,000 ✅ (43,121)
```

#### ✅ 检查点2：保存 Release Notes

---

### 步骤3：GBrain 知识沉淀实践（25分钟）

#### 3.1 三层 Wiki 架构

```
QMD 源文件（docs/）  →  GBrain SSOT（知识索引）  →  Wiki 页面（展示）
     版本控制              结构化查询                  用户可读
```

#### 3.2 为 DELETE 优化记录一条知识

```markdown
# Episode: DELETE 性能优化 —— 从 206 到 64,896 QPS

## 发生了什么
v2.9.0 DELETE 只有 206 QPS，远低于 E-09 地板（10,000）

## 根因分析
- 逐条 DELETE 导致 B+Tree 索引频繁更新
- WAL 同步写盘拖慢每次操作

## 优化方案
1. 批量提交：合并多个 DELETE 为一次事务
2. WAL 异步刷盘：Group Commit 减少 I/O
3. 索引批量更新：减少 B+Tree 维护开销

## 结果
DELETE QPS: 206 → 64,896（315x）
达到 E-09 地板 ✅

## 教训
- Pattern: "批量提交优先于逐条优化"
- 回归检测阈值 Δ > 20% = FAIL
```

#### ✅ 检查点3：保存 GBrain 知识卡片

---

### 步骤4：绘制 AI 工厂全景（15分钟）

```
画出 SQLRustGo 的 "AI-Native 软件工厂" 全景图：

Agent 编排 → 代码生成 → Harness 验证 → CI/CD 执行 → 知识沉淀
```

#### ✅ 检查点4：保存全景图

---

## 三、评分标准

| 检查项 | 分值 |
|--------|------|
| 版本标签 + Release Notes | 25分 |
| GBrain 知识卡片 | 35分 |
| AI 工厂全景图 | 25分 |
| 实验报告完整 | 15分 |

---

*最后更新: 2026-05-09*
