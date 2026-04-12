---
marp: true
theme: gaia
paginate: true
backgroundColor: #fff
color: #333
---

<!-- _class: lead -->

# 第八讲：测试驱动开发与Alpha版本

## AI增强的软件工程

---

# 课程大纲

1. **测试驱动开发（TDD）**（25分钟）
2. **AI辅助测试生成**（25分钟）
3. **Rust测试框架**（20分钟）
4. **Alpha版本验收**（20分钟）

---

# Part 1: 测试驱动开发（TDD）

---

## 1.1 TDD的核心思想

### 测试先行

- 先写测试，再写代码
- 测试定义了代码的预期行为

### 快速反馈

- 测试立即反馈代码是否正确
- 快速发现和修复问题

### 持续重构

- 在测试保护下重构代码
- 保证重构不破坏现有功能

---

## 1.2 Red-Green-Refactor循环

```
┌──────────────────────────────────────────────────────┐
│                                                      │
│   Red ──> Green ──> Refactor ──> Red                │
│    │        │           │          │                 │
│    │        │           │          │                 │
│    ▼        ▼           ▼          ▼                 │
│ 编写失败  编写最少   重构代码   编写新测试            │
│ 的测试    代码通过                                        │
│                                                      │
└──────────────────────────────────────────────────────┘
```

### Red：编写失败的测试

- 测试描述预期行为
- 测试应该失败

### Green：编写最少代码使测试通过

- 只编写必要的代码
- 不需要完美，只需通过测试

### Refactor：重构代码

- 在测试保护下优化代码
- 保持测试通过

---

## 1.3 测试先行的好处

### 更好的设计

- 从使用者角度设计接口
- 代码更易于测试和使用

### 更高的测试覆盖率

- 每个功能都有对应的测试
- 测试覆盖率自然提高

### 更少的Bug

- 测试捕获大部分Bug
- 回归测试防止Bug重现

### 文档作用

- 测试就是活文档
- 展示代码的使用方式

---

## 1.4 TDD实践技巧

### 从简单测试开始

- 先写最简单的测试
- 逐步增加复杂度

### 保持测试独立

- 测试之间不应该有依赖
- 每个测试可以独立运行

### 测试命名规范

```rust
#[test]
fn test_lexer_should_recognize_select_keyword() {
    // 测试内容
}

#[test]
fn test_parser_should_parse_select_statement() {
    // 测试内容
}
```

### 使用AI辅助TDD

- AI可以生成测试用例
- AI可以生成实现代码
- AI可以建议重构方案

---

# Part 2: AI辅助测试生成

---

## 2.1 使用AI生成测试用例

### 提示词设计

```
为以下词法分析器代码生成测试用例：
[Lexer代码]
要求：
1. 测试关键字识别
2. 测试标识符识别
3. 测试数字字面量识别
4. 测试字符串字面量识别
5. 测试运算符识别
6. 使用Rust #[test]属性
```

---

## 2.1 使用AI生成测试用例（续）

### AI输出示例

```rust
#[test]
fn test_keywords() {
    let mut lexer = Lexer::new("SELECT FROM WHERE");
    assert_eq!(lexer.next_token().unwrap(), Token::Select);
    assert_eq!(lexer.next_token().unwrap(), Token::From);
    assert_eq!(lexer.next_token().unwrap(), Token::Where);
}

#[test]
fn test_identifiers() {
    let mut lexer = Lexer::new("users table_name");
    assert_eq!(lexer.next_token().unwrap(), Token::Identifier("users".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::Identifier("table_name".to_string()));
}
```

---

## 2.2 使用AI分析测试覆盖率

### 识别未覆盖的代码路径

- AI可以分析代码，识别未测试的分支
- AI可以建议补充测试

### 生成补充测试建议

```
分析以下代码的测试覆盖率，建议补充测试：
[代码]
[现有测试]
要求：
1. 识别未覆盖的代码路径
2. 建议边界条件测试
3. 建议异常情况测试
```

### 边界条件发现

- AI可以发现容易忽略的边界条件
- 如：空字符串、最大值、最小值等

---

## 2.3 实例：SQLRustGo测试

### 词法分析器测试

```rust
#[test]
fn test_lexer_integer_literal() {
    let mut lexer = Lexer::new("123 456");
    assert_eq!(lexer.next_token().unwrap(), Token::IntegerLiteral(123));
    assert_eq!(lexer.next_token().unwrap(), Token::IntegerLiteral(456));
}

#[test]
fn test_lexer_string_literal() {
    let mut lexer = Lexer::new("'hello' \"world\"");
    assert_eq!(lexer.next_token().unwrap(), Token::StringLiteral("hello".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::StringLiteral("world".to_string()));
}
```

---

## 2.3 实例：SQLRustGo测试（续）

### 语法分析器测试

```rust
#[test]
fn test_parser_select_with_where() {
    let mut parser = Parser::new("SELECT * FROM users WHERE age > 18");
    let stmt = parser.parse().unwrap();
    match stmt {
        Statement::Select(select) => {
            assert_eq!(select.columns, vec!["*"]);
            assert_eq!(select.from_table, "users");
            assert!(select.where_clause.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}
```

---

# Part 3: Rust测试框架

---

## 3.1 #[test] 属性

### 测试函数定义

```rust
#[test]
fn test_addition() {
    assert_eq!(2 + 2, 4);
}
```

### 测试命名规范

- 使用`test_`前缀
- 描述测试的行为
- 如：`test_lexer_should_recognize_keywords`

---

## 3.2 assert! 宏系列

### assert!(expr)

```rust
#[test]
fn test_true() {
    assert!(true);
}
```

### assert_eq!(left, right)

```rust
#[test]
fn test_equality() {
    assert_eq!(2 + 2, 4);
}
```

### assert_ne!(left, right)

```rust
#[test]
fn test_inequality() {
    assert_ne!(2 + 2, 5);
}
```

### 自定义错误消息

```rust
assert_eq!(result, expected, "计算结果不正确，期望{}，实际{}", expected, result);
```

---

## 3.3 测试组织

### 单元测试（#[cfg(test)]）

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
```

### 集成测试（tests/目录）

```
project/
├── src/
│   └── lib.rs
└── tests/
    └── integration_test.rs
```

### 文档测试（doc tests）

```rust
/// 将两个数相加
/// 
/// # Examples
/// ```
/// let result = add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

---

## 3.4 测试覆盖率工具

### cargo-tarpaulin

```bash
cargo tarpaulin --out Html
```

### cargo-llvm-cov

```bash
cargo llvm-cov --html
```

### 覆盖率报告

- 行覆盖率
- 分支覆盖率
- 函数覆盖率

---

# Part 4: Alpha版本验收

---

## 4.1 Alpha版本定义

### 功能完整性

- 核心功能已实现
- 主要功能可运行

### 可运行

- 系统可以启动和运行
- 基本流程可以走通

### 有测试

- 单元测试覆盖核心功能
- 测试覆盖率 ≥ 70%

### 有文档

- README文档
- API文档
- 设计文档

---

## 4.2 功能验收标准

### SQL解析

- ✅ 支持SELECT语句
- ✅ 支持INSERT语句
- ✅ 支持UPDATE语句
- ✅ 支持DELETE语句
- ✅ 支持CREATE TABLE语句

### 存储引擎

- ✅ 支持数据读写
- ✅ 支持基本的页管理
- ✅ 支持缓冲池

### 执行引擎

- ✅ 支持基本查询执行
- ✅ 返回查询结果

---

## 4.3 质量门禁

| 检查项 | 命令 | 状态要求 |
|--------|------|---------|
| 编译 | `cargo build` | 通过 |
| 测试 | `cargo test` | 全部通过 |
| Clippy | `cargo clippy` | 无警告 |
| 格式化 | `cargo fmt --check` | 通过 |
| 覆盖率 | `cargo tarpaulin` | ≥ 70% |

---

## 4.4 Alpha版本发布

### 创建版本标签

```bash
git tag -a v0.1.0-alpha -m "Alpha版本发布"
git push origin v0.1.0-alpha
```

### 编写Release Notes

```markdown
# SQLRustGo v0.1.0-alpha

## 新功能
- 支持基本的SQL解析
- 支持SELECT、INSERT、UPDATE、DELETE语句
- 支持基本的存储引擎

## 已知问题
- 不支持JOIN操作
- 不支持事务

## 下一步计划
- 添加JOIN支持
- 添加事务支持
```

### 发布公告

- 在GitHub创建Release
- 编写发布说明
- 附上安装和使用指南

---

# 核心知识点总结

---

## 1. 测试驱动开发

- **Red-Green-Refactor循环**
- **测试先行的好处**
- **TDD实践技巧**

## 2. AI辅助测试

- **测试用例生成**
- **覆盖率分析**
- **边界条件发现**

## 3. Rust测试框架

- **#[test]属性**
- **assert!宏**
- **测试组织**
- **覆盖率工具**

## 4. Alpha版本验收

- **功能验收标准**
- **质量门禁**
- **发布流程**

---

# 上半学期总结

---

## 知识点回顾

| 周次 | 理论知识 | 实践技能 | 项目产出 |
|------|----------|----------|----------|
| 1 | 软件工程发展历史、Greenfield/Brownfield、AI局限性 | 环境搭建 | 开发环境 |
| 2 | 结构化设计、UML概述、用例图 | PlantUML使用 | 用例图文档 |
| 3 | 面向对象设计、SOLID原则、类图 | 类图绘制 | 类图文档 |
| 4 | 顺序图、状态图、架构图、快速原型法 | UML综合实践 | 设计文档 |
| 5 | 架构设计原理、SQLRustGo架构 | 架构图绘制 | 架构文档 |
| 6 | 功能模块划分、接口设计 | 模块设计 | 模块文档 |
| 7 | AI辅助开发、核心模块实现 | 编码实践 | 核心代码 |
| 8 | TDD、AI辅助测试、Alpha版本验收 | 测试编写 | Alpha版本 |

---

# 课后作业

---

## 任务

1. 补充单元测试
2. 提高测试覆盖率至70%+
3. 运行所有质量检查
4. 完成Alpha版本

## 预习

- 软件治理与分支策略
- Git协作开发

---

<!-- _class: lead -->

# 谢谢！

## 下半学期：协同开发与工程治理
