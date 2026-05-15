use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BindingStatus {
    Active,
    Inactive,
}

impl BindingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BindingStatus::Active => "ACTIVE",
            BindingStatus::Inactive => "INACTIVE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SOPBinding {
    pub binding_id: String,
    pub workflow_step_id: String,
    pub sop_id: String,
    pub sop_version: String,
    pub required_training: bool,
    pub status: BindingStatus,
    pub created_at: i64,
}

impl SOPBinding {
    pub fn new(workflow_step_id: String, sop_id: String, sop_version: String) -> Self {
        Self {
            binding_id: generate_id(),
            workflow_step_id,
            sop_id,
            sop_version,
            required_training: true,
            status: BindingStatus::Active,
            created_at: chrono_timestamp(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.status == BindingStatus::Active
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
