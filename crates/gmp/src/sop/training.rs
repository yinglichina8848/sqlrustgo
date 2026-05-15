use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrainingStatus {
    Valid,
    Expired,
    Superseded,
}

impl TrainingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrainingStatus::Valid => "VALID",
            TrainingStatus::Expired => "EXPIRED",
            TrainingStatus::Superseded => "SUPERSEDED",
        }
    }

    pub fn parse_status(s: &str) -> Option<TrainingStatus> {
        match s.to_uppercase().as_str() {
            "VALID" => Some(TrainingStatus::Valid),
            "EXPIRED" => Some(TrainingStatus::Expired),
            "SUPERSEDED" => Some(TrainingStatus::Superseded),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRecord {
    pub record_id: String,
    pub operator_id: String,
    pub sop_id: String,
    pub sop_version: String,
    pub completion_date: i64,
    pub expiry_date: Option<i64>,
    pub trainer_signature: String,
    pub status: TrainingStatus,
    pub superseded_by: Option<String>,
    pub created_at: i64,
}

impl TrainingRecord {
    pub fn new(
        operator_id: String,
        sop_id: String,
        sop_version: String,
        completion_date: i64,
        trainer_signature: String,
    ) -> Self {
        Self {
            record_id: generate_id(),
            operator_id,
            sop_id,
            sop_version,
            completion_date,
            expiry_date: None,
            trainer_signature,
            status: TrainingStatus::Valid,
            superseded_by: None,
            created_at: chrono_timestamp(),
        }
    }

    pub fn with_expiry_date(mut self, expiry_date: i64) -> Self {
        self.expiry_date = Some(expiry_date);
        self
    }

    pub fn is_valid(&self) -> bool {
        if self.status == TrainingStatus::Superseded {
            return false;
        }
        if let Some(expiry) = self.expiry_date {
            return chrono_timestamp() < expiry;
        }
        true
    }

    pub fn check_expiry(&mut self) {
        if let Some(expiry) = self.expiry_date {
            if chrono_timestamp() >= expiry {
                self.status = TrainingStatus::Expired;
            }
        }
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
