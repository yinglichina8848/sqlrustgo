---
marp: true
theme: gaia
paginate: true
backgroundColor: #fff
color: #333
---

<!-- _class: lead -->

# 第十三讲：安全扫描与GitNexus代码分析

## Harness工程：形式化证明与量化风险评估

---

# 课程大纲

1. **安全扫描的Harness视角**（25分钟）
2. **GitNexus代码智能**（25分钟）
3. **形式化证明机制**（25分钟）
4. **实践练习**（15分钟）

---

# Part 1: 安全扫描的Harness视角

---

## 1.1 AI生成代码的安全困境

### LLM安全意识的"表面化"

```
开发者：帮我写一个用户查询函数
AI：     好的，代码如下...

实际风险：
  • AI可能生成有SQL注入漏洞的代码
  • AI可能硬编码密码或API密钥
  • AI可能使用不安全的加密算法
  • AI可能忽略边界条件导致缓冲区溢出

问题根源：AI训练数据包含不安全代码模式
         AI无法真正"理解"安全原则
```

### AI安全问题的分类

```
显式安全问题（AI会避免）：
  • 明显的SQL注入
  • 硬编码密码

隐式安全问题（AI容易忽略）：
  • 时序攻击（timing attack）
  • 侧信道信息泄露
  • 复杂的竞态条件
  • 依赖供应链漏洞
```

---

## 1.2 安全扫描的三层架构

### Layer 1：SAST（静态应用安全测试）

```
原理：分析源代码，无需执行

工具：
  • Clippy (Rust) → 编译时安全检查
  • cargo audit → 依赖漏洞扫描

GitNexus SAST：
  • gitnexus_impact() → 代码变更影响分析
  • 主动识别HIGH/CRITICAL风险变更
```

### Layer 2：SCA（软件组成分析）

```
原理：扫描第三方依赖，发现已知漏洞

工具：
  • cargo audit → Rust依赖CVE扫描

SQLRustGo Gate要求：
  • cargo audit || true → 已知漏洞可豁免
  • R-05漏洞：审计报告已有，但暂未修复
  • 豁免需记录原因，符合ISO 9001可追溯原则
```

### Layer 3：DAST（动态应用安全测试）

```
原理：运行时分析，模拟攻击

工具：
  • 集成测试 → 验证SQL注入防护
  • TPC-H → 验证复杂Query正确性

在Gate中的角色：
  • 功能测试通过 = 基本安全验证通过
  • TPC-H通过 = 复杂Query无注入风险
```

---

## 1.3 cargo audit的Gate豁免机制

### 为什么需要豁免

```
正常流程：
  cargo audit → 发现CVE → Gate Fail → PR Blocked → 必须修复

实际情况：
  • 某些CVE影响微乎其微
  • 修复需要等待上游发布
  • 修复可能引入其他问题

E-09 Floor类似机制：
  已知/可接受风险 → 记录豁免 → 继续推进
```

### 豁免的合规要求

```
ISO 9001合规要求：
  • 豁免必须有书面记录
  • 豁免必须有根因分析
  • 豁免必须有计划修复
  • 豁免必须定期复审

错误的豁免：
  cargo audit || true → 忽略所有漏洞 ← 违规

正确的豁免：
  cargo audit
  if [ $? -ne 0 ]; then
    # 分析漏洞
    # 记录豁免原因到 Issue
    # 设置计划修复日期
    exit 0  # 或 exit 1 取决于风险
  fi
```

---

# Part 2: GitNexus代码智能

---

## 2.1 GitNexus是什么

### 代码智能的定义

> **GitNexus是一个代码索引和分析系统，将代码库转化为可查询的知识图谱。**

### 传统grep vs GitNexus

```
传统grep：
  • 关键字搜索
  • 逐文件匹配
  • 无法理解语义

GitNexus：
  • 符号（symbol）索引
  • 调用关系图谱
  • 语义理解
  • 影响范围分析
```

### SQLRustGo的索引规模

```
GitNexus索引统计：
  • 52,324 个符号（symbols）
  • 80,141 个关系（relationships）
  • 300 个执行流（execution flows）

查询能力：
  • 找某个函数的所有调用者
  • 找某个类型的所有继承者
  • 评估修改某代码的"爆炸半径"
```

---

## 2.2 gitnexus_impact：变更影响分析

### 核心功能

> **在修改代码前，先问GitNexus：这个改动会影响什么？**

### 影响等级定义

```
LOW（低风险）：
  • 仅影响测试代码
  • 仅影响文档
  • 内部实现细节变更

HIGH（高风险）：
  • 影响核心公共API
  • 影响多个执行流
  • 涉及安全相关代码

CRITICAL（极高风险）：
  • 影响数据库事务核心逻辑
  • 影响锁机制
  • 可能导致数据损坏
```

### 影响分析的执行流程

```
开发者计划修改函数 X
    │
    ▼
调用 gitnexus_impact({ target: "X", direction: "upstream" })
    │
    ▼
GitNexus返回：
  {
    direct_callers: [A, B, C],
    affected_processes: [QueryExecution, Transaction],
    risk_level: "HIGH"
  }
    │
    ▼
Gate要求：
  • LOW → 正常合并
  • HIGH → 需要额外Review
  • CRITICAL → 必须架构师批准
```

---

## 2.3 gitnexus_detect_changes：变更范围检测

### 核心功能

> **提交前检查：这个PR实际改了什么？是否只改了预期的文件？**

### 执行时机

```
错误的做法：
  改了很多文件 → 直接commit → push → PR
  问题：可能漏掉相关测试，可能有意外变更

正确的做法：
  git add . → git commit
  gitnexus_detect_changes()
  确认：只改了预期的3个文件
  → push → PR
```

### 检测结果示例

```
gitnexus_detect_changes() 输出：

检测到变更：
  ✗ executor/query.rs
  ✗ executor/join.rs  ← 预期外
  ✗ README.md         ← 预期外

预期变更：
  ✓ executor/query.rs

风险：
  ⚠️ 2个文件未在预期范围内
  ⚠️ join.rs 可能需要同步测试
  ⚠️ README.md 变更需文档检查
```

---

## 2.4 GitNexus的Gate集成

### PRMerge前的强制检查

```
PR创建 → Gate触发
    │
    ├──→ gitnexus_impact() 分析变更
    │
    ├──→ HIGH/CRITICAL → PR Blocked + 通知架构师
    │
    └──→ gitnexus_detect_changes() 验证范围
              │
              └──→ 范围不符 → 要求补充/澄清
```

### GitNexus在代码审查中的作用

```
传统Review：
  Reviewer人眼查找问题
  问题：人眼有限，可能遗漏

GitNexus增强Review：
  1. GitNexus预先分析变更
  2. 高亮风险代码点
  3. Reviewer聚焦高风险区域
  4. 人工 + AI = 完整覆盖
```

---

# Part 3: 形式化证明机制

---

## 3.1 形式化证明的必要性

### AI生成的代码为什么需要形式化验证

```
传统验证：
  测试100%通过 → 代码正确？

形式化验证：
  ∀ 输入 → 证明 invariant → 代码正确

区别：
  • 测试：通过有限的测试用例验证
  • 形式化证明：通过数学方法证明所有情况正确
```

### ISO 9001与形式化证明

```
ISO 9001 质量原则：
  • 以客户为中心 → 系统正确性是客户核心需求
  • 过程方法 → 形式化证明是验证过程的标准化
  • 基于证据的决策 → GitNexus分析是证据
  • 持续改进 → 证明失败 → 流程优化

CMM-5（优化级）：
  • 形式化证明是CMM-5的核心特征
  • 证明系统满足定义的invariant
```

---

## 3.2 SQLRustGo的形式化证明实践

### 核心Invariant定义

```
SQLRustGo事务 invariant：

IF 事务T开始
AND IF T执行了COMMIT
THEN 所有T的修改都对其他事务可见
AND T的修改是持久的

验证方式：
  • 并发测试（模拟100个并发事务）
  • 故障恢复测试（模拟断电重启）
  • GitNexus证明（变更不破坏事务边界）
```

### 形式化证明的执行层次

```
Layer 1：单元测试
  证明：函数对特定输入返回预期输出

Layer 2：属性测试（Property Testing）
  证明：函数对任意输入满足属性

Layer 3：GitNexus证明
  证明：代码变更不影响已证明的invariant

Layer 4：形式化规格（未来）
  证明：系统整体满足规格
```

---

## 3.3 IF/THEN规则的Gate要求

### PR描述中的形式化表达

```
传统PR描述：
  修复了一个Bug

Gate要求PR描述：
  修复了一个Bug

  IF 条件：
    用户并发提交相同主键的INSERT
  THEN 结果：
    只有一个成功，另一个返回DuplicateKeyError
  INVARIANT：
    主键唯一性不被破坏

  测试验证：
    ✓ 100并发INSERT相同主键 → 1成功99失败
    ✓ 之后查询 → 主键唯一性保持
```

### 为什么需要这种格式

```
传统描述的问题：
  • "修复了一个Bug" → 后续无法追溯
  • 无法判断修复是否引入新问题
  • 无法验证 invariant 是否保持

IF/THEN格式的价值：
  • 明确的输入/输出/不变式
  • 后续Reviewer可以验证逻辑
  • GitNexus可以关联相关测试
  • 符合 ISO 9001 可追溯原则
```

---

## 3.4 形式化证明的局限性

### 当前LLM的能力边界

```
GitNexus能做到：
  ✓ 分析代码调用关系
  ✓ 识别影响范围
  ✓ 量化风险等级
  ✓ 关联测试用例

GitNexus做不到：
  ✗ 数学形式化证明
  ✗ 穷举所有输入组合
  ✗ 绝对保证正确性

结论：
  GitNexus是Harness的一部分，不是形式化证明的替代
  形式化证明需要人工 + 工具的结合
```

### CMM-4/5的渐进路径

```
当前状态（CMM-4）：
  • GitNexus提供量化风险分析
  • Gate提供自动化检查
  • IF/THEN规则作为沟通语言

目标状态（CMM-5）：
  • 形式化规格驱动开发
  • 自动化invariant验证
  • 持续流程优化
```

---

# Part 4: 实践练习

---

## 4.1 运行cargo audit

### 任务

1. 安装cargo-audit
2. 运行依赖扫描
3. 分析扫描结果

### 命令

```bash
cargo install cargo-audit
cargo audit
```

### 解读结果

```
正常结果：
  Scanning ... done
  No vulnerable packages found

发现问题：
  Crate: serde_json
  Version: 1.0.108
  Title: Uncontrolled recursion in serde_json
  CVE: CVE-2024-XXXX
  Solution: Upgrade to >= 1.0.109
```

---

## 4.2 使用GitNexus分析代码

### 任务

1. 运行gitnexus_impact分析一个函数
2. 查看影响范围报告
3. 评估风险等级

### 关键命令

```bash
# 分析影响
npx gitnexus impact --target <function_name>

# 查看符号上下文
npx gitnexus context <symbol_name>

# 检测变更范围
npx gitnexus detect-changes
```

---

## 4.3 编写IF/THEN格式的PR描述

### 任务

1. 选择一个已合并的PR
2. 用IF/THEN格式重写描述
3. 对比两种格式的差异

### 模板

```markdown
## 变更说明

## IF 条件
[描述触发条件]

## THEN 结果
[描述执行结果]

## INVARIANT
[描述保持的不变式]

## 测试验证
- [ ] 测试1
- [ ] 测试2
```

---

# 核心知识点总结

---

## 1. 安全扫描

- **三层架构**：SAST + SCA + DAST
- **cargo audit**：依赖CVE扫描
- **Gate豁免**：必须记录原因，符合ISO 9001

## 2. GitNexus代码智能

- **核心功能**：符号索引、调用图谱、影响分析
- **gitnexus_impact**：变更风险评估
- **gitnexus_detect_changes**：变更范围验证

## 3. 形式化证明

- **IF/THEN规则**：PR描述的标准化格式
- **Invariant**：必须保持的系统属性
- **渐进路径**：CMM-4 → CMM-5

---

# 课后作业

---

## 任务

1. 运行 `cargo audit` 扫描你的项目依赖
2. 用 gitnexus_impact 分析一个计划的代码变更
3. 用IF/THEN格式重写一个PR描述

## 预习

- 发布门禁与检查清单
- BetaPhase2 Gate系统

---

<!-- _class: lead -->

# 谢谢！

## 下节课：发布门禁与BetaPhase2

