use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationRecord {
    pub record_id: String,
    pub device_id: String,
    pub calibration_date: i64,
    pub performed_by: String,
    pub next_calibration_date: Option<i64>,
    pub measurement_data: HashMap<String, f64>,
    pub is_valid: bool,
}

impl CalibrationRecord {
    pub fn new(device_id: &str, performed_by: &str, next_calibration_date: Option<i64>) -> Self {
        let record_id = format!("CAL-{}-{}", device_id, current_timestamp());
        Self {
            record_id,
            device_id: device_id.to_string(),
            calibration_date: current_timestamp(),
            performed_by: performed_by.to_string(),
            next_calibration_date,
            measurement_data: HashMap::new(),
            is_valid: true,
        }
    }

    pub fn with_measurement(mut self, key: &str, value: f64) -> Self {
        self.measurement_data.insert(key.to_string(), value);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(next) = self.next_calibration_date {
            current_timestamp() > next
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub is_calibrated: bool,
    pub last_calibration_date: Option<i64>,
    pub requires_recalibration: bool,
}

impl Device {
    pub fn new(device_id: &str, device_name: &str, device_type: &str) -> Self {
        Self {
            device_id: device_id.to_string(),
            device_name: device_name.to_string(),
            device_type: device_type.to_string(),
            is_calibrated: false,
            last_calibration_date: None,
            requires_recalibration: false,
        }
    }

    pub fn mark_needs_calibration(&mut self) {
        self.is_calibrated = false;
        self.requires_recalibration = true;
    }

    pub fn mark_calibrated(&mut self, calibration_date: i64) {
        self.is_calibrated = true;
        self.last_calibration_date = Some(calibration_date);
        self.requires_recalibration = false;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationCheckResult {
    pub device_id: String,
    pub is_calibrated: bool,
    pub requires_recalibration: bool,
    pub last_calibration_date: Option<i64>,
    pub next_calibration_date: Option<i64>,
    pub days_until_expiry: Option<i64>,
}

pub struct DeviceCalibrationManager {
    devices: HashMap<String, Device>,
    calibration_records: Vec<CalibrationRecord>,
}

impl DeviceCalibrationManager {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            calibration_records: Vec::new(),
        }
    }

    pub fn register_device(&mut self, device: Device) {
        self.devices.insert(device.device_id.clone(), device);
    }

    pub fn perform_calibration(
        &mut self,
        device_id: &str,
        performed_by: &str,
        next_calibration_date: Option<i64>,
    ) -> Result<&CalibrationRecord, &'static str> {
        let device = self.devices.get_mut(device_id).ok_or("Device not found")?;

        let record = CalibrationRecord::new(device_id, performed_by, next_calibration_date);
        let calibration_date = record.calibration_date;

        device.mark_calibrated(calibration_date);
        self.calibration_records.push(record);
        Ok(self.calibration_records.last().unwrap())
    }

    pub fn check_calibration_status(&self, device_id: &str) -> Option<CalibrationCheckResult> {
        let device = self.devices.get(device_id)?;

        let last_record = self
            .calibration_records
            .iter()
            .filter(|r| r.device_id == device_id)
            .max_by_key(|r| r.calibration_date);

        let (is_calibrated, next_calibration_date) = if let Some(record) = last_record {
            let expired = record.is_expired();
            (!expired, record.next_calibration_date)
        } else {
            (false, None)
        };

        let days_until_expiry = next_calibration_date.map(|next| {
            let now = current_timestamp();
            (next - now) / 86400
        });

        Some(CalibrationCheckResult {
            device_id: device_id.to_string(),
            is_calibrated,
            requires_recalibration: !is_calibrated,
            last_calibration_date: device.last_calibration_date,
            next_calibration_date,
            days_until_expiry,
        })
    }

    pub fn get_device(&self, device_id: &str) -> Option<&Device> {
        self.devices.get(device_id)
    }

    pub fn get_calibration_history(&self, device_id: &str) -> Vec<&CalibrationRecord> {
        self.calibration_records
            .iter()
            .filter(|r| r.device_id == device_id)
            .collect()
    }

    pub fn get_devices_needing_calibration(&self) -> Vec<&Device> {
        self.devices
            .values()
            .filter(|d| d.requires_recalibration)
            .collect()
    }

    pub fn flag_device_needs_calibration(&mut self, device_id: &str) -> Result<(), &'static str> {
        let device = self.devices.get_mut(device_id).ok_or("Device not found")?;
        device.mark_needs_calibration();
        Ok(())
    }

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    pub fn calibrated_device_count(&self) -> usize {
        self.devices.values().filter(|d| d.is_calibrated).count()
    }
}

impl Default for DeviceCalibrationManager {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_device() {
        let mut manager = DeviceCalibrationManager::new();
        let device = Device::new("dev-001", "Thermometer", "Temperature");
        manager.register_device(device);
        assert_eq!(manager.device_count(), 1);
    }

    #[test]
    fn test_perform_calibration() {
        let mut manager = DeviceCalibrationManager::new();
        let device = Device::new("dev-001", "Thermometer", "Temperature");
        manager.register_device(device);
        let result = manager.perform_calibration("dev-001", "tech-001", None);
        assert!(result.is_ok());
        assert!(manager.get_device("dev-001").unwrap().is_calibrated);
    }

    #[test]
    fn test_calibration_expiry() {
        let mut manager = DeviceCalibrationManager::new();
        let device = Device::new("dev-001", "Thermometer", "Temperature");
        manager.register_device(device);

        let mut expired_record =
            CalibrationRecord::new("dev-001", "tech-001", Some(current_timestamp() - 86400));
        expired_record.is_valid = false;
        manager.calibration_records.push(expired_record);

        let status = manager.check_calibration_status("dev-001").unwrap();
        assert!(!status.is_calibrated);
    }

    #[test]
    fn test_flag_device_needs_calibration() {
        let mut manager = DeviceCalibrationManager::new();
        let device = Device::new("dev-001", "Thermometer", "Temperature");
        manager.register_device(device);
        manager.flag_device_needs_calibration("dev-001").unwrap();
        let device = manager.get_device("dev-001").unwrap();
        assert!(device.requires_recalibration);
    }

    #[test]
    fn test_get_devices_needing_calibration() {
        let mut manager = DeviceCalibrationManager::new();
        manager.register_device(Device::new("dev-001", "Thermometer", "Temperature"));
        manager.register_device(Device::new("dev-002", "Scale", "Weight"));
        manager.flag_device_needs_calibration("dev-001").unwrap();
        let needing_cal = manager.get_devices_needing_calibration();
        assert_eq!(needing_cal.len(), 1);
        assert_eq!(needing_cal[0].device_id, "dev-001");
    }

    #[test]
    fn test_calibration_history() {
        let mut manager = DeviceCalibrationManager::new();
        let device = Device::new("dev-001", "Thermometer", "Temperature");
        manager.register_device(device);
        manager
            .perform_calibration("dev-001", "tech-001", None)
            .unwrap();
        manager
            .perform_calibration("dev-001", "tech-002", None)
            .unwrap();
        let history = manager.get_calibration_history("dev-001");
        assert_eq!(history.len(), 2);
    }
}
