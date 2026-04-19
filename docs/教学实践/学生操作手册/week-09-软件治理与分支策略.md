# 第9周：软件治理与分支策略

> 实验时间：2学时（🕹️ 手动档第2周）
> 实验类型：验证性+设计性

---

## 一、实验目标

- [ ] 理解Git分支策略的概念和作用
- [ ] 能够配置分支保护规则
- [ ] 掌握多AI协同开发模式
- [ ] 能够创建和管理功能分支

---

## 二、实验环境

| 项目 | 要求 |
|------|------|
| 操作系统 | macOS / Linux / Windows 10+ |
| Git | 最新版本 |
| GitHub账号 | 需要仓库管理权限 |
| 项目代码 | SQLRustGo |

---

## 三、核心概念：为什么需要软件治理？

```
┌─────────────────────────────────────────────────────────────────────┐
│                    单点开发 → 并行开发的转折点                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  🚗 自动档阶段（1-6周）：                                           │
│  • 一个人+一个AI                                                   │
│  • 代码直接推送到主分支                                             │
│  • 不需要治理                                                       │
│                                                                      │
│  🕹️ 手动档阶段（7-12周）：                                         │
│  • 多个人+多个AI同时开发                                           │
│  • 代码需要审查和合并                                               │
│  • ⚠️ 需要治理！                                                    │
│                                                                      │
│  🔧 问题：AI之间会"打架"、会"跑偏"、质量无法保证                    │
│  🔑 解决方案：分支隔离 + PR审查 + CI/CD门禁                         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 四、操作步骤

### 步骤1：分析SQLRustGo现有分支策略（15分钟）

#### 1.1 查看所有分支

```bash
git branch -a
```

#### 1.2 查看分支历史

```bash
git log --oneline --graph --all --decorate
```

#### 1.3 查看远程分支

```bash
git fetch origin
git branch -r
```

**分析要点**：
- 当前的分支结构是什么？
- main分支和develop分支的关系？
- feature分支的命名规范？

#### ✅ 检查点1：记录分支结构

| 分支名 | 用途 | 保护状态 |
|--------|------|---------|
| main | 生产分支 | 已保护 |
| develop/v2.6.0 | 开发分支 | 已保护 |
| feature/* | 功能开发 | 未保护 |

---

### 步骤2：配置开发分支（20分钟）

#### 2.1 确保本地develop分支最新

```bash
git checkout develop/v2.6.0
git pull origin develop/v2.6.0
```

#### 2.2 创建功能分支

```bash
# 创建本周实验分支
git checkout -b feature/week9-lab
```

#### 2.3 创建初始提交

```bash
git commit --allow-empty -m "chore: create feature branch for week9 lab"
```

#### 2.4 推送到远程

```bash
git push origin feature/week9-lab
```

#### ✅ 检查点2：截图分支创建成功

---

### 步骤3：配置分支保护规则（25分钟）

#### 3.1 进入GitHub仓库设置

1. 访问 SQLRustGo 仓库
2. 进入 Settings → Branches
3. 点击 "Add branch protection rule"

#### 3.2 配置保护规则

```
Branch name pattern: develop/v2.6.0

✅ Require pull request reviews before merging
   - Required approving reviews: 1

✅ Require status checks to pass before merging
   - Require branches to be up to date before merging

❌ Allow force pushes
❌ Allow deletions
```

#### 3.3 为其他分支配置类似规则

| 分支模式 | 保护规则 |
|---------|---------|
| main | 严格保护（2人审批+CI通过） |
| develop/* | 中等保护（1人审批+CI通过） |
| feature/* | 轻度保护（CI通过即可） |

#### ✅ 检查点3：截图分支保护规则配置

---

### 步骤4：测试分支保护（20分钟）

#### 4.1 尝试直接推送develop分支（应该失败）

```bash
git checkout develop/v2.6.0
echo "test" >> README.md
git commit -m "test: direct commit attempt"
git push origin develop/v2.6.0  # 应该被拒绝！
```

**预期结果**：推送被拒绝，显示分支受保护

#### 4.2 通过PR方式合并代码

```bash
# 切换到功能分支
git checkout feature/week9-lab

# 添加一些修改
echo "test via PR" >> README.md
git commit -m "docs: test PR workflow"

# 推送
git push origin feature/week9-lab
```

#### 4.3 在GitHub上创建PR

1. 访问推送后的分支页面
2. 点击 "Compare & pull request"
3. 填写PR信息
4. 提交PR

#### ✅ 检查点4：记录PR创建和推送结果

---

## 五、多AI协同开发模式

### 5.1 多AI协作的问题

```
┌─────────────────────────────────────────────────────────────────────┐
│                    多AI协作的三大难题                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  1. 🐓 打地盘：两个AI同时修改同一文件                                │
│     → 解决：不同功能在不同分支开发                                   │
│                                                                      │
│  2. 🐢 跑偏了：AI生成的代码不符合项目规范                            │
│     → 解决：PR审查 + CI门禁                                         │
│                                                                      │
│  3. 📦 质量差：AI生成的代码有bug                                     │
│     → 解决：测试覆盖 + 代码审查                                      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.2 分支命名规范

| 类型 | 命名格式 | 示例 |
|------|---------|------|
| 功能分支 | feature/功能名 | feature/add-join |
| 修复分支 | fix/问题描述 | fix/memory-leak |
| 实验分支 | experiment/实验名 | experiment/new-optimizer |

### 5.3 工作流程

```
develop/v2.6.0 ←←←←←←←←←←←←←←←←←←←←
     ↑                                     
     │ 1. 创建功能分支                     
     │    git checkout -b feature/xxx      
     │                                     
     │ 2. 开发并提交                       
     │    git commit ...                   
     │                                     
     │ 3. 推送并创建PR                     
     │    git push origin feature/xxx      
     │                                     
     │ 4. 代码审查                          
     │    人工/AI审查                      
     │                                     
     └←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←
          5. 合并到develop
```

---

## 六、实验报告

### 6.1 报告内容

1. **分支策略分析报告**
   - 当前分支结构
   - 分支命名规范
   - 保护规则现状

2. **分支保护规则配置截图**
   - GitHub分支保护设置
   - 规则详情

3. **PR创建和审核流程记录**
   - PR链接
   - 审核意见（如有）

4. **遇到的问题和解决方法**

### 6.2 提交方式

```bash
git checkout -b experiment/week-09-你的学号
mkdir -p reports/week-09
# 放入报告和截图
git add reports/week-09/
git commit -m "experiment: submit week-09 report"
git push origin experiment/week-09-你的学号
```

---

## 七、思考与反思

### 🎯 本周能力阶段：Context Engineering（上下文工程）

**本周重点**：学会提供足够的项目上下文，让AI理解代码结构

### 问题1：优化与改进
- 在描述分支策略时，我是否提供了足够的项目背景信息？
- AI给出的建议是否考虑了我们的实际项目结构？

### 问题2：AI不可替代的能力
- AI能理解"为什么我们需要分支保护"这个业务需求吗？
- 人类的哪些判断（如团队规模、发布节奏）影响了分支策略的选择？

### 问题3：自我提升
- 我是否开始学会在提问时提供上下文信息？
- 哪些信息是冗余的？哪些是必要的？

---

## 八、评分标准

| 检查项 | 分值 |
|--------|------|
| 分支策略分析完整 | 20分 |
| 分支保护规则配置正确 | 30分 |
| PR工作流实践 | 25分 |
| 多AI协作模式理解 | 15分 |
| 实验报告完整 | 10分 |

---

*最后更新: 2026-04-19*
*🕹️ 手动档第2周 - 软件治理入门*
