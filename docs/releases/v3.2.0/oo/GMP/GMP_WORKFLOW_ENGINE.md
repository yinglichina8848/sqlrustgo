# OO-9: GMP Workflow Engine 设计文档

> **版本**: v1.0
> **日期**: 2026-05-16
> **基于**: v3.2.0
> **维护人**: hermes-z6g4
> **状态**: 设计中

---

## 一、概述

### 1.1 目标

实现 GMP Workflow Engine（工作流引擎），管理复杂的多步骤 GMP 流程：

- **状态机**: 管理工作流状态转换
- **事件驱动**: 基于事件触发工作流步骤
- **持久化**: 工作流状态持久化
- **审计追踪**: 完整的工作流执行审计

### 1.2 核心理念

```
Workflow Engine = State Machine + Event Processing + Persistence + Audit
```

---

## 二、技术架构

### 2.1 组件关系

```
┌─────────────────────────────────────────────────────────────────┐
│                    Workflow Engine System                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │   Workflow  │───▶│   State     │───▶│   Event         │  │
│  │   Registry  │    │   Machine   │    │   Processor     │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│         │                   │                      │            │
│         │                   │                      │            │
│         ▼                   ▼                      ▼            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │  Workflow   │    │   Step      │    │   Workflow      │  │
│  │  Instance   │    │   Executor  │    │   Persistence   │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              Audit & Notification                         │   │
│  │  - Workflow History                                      │   │
│  │  - Step Completion Events                                │   │
│  │  - SLA Monitoring                                        │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、工作流定义

### 3.1 工作流 DSL

```sql
CREATE WORKFLOW batch_release (
    VERSION '1.0'
    DESCRIPTION 'GMP Batch Release Workflow'

    STATE INITIAL     -> PENDING_QC
    STATE PENDING_QC  -> QC_APPROVED, QC_REJECTED
    STATE QC_APPROVED -> PENDING_PRODUCTION_SIGN_OFF
    STATE PENDING_PRODUCTION_SIGN_OFF -> SIGNED, REJECTED
    STATE SIGNED      -> RELEASED
    STATE RELEASED     (FINAL)

    STEP qc_inspection (
        ASSIGNED_TO qc_role
        TIMEOUT 72 HOURS
        NOTIFY ON COMPLETE
    )

    STEP production_sign_off (
        REQUIRED_SIGNATURES 2
        REQUIRED_ROLES (production_manager, quality_director)
        TIMEOUT 48 HOURS
    )

    ON STATE TRANSITION DO audit_log
    ON STEP FAILURE DO notify_managers
);
```

### 3.2 工作流状态机

```
     ┌──────────────┐
     │   INITIAL   │
     └──────┬───────┘
            │
            ▼
     ┌──────────────┐
     │  PENDING_QC  │◀─────────┐
     └──────┬───────┘          │
            │                  │
    ┌───────┴───────┐          │
    │               │          │
    ▼               ▼          │
┌────────┐    ┌──────────┐     │
│   QC   │    │    QC    │     │
│APPROVED│    │ REJECTED │─────┘
└───┬────┘    └──────────┘
    │
    ▼
┌──────────────────────────┐
│ PENDING_PRODUCTION_SIGN │
└───────────┬──────────────┘
            │
    ┌───────┴───────┐
    │               │
    ▼               ▼
┌────────┐    ┌──────────┐
│ SIGNED │    │ REJECTED │─────┐
└───┬────┘    └──────────┘     │
    │                           │
    ▼                           │
┌──────────┐                    │
│ RELEASED │────────────────────┘
└──────────┘
```

---

## 四、数据结构

### 4.1 工作流定义表 (gmp_workflow_definition)

```sql
CREATE TABLE gmp_workflow_definition (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                TEXT UNIQUE NOT NULL,
    version             TEXT NOT NULL DEFAULT '1.0',
    description         TEXT,
    definition_json     JSONB NOT NULL,
    initial_state       TEXT NOT NULL,
    final_states        TEXT[] NOT NULL,
    created_at          TIMESTAMP DEFAULT NOW(),
    updated_at          TIMESTAMP DEFAULT NOW(),
    active              BOOLEAN DEFAULT TRUE
);
```

### 4.2 工作流实例表 (gmp_workflow_instance)

```sql
CREATE TABLE gmp_workflow_instance (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_name       TEXT NOT NULL,
    workflow_version    TEXT NOT NULL,
    current_state       TEXT NOT NULL,
    context             JSONB NOT NULL DEFAULT '{}',
    status              TEXT NOT NULL DEFAULT 'ACTIVE',
    started_by          TEXT NOT NULL,
    started_at          BIGINT NOT NULL,
    updated_at          BIGINT NOT NULL,
    completed_at        BIGINT,
    due_at              BIGINT,
    created_at          TIMESTAMP DEFAULT NOW()
);
```

### 4.3 工作流步骤表 (gmp_workflow_step)

```sql
CREATE TABLE gmp_workflow_step (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    instance_id         UUID NOT NULL REFERENCES gmp_workflow_instance(id),
    step_name           TEXT NOT NULL,
    step_order          INT NOT NULL,
    assigned_to         TEXT,
    assigned_role       TEXT,
    status              TEXT NOT NULL DEFAULT 'PENDING',
    input_data          JSONB,
    output_data         JSONB,
    started_at          BIGINT,
    completed_at        BIGINT,
    timeout_at          BIGINT,
    created_at          TIMESTAMP DEFAULT NOW()
);
```

---

## 五、API 设计

### 5.1 核心 Trait: `WorkflowEngine`

```rust
/// 工作流引擎接口
pub trait WorkflowEngine: Send + Sync {
    /// 创建工作流实例
    fn create_instance(
        &self,
        workflow_name: &str,
        context: JsonValue,
        started_by: &str,
    ) -> Result<WorkflowInstance, WorkflowError>;

    /// 执行状态转换
    fn transition(
        &self,
        instance_id: Uuid,
        target_state: &str,
        actor_id: &str,
    ) -> Result<TransitionResult, WorkflowError>;

    /// 执行步骤
    fn execute_step(
        &self,
        instance_id: Uuid,
        step_name: &str,
        input_data: JsonValue,
        actor_id: &str,
    ) -> Result<StepResult, WorkflowError>;

    /// 获取实例状态
    fn get_instance(
        &self,
        instance_id: Uuid,
    ) -> Result<WorkflowInstance, WorkflowError>;

    /// 获取待处理步骤
    fn get_pending_steps(
        &self,
        actor_id: &str,
    ) -> Result<Vec<WorkflowStep>, WorkflowError>;
}
```

### 5.2 数据结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: Uuid,
    pub workflow_name: String,
    pub current_state: String,
    pub context: JsonValue,
    pub status: InstanceStatus,
    pub started_by: String,
    pub started_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub instance_id: Uuid,
    pub step_name: String,
    pub step_order: i32,
    pub assigned_to: Option<String>,
    pub assigned_role: Option<String>,
    pub status: StepStatus,
    pub input_data: Option<JsonValue>,
    pub output_data: Option<JsonValue>,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub timeout_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstanceStatus {
    Active,
    Suspended,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
    Failed,
    TimedOut,
}
```

---

## 六、事件处理

### 6.1 事件类型

```rust
#[derive(Debug, Clone)]
pub enum WorkflowEvent {
    StateTransition {
        instance_id: Uuid,
        from_state: String,
        to_state: String,
        actor_id: String,
    },
    StepStarted {
        instance_id: Uuid,
        step_name: String,
        actor_id: String,
    },
    StepCompleted {
        instance_id: Uuid,
        step_name: String,
        actor_id: String,
        output: JsonValue,
    },
    StepFailed {
        instance_id: Uuid,
        step_name: String,
        error: String,
    },
    Timeout {
        instance_id: Uuid,
        step_name: String,
    },
}
```

### 6.2 事件处理器

```rust
/// 工作流事件处理器
pub trait WorkflowEventHandler: Send + Sync {
    fn handle(&self, event: &WorkflowEvent) -> Result<(), WorkflowError>;
}

/// 审计日志处理器
pub struct AuditLogHandler;

impl WorkflowEventHandler for AuditLogHandler {
    fn handle(&self, event: &WorkflowEvent) -> Result<(), WorkflowError> {
        // 记录事件到审计表
        audit_log.record_event(event);
        Ok(())
    }
}

/// 通知处理器
pub struct NotificationHandler;

impl WorkflowEventHandler for NotificationHandler {
    fn handle(&self, event: &WorkflowEvent) -> Result<(), WorkflowError> {
        match event {
            WorkflowEvent::StepCompleted { .. } => notify_stakeholders(event),
            WorkflowEvent::Timeout { .. } => notify_managers(event),
            _ => Ok(()),
        }
    }
}
```

---

## 七、持久化

### 7.1 状态持久化

```rust
/// 工作流持久化器
pub trait WorkflowPersistence {
    fn save_instance(&self, instance: &WorkflowInstance) -> Result<(), Error>;
    fn load_instance(&self, id: Uuid) -> Result<Option<WorkflowInstance>, Error>;
    fn save_step(&self, step: &WorkflowStep) -> Result<(), Error>;
    fn load_steps(&self, instance_id: Uuid) -> Result<Vec<WorkflowStep>, Error>;
}
```

---

## 八、实现状态

| 阶段 | 任务 | 状态 | PR |
|------|------|------|-----|
| 1 | 工作流引擎核心 | ✅ | #1046 |
| 2 | 状态机实现 | ✅ | #1046 |
| 3 | 步骤执行器 | ✅ | #1046 |
| 4 | 事件处理 | ✅ | #1046 |
| 5 | 持久化 | ✅ | #1046 |

---

*本文档由 hermes-agent 创建*
*版本 1.0 - 2026-05-16*