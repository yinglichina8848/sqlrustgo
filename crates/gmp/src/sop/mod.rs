mod binding;
mod training;
mod training_binding;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SopStatus {
    Active,
    Inactive,
    Superseded,
}

impl SopStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SopStatus::Active => "ACTIVE",
            SopStatus::Inactive => "INACTIVE",
            SopStatus::Superseded => "SUPERSEDED",
        }
    }

    pub fn parse_status(s: &str) -> Option<SopStatus> {
        match s.to_uppercase().as_str() {
            "ACTIVE" => Some(SopStatus::Active),
            "INACTIVE" => Some(SopStatus::Inactive),
            "SUPERSEDED" => Some(SopStatus::Superseded),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardOperatingProcedure {
    pub sop_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub qualification_requirements: Vec<String>,
    pub status: SopStatus,
    pub created_at: i64,
    pub updated_at: i64,
    pub previous_version_id: Option<String>,
}

impl StandardOperatingProcedure {
    pub fn new(name: String, version: String, description: String) -> Self {
        let now = chrono_timestamp();
        Self {
            sop_id: generate_id(),
            name,
            version,
            description,
            qualification_requirements: Vec::new(),
            status: SopStatus::Active,
            created_at: now,
            updated_at: now,
            previous_version_id: None,
        }
    }

    pub fn with_qualification_requirements(mut self, requirements: Vec<String>) -> Self {
        self.qualification_requirements = requirements;
        self
    }

    pub fn is_active(&self) -> bool {
        self.status == SopStatus::Active
    }
}

fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn generate_id() -> String {
    use std::fmt::Write;
    let mut id = String::new();
    let timestamp = chrono_timestamp();
    let random: u32 = rand::random();
    write!(&mut id, "{:x}-{:x}", timestamp, random).ok();
    id
}

pub use binding::{BindingStatus, SOPBinding};
pub use training::{TrainingRecord, TrainingStatus};
pub use training_binding::{GmpOperation, SopTrainingBinding, TrainingVerificationResult};

pub const TABLE_SOP: &str = "gmp_standard_operating_procedures";
pub const TABLE_TRAINING_RECORDS: &str = "gmp_training_records";
pub const TABLE_SOP_BINDINGS: &str = "gmp_sop_bindings";
