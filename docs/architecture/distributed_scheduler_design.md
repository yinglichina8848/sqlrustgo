# SQLRustGo 2.0 分布式调度器设计

> **版本**: 2.0 (规划中)
> **更新日期**: 2026-03-05

---

## 1. 概述

SQLRustGo 2.0 引入**分布式执行框架**，支持多节点并行查询执行。

### 核心目标

- **水平扩展**: 通过添加节点提升性能
- **数据分片**: 支持数据分布式存储
- **查询并行**: 支持分布式查询优化
- **容错处理**: 支持节点故障恢复

---

## 2. 系统架构

```mermaid
graph TB
    subgraph "Client"
        SQL["SQL Query"]
    end

    subgraph "Coordinator Node"
        Parser["Parser"]
        Planner["Distributed Planner"]
        Optimizer["Optimizer"]
        Scheduler["Task Scheduler"]
        Coordinator["Query Coordinator"]
    end

    subgraph "Worker Nodes"
        subgraph "Node 1"
            Worker1["Worker 1"]
            Storage1["Storage"]
        end
        subgraph "Node 2"
            Worker2["Worker 2"]
            Storage2["Storage"]
        end
        subgraph "Node 3"
            Worker3["Worker 3"]
            Storage3["Storage"]
        end
    end

    SQL --> Parser
    Parser --> Planner
    Planner --> Optimizer
    Optimizer --> Scheduler
    Scheduler --> Coordinator
    Coordinator --> Worker1
    Coordinator --> Worker2
    Coordinator --> Worker3
    Worker1 --> Storage1
    Worker2 --> Storage2
    Worker3 --> Storage3
```

---

## 3. DAG 执行图

```mermaid
flowchart TB
    subgraph "Query DAG"
        Q["Query"]
        
        subgraph "Stage 1"
            S1["Scan"]
            F1["Filter"]
        end
        
        subgraph "Stage 2"
            J["Join"]
        end
        
        subgraph "Stage 3"
            A["Aggregate"]
        end
        
        subgraph "Stage 4"
            O["Order By"]
            L["Limit"]
        end
        
        R["Result"]
    end
    
    Q --> S1
    S1 --> F1
    F1 --> J
    J --> A
    A --> O
    O --> L
    L --> R
    
    style S1 fill:#e1f5fe
    style J fill:#fff3e0
    style A fill:#e8f5e9
    style O fill:#fce4ec
    style L fill:#f3e5f5
```

---

## 4. Scheduler 任务

调度器负责：

```mermaid
graph LR
    subgraph "Scheduler Responsibilities"
        Split["Task Splitting"]
        Assign["Node Assignment"]
        Monitor["Execution Monitor"]
        Retry["Failure Retry"]
    end
```

### 4.1 任务拆分

```mermaid
flowchart LR
    Query["SQL Query"]
    
    subgraph "Logical Plan"
        Scan["Scan"]
        Join["Join"]
        Agg["Aggregate"]
    end
    
    subgraph "DAG"
        Stage1["Stage 1: Scan"]
        Stage2["Stage 2: Join"]
        Stage3["Stage 3: Aggregate"]
    end
    
    Query --> Scan
    Scan --> Join
    Join --> Agg
    
    Scan --> Stage1
    Join --> Stage2
    Agg --> Stage3
```

---

## 5. Task 调度模型

```mermaid
classDiagram
    class QueryTask {
        +task_id: TaskId
        +stage_id: StageId
        +plan: PhysicalPlan
        +input_partitions: Vec~Partition~
        +output_partition: Option~Partition~
    }

    class Stage {
        +stage_id: StageId
        +tasks: Vec~QueryTask~
        +parallelism: usize
        +partition_strategy: PartitionStrategy
    }

    class Query {
        +query_id: QueryId
        +stages: Vec~Stage~
        +status: QueryStatus
    }

    Query "1" --> "*" Stage
    Stage "1" --> "*" QueryTask
```

---

## 6. Stage 执行

### 6.1 Stage 定义

Stage = 一组并行执行的 Task

```mermaid
graph LR
    subgraph "Stage N"
        T1["Task 1<br/>Node 1"]
        T2["Task 2<br/>Node 2"]
        T3["Task 3<br/>Node 3"]
    end
    
    Input["Input Data"]
    Output["Output Data"]
    
    Input --> T1
    Input --> T2
    Input --> T3
    
    T1 --> Output
    T2 --> Output
    T3 --> Output
```

### 6.2 Stage 依赖

```mermaid
flowchart LR
    S1["Stage 1<br/>Scan"] --> S2["Stage 2<br/>Join"]
    S2 --> S3["Stage 3<br/>Aggregate"]
    S3 --> S4["Stage 4<br/>Result"]
    
    S1 -->|Task 1| W1["Worker 1"]
    S1 -->|Task 2| W2["Worker 2"]
    S1 -->|Task 3| W3["Worker 3"]
    
    style S1 fill:#e1f5fe
    style S2 fill:#fff3e0
    style S3 fill:#e8f5e9
    style S4 fill:#fce4ec
```

---

## 7. Exchange 算子

分布式关键算子：

```mermaid
graph TB
    subgraph "Exchange Types"
        Hash["Hash Shuffle"]
        Broadcast["Broadcast"]
        Local["Local"]
    end
```

### 7.1 随机播放

```mermaid
flowchart LR
    subgraph "Input"
        P1["Partition 1<br/>Key=A"]
        P2["Partition 2<br/>Key=B"]
        P3["Partition 3<br/>Key=C"]
    end
    
    subgraph "Shuffle"
        S["Shuffle Operator"]
    end
    
    subgraph "Output"
        N1["Node 1<br/>Key=A"]
        N2["Node 2<br/>Key=B"]
        N3["Node 3<br/>Key=C"]
    end
    
    P1 --> S
    P2 --> S
    P3 --> S
    
    S --> N1
    S --> N2
    S --> N3
```

### 7.2 广播

```mermaid
flowchart LR
    subgraph "Input"
        Small["Small Table<br/>10MB"]
    end
    
    subgraph "Broadcast"
        B["Broadcast Operator"]
    end
    
    subgraph "Output"
        W1["Worker 1"]
        W2["Worker 2"]
        W3["Worker 3"]
    end
    
    Small --> B
    B --> W1
    B --> W2
    B --> W3
```

---

## 8. 调度策略

| 策略 | 说明 | 适用场景 |
|------|------|----------|
| **Hash** | 按 key 分区 | 等值 Join |
|**播送**| 小表广播 | 小表 Join |
| **Local** | 本地执行 | 本地数据 |
|**随机的**| 随机分配 | 负载均衡 |

### 8.1 策略选择

```mermaid
flowchart TB
    Start["Choose Strategy"] --> CheckSize{Table Size?}
    
    CheckSize -->|< 10MB| Broadcast["Use Broadcast"]
    CheckSize -->|>= 10MB| CheckDistribution{Data<br/>Distribution?}
    
    CheckDistribution -->|Skewed| Custom["Custom Hash"]
    CheckDistribution -->|Uniform| Hash["Use Hash"]
    
    Broadcast --> Output
    Hash --> Output
    Custom --> Output
    
    style Broadcast fill:#c8e6c9
    style Hash fill:#bbdefb
    style Custom fill:#fff9c4
```

---

## 9. 容错

### 9.1 容错机制

```mermaid
flowchart TB
    subgraph "Failure Detection"
        Heartbeat["Heartbeat Monitor"]
        Timeout["Timeout Detector"]
    end
    
    subgraph "Recovery"
        Retry["Task Retry"]
        Reschedule["Reschedule"]
        Failover["Failover to Backup"]
    end
    
    Heartbeat --> Timeout
    Timeout --> Retry
    Retry --> Reschedule
    Reschedule --> Failover
```

### 9.2 恢复策略

| 故障类型 | 检测方式 | 恢复策略 |
|----------|----------|----------|
| Worker 节点宕机 | 心跳检测 | 任务重新调度 |
| 网络分区 | 超时检测 | 重新执行 |
| 数据丢失 |校验和| 重新获取 |

---

## 10. SQLRustGo 分布式执行流程

```mermaid
sequenceDiagram
    participant Client
    participant Coordinator
    participant Worker1
    participant Worker2
    participant Worker3

    Client->>Coordinator: SQL Query
    Coordinator->>Coordinator: Parse & Plan
    Coordinator->>Coordinator: Split into Stages

    rect rgb(240, 248, 255)
        Note over Coordinator,Worker1: Stage 1: Scan
        Coordinator->>Worker1: Execute Scan Task 1
        Coordinator->>Worker2: Execute Scan Task 2
        Coordinator->>Worker3: Execute Scan Task 3
        Worker1-->>Coordinator: Partition 1 Data
        Worker2-->>Coordinator: Partition 2 Data
        Worker3-->>Coordinator: Partition 3 Data
    end

    rect rgb(255, 250, 240)
        Note over Coordinator,Worker1: Stage 2: Shuffle & Join
        Coordinator->>Worker1: Shuffle & Join Task
        Coordinator->>Worker2: Shuffle & Join Task
        Worker1-->>Coordinator: Join Result 1
        Worker2-->>Coordinator: Join Result 2
    end

    rect rgb(240, 255, 240)
        Note over Coordinator,Worker1: Stage 3: Aggregate
        Coordinator->>Worker1: Aggregate Task
        Worker1-->>Coordinator: Final Result
    end

    Coordinator-->>Client: ResultSet
```

---

## 11. 性能优化

### 11.1 优化策略

| 优化手段 | 效果 |
|----------|------|
|谓词下推| 减少网络传输 |
|柱状传输| 减少序列化开销 |
|数据局部性| 减少跨节点访问 |
|流水线| 减少等待时间 |

### 11.2 性能目标

| 指标 | 1.x (单机) | 2.0 (3节点) |
|------|------------|-------------|
| 100万行 Join | <1s | <500ms |
| 1000万行聚合 | <5s | <2s |
| 水平扩展 | 1x | ~2.5x |

---

## 12. 技术对标

如果以下三件事做对：

- 执行接口 ✅
- 级联优化器✅
- 分布式调度 ✅

SQLRustGo 的技术路线将与这些系统一致：

| 数据库 | 架构 |
|--------|------|
|蟑螂数据库|瀑布+白天|
|绿梅|瀑布+白天|
|SQL服务器|瀑布+白天|
| Trino |有向无环图执行|

---

## 13. 相关文档

- [SQLRustGo Architecture](./sqlrustgo_architecture.md)
- [Cascades Optimizer](./cascades_optimizer_design.md)
- [Whitepaper](../whitepaper/sqlrustgo_1.2_release_whitepaper.md)
- [2.0 Distributed Framework](../whitepaper/sqlrustgo_2.0_distributed_framework.md)
