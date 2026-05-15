//! GMP Workflow Engine Module
//!
//! Provides state-machine-based workflow management for GMP-compliant processes.
//!
//! # Architecture
//!
//! - [`engine`] - Core workflow engine managing lifecycle
//! - [`state`] - State machine with defined transitions
//! - [`approval`] - Multi-level approval chain
//! - [`timeout`] - Timeout detection and auto-reject
//!
//! # Example
//!
//! ```rust,ignore
//! use sqlrustgo_gmp::workflow::{WorkflowEngine, WorkflowState};
//!
//! let mut engine = WorkflowEngine::new();
//! engine.create_definition("batch_release", vec!["draft", "review", "approval", "released"])?;
//! let instance_id = engine.start_workflow("batch_release", json!({"batch_id": 123}))?;
//! ```

pub mod approval;
pub mod definition;
pub mod engine;
pub mod instance;
pub mod state;
pub mod timeout;

pub use approval::{ApprovalAction, ApprovalChain, ApprovalRecord};
pub use definition::WorkflowDefinition;
pub use engine::WorkflowEngine;
pub use instance::WorkflowInstance;
pub use state::{WorkflowState, WorkflowTransition};
pub use timeout::TimeoutChecker;
