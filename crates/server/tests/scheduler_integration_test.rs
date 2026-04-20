//! Scheduler Integration Tests
//!
//! Tests for the scheduler API endpoints.

use sqlrustgo_server::scheduler::{
    Agent, AgentStatus, CreateTaskRequest, CreateWorkflowRequest, ExecuteWorkflowRequest,
    RegisterAgentRequest, SchedulerState, SyncAgentStatusRequest, Task, TaskPriority, TaskStatus,
    Workflow, WorkflowStep,
};

/// Helper to create a fresh scheduler state for each test
fn create_test_state() -> SchedulerState {
    SchedulerState::new()
}

#[test]
fn test_create_and_get_task() {
    let state = create_test_state();

    // Create a task
    let task = Task::new(
        "test_task".to_string(),
        serde_json::json!({"data": "value"}),
    );
    let id = state.create_task(task.clone());

    // Get the task
    let retrieved = state.get_task(&id);
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.name, "test_task");
    assert_eq!(retrieved.status, TaskStatus::Pending);
}

#[test]
fn test_create_task_with_priority() {
    let state = create_test_state();

    let mut task = Task::new(
        "high_priority_task".to_string(),
        serde_json::json!({"urgent": true}),
    );
    task.priority = TaskPriority::High;
    let id = state.create_task(task);

    let retrieved = state.get_task(&id).unwrap();
    assert_eq!(retrieved.priority, TaskPriority::High);
}

#[test]
fn test_list_tasks() {
    let state = create_test_state();

    // Create multiple tasks
    for i in 0..5 {
        let task = Task::new(format!("task_{}", i), serde_json::json!({"index": i}));
        state.create_task(task);
    }

    let tasks = state.list_tasks();
    assert_eq!(tasks.len(), 5);
}

#[test]
fn test_delete_task() {
    let state = create_test_state();

    let task = Task::new("to_delete".to_string(), serde_json::json!({}));
    let id = state.create_task(task);

    assert!(state.delete_task(&id));
    assert!(state.get_task(&id).is_none());

    // Deleting non-existent task should return false
    assert!(!state.delete_task("non_existent"));
}

#[test]
fn test_create_and_get_workflow() {
    let state = create_test_state();

    let steps = vec![
        WorkflowStep::Task {
            task_template: "step1".to_string(),
            params: serde_json::json!({}),
        },
        WorkflowStep::Task {
            task_template: "step2".to_string(),
            params: serde_json::json!({}),
        },
    ];
    let workflow = Workflow::new("test_workflow".to_string(), steps);
    let id = state.create_workflow(workflow.clone());

    let retrieved = state.get_workflow(&id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test_workflow");
}

#[test]
fn test_list_workflows() {
    let state = create_test_state();

    for i in 0..3 {
        let workflow = Workflow::new(
            format!("workflow_{}", i),
            vec![WorkflowStep::Task {
                task_template: format!("template_{}", i),
                params: serde_json::json!({}),
            }],
        );
        state.create_workflow(workflow);
    }

    let workflows = state.list_workflows();
    assert_eq!(workflows.len(), 3);
}

#[test]
fn test_delete_workflow() {
    let state = create_test_state();

    let workflow = Workflow::new(
        "to_delete".to_string(),
        vec![WorkflowStep::Task {
            task_template: "t".to_string(),
            params: serde_json::json!({}),
        }],
    );
    let id = state.create_workflow(workflow);

    assert!(state.delete_workflow(&id));
    assert!(state.get_workflow(&id).is_none());
}

#[test]
fn test_register_and_get_agent() {
    let state = create_test_state();

    let agent = Agent {
        id: "agent_1".to_string(),
        name: "test_agent".to_string(),
        status: AgentStatus::Idle,
        capabilities: vec!["sql".to_string(), "rag".to_string()],
        current_task_id: None,
        tasks_completed: 0,
        created_at: 0,
    };
    let id = state.register_agent(agent);

    let retrieved = state.get_agent(&id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test_agent");
}

#[test]
fn test_list_agents() {
    let state = create_test_state();

    for i in 0..4 {
        let agent = Agent {
            id: format!("agent_{}", i),
            name: format!("agent_{}", i),
            status: AgentStatus::Idle,
            capabilities: vec![],
            current_task_id: None,
            tasks_completed: 0,
            created_at: 0,
        };
        state.register_agent(agent);
    }

    let agents = state.list_agents();
    assert_eq!(agents.len(), 4);
}

#[test]
fn test_update_agent_status() {
    let state = create_test_state();

    let agent = Agent {
        id: "agent_1".to_string(),
        name: "test_agent".to_string(),
        status: AgentStatus::Idle,
        capabilities: vec![],
        current_task_id: None,
        tasks_completed: 0,
        created_at: 0,
    };
    let id = state.register_agent(agent);

    // Update status to busy with a task
    let result = state.update_agent_status(&id, AgentStatus::Busy, Some("task_123".to_string()));
    assert!(result);

    let retrieved = state.get_agent(&id).unwrap();
    assert_eq!(retrieved.status, AgentStatus::Busy);
    assert_eq!(retrieved.current_task_id, Some("task_123".to_string()));
}

#[test]
fn test_execution_history() {
    let state = create_test_state();

    use sqlrustgo_server::scheduler::ExecutionRecord;

    let record1 = ExecutionRecord::new("workflow_1".to_string());
    let record2 = ExecutionRecord::new("workflow_2".to_string());

    state.record_execution(record1);
    state.record_execution(record2);

    let history = state.get_execution_history(10);
    assert_eq!(history.len(), 2);
}

#[test]
fn test_sequential_workflow_step() {
    let steps = vec![
        WorkflowStep::Task {
            task_template: "step1".to_string(),
            params: serde_json::json!({"key": "value1"}),
        },
        WorkflowStep::Task {
            task_template: "step2".to_string(),
            params: serde_json::json!({"key": "value2"}),
        },
    ];

    let workflow = Workflow::new(
        "sequential_workflow".to_string(),
        vec![WorkflowStep::Sequential { steps }],
    );

    assert_eq!(workflow.steps.len(), 1);
}

#[test]
fn test_parallel_workflow_step() {
    let steps = vec![
        WorkflowStep::Task {
            task_template: "parallel1".to_string(),
            params: serde_json::json!({}),
        },
        WorkflowStep::Task {
            task_template: "parallel2".to_string(),
            params: serde_json::json!({}),
        },
    ];

    let workflow = Workflow::new(
        "parallel_workflow".to_string(),
        vec![WorkflowStep::Parallel { steps }],
    );

    assert_eq!(workflow.steps.len(), 1);
}

#[test]
fn test_conditional_workflow_step() {
    let workflow = Workflow::new(
        "conditional_workflow".to_string(),
        vec![WorkflowStep::Conditional {
            condition: "user.is_admin".to_string(),
            then_step: Box::new(WorkflowStep::Task {
                task_template: "admin_task".to_string(),
                params: serde_json::json!({}),
            }),
            else_step: Some(Box::new(WorkflowStep::Task {
                task_template: "user_task".to_string(),
                params: serde_json::json!({}),
            })),
        }],
    );

    assert_eq!(workflow.steps.len(), 1);
}

#[test]
fn test_task_serialization() {
    let task = Task::new(
        "serialization_test".to_string(),
        serde_json::json!({"nested": {"data": [1, 2, 3]}}),
    );

    let json = serde_json::to_string(&task).unwrap();
    let deserialized: Task = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, task.name);
    assert_eq!(deserialized.status, task.status);
}

#[test]
fn test_workflow_serialization() {
    let workflow = Workflow::new(
        "serialization_test".to_string(),
        vec![WorkflowStep::Task {
            task_template: "t".to_string(),
            params: serde_json::json!({"x": 1}),
        }],
    );

    let json = serde_json::to_string(&workflow).unwrap();
    let deserialized: Workflow = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, workflow.name);
}

#[test]
fn test_agent_serialization() {
    let agent = Agent {
        id: "agent_test".to_string(),
        name: "Test Agent".to_string(),
        status: AgentStatus::Busy,
        capabilities: vec!["test".to_string()],
        current_task_id: Some("task_1".to_string()),
        tasks_completed: 42,
        created_at: 1000,
    };

    let json = serde_json::to_string(&agent).unwrap();
    let deserialized: Agent = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, agent.id);
    assert_eq!(deserialized.status, AgentStatus::Busy);
}

#[test]
fn test_request_deserialization() {
    let task_json = r#"{"name": "api_task", "payload": {"data": "test"}, "priority": "high"}"#;
    let req: CreateTaskRequest = serde_json::from_str(task_json).unwrap();

    assert_eq!(req.name, "api_task");
    assert_eq!(req.priority, Some(TaskPriority::High));
}

#[test]
fn test_workflow_request_deserialization() {
    let workflow_json = r#"{
        "name": "api_workflow",
        "steps": [
            {"type": "task", "task_template": "step1", "params": {}}
        ]
    }"#;
    let req: CreateWorkflowRequest = serde_json::from_str(workflow_json).unwrap();

    assert_eq!(req.name, "api_workflow");
    assert_eq!(req.steps.len(), 1);
}

#[test]
fn test_execute_workflow_request_deserialization() {
    let exec_json = r#"{"workflow_id": "wf_123", "params": {"timeout": 30}}"#;
    let req: ExecuteWorkflowRequest = serde_json::from_str(exec_json).unwrap();

    assert_eq!(req.workflow_id, "wf_123");
}

#[test]
fn test_register_agent_request_deserialization() {
    let agent_json = r#"{"name": "new_agent", "capabilities": ["sql", "rag", "vector"]}"#;
    let req: RegisterAgentRequest = serde_json::from_str(agent_json).unwrap();

    assert_eq!(req.name, "new_agent");
    assert_eq!(req.capabilities.len(), 3);
}

#[test]
fn test_sync_agent_status_request_deserialization() {
    let sync_json = r#"{
        "agent_id": "agent_1",
        "status": "busy",
        "current_task_id": "task_456",
        "tasks_completed": 100
    }"#;
    let req: SyncAgentStatusRequest = serde_json::from_str(sync_json).unwrap();

    assert_eq!(req.agent_id, "agent_1");
    assert_eq!(req.status, AgentStatus::Busy);
    assert_eq!(req.tasks_completed, Some(100));
}
