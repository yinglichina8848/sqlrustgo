//! Event scheduler definitions for the catalog

use serde::{Deserialize, Serialize};

/// Event schedule timing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventSchedule {
    OneTime,
    Interval {
        interval_value: String,
        interval_unit: String,
    },
}

/// Event definition stored in catalog
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub schema: String,
    pub schedule: EventSchedule,
    pub body: String,
    pub enable: bool,
    pub comment: Option<String>,
    pub created: String,
    pub last_altered: String,
    pub definer: String,
    pub sql_mode: String,
    pub status: String,
    pub on_completion: String,
    pub starts: Option<String>,
    pub ends: Option<String>,
}