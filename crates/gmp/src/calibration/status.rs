use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CalibrationStatus {
    Current,
    Due,
    Expired,
}

impl CalibrationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CalibrationStatus::Current => "CURRENT",
            CalibrationStatus::Due => "DUE",
            CalibrationStatus::Expired => "EXPIRED",
        }
    }

    pub fn parse_status(s: &str) -> Option<CalibrationStatus> {
        match s.to_uppercase().as_str() {
            "CURRENT" => Some(CalibrationStatus::Current),
            "DUE" => Some(CalibrationStatus::Due),
            "EXPIRED" => Some(CalibrationStatus::Expired),
            _ => None,
        }
    }
}
