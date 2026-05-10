# 第13周：安全扫描与审计

> 实验时间：2学时（🔧 修车技能第1周）
> 实验类型：验证性

---

## 一、实验目标

- [ ] 理解软件安全基础
- [ ] 能够运行依赖安全扫描
- [ ] 能够运行代码安全扫描
- [ ] 能够生成安全报告

---

## 二、🕹️→🔧 阶段转变：从手动档到修车技能

**本周开始进入"修车技能"阶段！**

| 手动档阶段 | 修车技能阶段 |
|-----------|------------|
| 我写代码，AI帮我审查 | 我要找Bug，AI帮我分析 |
| "帮我看看这段代码对不对" | "帮我分析为什么程序崩溃了" |
| 追求理解原理 | 追求定位和解决问题的能力 |

**本周的核心转变**：
- 从"写代码"转向"分析和调试"
- 学习使用专业工具定位问题
- 理解"为什么会出错"比"怎么写对"更重要

---

## 三、实验环境

| 项目 | 要求 |
|------|------|
| Rust工具链 | 1.75+ |
| GitHub账号 | 需要仓库管理权限 |
| 项目代码 | SQLRustGo |

---

## 四、操作步骤

### 步骤1：依赖安全扫描（25分钟）

#### 1.1 安装cargo-audit

```bash
cargo install cargo-audit
```

#### 1.2 运行依赖扫描

```bash
cargo audit
```

#### 1.3 分析扫描结果

```
Found 3 vulnerabilities (2 moderate, 1 high)

Package: serde_json
Version: 1.0.91
Date: 2024-01-15
ID: RUSTSEC-2024-0001
URL: https://rustsec.org/advisories/RUSTSEC-2024-0001
Title: Denial of service in serde_json
Solution: upgrade to: 1.0.108
```

| 漏洞 | 严重程度 | 影响 | 修复方案 |
|------|---------|------|---------|
| RUSTSEC-2024-0001 | 高危 | DoS | 升级到1.0.108 |
| RUSTSEC-2024-0002 | 中等 | 信息泄露 | 升级到2.0.0 |
| RUSTSEC-2024-0003 | 中等 | 内存问题 | 升级到1.0.95 |

#### ✅ 检查点1：记录依赖扫描结果

---

### 步骤2：配置Dependabot（15分钟）

#### 2.1 在GitHub上配置

1. 进入仓库 Settings → Security & analysis
2. 启用 Dependabot alerts
3. 启用 Dependabot security updates

#### 2.2 创建dependabot配置文件

创建 `.github/dependabot.yml`：

```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 10

  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "weekly"
```

#### ✅ 检查点2：保存Dependabot配置

---

### 步骤3：代码安全扫描（25分钟）

#### 3.1 运行Clippy安全检查

```bash
cargo clippy --all-features -- -D warnings
```

#### 3.2 关注安全相关警告

| 警告类型 | 含义 | 风险 |
|---------|------|------|
| `unwrap()` | 未处理的错误 | 崩溃 |
| `unsafe` | 不安全代码 | 内存安全 |
| `TODO` | 未完成代码 | 功能缺失 |

#### 3.3 AI辅助安全审查

```
提示词示例：

审查以下SQLRustGo代码的安全问题：
[粘贴存储引擎相关代码]

请检查：
1. SQL注入风险
2. 缓冲区溢出风险
3. 敏感信息泄露
4. 不安全的加密使用
5. 其他安全问题
```

#### ✅ 检查点3：记录安全扫描结果

---

### 步骤4：生成安全报告（15分钟）

#### 4.1 整理安全检查结果

创建 `docs/security/security-report.md`：

```markdown
# 安全审计报告

## 1. 依赖安全

| 依赖 | 版本 | 漏洞数 | 状态 |
|------|------|--------|------|
| serde_json | 1.0.91 | 1 | 待修复 |
| tokio | 1.28.0 | 0 | ✅ 安全 |
| ... | ... | ... | ... |

## 2. 代码安全

| 检查项 | 结果 | 风险等级 | 状态 |
|--------|------|---------|------|
| unwrap()使用 | 15处 | 中 | 待优化 |
| unsafe代码 | 3处 | 高 | 需审查 |
| TODO | 20处 | 低 | 计划中 |

## 3. 建议修复项

### 高优先级
1. 修复 serde_json 漏洞
2. 审查 unsafe 代码

### 中优先级
3. 减少 unwrap() 使用
```

#### 4.2 提交安全报告

```bash
git add docs/security/
git commit -m "docs: add security audit report"
```

#### ✅ 检查点4：保存安全报告

---

## 五、实验报告

### 5.1 报告内容

1. **依赖扫描结果**
2. **代码扫描结果**
3. **安全报告**
4. **修复建议**

### 5.2 提交方式

```bash
git checkout -b experiment/week-13-你的学号
mkdir -p reports/week-13
# 放入报告
git add reports/week-13/
git commit -m "experiment: submit week-13 report"
git push origin experiment/week-13-你的学号
```

---

## 六、思考与反思

### 🎯 本周能力阶段：Harness Engineering（规则约束工程）

**本周重点**：设计安全规则，约束AI生成的代码

### 问题1：优化与改进
- 我是否开始学会定义"安全边界"？
- AI能理解"安全"这个概念吗？

### 问题2：AI不可替代的能力
- AI知道"这个功能在某些场景下可能有安全风险"吗？
- 人类如何平衡"功能实现"和"安全风险"？

### 问题3：自我提升
- 我是否开始学会从"安全"角度审查代码？
- 我是否能识别AI生成代码中的潜在安全风险？

---

## 七、评分标准

| 检查项 | 分值 |
|--------|------|
| 依赖安全扫描 | 25分 |
| Dependabot配置 | 15分 |
| 代码安全扫描 | 25分 |
| 安全报告 | 20分 |
| 实验报告完整 | 15分 |

---

*最后更新: 2026-04-19*
*🔧 修车技能第1周 - 安全扫描与审计*
