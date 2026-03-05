# SQLRustGo System Architecture

> **版本**: 1.2
> **更新日期**: 2026-03-05

---

## 1. 系统总体架构

```mermaid
graph TB
    subgraph "Client Layer"
        SQL["SQL Query"]
    end

    subgraph "Query Processing"
        Parser["Parser"]
        LogicalPlan["Logical Plan"]
        Optimizer["Optimizer<br/>Rule + CBO"]
        PhysicalPlan["Physical Plan"]
    end

    subgraph "Execution Engine"
        Executor["Executor"]
        Pipeline["Pipeline Engine"]
    end

    subgraph "Storage Layer"
        StorageEngine["StorageEngine<br/>Trait"]
        FileStorage["FileStorage"]
        MemoryStorage["MemoryStorage"]
    end

    SQL --> Parser
    Parser --> LogicalPlan
    LogicalPlan --> Optimizer
    Optimizer --> PhysicalPlan
    PhysicalPlan --> Executor
    Executor --> Pipeline
    Executor --> StorageEngine
    StorageEngine --> FileStorage
    StorageEngine --> MemoryStorage
```

---

## 2. 查询执行流程

```mermaid
flowchart LR
    subgraph "Frontend"
        SQL["SQL Text"]
        Parser["Parser"]
        AST["AST"]
        LogicalPlan["Logical Plan"]
    end

    subgraph "Optimization"
        Optimizer["Optimizer<br/>Rule + Cost"]
        Memo["Memo<br/>Intermediate Representation"]
        Rules["Transformation<br/>Rules"]
        CostModel["Cost Model"]
    end

    subgraph "Execution"
        Planner["Physical Planner"]
        Executor["Executor"]
        Operators["Operators"]
        Storage["Storage Engine"]
    end

    SQL --> Parser
    Parser --> AST
    AST --> LogicalPlan
    LogicalPlan --> Optimizer
    Optimizer --> Memo
    Memo --> Rules
    Memo --> CostModel
    Rules -.-> Memo
    CostModel -.-> Memo
    Optimizer --> Planner
    Planner --> Executor
    Executor --> Operators
    Operators --> Storage
```

---

## 3. SQLRustGo 模块架构

```mermaid
graph TB
    subgraph "sqlrustgo"
        subgraph "parser"
            Lexer["lexer.rs"]
            Parser["parser.rs"]
            AST["ast.rs"]
        end

        subgraph "planner"
            Logical["logical.rs"]
            Physical["physical.rs"]
        end

        subgraph "optimizer"
            Memo["memo.rs"]
            Rules["rules.rs"]
            Cost["cost.rs"]
        end

        subgraph "executor"
            Operator["operator.rs"]
            Batch["batch.rs"]
            Vectors["vectors/"]
            Engine["engine.rs"]
        end

        subgraph "storage"
            Engine["engine.rs"]
            File["file.rs"]
            Memory["memory.rs"]
            Stats["stats.rs"]
        end

        subgraph "catalog"
            Catalog["catalog.rs"]
        end
    end
```

---

## 4. 执行引擎结构

```mermaid
graph TB
    subgraph "Executor Core"
        Engine["Executor"]
        RecordBatch["RecordBatch"]
    end

    subgraph "Operators"
        Scan["Scan"]
        Filter["Filter"]
        Project["Projection"]
        Join["Join"]
        Aggregate["Aggregate"]
        Sort["Sort"]
        Limit["Limit"]
    end

    subgraph "Vectorized Arrays"
        IntArray["IntArray"]
        FloatArray["FloatArray"]
        StringArray["StringArray"]
        BoolArray["BoolArray"]
    end

    Engine --> RecordBatch
    RecordBatch --> Scan
    RecordBatch --> Filter
    RecordBatch --> Project
    RecordBatch --> Join
    RecordBatch --> Aggregate
    RecordBatch --> Sort
    RecordBatch --> Limit

    Scan --> IntArray
    Scan --> FloatArray
    Scan --> StringArray
    Scan --> BoolArray
```

---

## 5. Pipeline 执行模型

```mermaid
sequenceDiagram
    participant Client as Client
    participant Executor as Executor
    participant Op1 as Scan Operator
    participant Op2 as Filter Operator
    participant Op3 as Project Operator
    participant Storage as Storage Engine

    Client->>Executor: execute(query)

    rect rgb(240, 248, 255)
        Note over Executor,Op1: open() phase
        Executor->>Op1: open()
        Op1->>Op1: initialize
        Executor->>Op2: open()
        Op2->>Op2: initialize
        Executor->>Op3: open()
        Op3->>Op3: initialize
    end

    rect rgb(255, 250, 240)
        Note over Executor,Op1: next_batch() phase
        loop While data available
            Op1->>Storage: read_batch()
            Storage-->>Op1: RecordBatch
            Op1-->>Op2: RecordBatch
            Op2->>Op2: filter()
            Op2-->>Op3: RecordBatch
            Op3->>Op3: project()
            Op3-->>Executor: ResultBatch
        end
    end

    rect rgb(240, 255, 240)
        Note over Executor,Op1: close() phase
        Executor->>Op1: close()
        Executor->>Op2: close()
        Executor->>Op3: close()
    end

    Executor-->>Client: ResultSet
```

---

## 6. Storage Engine 架构

```mermaid
classDiagram
    class StorageEngine {
        <<trait>>
        +read(table: &str) Vec~Record~
        +write(table: &str, records: Vec~Record~) Result~()~
        +scan(table: &str, filter: Option~Filter~) Vec~Record~
        +get_stats(table: &str) Result~TableStats~
    }

    class FileStorage {
        +read(table: &str) Vec~Record~
        +write(table: &str, records: Vec~Record~) Result~()~
        +scan(table: &str, filter: Option~Filter~) Vec~Record~
    }

    class MemoryStorage {
        +read(table: &str) Vec~Record~
        +write(table: &str, records: Vec~Record~) Result~()~
        +scan(table: &str, filter: Option~Filter~) Vec~Record~
    }

    StorageEngine <|.. FileStorage
    StorageEngine <|.. MemoryStorage
```

---

## 7. 统计信息系统

```mermaid
graph LR
    subgraph "Statistics Collection"
        ANALYZE["ANALYZE Command"]
        Collector["StatsCollector"]
    end

    subgraph "Statistics Data"
        TableStats["TableStats"]
        ColumnStats["ColumnStats"]
        Histogram["Histogram"]
    end

    subgraph "Usage"
        CostModel["Cost Model"]
        Optimizer["Optimizer"]
    end

    ANALYZE --> Collector
    Collector --> TableStats
    TableStats --> ColumnStats
    ColumnStats --> Histogram
    TableStats --> CostModel
    CostModel --> Optimizer
```

---

## 8. CBO 成本模型

```mermaid
graph TB
    subgraph "Cost Factors"
        CPU["CPU Cost"]
        IO["I/O Cost"]
        Memory["Memory Cost"]
        Network["Network Cost"]
    end

    subgraph "Cost Calculation"
        ScanCost["Scan Cost"]
        JoinCost["Join Cost"]
        FilterCost["Filter Cost"]
    end

    subgraph "Decision"
        JoinOrder["Join Order"]
        ScanMethod["Scan Method"]
    end

    CPU --> ScanCost
    IO --> ScanCost
    CPU --> JoinCost
    Network --> JoinCost
    CPU --> FilterCost

    ScanCost --> JoinOrder
    JoinCost --> JoinOrder
    FilterCost --> JoinOrder
    ScanCost --> ScanMethod
    IO --> ScanMethod
```

---

## 9. 版本演进

```mermaid
stateDiagram-v2
    [*] --> v1.0: Initial Release
    v1.0 --> v1.1: Basic Engine
    v1.1 --> v1.2: Vector + CBO
    v1.2 --> v1.3: Full Vectorization
    v1.3 --> v2.0: Distributed
    v2.0 --> v3.0: Cloud Native

    note right of v1.2: Interface Freeze
    note right of v2.0: Distributed Execution
    note right of v3.0: Kubernetes Integration
```

---

## 10. 相关文档

- [Cascades Optimizer Design](./cascades_optimizer_design.md)
- [Distributed Scheduler Design](./distributed_scheduler_design.md)
- [Whitepaper](../whitepaper/sqlrustgo_1.2_release_whitepaper.md)
- [Interface Freeze](../whitepaper/sqlrustgo_1.2_interface_freeze.md)
