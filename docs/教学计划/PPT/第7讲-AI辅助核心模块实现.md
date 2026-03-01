---
marp: true
theme: gaia
paginate: true
backgroundColor: #fff
color: #333
---

<!-- _class: lead -->

# 第七讲：AI辅助核心模块实现

## AI增强的软件工程

---

# 课程大纲

1. **AI辅助开发概述**（20分钟）
2. **AI辅助实现词法分析器**（30分钟）
3. **AI辅助实现语法分析器**（30分钟）
4. **AI辅助实现存储引擎**（30分钟）

---

# Part 1: AI辅助开发概述

---

## 1.1 What：AI辅助开发是什么

### 定义

使用AI工具辅助软件开发的各个环节

### AI辅助开发的阶段

- **需求分析**：AI分析需求文档，识别功能点
- **设计阶段**：AI生成设计方案、架构图、接口设计
- **编码阶段**：AI生成代码、补全代码、重构代码
- **测试阶段**：AI生成测试用例、分析覆盖率
- **文档阶段**：AI生成文档、翻译文档

---

## 1.1 What：AI辅助开发是什么（续）

### AI辅助开发工具

- **AI-IDE**：Cursor、TRAE IDE、Claude Code
- **AI编程助手**：GitHub Copilot、TabNine
- **AI聊天机器人**：ChatGPT、Claude、Gemini
- **AI代码审查**：SonarQube AI、DeepCode

---

## 1.2 Why：为什么使用AI辅助开发

### 提高效率

- AI可以快速生成样板代码
- AI可以自动补全代码
- AI可以批量生成测试用例
- **开发效率提升2-5倍**

### 降低门槛

- 新手可以借助AI快速上手
- AI可以解释复杂代码
- AI可以提供最佳实践建议

---

## 1.2 Why：为什么使用AI辅助开发（续）

### 提高质量

- AI可以生成规范的代码
- AI可以自动发现Bug
- AI可以建议重构方案

### 加速学习

- AI可以解释概念
- AI可以提供示例
- AI可以回答问题

### 业界案例

- **GitHub**：Copilot帮助开发者提高55%的编码速度
- **Microsoft**：AI辅助开发减少30%的Bug
- **Google**：AI代码审查提高代码质量

---

## 1.3 How：如何有效使用AI辅助开发

### AI辅助开发流程

1. **明确需求**：清楚描述要实现的功能
2. **设计提示词**：编写清晰、完整的提示词
3. **生成代码**：使用AI生成初始代码
4. **代码审查**：人工审查AI生成的代码
5. **测试验证**：编写测试用例，验证功能
6. **迭代优化**：根据反馈优化代码

---

## 1.3 How：如何有效使用AI辅助开发（续）

### 提示词工程

- **清晰性**：明确任务目标，避免歧义
- **完整性**：提供必要上下文，指定约束条件
- **结构性**：使用结构化格式，分步骤描述
- **可迭代性**：便于反馈修正，支持渐进优化

### 代码审查要点

- **正确性**：代码是否实现了预期功能
- **安全性**：是否存在安全漏洞
- **性能**：是否存在性能问题
- **可读性**：代码是否易于理解
- **可维护性**：代码是否易于修改

---

## 1.3 How：如何有效使用AI辅助开发（续）

### AI辅助开发的局限性

- **上下文窗口限制**：AI无法理解整个大型项目
- **创造性限制**：AI难以创造全新的解决方案
- **领域知识限制**：AI缺乏特定领域的专业知识
- **责任问题**：AI生成的代码需要人工负责

---

# Part 2: AI辅助实现词法分析器

---

## 2.1 词法分析原理

### 词法分析的作用

将SQL字符串转换为Token流

### Token类型

- **关键字**：SELECT, FROM, WHERE, INSERT, UPDATE, DELETE
- **标识符**：表名、列名
- **字面量**：字符串、整数、浮点数、布尔值
- **运算符**：=, <>, <, >, <=, >=, +, -, *, /
- **分隔符**：, ( ) ; .

### 正则表达式

描述Token的模式

### 有限状态机（FSM）

实现词法分析器

---

## 2.2 What：使用AI生成Token定义

### 提示词设计

```
设计一个SQL词法分析器的Token枚举，支持：
- SQL关键字：SELECT, FROM, WHERE, INSERT, UPDATE, DELETE, CREATE, DROP, TABLE
- 标识符：表名、列名
- 字面量：字符串、整数、浮点数、布尔值
- 运算符：=, <>, <, >, <=, >=, +, -, *, /, AND, OR, NOT
- 分隔符：, ( ) ; .
使用Rust枚举实现，包含Debug和Clone trait。
```

---

## 2.2 What：使用AI生成Token定义（续）

### AI输出

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // 关键字
    Select, From, Where, Insert, Update, Delete,
    Create, Drop, Table,
    
    // 标识符和字面量
    Identifier(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BooleanLiteral(bool),
    
    // 运算符
    Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    Plus, Minus, Star, Slash,
    And, Or, Not,
    
    // 分隔符
    Comma, LeftParen, RightParen, Semicolon, Dot,
    
    // 结束标记
    Eof,
}
```

---

## 2.3 How：使用AI生成词法分析器

### 提示词设计

```
基于以下Token定义，实现一个SQL词法分析器：
[Token定义代码]
要求：
1. 实现Lexer结构体，包含input和position字段
2. 实现next_token()方法，返回下一个Token
3. 支持跳过空白字符（空格、制表符、换行）
4. 支持识别关键字和标识符（区分大小写）
5. 支持识别整数和浮点数字面量
6. 支持识别字符串字面量（单引号和双引号）
7. 支持识别运算符和分隔符
8. 使用Rust实现，考虑错误处理
```

---

## 2.3 How：使用AI生成词法分析器（续）

### 代码审查要点

- 检查逻辑是否正确
- 检查错误处理是否完善
- 检查边界条件是否处理

### 测试验证

```rust
#[test]
fn test_lexer_select() {
    let mut lexer = Lexer::new("SELECT * FROM users");
    assert_eq!(lexer.next_token().unwrap(), Token::Select);
    assert_eq!(lexer.next_token().unwrap(), Token::Star);
    assert_eq!(lexer.next_token().unwrap(), Token::From);
    assert_eq!(lexer.next_token().unwrap(), Token::Identifier("users".to_string()));
}
```

---

# Part 3: AI辅助实现语法分析器

---

## 3.1 语法分析原理

### 语法分析的作用

将Token流转换为AST

### 上下文无关文法

描述SQL语法

### 抽象语法树（AST）

表示SQL语句的结构

### 递归下降解析

实现语法分析器

---

## 3.2 What：使用AI生成AST定义

### 提示词设计

```
设计SQL语句的AST节点，支持：
- SELECT语句：columns（列列表）, from_table（表名）, where_clause（WHERE条件）, limit（限制数量）
- INSERT语句：table（表名）, columns（列列表）, values（值列表）
- UPDATE语句：table（表名）, set_clauses（SET子句）, where_clause（WHERE条件）
- DELETE语句：table（表名）, where_clause（WHERE条件）
- CREATE TABLE语句：table_name（表名）, columns（列定义列表）
- DROP TABLE语句：table_name（表名）
使用Rust结构体和枚举实现，包含Debug和Clone trait。
```

---

## 3.3 How：使用AI生成语法分析器

### 提示词设计

```
基于以下Token和AST定义，实现一个SQL语法分析器：
[Token定义代码]
[AST定义代码]
要求：
1. 实现Parser结构体，包含lexer字段
2. 实现parse()方法，解析SQL字符串并返回AST
3. 支持解析SELECT、INSERT、UPDATE、DELETE、CREATE TABLE、DROP TABLE语句
4. 使用Rust实现，考虑错误处理
5. 提供清晰的错误信息
```

---

## 3.3 How：使用AI生成语法分析器（续）

### 代码审查要点

- 检查解析逻辑是否正确
- 检查错误处理是否完善
- 检查是否支持所有SQL语句

### 测试验证

```rust
#[test]
fn test_parser_select() {
    let mut parser = Parser::new("SELECT id, name FROM users WHERE age > 18");
    let stmt = parser.parse().unwrap();
    match stmt {
        Statement::Select(select) => {
            assert_eq!(select.columns, vec!["id", "name"]);
            assert_eq!(select.from_table, "users");
        }
        _ => panic!("Expected SELECT statement"),
    }
}
```

---

# Part 4: AI辅助实现存储引擎

---

## 4.1 存储引擎原理

### 页式存储

数据以页为单位存储

### 缓冲池

管理内存中的页

### B+树索引

加速数据查询

### WAL日志

保证事务持久性

---

## 4.2 What：使用AI设计页结构

### 提示词设计

```
设计数据库存储页结构，要求：
1. 页大小：8KB
2. 页头：页ID（4字节）、页类型（1字节）、空闲空间指针（2字节）
3. 数据区：存储实际数据
4. 方法：read(offset, len)读取数据、write(offset, data)写入数据
5. 使用Rust实现，考虑内存安全
6. 支持序列化和反序列化
```

---

## 4.3 How：使用AI实现缓冲池

### 提示词设计

```
实现数据库缓冲池管理器，要求：
1. 容量可配置（默认100页）
2. 使用LRU置换算法
3. 支持get(page_id)获取页面
4. 支持put(page_id, page)插入页面
5. 支持脏页标记
6. 支持flush()刷盘所有脏页
7. 使用Rust实现，考虑线程安全
8. 使用HashMap和LinkedList实现LRU
```

---

## 4.3 How：使用AI实现缓冲池（续）

### 代码审查要点

- 检查LRU算法是否正确
- 检查并发安全是否保证
- 检查脏页管理是否完善

### 测试验证

```rust
#[test]
fn test_buffer_pool() {
    let mut pool = BufferPool::new(10);
    let page = Page::new(1);
    pool.put_page(1, page).unwrap();
    
    let retrieved = pool.get_page(1).unwrap();
    assert_eq!(retrieved.id, 1);
}
```

---

# 核心知识点总结

---

## 1. AI辅助开发

- **What**：AI辅助开发的定义、阶段、工具
- **Why**：提高效率、降低门槛、提高质量、加速学习
- **How**：开发流程、提示词工程、代码审查、局限性

## 2. AI辅助实现词法分析器

- Token定义
- Lexer实现
- 测试验证

## 3. AI辅助实现语法分析器

- AST定义
- Parser实现
- 测试验证

## 4. AI辅助实现存储引擎

- Page结构
- BufferPool实现
- 测试验证

---

# 课后作业

---

## 任务

1. 完成词法分析器实现
2. 完成语法分析器实现
3. 完成页结构和缓冲池实现
4. 编写测试用例

## 预习

- 测试驱动开发（TDD）
- Alpha版本验收

---

<!-- _class: lead -->

# 谢谢！

## 下节课：测试驱动开发与Alpha版本
