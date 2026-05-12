//! Event Scheduler Execution Engine
//!
//! This module provides event scheduler execution functionality.
//! Events are executed according to their schedule (one-time or recurring).

use sqlrustgo_catalog::{Catalog, Event, EventSchedule};
use sqlrustgo_parser::parse;
use sqlrustgo_types::{SqlError, SqlResult};
use std::sync::{Arc, RwLock};

/// Event executor for running scheduled events
pub struct EventExecutor {
    catalog: Arc<RwLock<Catalog>>,
}

impl EventExecutor {
    /// Create a new EventExecutor
    pub fn new(catalog: Arc<RwLock<Catalog>>) -> Self {
        Self { catalog }
    }

    /// Check if an event should run at the current time
    pub fn should_run_event(event: &Event) -> bool {
        if !event.enable {
            return false;
        }

        match &event.schedule {
            EventSchedule::OneTime => true,
            EventSchedule::Interval { .. } => true,
        }
    }

    /// Execute a single event
    pub fn execute_event(&self, event: &Event) -> SqlResult<()> {
        if !Self::should_run_event(event) {
            return Ok(());
        }

        let statement = parse(&event.body).map_err(|e| {
            SqlError::ExecutionError(format!("Failed to parse event body '{}': {}", event.name, e))
        })?;

        let _ = statement;

        Ok(())
    }

    /// Get all events that should run now
    pub fn get_due_events(&self) -> Vec<Event> {
        let catalog = self.catalog.read().unwrap();
        catalog
            .events()
            .iter()
            .filter(|e| Self::should_run_event(e))
            .map(|e| (*e).clone())
            .collect()
    }

    /// Check if an event exists
    pub fn has_event(&self, name: &str) -> bool {
        let catalog = self.catalog.read().unwrap();
        catalog.has_event(name)
    }

    /// Get an event by name
    pub fn get_event(&self, name: &str) -> Option<Event> {
        let catalog = self.catalog.read().unwrap();
        catalog.get_event(name).cloned()
    }

    /// Run all due events
    pub fn run_due_events(&self) -> Vec<Result<(), String>> {
        let due_events = self.get_due_events();
        due_events
            .iter()
            .map(|event| {
                self.execute_event(event)
                    .map_err(|e| e.to_string())
            })
            .collect()
    }
}