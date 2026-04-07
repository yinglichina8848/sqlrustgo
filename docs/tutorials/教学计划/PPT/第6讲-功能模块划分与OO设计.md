---
title: 第六讲：功能模块划分与接口设计
theme: gaia
---

<!-- _class: 领导 -->

# 第六讲：功能模块划分与接口设计

## AI增强的软件工程

---

# 课程大纲

1. **功能模块划分**（25分钟）
2. **SQLRustGo功能模块划分**（30分钟）
3. **接口设计原则与实践**（20分钟）
4. **AI辅助接口设计实践**（10分钟）

---

# Part 1: 功能模块划分

---

## 1.1 What：什么是功能模块

### 定义

功能模块是具有独立功能的软件单元，通过接口对外提供服务

### 模块的特性

- **独立性**：模块可以独立开发、测试、部署
- **可重用性**：模块可以在不同场景中重用
- **可替换性**：模块可以被其他实现替换
- **封装性**：模块隐藏内部实现细节

---

## 1.1 What：什么是功能模块（续）

### 模块的粒度

- **粗粒度模块**：功能完整、接口复杂（如存储引擎）
- **细粒度模块**：功能单一、接口简单（如词法分析器）

### 模块划分原则

- **高内聚**：模块内部元素紧密相关
- **低耦合**：模块之间依赖最小化
- **单一职责**：一个模块只负责一个功能
- **接口稳定**：模块接口应该稳定，内部实现可以变化

---

## 1.2 Why：为什么需要功能模块划分

### 复杂性管理

- 大型系统包含数百个功能
- 模块化将复杂问题分解为简单问题
- 降低认知负荷

### 团队协作

- 不同团队可以负责不同模块
- 减少代码冲突
- 提高并行开发效率

---

## 1.2 Why：为什么需要功能模块划分（续）

### 可维护性

- 模块化使代码更易理解和修改
- Bug定位更精确
- 重构范围更可控

### 可测试性

- 模块可以独立测试
- 测试覆盖率更高
- Mock和Stub更容易实现

### 可扩展性

- 新功能可以作为新模块添加
- 现有模块可以独立升级
- 支持插件化架构

---

## 1.2 Why：为什么需要功能模块划分（续）

### 业界案例

- **Linux内核**：模块化设计支持动态加载驱动
- **Chrome浏览器**：多进程架构提高稳定性
- **VS Code**：插件架构支持扩展

---

## 1.3 How：如何进行功能模块划分

### 模块划分方法

| 方法 | 依据 | 示例 | 优点 | 缺点 |
|------|------|------|------|------|
| 按功能划分 | 业务功能 | 用户管理、订单管理 | 符合业务逻辑 | 可能导致功能耦合 |
| 按层次划分 | 系统层次 | 表示层、业务层、数据层 | 结构清晰 | 跨层次调用复杂 |
| 按数据划分 | 数据实体 | 用户模块、商品模块 | 数据一致性好 | 可能产生数据冗余 |

---

## 1.3 How：如何进行功能模块划分（续）

### 模块划分流程

1. **需求分析**：识别系统功能
2. **功能聚类**：将相关功能归类
3. **模块识别**：识别候选模块
4. **接口设计**：设计模块接口
5. **依赖分析**：分析模块依赖关系
6. **迭代优化**：根据反馈优化模块划分

### 模块划分评估

- **内聚度评估**：模块内部元素的相关程度
- **耦合度评估**：模块之间的依赖程度
- **复杂度评估**：模块的复杂程度
- **可测试性评估**：模块是否易于测试

---

## 1.3 How：如何进行功能模块划分（续）

### AI辅助模块划分

```
分析以下数据库系统的需求，进行功能模块划分：
[需求文档]
要求：
1. 识别核心功能模块
2. 说明每个模块的职责
3. 分析模块之间的依赖关系
4. 评估模块划分的合理性（高内聚、低耦合）
```

---

# Part 1.4: 使用UML进行OOA/OOD模块设计

---

## 1.4.1 UML在模块设计中的作用

UML（统一建模语言）提供了一套标准化的图形建模工具，可以帮助我们在面向对象分析（OOA） 和面向对象设计（OOD） 阶段系统地进行模块划分与接口设计。

| UML图类型 | 在模块设计中的作用 | 对应OOA/OOD阶段 |
|----------|------------------|----------------|
| 用例图 | 捕获系统功能需求，识别参与者，界定系统边界，为模块划分提供顶层功能清单 | OOA |
| 概念类图 | 发现业务实体及关系，识别候选模块（每个类或一组类可能成为一个模块） | OOA |
| 活动图/状态图 | 分析核心业务流程，帮助识别功能内聚的模块边界 | OOA |
| 设计类图 | 定义模块的静态结构：类的属性、方法、可见性；明确模块对外接口 | OOD |
| 顺序图 | 描述对象间的交互时序，验证模块接口的完整性和调用关系 | OOD |
| 组件图 | 将类或包组织为可部署的组件（模块），定义组件提供的接口和依赖的接口 | OOD |
| 包图 | 划分顶层模块，展示模块间的依赖关系，指导项目目录结构 | OOD |

---

## 1.4.2 OOA阶段：从需求到候选模块

### 步骤1：绘制用例图

根据需求文档，识别所有参与者（用户、外部系统）和用例（系统提供的完整功能）。

用例图的系统边界框直接对应模块划分的顶层范围。

**示例：SQLRustGo核心用例图**

```plantuml
@startuml
left to right direction
actor 用户
actor 管理员

rectangle SQLRustGo {
  usecase "执行查询" as UC1
  usecase "插入数据" as UC2
  usecase "更新数据" as UC3
  usecase "删除数据" as UC4
  usecase "创建表" as UC5
  usecase "管理索引" as UC6
}

用户 --> UC1
用户 --> UC2
用户 --> UC3
用户 --> UC4
管理员 --> UC5
管理员 --> UC6
@enduml
```

---

## 1.4.2 OOA阶段：从需求到候选模块（续）

### 步骤2：建立概念类图

使用名词分析法从用例描述中提取候选类（业务实体）。

识别类之间的关联、聚合、组合、继承关系。

概念类图中只写类名和属性名，不写方法（分析阶段不关心实现）。

**示例：SQLRustGo概念类图（简化）**

```plantuml
@startuml
class 查询请求 {
  +sql文本
  +参数列表
}

class 执行计划 {
  +操作符树
  +预估成本
}

class 数据表 {
  +表名
  +列定义
  +统计信息
}

class 索引 {
  +索引名
  +索引列
  +类型(BTree/Hash)
}

查询请求 "1" --> "1" 执行计划
执行计划 "1" --> "*" 数据表
数据表 "1" --> "*" 索引
@enduml
```

---

## 1.4.2 OOA阶段：从需求到候选模块（续）

### 步骤3：基于概念类图初步分组（候选模块）

将高内聚的类归入同一个模块。例如：

- **Parser模块**：包含查询请求、词法单元、语法树节点等类。
- **Planner模块**：包含执行计划、优化规则、成本估算器等类。
- **Executor模块**：包含操作符、执行上下文、记录批次等类。
- **Storage模块**：包含数据表、索引、页、缓冲池、事务日志等类。

---

## 1.4.3 OOD阶段：细化模块与接口设计

### 步骤4：绘制设计类图（细化接口）

将概念类转化为具体的设计类，添加方法签名、属性类型、可见性（+ public, - private）。

明确接口（interface/trait），将模块的公共API抽象出来。

**示例：Storage模块的设计类图（部分）**

```plantuml
@startuml
interface StorageEngine {
  + read(table: String, key: Key): Result<Record>
  + write(table: String, record: Record): Result<()>
  + scan(table: String, filter: Filter): Result<Vec<Record>>
  + create_table(schema: Schema): Result<()>
}

class BPlusTree {
  - root_page_id: PageId
  - buffer_pool: BufferPool
  + insert(key: Key, value: Value): Result<()>
  + search(key: Key): Result<Option<Value>>
  + range_scan(start: Key, end: Key): Result<Vec<(Key, Value)>>
}

class BufferPool {
  - frames: Vec<Page>
  - lru: LruCache
  + get_page(id: PageId): Result<&Page>
  + put_page(page: Page): Result<()>
  + flush_all(): Result<()>
}

StorageEngine <|.. BPlusTree
BPlusTree --> BufferPool
@enduml
```

---

## 1.4.3 OOD阶段：细化模块与接口设计（续）

### 步骤5：绘制顺序图（验证接口交互）

选择一个核心用例（如"执行查询"），绘制对象间的消息交互。

检查模块接口的调用顺序、参数传递、返回值是否合理，发现遗漏或冗余的接口。

**示例：SQL查询执行的顺序图**

```plantuml
@startuml
actor 用户
participant "Parser模块" as P
participant "Planner模块" as PL
participant "Executor模块" as E
participant "Storage模块" as S

用户 -> P: SQL文本
P -> P: 词法/语法分析
P --> PL: AST

PL -> PL: 逻辑优化
PL -> PL: 物理优化
PL --> E: 物理执行计划

E -> E: 构建算子树
loop 每个批次
  E -> S: next_batch()
  S --> E: RecordBatch
  E -> E: 执行算子运算
end
E --> 用户: 结果集
@enduml
```

---

## 1.4.3 OOD阶段：细化模块与接口设计（续）

### 步骤6：绘制组件图（定义模块边界与依赖）

每个组件对应一个独立部署/编译单元（如Rust crate、Java jar）。

用接口（lollipop符号）表示组件提供的服务，用依赖（虚线箭头）表示组件需要的接口。

组件图是模块划分的最终输出，直接指导项目代码结构。

**示例：SQLRustGo组件图**

```plantuml
@startuml
component "Parser" as P
component "Planner" as PL
component "Executor" as E
component "Storage" as S
component "Common" as C

P ..> C : <<use>>
PL ..> C : <<use>>
E ..> C : <<use>>
S ..> C : <<use>>

P --> PL : AST
PL --> E : PhysicalPlan
E --> S : StorageEngine API
@enduml
```

---

## 1.4.4 将UML模型映射到SQLRustGo模块划分

| UML模型 | 映射结果 | 在代码中的体现 |
|---------|----------|----------------|
| 组件图 | 模块边界 | Cargo.toml中的crate依赖 |
| 设计类图中的接口（trait） | 模块公共API | pub trait StorageEngine { ... } |
| 设计类图中的具体类 | 模块内部实现 | pub struct BPlusTree { ... } |
| 顺序图中的交互 | 模块间的调用关系 | 函数调用、消息传递 |
| 包图 | 源代码目录结构 | src/parser/, src/optimizer/等 |

**实践建议：**

- 在编写任何代码之前，先用UML组件图和包图完成顶层模块划分。
- 为每个核心模块定义接口（trait），通过设计类图固定下来。
- 使用顺序图验证关键流程的接口调用，确保模块之间的依赖合理（无循环依赖）。
- 将UML模型作为设计文档的一部分，提交到代码仓库（可使用PlantUML源码），便于团队同步和后续维护。

---

## 1.4.5 AI辅助UML建模

AI工具（如Claude、ChatGPT）可以根据需求描述快速生成PlantUML代码：

**提示词示例：**

```text
为SQLRustGo数据库系统设计UML组件图，包含以下模块：
- Parser（解析器）
- Optimizer（优化器）
- Executor（执行器）
- Storage（存储引擎）
- Common（公共组件）
要求：显示模块间的依赖关系和提供的接口。
```

AI可以输出可直接渲染的PlantUML源码，大幅提升建模效率。但需要人工审查模型的正确性和合理性。

---

## 1.4.6 完整模块设计案例：Parser模块从OOA到OOD

本案例展示如何从需求到完整设计，串联所有7张UML图：

**Step 1: 用例图**

识别Parser模块的参与者和功能：

```plantuml
@startuml
left to right direction
actor "Planner模块"
actor "客户端"

rectangle "Parser模块" {
  usecase "词法分析" as UC1
  usecase "语法分析" as UC2
  usecase "生成AST" as UC3
  usecase "SQL验证" as UC4
}

客户端 --> UC1
客户端 --> UC2
客户端 --> UC3
客户端 --> UC4
"Planner模块" --> UC3
@enduml
```

**Step 2: 概念类图**

从用例中提取业务实体：

```plantuml
@startuml
class "SQL语句" {
  +sql文本
  +参数列表
}

class "词法单元" {
  +类型
  +值
  +位置
}

class "抽象语法树" {
  +根节点
  +语句列表
}

class "语法节点" {
  +类型
  +子节点
}

"SQL语句" "1" --> "*" "词法单元"
"词法单元" "*" --> "1" "抽象语法树"
"抽象语法树" "1" --> "*" "语法节点"
@enduml
```

**Step 3: 活动图**

描述Parser的工作流程：

```plantuml
@startuml
start
:接收SQL语句;
:词法分析生成Token流;
diamond "语法正确?"
  -> 是: 继续
  -> 否: 生成语法错误
:语法分析构建AST;
diamond "语义正确?"
  -> 是: 输出AST
  -> 否: 生成语义错误
stop
@enduml
```

**Step 4: 顺序图**

验证对象间的交互：

```plantuml
@startuml
actor "客户端" as Client
participant "Parser" as P
participant "Lexer" as L
participant "AST" as A

Client -> P: parse("SELECT * FROM users")
P -> L: tokenize("SELECT * FROM users")
L --> P: [Token流]
P -> P: 语法分析
P --> A: 构建AST
A --> P: 返回AST
P --> Client: 返回AST
@enduml
```

**Step 5: 状态图**

描述Parser的状态变化：

```plantuml
@startuml
[*] --> Idle

Idle --> Tokenizing: 开始词法分析
Tokenizing --> Parsing: 词法分析完成
Parsing --> Validating: 语法分析完成
Validating --> Success: 验证通过
Validating --> Error: 验证失败
Success --> Idle: 重置
Error --> Idle: 重置
@enduml
```

**Step 6: 组件图**

定义模块边界和依赖：

```plantuml
@startuml
component "Parser" as P
component "Common" as C

P ..> C : <<use>>

P --> C : Token定义
P --> C : 错误处理
@enduml
```

**Step 7: 设计类图**

详细定义接口和实现：

```plantuml
@startuml
interface Lexer {
  + tokenize(sql: &str): Result<Vec<Token>, LexError>
}

interface Parser {
  + parse(tokens: &[Token]): Result<AST, ParseError>
  + validate(ast: &AST): Result<(), ValidationError>
}

class SqlLexer {
  - keywords: HashSet<String>
  - symbols: HashSet<char>
  + tokenize(sql: &str): Result<Vec<Token>, LexError>
}

class SqlParser {
  - lexer: Box<dyn Lexer>
  + parse(tokens: &[Token]): Result<AST, ParseError>
  + validate(ast: &AST): Result<(), ValidationError>
}

class Token {
  + token_type: TokenType
  + value: String
  + position: Position
}

class AST {
  + root: Box<dyn Node>
  + statements: Vec<Box<dyn Node>>
  + accept(visitor: &mut dyn Visitor): Result<(), Error>
}

Lexer <|.. SqlLexer
Parser <|.. SqlParser
SqlParser --> Lexer
SqlParser --> AST
AST --> Token
@enduml
```

---

## 1.4.7 UML验证检查表

| 图表类型 | 检查项目 | 常见问题 | 解决方案 |
|---------|----------|----------|----------|
| **用例图** | 参与者是否完整？ | 遗漏外部系统 | 列出所有与模块交互的实体 |
| | 用例是否覆盖所有功能？ | 功能遗漏 | 对照需求文档逐条检查 |
| | 关系是否正确？ | 箭头方向错误 | 参与者发起用例，不是相反 |
| **概念类图** | 类是否完整？ | 遗漏业务实体 | 从用例描述中提取所有名词 |
| | 关系是否合理？ | 关系类型错误 | 明确是关联、聚合还是组合 |
| | 属性是否必要？ | 属性过多 | 只保留核心属性，避免实现细节 |
| **活动图** | 流程是否完整？ | 缺少分支或终止 | 确保所有路径都有终点 |
| | 决策是否合理？ | 逻辑不清晰 | 使用明确的条件判断 |
| **顺序图** | 交互是否合理？ | 消息顺序错误 | 按照实际调用顺序排列 |
| | 激活条是否正确？ | 激活条缺失 | 显示对象的活动时间 |
| **状态图** | 状态是否完整？ | 遗漏状态 | 分析对象的所有可能状态 |
| | 转换是否合理？ | 转换条件不明确 | 为每个转换添加触发事件 |
| **组件图** | 依赖是否单向？ | 循环依赖 | 确保依赖方向一致（自下而上） |
| | 接口是否清晰？ | 接口不明确 | 使用lollipop符号表示提供的接口 |
| **设计类图** | 接口是否合理？ | 接口过大 | 遵循ISP原则，接口小型化 |
| | 可见性是否正确？ | 过度暴露 | 只暴露必要的方法和属性 |
| | 关系是否清晰？ | 关系复杂 | 简化关系，使用依赖注入 |

---

## 1.4.8 AI辅助UML建模的迭代示例

**第一轮：生成初步用例图**

提示词：
```
为SQLRustGo的Parser模块生成用例图，参与者包括Planner和客户端，用例包括词法分析、语法分析、生成AST、SQL验证。
```

AI输出可能缺少系统边界。

**第二轮：优化用例图**

提示词：
```
修改之前的用例图，添加系统边界矩形，明确标注"Parser模块"，并确保所有参与者都在系统边界外。
```

**第三轮：生成概念类图**

提示词：
```
基于用例图，为Parser模块生成概念类图，包含SQL语句、词法单元、抽象语法树、语法节点等类，以及它们之间的关系。
```

AI输出可能缺少属性。

**第四轮：完善概念类图**

提示词：
```
为概念类图添加属性：SQL语句包含sql文本和参数列表，词法单元包含类型、值和位置，抽象语法树包含根节点和语句列表。
```

通过多轮迭代，可以逐步完善AI生成的UML图，确保其正确性和完整性。

---

# Part 2: SQLRustGo功能模块划分

---

## 2.1 模块划分方案

```
┌─────────────────────────────────────────────────────────┐
│                    Parser 模块                           │
│  • Lexer：词法分析器                                     │
│  • Parser：语法分析器                                    │
│  • AST：抽象语法树                                       │
│  职责：将SQL字符串转换为AST                              │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                    Planner 模块                          │
│  • LogicalPlanner：逻辑规划器                            │
│  • PhysicalPlanner：物理规划器                           │
│  • Optimizer：查询优化器                                 │
│  职责：生成优化的执行计划                                │
└─────────────────────────────────────────────────────────┘
```

---

## 2.1 模块划分方案（续）

```
                          ↓
┌─────────────────────────────────────────────────────────┐
│                    Executor 模块                         │
│  • ExecutionEngine：执行引擎                             │
│  • Operators：执行算子                                   │
│  职责：执行查询并返回结果                                │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                    Storage 模块                          │
│  • PageManager：页管理器                                 │
│  • BufferPool：缓冲池                                   │
│  • BPlusTree：B+树索引                                  │
│  • WAL：预写日志                                        │
│  • TransactionManager：事务管理器                        │
│  职责：数据存储、索引、事务                              │
└─────────────────────────────────────────────────────────┘
```

---

## 2.1 模块划分方案（续）

```
                          ↓
┌─────────────────────────────────────────────────────────┐
│                    Common 模块                           │
│  • Value：数据值类型                                     │
│  • DataType：数据类型枚举                                │
│  • SqlError：错误类型                                   │
│  职责：公共数据类型和工具                                │
└─────────────────────────────────────────────────────────┘
```

---

## 2.2 模块依赖关系

### 依赖方向

- **Parser → Common**：依赖公共数据类型
- **Planner → Parser, Common**：依赖AST和公共类型
- **Executor → Planner, Storage, Common**：依赖执行计划和存储层
- **Storage → Common**：依赖公共数据类型
- **所有模块 → Common**：依赖公共类型

### 依赖原则

- **上层依赖下层，下层不依赖上层**
- **避免循环依赖**
- **依赖抽象而非具体实现**

---

## 2.3 模块接口设计

### Parser模块接口

```rust
pub trait Lexer {
    fn next_token(&mut self) -> Result<Token, SqlError>;
}

pub trait Parser {
    fn parse(&mut self, sql: &str) -> Result<Statement, SqlError>;
}
```

### Planner模块接口

```rust
pub trait LogicalPlanner {
    fn plan(&self, stmt: &Statement) -> Result<LogicalPlan, SqlError>;
}

pub trait PhysicalPlanner {
    fn plan(&self, logical: &LogicalPlan) -> Result<PhysicalPlan, SqlError>;
}
```

---

## 2.3 模块接口设计（续）

### Executor模块接口

```rust
pub trait ExecutionEngine {
    fn execute(&mut self, plan: &PhysicalPlan) -> Result<ExecutionResult, SqlError>;
}
```

### Storage模块接口

```rust
pub trait PageManager {
    fn read_page(&self, page_id: PageId) -> Result<Page, SqlError>;
    fn write_page(&mut self, page: &Page) -> Result<(), SqlError>;
}

pub trait BufferPool {
    fn get_page(&mut self, page_id: PageId) -> Result<&Page, SqlError>;
    fn put_page(&mut self, page: Page) -> Result<(), SqlError>;
}
```

---

# Part 3: 接口设计原则与实践

---

## 3.1 What：什么是接口设计

### 接口的定义

模块之间交互的契约

### 接口的组成

- **方法签名**：方法名、参数、返回值
- **行为契约**：前置条件、后置条件、不变式
- **错误处理**：错误类型、错误传播

### 接口的类型

- **同步接口**：调用者等待结果
- **异步接口**：调用者不等待结果
- **流式接口**：返回数据流

---

## 3.2 Why：接口设计的重要性

### 解耦

- 接口降低模块之间的耦合度

### 可测试性

- 接口使Mock和Stub更容易实现

### 可替换性

- 接口允许替换实现而不影响调用者

### 可扩展性

- 接口支持新功能的添加

### 文档作用

- 接口本身就是文档

---

## 3.3 How：接口设计原则

### 接口隔离原则（ISP）

- 接口要小而专一
- 客户端不应该依赖它不需要的接口

### 最小知识原则（Law of Demeter）

- 模块只与直接的朋友通信
- 减少模块之间的依赖

### 契约式设计（Design by Contract）

- 明确前置条件、后置条件、不变式
- 使用类型系统保证契约

---

## 3.3 How：接口设计原则（续）

### 错误处理原则

- 使用Result类型表示可能失败的操作
- 错误类型应该包含足够的信息
- 错误应该向上传播到合适的处理层

### Rust接口设计实践

- 使用trait定义接口
- 使用泛型支持多种实现
- 使用生命周期标注引用关系
- 使用Send和Sync标记线程安全

---

# Part 4: AI辅助接口设计实践

---

## 4.1 使用AI设计接口

### 提示词设计

```
设计数据库缓冲池的接口，要求：
1. 支持获取页面：get_page(page_id)
2. 支持插入页面：put_page(page)
3. 支持LRU置换算法
4. 支持脏页标记
5. 使用Rust trait定义
6. 考虑线程安全
```

### 评估接口设计

- 接口是否简洁？
- 接口是否完整？
- 接口是否稳定？

---

## 4.2 接口评审

### 评审要点

- 接口是否简洁？
- 接口是否完整？
- 接口是否稳定？
- 错误处理是否合理？

### 评审方法

- 同行评审（Peer Review）
- 接口评审会议
- AI辅助评审

---

# 核心知识点总结

---

## 1. 功能模块划分

- **What**：功能模块的定义、特性、粒度、划分原则
- **Why**：复杂性管理、团队协作、可维护性、可测试性、可扩展性
- **How**：划分方法、划分流程、评估方法、AI辅助划分

## 2. UML进行OOA/OOD模块设计

- **What**：UML图类型及作用
- **Why**：系统地进行模块划分与接口设计
- **How**：OOA阶段（用例图、概念类图）、OOD阶段（设计类图、顺序图、组件图）
- **实践**：将UML模型映射到SQLRustGo模块划分
- **AI辅助**：使用AI生成PlantUML代码

## 3. SQLRustGo模块划分

- Parser模块、Planner模块、Executor模块、Storage模块、Common模块
- 模块依赖关系
- 模块接口设计

## 4. 接口设计

- 接口设计原则：ISP、最小知识原则、契约式设计、错误处理原则
- Rust接口设计实践
- AI辅助接口设计

---

# 课后作业

---

## 任务

1. 完成SQLRustGo模块划分文档
2. 设计所有模块的接口
3. 绘制模块依赖图

## 预习

- AI辅助开发概述
- 词法分析器实现

---

<!-- _class: 领导 -->

# 谢谢！

## 下节课：AI辅助核心模块实现
