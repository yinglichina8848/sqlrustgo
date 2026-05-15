use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CalibrationResult {
    Pass,
    Fail,
}

impl CalibrationResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            CalibrationResult::Pass => "PASS",
            CalibrationResult::Fail => "FAIL",
        }
    }

    pub fn parse_status(s: &str) -> Option<CalibrationResult> {
        match s.to_uppercase().as_str() {
            "PASS" => Some(CalibrationResult::Pass),
            "FAIL" => Some(CalibrationResult::Fail),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationRecord {
    pub record_id: String,
    pub device_id: String,
    pub calibration_date: i64,
    pub result: CalibrationResult,
    pub measured_values: Vec<CalibrationMeasurement>,
    pub technician_signature: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationMeasurement {
    pub parameter: String,
    pub expected: f64,
    pub measured: f64,
    pub tolerance: f64,
    pub passed: bool,
}

impl CalibrationRecord {
    pub fn new(
        device_id: String,
        calibration_date: i64,
        result: CalibrationResult,
        technician_signature: String,
    ) -> Self {
        Self {
            record_id: generate_id(),
            device_id,
            calibration_date,
            result,
            measured_values: Vec::new(),
            technician_signature,
            created_at: chrono_timestamp(),
        }
    }

    pub fn with_measurements(mut self, measurements: Vec<CalibrationMeasurement>) -> Self {
        self.measured_values = measurements;
        self
    }

    pub fn all_measurements_passed(&self) -> bool {
        self.measured_values.iter().all(|m| m.passed)
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
