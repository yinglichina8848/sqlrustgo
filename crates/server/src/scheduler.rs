//! OpenClaw Scheduler Module
//!
//! Provides task flow orchestration and agent collaboration capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// ============================================================================
// Task Types
// ============================================================================

/// Task status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Medium
    }
}

/// A single task in the scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub payload: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub agent_id: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
}

impl Task {
    pub fn new(name: String, payload: serde_json::Value) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            status: TaskStatus::Pending,
            priority: TaskPriority::Medium,
            payload,
            result: None,
            error: None,
            created_at: now,
            updated_at: now,
            agent_id: None,
            retry_count: 0,
            max_retries: 3,
        }
    }
}

// ============================================================================
// Workflow Types
// ============================================================================

/// Step type in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowStep {
    Task {
        task_template: String,
        params: serde_json::Value,
    },
    Parallel {
        steps: Vec<WorkflowStep>,
    },
    Sequential {
        steps: Vec<WorkflowStep>,
    },
    Conditional {
        condition: String,
        then_step: Box<WorkflowStep>,
        else_step: Option<Box<WorkflowStep>>,
    },
}

/// A workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<WorkflowStep>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Workflow {
    pub fn new(name: String, steps: Vec<WorkflowStep>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            steps,
            created_at: now,
            updated_at: now,
        }
    }
}

// ============================================================================
// Agent Types
// ============================================================================

/// Agent status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Busy,
    Offline,
}

/// Agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
    pub current_task_id: Option<String>,
    pub tasks_completed: u64,
    pub created_at: u64,
}

// ============================================================================
// Scheduler State
// ============================================================================

/// Global scheduler state
#[derive(Debug, Clone)]
pub struct SchedulerState {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    workflows: Arc<RwLock<HashMap<String, Workflow>>>,
    agents: Arc<RwLock<HashMap<String, Agent>>>,
    execution_history: Arc<RwLock<Vec<ExecutionRecord>>>,
}

impl Default for SchedulerState {
    fn default() -> Self {
        Self::new()
    }
}

impl SchedulerState {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            workflows: Arc::new(RwLock::new(HashMap::new())),
            agents: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // Task operations
    pub fn create_task(&self, task: Task) -> String {
        let id = task.id.clone();
        self.tasks.write().unwrap().insert(id.clone(), task);
        id
    }

    pub fn get_task(&self, id: &str) -> Option<Task> {
        self.tasks.read().unwrap().get(id).cloned()
    }

    pub fn update_task(&self, id: &str, mut task: Task) -> bool {
        task.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if self
            .tasks
            .write()
            .unwrap()
            .insert(id.to_string(), task)
            .is_some()
        {
            true
        } else {
            false
        }
    }

    pub fn list_tasks(&self) -> Vec<Task> {
        self.tasks.read().unwrap().values().cloned().collect()
    }

    pub fn delete_task(&self, id: &str) -> bool {
        self.tasks.write().unwrap().remove(id).is_some()
    }

    // Workflow operations
    pub fn create_workflow(&self, workflow: Workflow) -> String {
        let id = workflow.id.clone();
        self.workflows.write().unwrap().insert(id.clone(), workflow);
        id
    }

    pub fn get_workflow(&self, id: &str) -> Option<Workflow> {
        self.workflows.read().unwrap().get(id).cloned()
    }

    pub fn list_workflows(&self) -> Vec<Workflow> {
        self.workflows.read().unwrap().values().cloned().collect()
    }

    pub fn delete_workflow(&self, id: &str) -> bool {
        self.workflows.write().unwrap().remove(id).is_some()
    }

    // Agent operations
    pub fn register_agent(&self, agent: Agent) -> String {
        let id = agent.id.clone();
        self.agents.write().unwrap().insert(id.clone(), agent);
        id
    }

    pub fn get_agent(&self, id: &str) -> Option<Agent> {
        self.agents.read().unwrap().get(id).cloned()
    }

    pub fn list_agents(&self) -> Vec<Agent> {
        self.agents.read().unwrap().values().cloned().collect()
    }

    pub fn update_agent_status(
        &self,
        id: &str,
        status: AgentStatus,
        current_task_id: Option<String>,
    ) -> bool {
        if let Some(agent) = self.agents.write().unwrap().get_mut(id) {
            agent.status = status;
            agent.current_task_id = current_task_id;
            true
        } else {
            false
        }
    }

    // Execution history
    pub fn record_execution(&self, record: ExecutionRecord) {
        self.execution_history.write().unwrap().push(record);
    }

    pub fn get_execution_history(&self, limit: usize) -> Vec<ExecutionRecord> {
        let history = self.execution_history.read().unwrap();
        history.iter().rev().take(limit).cloned().collect()
    }
}

// ============================================================================
// Execution Types
// ============================================================================

/// Execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// An execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: String,
    pub workflow_id: String,
    pub status: ExecutionStatus,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub task_results: HashMap<String, serde_json::Value>,
    pub error: Option<String>,
}

impl ExecutionRecord {
    pub fn new(workflow_id: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            workflow_id,
            status: ExecutionStatus::Pending,
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            completed_at: None,
            task_results: HashMap::new(),
            error: None,
        }
    }
}

// ============================================================================
// API Request/Response Types
// ============================================================================

/// Create task request
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,
    pub payload: serde_json::Value,
    pub max_retries: Option<u32>,
}

/// Create task response
#[derive(Debug, Serialize)]
pub struct CreateTaskResponse {
    pub id: String,
    pub status: String,
}

/// Get task response
#[derive(Debug, Serialize)]
pub struct GetTaskResponse {
    pub task: Option<Task>,
    pub error: Option<String>,
}

/// List tasks response
#[derive(Debug, Serialize)]
pub struct ListTasksResponse {
    pub tasks: Vec<Task>,
    pub total: usize,
}

/// Create workflow request
#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<WorkflowStep>,
}

/// Create workflow response
#[derive(Debug, Serialize)]
pub struct CreateWorkflowResponse {
    pub id: String,
    pub status: String,
}

/// Get workflow response
#[derive(Debug, Serialize)]
pub struct GetWorkflowResponse {
    pub workflow: Option<Workflow>,
    pub error: Option<String>,
}

/// Execute workflow request
#[derive(Debug, Deserialize)]
pub struct ExecuteWorkflowRequest {
    pub workflow_id: String,
    pub params: Option<serde_json::Value>,
}

/// Execute workflow response
#[derive(Debug, Serialize)]
pub struct ExecuteWorkflowResponse {
    pub execution_id: String,
    pub status: String,
    pub message: String,
}

/// Get execution response
#[derive(Debug, Serialize)]
pub struct GetExecutionResponse {
    pub execution: Option<ExecutionRecord>,
    pub error: Option<String>,
}

/// Agent registration request
#[derive(Debug, Deserialize)]
pub struct RegisterAgentRequest {
    pub name: String,
    pub capabilities: Vec<String>,
}

/// Register agent response
#[derive(Debug, Serialize)]
pub struct RegisterAgentResponse {
    pub id: String,
    pub status: String,
}

/// List agents response
#[derive(Debug, Serialize)]
pub struct ListAgentsResponse {
    pub agents: Vec<Agent>,
    pub total: usize,
}

/// Sync agent status request
#[derive(Debug, Deserialize)]
pub struct SyncAgentStatusRequest {
    pub agent_id: String,
    pub status: AgentStatus,
    pub current_task_id: Option<String>,
    pub tasks_completed: Option<u64>,
}

/// Sync agent status response
#[derive(Debug, Serialize)]
pub struct SyncAgentStatusResponse {
    pub success: bool,
    pub message: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("test_task".to_string(), serde_json::json!({"key": "value"}));
        assert_eq!(task.name, "test_task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(!task.id.is_empty());
    }

    #[test]
    fn test_task_status_default() {
        let status = TaskStatus::default();
        assert_eq!(status, TaskStatus::Pending);
    }

    #[test]
    fn test_workflow_creation() {
        let steps = vec![WorkflowStep::Task {
            task_template: "template1".to_string(),
            params: serde_json::json!({}),
        }];
        let workflow = Workflow::new("test_workflow".to_string(), steps);
        assert_eq!(workflow.name, "test_workflow");
        assert!(!workflow.id.is_empty());
    }

    #[test]
    fn test_scheduler_state_create_task() {
        let state = SchedulerState::new();
        let task = Task::new("test".to_string(), serde_json::json!({}));
        let id = state.create_task(task);
        assert!(!id.is_empty());
        assert_eq!(state.get_task(&id).unwrap().name, "test");
    }

    #[test]
    fn test_scheduler_state_list_tasks() {
        let state = SchedulerState::new();
        let task1 = Task::new("task1".to_string(), serde_json::json!({}));
        let task2 = Task::new("task2".to_string(), serde_json::json!({}));
        state.create_task(task1);
        state.create_task(task2);
        assert_eq!(state.list_tasks().len(), 2);
    }

    #[test]
    fn test_scheduler_state_delete_task() {
        let state = SchedulerState::new();
        let task = Task::new("test".to_string(), serde_json::json!({}));
        let id = state.create_task(task);
        assert!(state.delete_task(&id));
        assert!(state.get_task(&id).is_none());
    }

    #[test]
    fn test_workflow_sequential_step() {
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
        let workflow = Workflow::new(
            "sequential".to_string(),
            vec![WorkflowStep::Sequential { steps }],
        );
        assert!(!workflow.id.is_empty());
    }

    #[test]
    fn test_agent_status_serialization() {
        let status = AgentStatus::Busy;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("busy"));
    }

    #[test]
    fn test_execution_record_creation() {
        let record = ExecutionRecord::new("workflow_123".to_string());
        assert_eq!(record.workflow_id, "workflow_123");
        assert_eq!(record.status, ExecutionStatus::Pending);
        assert!(record.id.len() > 0);
    }

    #[test]
    fn test_create_task_request_deserialization() {
        let json = r#"{"name": "my_task", "payload": {"data": 123}}"#;
        let req: CreateTaskRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "my_task");
    }

    #[test]
    fn test_create_workflow_request_deserialization() {
        let json = r#"{
            "name": "my_workflow",
            "steps": [{"type": "task", "task_template": "t1", "params": {}}]
        }"#;
        let req: CreateWorkflowRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "my_workflow");
    }

    #[test]
    fn test_agent_registration() {
        let state = SchedulerState::new();
        let agent = Agent {
            id: Uuid::new_v4().to_string(),
            name: "test_agent".to_string(),
            status: AgentStatus::Idle,
            capabilities: vec!["sql".to_string(), "rag".to_string()],
            current_task_id: None,
            tasks_completed: 0,
            created_at: 0,
        };
        let id = state.register_agent(agent);
        assert!(!id.is_empty());
    }

    #[test]
    fn test_execution_history() {
        let state = SchedulerState::new();
        let record1 = ExecutionRecord::new("wf1".to_string());
        let record2 = ExecutionRecord::new("wf2".to_string());
        state.record_execution(record1);
        state.record_execution(record2);
        let history = state.get_execution_history(10);
        assert_eq!(history.len(), 2);
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
            "parallel".to_string(),
            vec![WorkflowStep::Parallel { steps }],
        );
        assert_eq!(workflow.steps.len(), 1);
    }
}
