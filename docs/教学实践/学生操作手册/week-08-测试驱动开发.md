# 第8周：测试驱动开发与Alpha版本

> 实验时间：2学时（🕹️ 手动档第1周）
> 实验类型：验证性+设计性
> **本阶段核心转变**：从"让AI帮我做"到"我自己做，AI辅助审查"

---

## 一、实验目标

- [ ] 掌握测试驱动开发（TDD）方法
- [ ] 理解白盒测试与覆盖率概念
- [ ] 能够使用AI辅助生成测试用例
- [ ] 能够提高测试覆盖率至70%以上
- [ ] 能够完成Alpha版本发布

---

## 二、白盒测试与覆盖率详解

### 2.1 什么是白盒测试？

| 维度 | 黑盒测试 | 白盒测试 |
|------|---------|---------|
| 视角 | 外部功能 | 内部实现 |
| 依据 | 需求规格 | 代码结构 |
| 重点 | 输入-输出 | 逻辑路径 |

### 2.2 覆盖率度量层次

```
路径覆盖 (Path Coverage) - 最高要求
  ↓
分支覆盖 (Branch Coverage)
  ↓
语句覆盖 (Statement Coverage) - 最低要求
```

### 2.3 语句覆盖 (Statement Coverage)

- **定义**：每个可执行的代码语句至少被执行一次
- **公式**：语句覆盖率 = (已执行语句数 / 总语句数) × 100%
- **目标**：通常要求 ≥ 70%

**示例**：
```rust
fn calculate(a: i32, b: i32, op: &str) -> i32 {
    if op == "add" {      // 语句1
        a + b              // 语句2
    } else if op == "sub" { // 语句3
        a - b              // 语句4
    } else {              // 语句5
        0                  // 语句6
    }
}
```
- 测试用例 `"add"` 执行语句：1, 2 → 覆盖率 2/6 = 33%
- 测试用例 `"add" + "sub"` 执行语句：1, 2, 3, 4 → 覆盖率 4/6 = 67%

### 2.4 分支覆盖 (Branch Coverage)

- **定义**：每个条件分支的真假两个分支至少执行一次
- **公式**：分支覆盖率 = (已执行分支数 / 总分支数) × 100%
- **目标**：通常要求 ≥ 60%

### 2.5 路径覆盖 (Path Coverage)

- **定义**：每个可能的代码执行路径至少执行一次
- **注意**：路径数可能指数级增长，通常使用基本路径覆盖

### 2.6 版本命名规范

```
推荐格式：alpha/v0.1.0-alpha1

dev/v1.0.0     - 开发版本
alpha/v1.0.0   - 内部测试
beta/v1.0.0    - 外部测试
rc/v1.0.0      - 候选发布
release/v1.0.0 - 正式发布
```

---

## 三、🚗→🕹️ 阶段转变说明

**本周是"手动档"的开始！**

| 自动档阶段 | 手动档阶段 |
|-----------|-----------|
| AI生成代码，我来运行 | **我自己写代码，AI帮我审查** |
| "帮我做一个..." | "帮我看看我写的对不对" |
| 追求结果正确 | 追求理解原理 |

**本周的核心转变**：
- 不再完全依赖AI生成代码
- 自己动手写测试用例
- AI的角色从"执行者"变成"审查者"

---

## 四、实验环境

| 项目 | 要求 |
|------|------|
| 操作系统 | macOS / Linux / Windows 10+ |
| Rust工具链 | 1.75+ |
| 开发工具 | TRAE IDE / VS Code |
| Git | 最新版本 |
| 项目代码 | SQLRustGo develop/v2.6.0分支 |

---

## 五、操作步骤

### 步骤1：运行现有测试并分析覆盖率（20分钟）

#### 1.1 克隆最新代码

```bash
# 确保在开发分支上
git checkout develop/v2.6.0
git pull origin develop/v2.6.0
```

#### 1.2 运行所有测试

```bash
cargo test --all-features
```

**预期输出**：
```
running xx tests
...
test result: ok. xx passed; 0 failed; 0 ignored
```

#### 1.3 安装覆盖率工具

```bash
cargo install cargo-tarpaulin
```

#### 1.4 生成覆盖率报告

```bash
cargo tarpaulin --out Html --output-dir coverage
```

#### 1.5 查看覆盖率

```bash
# 打开生成的HTML报告
open coverage/tarpaulin-report.html
```

**分析要点**：
- 当前覆盖率是多少？（目标：≥70%）
- 哪些模块覆盖率较低？
- 哪些代码路径未被测试覆盖？

#### ✅ 检查点1：截图保存
- 截图覆盖率报告
- 记录覆盖率数据

---

### 步骤2：使用AI辅助生成测试用例（30分钟）

#### 2.1 分析现有测试结构

```bash
# 查看现有测试文件
find . -name "*.rs" -path "*/tests/*" | head -20

# 查看模块测试
ls -la crates/*/src/
```

#### 2.2 使用AI生成测试提示词

在TRAE IDE中输入以下提示词：

```
我需要为SQLRustGo的词法分析器生成测试用例。
现有代码位于 src/parser/lexer.rs
请生成以下测试：
1. 关键字识别测试（SELECT, FROM, WHERE, INSERT, UPDATE, DELETE等）
2. 标识符识别测试
3. 数字字面量测试
4. 字符串字面量测试
5. 运算符识别测试
6. 边界条件测试（空输入、单字符、特殊字符等）

请使用Rust的 #[test] 属性编写测试代码。
```

#### 2.3 扩展任务：为语法分析器生成测试用例

```
为SQLRustGo的语法分析器生成测试用例。
覆盖：
1. SELECT语句解析
2. INSERT语句解析
3. UPDATE语句解析
4. DELETE语句解析
5. 错误语法检测
```

#### 2.4 整合AI生成的测试代码

将AI生成的测试代码整合到项目中：

```bash
# 创建测试目录（如不存在）
mkdir -p crates/parser/src

# 查看现有测试结构
cat crates/parser/src/lib.rs | grep -A5 "#[cfg(test)]"
```

#### ✅ 检查点2：保存AI提示词和生成的测试代码

---

### 步骤3：补充测试用例并验证覆盖率（30分钟）

#### 3.1 补充缺失的测试

根据AI生成的测试建议，补充缺失的测试用例。

#### 3.2 运行测试确保全部通过

```bash
# 运行特定模块的测试
cargo test --package parser
cargo test --package executor
cargo test --package storage
```

#### 3.3 重新生成覆盖率报告

```bash
cargo tarpaulin --out Html --packages parser,executor,storage
```

#### 3.4 验证覆盖率是否达到70%

```bash
# 查看覆盖率摘要
cargo tarpaulin --out Text
```

**如果覆盖率未达到70%**：
- 识别覆盖率低的模块
- 使用AI辅助生成更多测试用例
- 重复步骤3直到达标

#### ✅ 检查点3：记录覆盖率变化

| 模块 | 初始覆盖率 | 目标覆盖率 | 最终覆盖率 |
|------|-----------|-----------|-----------|
| parser | ? | ≥70% | ? |
| executor | ? | ≥70% | ? |
| storage | ? | ≥70% | ? |

---

### 步骤4：运行质量门禁检查（20分钟）

#### 4.1 编译检查

```bash
cargo build --all-features
```

#### 4.2 测试检查

```bash
cargo test --all-features
```

#### 4.3 Clippy检查

```bash
cargo clippy --all-features -- -D warnings
```

#### 4.4 格式化检查

```bash
cargo fmt --check --all
```

**所有检查必须通过才能继续！**

#### ✅ 检查点4：保存门禁检查结果

---

### 步骤5：创建Alpha版本（20分钟）

#### 5.1 版本命名规范

```
推荐格式：alpha/v0.1.0-alpha1

dev/v1.0.0     - 开发版本
alpha/v1.0.0   - 内部测试
beta/v1.0.0    - 外部测试
rc/v1.0.0      - 候选发布
release/v1.0.0 - 正式发布
```

#### 5.2 确保代码是最新的

```bash
git status
git pull origin develop/v2.6.0
```

#### 5.3 创建Alpha版本标签

```bash
git tag -a alpha/v0.1.0-alpha1 -m "Alpha版本发布 - 测试驱动开发完成"
```

#### 5.4 推送标签

```bash
git push origin alpha/v0.1.0-alpha1
```

#### 5.5 创建GitHub Release

在GitHub网页上操作：
1. 进入仓库 Releases 页面
2. 点击 "Draft a new release"
3. 填写信息：
   - Tag: alpha/v0.1.0-alpha1
   - Title: SQLRustGo v0.1.0-alpha
   - Release notes: 描述Alpha版本的特性

#### ✅ 检查点5：保存Release链接

---

## 六、实验报告

### 5.1 报告内容

1. **测试覆盖率报告**
   - 初始覆盖率截图
   - 最终覆盖率截图
   - 覆盖率变化分析

2. **补充的测试用例列表**
   - 测试用例名称
   - 测试目的
   - 覆盖的代码路径

3. **质量门禁检查结果**
   - cargo build 结果
   - cargo test 结果
   - cargo clippy 结果
   - cargo fmt 结果

4. **Alpha版本Release链接**
   - GitHub Release URL

5. **本周思考与反思**
   - 自动档→手动档的转变感受
   - AI辅助测试生成的收获
   - 遇到的问题和解决方法

### 5.2 提交方式

```bash
# 创建实验报告分支
git checkout -b experiment/week-08-你的学号

# 创建报告目录
mkdir -p reports/week-08

# 将报告和截图放入该目录

# 提交
git add reports/week-08/
git commit -m "experiment: submit week-08 report"
git push origin experiment/week-08-你的学号
```

---

## 七、思考与反思

### 🎯 本周能力阶段：Prompt Engineering（提示词工程）

**本周重点**：学会清晰准确地描述需求，让AI生成正确的测试代码

### 问题1：如何优化和改进？

- 我在向AI描述测试需求时，哪些表达让AI理解得更好？哪些让AI产生了错误理解？
- 如何把"要测试的东西"描述得更精确？

### 问题2：AI不可替代的能力

- AI生成的测试用例覆盖了"常见情况"吗？
- 哪些"边界情况"需要我手动补充？AI为什么没想到？

### 问题3：自我提升

- 我的提示词需要迭代几次才能得到满意的结果？
- 我是否开始理解什么样的提示词能获得更好的输出？

---

## 八、评分标准

| 检查项 | 分值 |
|--------|------|
| 测试覆盖率≥70% | 25分 |
| 补充测试用例质量 | 20分 |
| 质量门禁全部通过 | 25分 |
| Alpha版本发布成功 | 15分 |
| 实验报告完整 | 15分 |

---

*最后更新: 2026-04-19*
*🕹️ 手动档第1周 - 从自动档转向手动实现*
