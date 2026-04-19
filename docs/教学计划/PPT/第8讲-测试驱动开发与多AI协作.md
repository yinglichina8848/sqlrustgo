---
marp: true
theme: gaia
paginate: true
backgroundColor: #fff
color: #333
---

<!-- _class: lead -->

# 第八讲：测试驱动开发与多AI协作

## AI增强的软件工程 - 🕹️ 手动档第1周

---

# 课程大纲

1. **从单点到并行：AI协作模式的转变**（15分钟）
2. **白盒测试与覆盖率详解**（25分钟）
3. **测试驱动开发（TDD）完整实践**（20分钟）
4. **版本命名规范与软件治理**（15分钟）
5. **Alpha版本发布**（5分钟）

---

# Part 1: 从单点到并行：AI协作模式的转变

---

## 1.1 我们走过的路

```
┌─────────────────────────────────────────────────────────────────────┐
│                    AI协作模式的演进                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  第1-6周：🚗 单点开发                                              │
│  ════════════════════════════                                       │
│  • 一个AI工具 + 一个开发者                                          │
│  • 我下指令，AI执行                                                │
│  • 就像"我开车，AI是自动档"                                        │
│                                                                      │
│  第8周+：🚀 并行开发                                                │
│  ════════════════════════════                                       │
│  • 多个AI Agent + 多个功能模块                                     │
│  • AI之间需要协调、合并、审查                                        │
│  • 就像"一个车队"，需要交通规则                                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 1.2 为什么要转向并行开发？

### 单点开发的局限

- 速度慢：一个AI一次只能做一个任务
- 能力有限：复杂系统需要多人协作
- 效率低：等待AI生成时人在空闲

### 并行开发的优势

- 速度快：多个AI同时工作
- 能力强：分工合作完成复杂任务
- 效率高：人负责协调，AI负责执行

---

## 1.3 并行开发带来的新挑战

```
┌─────────────────────────────────────────────────────────────────────┐
│                    并行开发的新问题                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  🚨 问题1：AI之间会"打架"                                          │
│  ════════════════════════════                                       │
│  • 两个AI同时修改同一行代码                                          │
│  • 各自的代码风格不一致                                             │
│  • 一个人写的代码另一个人看不懂                                      │
│                                                                      │
│  🚨 问题2：AI会"跑偏"                                              │
│  ════════════════════════════                                       │
│  • AI忘记整体架构，各自为战                                         │
│  • 有人想做"添加功能"，有人想做"删除代码"                          │
│  • 提交顺序混乱，无法合并                                           │
│                                                                      │
│  🚨 问题3：质量无法保证                                            │
│  ════════════════════════════                                       │
│  • 没有统一的代码规范                                               │
│  • 没有测试验证正确性                                               │
│  • 不知道谁写的代码有问题                                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 1.4 解决方案：软件治理

```
┌─────────────────────────────────────────────────────────────────────┐
│                    解决方案：软件治理                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  🔧 分支策略：让AI各走各的路                                        │
│  ├── 功能分支隔离                                                  │
│  ├── 主干保护                                                      │
│  └── 合并规则                                                      │
│                                                                      │
│  🔧 PR工作流：让AI提交的代码有人审查                               │
│  ├── 代码审查                                                      │
│  ├── 自动化检查                                                    │
│  └── 人工批准                                                      │
│                                                                      │
│  🔧 CI/CD：让AI遵守规则                                           │
│  ├── 自动测试                                                      │
│  ├── 格式检查                                                      │
│  └── 质量门禁                                                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part 2: 白盒测试与覆盖率详解

---

## 2.1 什么是白盒测试？

### 黑盒 vs 白盒测试

| 维度 | 黑盒测试 | 白盒测试 |
|------|---------|---------|
| 视角 | 外部功能 | 内部实现 |
| 依据 | 需求规格 | 代码结构 |
| 重点 | 输入-输出 | 逻辑路径 |
| 工具 | 功能测试 | 覆盖率工具 |
| 例子 | 用户使用流程 | 语句覆盖 |

### 为什么需要白盒测试？

1. **验证代码质量**：确保每个语句都被执行
2. **发现隐藏bug**：覆盖边界条件和异常路径
3. **指导重构**：有测试保护，重构更安全
4. **多AI协作的"契约"**：测试通过=没破坏别人代码

---

## 2.2 覆盖率详解

### 覆盖率度量层次

```
┌─────────────────────────────────────────────────────────────────────┐
│                    覆盖率度量层次 (从低到高)                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │ 路径覆盖 (Path Coverage) - 最高要求                        │    │
│  │ 每个独立路径至少执行一次                                    │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                              ▲                                      │
│                              │                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │ 条件/分支覆盖 (Branch Coverage)                             │    │
│  │ 每个分支条件至少有一次真、有一次假                           │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                              ▲                                      │
│                              │                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │ 语句覆盖 (Statement Coverage) - 最低要求                    │    │
│  │ 每个可执行语句至少执行一次                                   │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 2.3 语句覆盖 (Statement Coverage)

### 什么是语句覆盖？

- **定义**：每个可执行的代码语句至少被执行一次
- **公式**：语句覆盖率 = (已执行语句数 / 总语句数) × 100%
- **目标**：通常要求 ≥ 70%

### 示例

```rust
fn calculate(a: i32, b: i32, op: &str) -> i32 {
    let result = if op == "add" {      // 语句1
        a + b                          // 语句2
    } else if op == "sub" {           // 语句3
        a - b                          // 语句4
    } else {                           // 语句5
        0                              // 语句6
    };
    result                             // 语句7
}
```

**测试用例1**: `calculate(1, 2, "add")`
- 执行语句：1, 2, 7
- 覆盖率：4/7 = 57%

**测试用例2**: `calculate(1, 2, "sub")`
- 执行语句：1, 3, 4, 7
- 覆盖率：4/7 = 57%

**测试用例1+2**: 
- 执行语句：1, 2, 3, 4, 7
- 覆盖率：5/7 = 71%

---

## 2.4 分支覆盖 (Branch Coverage)

### 什么是分支覆盖？

- **定义**：每个条件分支（if/else、match）的真假两个分支至少执行一次
- **公式**：分支覆盖率 = (已执行分支数 / 总分支数) × 100%
- **目标**：通常要求 ≥ 60%

### 示例

```rust
fn check_age(age: i32) -> &'static str {
    if age >= 18 {      // 分支1: age >= 18
        "adult"          // 分支1-true
    } else {            // 分支1: age < 18
        "minor"         // 分支1-false
    }
}

fn grade(score: i32) -> &'static str {
    if score >= 90 {    // 分支1
        "A"
    } else if score >= 80 { // 分支2
        "B"
    } else if score >= 70 { // 分支3
        "C"
    } else {            // 分支4
        "D"
    }
}
```

| 测试用例 | age/score | 覆盖分支 |
|---------|-----------|---------|
| check_age(20) | 成年 | 分支1-T |
| check_age(10) | 未成年 | 分支1-F |
| grade(95) | 95分 | 分支1-T |
| grade(85) | 85分 | 分支1-F, 分支2-T |
| grade(75) | 75分 | 分支1-F, 分支2-F, 分支3-T |
| grade(65) | 65分 | 全部分支覆盖 |

---

## 2.5 路径覆盖 (Path Coverage)

### 什么是路径覆盖？

- **定义**：每个可能的代码执行路径至少执行一次
- **公式**：路径覆盖率 = (已执行路径数 / 总路径数) × 100%
- **注意**：路径数可能指数级增长

### 示例

```rust
fn process(a: bool, b: bool) -> i32 {
    let mut result = 0;
    
    if a {              // 分支1
        result += 1;
    }
    
    if b {              // 分支2
        result += 10;
    }
    
    return result;
}
```

**路径分析**：
- 总路径数：2^2 = 4条
  1. a=T, b=T → result=11
  2. a=T, b=F → result=1
  3. a=F, b=T → result=10
  4. a=F, b=F → result=0

| 测试用例 | 覆盖路径 |
|---------|---------|
| process(true, true) | 路径1 |
| process(true, false) | 路径2 |
| process(false, true) | 路径3 |
| process(false, false) | 路径4 |

### 循环的路径爆炸

```rust
for i in 0..n {  // 如果n=10，有11种路径
    // ...
}
```

**实际做法**：使用**基本路径覆盖**，选择代表性的路径测试

---

## 2.6 覆盖率统计工具

### Rust覆盖率工具

```bash
# 1. cargo-tarpaulin (最常用)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage

# 2. cargo-llvm-cov
cargo install cargo-llvm-cov
cargo llvm-cov --html --output-dir coverage

# 3. grcov (生成多种格式)
cargo install grcov
```

### 覆盖率报告解读

```bash
# 生成覆盖率报告
cargo tarpaulin --out Text

# 输出示例
Filename                      Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
src/parser/lexer.rs               45                 8    82.22%          12                 1    91.67%         120                18    85.00%           0                 0         -
src/parser/parser.rs             78                15    80.77%          24                 3    87.50%         250                45    82.00%           0                 0         -
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                            123                23    81.30%          36                 4    88.89%         370                63    82.97%           0                 0         -
```

### 覆盖率指标说明

| 指标 | 说明 | 目标 |
|------|------|------|
| Line Coverage | 代码行覆盖率 | ≥ 70% |
| Function Coverage | 函数覆盖率 | ≥ 80% |
| Region Coverage | 代码区域覆盖率 | ≥ 70% |
| Branch Coverage | 分支覆盖率 | ≥ 60% |

---

# Part 3: 测试驱动开发（TDD）完整实践

---

## 3.1 TDD的核心思想

### 测试先行

- 先写测试，再写代码
- 测试定义了代码的预期行为
- 测试是"规格说明书"

### 快速反馈

- 测试立即反馈代码是否正确
- 快速发现和修复问题
- 小步前进，每次只改一点

### 持续重构

- 在测试保护下重构代码
- 保证重构不破坏现有功能
- 重构后测试仍然通过

---

## 3.2 Red-Green-Refactor循环

```
┌──────────────────────────────────────────────────────┐
│                                                      │
│   Red ──> Green ──> Refactor ──> Red              │
│    │        │           │          │                 │
│    │        │           │          │                 │
│    ▼        ▼           ▼          ▼                 │
│  编写失败  编写最少   重构代码   编写新测试         │
│  的测试    代码通过  (保持绿色)   (继续循环)       │
│                                                      │
└──────────────────────────────────────────────────────┘
```

### Red (红) - 编写失败的测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        // 先写测试，此时代码还没实现
        // 编译失败或测试失败
        assert_eq!(add(2, 3), 5);
    }
}

// 编译错误：function `add` not found
```

### Green (绿) - 编写最少的代码让测试通过

```rust
// 最简单粗暴的实现
fn add(a: i32, b: i32) -> i32 {
    5  // 硬编码让测试通过
}
```

### Refactor (黄) - 重构代码

```rust
// 重构为正确的实现
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

---

## 3.3 TDD实践示例：为SQLRustGo编写测试

### 场景：测试词法分析器

#### Step 1: 编写失败的测试 (Red)

```rust
// tests/lexer_test.rs

use sqlrustgo_lexer::{Lexer, Token};

#[test]
fn test_keyword_select() {
    let input = "SELECT";
    let mut lexer = Lexer::new(input);
    
    let token = lexer.next_token().unwrap();
    assert_eq!(token.token_type, TokenType::SELECT);
    assert_eq!(token.value, "SELECT");
}

#[test]
fn test_keyword_from() {
    let input = "FROM";
    let mut lexer = Lexer::new(input);
    
    let token = lexer.next_token().unwrap();
    assert_eq!(token.token_type, TokenType::FROM);
}

#[test]
fn test_identifier() {
    let input = "users";
    let mut lexer = Lexer::new(input);
    
    let token = lexer.next_token().unwrap();
    assert_eq!(token.token_type, TokenType::IDENTIFIER);
    assert_eq!(token.value, "users");
}

#[test]
fn test_number() {
    let input = "123";
    let mut lexer = Lexer::new(input);
    
    let token = lexer.next_token().unwrap();
    assert_eq!(token.token_type, TokenType::NUMBER);
    assert_eq!(token.value, "123");
}
```

#### Step 2: 实现代码让测试通过 (Green)

```rust
// src/lexer/mod.rs

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    SELECT,
    FROM,
    WHERE,
    IDENTIFIER,
    NUMBER,
    // ...
}

pub struct Lexer {
    input: String,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.to_string(),
            position: 0,
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        // 跳过空白字符
        self.skip_whitespace();
        
        if self.is_at_end() {
            return Ok(Token::new(TokenType::EOF, "".to_string()));
        }
        
        let ch = self.peek();
        
        // 识别关键字
        if ch.is_alphabetic() {
            return self.read_identifier();
        }
        
        // 识别数字
        if ch.is_numeric() {
            return self.read_number();
        }
        
        // ...
    }
    
    fn read_identifier(&mut self) -> Result<Token, LexError> {
        let mut result = String::new();
        while !self.is_at_end() && self.peek().is_alphanumeric() {
            result.push(self.peek());
            self.advance();
        }
        
        // 检查是否是关键字
        match result.to_uppercase().as_str() {
            "SELECT" => Ok(Token::new(TokenType::SELECT, result)),
            "FROM" => Ok(Token::new(TokenType::FROM, result)),
            "WHERE" => Ok(Token::new(TokenType::WHERE, result)),
            _ => Ok(Token::new(TokenType::IDENTIFIER, result)),
        }
    }
    
    fn read_number(&mut self) -> Result<Token, LexError> {
        let mut result = String::new();
        while !self.is_at_end() && self.peek().is_numeric() {
            result.push(self.peek());
            self.advance();
        }
        Ok(Token::new(TokenType::NUMBER, result))
    }
}
```

#### Step 3: 运行测试验证 (Green)

```bash
cargo test --package lexer
# 输出：test result: ok. 4 passed; 0 failed
```

#### Step 4: 重构 (Refactor)

```rust
// 提取关键字映射表
fn keyword_lookup(s: &str) -> TokenType {
    match s.to_uppercase().as_str() {
        "SELECT" => TokenType::SELECT,
        "FROM" => TokenType::FROM,
        "WHERE" => TokenType::WHERE,
        "INSERT" => TokenType::INSERT,
        "UPDATE" => TokenType::UPDATE,
        "DELETE" => TokenType::DELETE,
        _ => TokenType::IDENTIFIER,
    }
}
```

---

## 3.4 TDD对多AI协作的重要性

### 测试 = 协作的"契约"

| 单点开发 | 并行开发 |
|---------|---------|
| 测试验证功能正确 | 测试验证"我的代码不破坏你的" |
| 测试是开发者的后盾 | 测试是所有AI的共同标准 |
| 随时可以重构 | 任何人改代码都要过测试 |

### 测试让AI之间和平共处

- AI_A修改了功能 → 测试通过 → 证明没破坏AI_B的功能
- 没有测试 → 合并后 → 谁也不知道为什么坏了

---

# Part 4: 版本命名规范与软件治理

---

## 4.1 为什么需要版本规范？

### 没有规范的版本命名

```
❌ 混乱的版本号：
- v1
- v1.0
- version1
- version_1.0.0
- release-1.0
- final_v1.0
```

### 规范的版本命名

```
✅ 清晰的版本号：
- dev/v1.0.0     # 开发版本
- alpha/v1.0.0   # 内部测试
- beta/v1.0.0   # 外部测试
- rc/v1.0.0     # 候选发布
- release/v1.0.0 # 正式发布
```

---

## 4.2 推荐的版本命名规范

### 语义化版本 + 环境前缀

```
┌─────────────────────────────────────────────────────────────────────┐
│                    版本命名规范                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  📦 dev/v1.0.0-dev1                                               │
│  ═══════════════════════════                                       │
│  • 开发中版本                                                      │
│  • 每日构建，可能包含未完成功能                                     │
│  • 开发团队内部使用                                                 │
│                                                                      │
│  📦 alpha/v1.0.0-alpha1                                           │
│  ═══════════════════════════                                       │
│  • 内部测试版本                                                    │
│  • 功能基本完成，侧重测试                                          │
│  • QA团队测试为主                                                 │
│                                                                      │
│  📦 beta/v1.0.0-beta1                                             │
│  ═══════════════════════════                                       │
│  • 外部测试版本                                                    │
│  • 公开招募测试用户                                                │
│  • 收集反馈，修复bug                                               │
│                                                                      │
│  📦 rc/v1.0.0-rc1 → rc/v1.0.0-rc2                                │
│  ═══════════════════════════                                       │
│  • 候选发布版本                                                    │
│  • 除非发现严重bug，否则不会改变                                    │
│  • 冻结新功能，专注bug修复                                         │
│                                                                      │
│  📦 release/v1.0.0                                                 │
│  ═══════════════════════════                                       │
│  • 正式发布版本                                                   │
│  • 生产环境使用                                                    │
│  • 遵循语义化版本 (SemVer)                                         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### SQLRustGo版本命名实践

```bash
# 开发分支
git checkout -b dev/v1.0.0

# Alpha版本
git tag -a alpha/v1.0.0-alpha1 -m "Alpha version"

# Beta版本  
git tag -a beta/v1.0.0-beta1 -m "Beta version"

# RC版本
git tag -a rc/v1.0.0-rc1 -m "Release candidate"

# 正式发布
git tag -a release/v1.0.0 -m "Release v1.0.0"
```

---

## 4.3 版本与分支策略

```
┌─────────────────────────────────────────────────────────────────────┐
│                    分支与版本对应关系                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  main ──────────────────────────────────────────────────────►        │
│   │                                                                │
│   │  release/v1.0.0 ──► 正式发布                                   │
│   │                                                                │
│   ├                                                                │
│   │  rc/v1.0.0-rc1 ──► 候选发布                                    │
│   │                                                                │
│   ├                                                                │
│   │  beta/v1.0.0-beta1 ──► 公开测试                               │
│   │                                                                │
│   ├                                                                │
│ develop ─────────────────────────────────────────►                  │
│   │                                                                │
│   │  alpha/v1.0.0-alpha1 ──► 内部测试                              │
│   │                                                                │
│   ├                                                                │
│   │  dev/v1.0.0-dev1 ──► 开发版本                                 │
│   │                                                                │
│   ├                                                                │
│ feature/* ──► 功能开发分支                                         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 4.4 本周实验任务

### 任务1：运行测试并分析覆盖率（20分钟）
- 执行 `cargo tarpaulin`
- 分析覆盖率报告
- 理解语句覆盖、分支覆盖的区别

### 任务2：AI辅助生成测试用例（30分钟）
- 使用AI为parser生成测试
- 补充边界条件测试

### 任务3：补充测试用例（30分钟）
- 补充AI未覆盖的边界情况
- 确保覆盖率≥70%

### 任务4：运行质量门禁（20分钟）
- 执行所有门禁检查
- 准备发布Alpha版本

### 任务5：创建Alpha版本（20分钟）
- 使用 `alpha/v0.1.0-alpha1` 格式
- 推送版本标签
- 创建GitHub Release

---

## 4.5 本周要点总结

### 核心转变

- 从"一个人+一个AI"到"多AI协作"
- 从"自己管自己"到"规则约束"

### 测试覆盖率

- 语句覆盖：每个语句执行一次
- 分支覆盖：每个分支真假各一次
- 路径覆盖：每个执行路径一次（难以完全实现）

### 版本规范

- dev/v1.0.0 - 开发版本
- alpha/v1.0.0 - 内部测试
- beta/v1.0.0 - 外部测试
- rc/v1.0.0 - 候选发布
- release/v1.0.0 - 正式发布

---

# 课后作业

---

## 思考题

1. 语句覆盖率达到100%是否意味着没有bug？为什么？

2. 如果有3个AI同时开发，一个改Parser，一个改Executor，一个改Storage，可能会出现什么冲突？

3. 为什么说"测试"是AI协作的"契约"？

4. 版本命名不规范会带来什么问题？

---

## 预习

- 软件治理与分支策略
- Git协作流程
- 分支保护规则

---

<!-- _class: lead -->

# 谢谢！

## 下节课：软件治理与分支策略

---
