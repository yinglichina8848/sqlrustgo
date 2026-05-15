use super::status::CalibrationStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationInterval {
    pub days: u32,
    pub hours: u32,
}

impl CalibrationInterval {
    pub fn new(days: u32) -> Self {
        Self { days, hours: 0 }
    }

    pub fn from_days(days: u32) -> Self {
        Self { days, hours: 0 }
    }

    pub fn to_seconds(&self) -> i64 {
        ((self.days as i64) * 24 * 60 * 60) + ((self.hours as i64) * 60 * 60)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationDevice {
    pub device_id: String,
    pub device_type: String,
    pub calibration_interval: CalibrationInterval,
    pub tolerance_criteria: String,
    pub status: CalibrationStatus,
    pub last_calibration_at: Option<i64>,
    pub next_calibration_at: Option<i64>,
    pub registered_at: i64,
}

impl CalibrationDevice {
    pub fn new(
        device_id: String,
        device_type: String,
        calibration_interval_days: u32,
        tolerance_criteria: String,
    ) -> Self {
        let now = chrono_timestamp();
        let interval = CalibrationInterval::from_days(calibration_interval_days);
        let next_cal = now + interval.to_seconds();
        Self {
            device_id,
            device_type,
            calibration_interval: interval,
            tolerance_criteria,
            status: CalibrationStatus::Current,
            last_calibration_at: None,
            next_calibration_at: Some(next_cal),
            registered_at: now,
        }
    }

    pub fn is_calibration_due(&self) -> bool {
        if let Some(next) = self.next_calibration_at {
            return chrono_timestamp() >= next;
        }
        false
    }

    pub fn is_calibration_expired(&self) -> bool {
        if let Some(next) = self.next_calibration_at {
            let grace_period = 24 * 60 * 60;
            return chrono_timestamp() >= next + grace_period;
        }
        false
    }

    pub fn update_status(&mut self) {
        if self.is_calibration_expired() {
            self.status = CalibrationStatus::Expired;
        } else if self.is_calibration_due() {
            self.status = CalibrationStatus::Due;
        } else {
            self.status = CalibrationStatus::Current;
        }
    }
}

fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
