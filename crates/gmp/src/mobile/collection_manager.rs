use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub device_id: String,
    pub certificate_subject: String,
    pub registered_at: i64,
    pub is_active: bool,
    pub last_seen_at: Option<i64>,
}

impl TrustedDevice {
    pub fn new(device_id: &str, certificate_subject: &str) -> Self {
        Self {
            device_id: device_id.to_string(),
            certificate_subject: certificate_subject.to_string(),
            registered_at: current_timestamp(),
            is_active: true,
            last_seen_at: None,
        }
    }

    pub fn mark_seen(&mut self) {
        self.last_seen_at = Some(current_timestamp());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDataSubmission {
    pub device_id: String,
    pub data: String,
    pub device_timestamp: i64,
    pub received_at: i64,
    pub signature: Vec<u8>,
}

impl DeviceDataSubmission {
    pub fn new(device_id: &str, data: &str, device_timestamp: i64, signature: Vec<u8>) -> Self {
        Self {
            device_id: device_id.to_string(),
            data: data.to_string(),
            device_timestamp,
            received_at: current_timestamp(),
            signature,
        }
    }
}

pub struct MobileTrustedCollection {
    trusted_devices: HashMap<String, TrustedDevice>,
    submissions: Vec<DeviceDataSubmission>,
}

impl MobileTrustedCollection {
    pub fn new() -> Self {
        Self {
            trusted_devices: HashMap::new(),
            submissions: Vec::new(),
        }
    }

    pub fn register_device(
        &mut self,
        device_id: &str,
        certificate_subject: &str,
    ) -> Result<&TrustedDevice, &'static str> {
        if self.trusted_devices.contains_key(device_id) {
            return Err("Device already registered");
        }
        let device = TrustedDevice::new(device_id, certificate_subject);
        self.trusted_devices.insert(device_id.to_string(), device);
        Ok(self.trusted_devices.get(device_id).unwrap())
    }

    pub fn is_device_trusted(&self, device_id: &str) -> bool {
        self.trusted_devices
            .get(device_id)
            .map(|d| d.is_active)
            .unwrap_or(false)
    }

    pub fn submit_data(
        &mut self,
        device_id: &str,
        data: &str,
        device_timestamp: i64,
        signature: Vec<u8>,
    ) -> Result<&DeviceDataSubmission, &'static str> {
        if !self.is_device_trusted(device_id) {
            return Err("Device not trusted");
        }
        let submission = DeviceDataSubmission::new(device_id, data, device_timestamp, signature);
        if let Some(device) = self.trusted_devices.get_mut(device_id) {
            device.mark_seen();
        }
        self.submissions.push(submission);
        Ok(self.submissions.last().unwrap())
    }

    pub fn get_device(&self, device_id: &str) -> Option<&TrustedDevice> {
        self.trusted_devices.get(device_id)
    }

    pub fn get_submissions(&self) -> &[DeviceDataSubmission] {
        &self.submissions
    }

    pub fn revoke_device(&mut self, device_id: &str) -> Result<(), &'static str> {
        if let Some(device) = self.trusted_devices.get_mut(device_id) {
            device.is_active = false;
            Ok(())
        } else {
            Err("Device not found")
        }
    }

    pub fn trusted_device_count(&self) -> usize {
        self.trusted_devices
            .values()
            .filter(|d| d.is_active)
            .count()
    }
}

impl Default for MobileTrustedCollection {
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
        let mut collection = MobileTrustedCollection::new();
        let result = collection.register_device("device-001", "CN=TestDevice");
        assert!(result.is_ok());
        assert_eq!(collection.trusted_device_count(), 1);
    }

    #[test]
    fn test_is_device_trusted() {
        let mut collection = MobileTrustedCollection::new();
        collection
            .register_device("device-001", "CN=TestDevice")
            .unwrap();
        assert!(collection.is_device_trusted("device-001"));
        assert!(!collection.is_device_trusted("device-002"));
    }

    #[test]
    fn test_submit_data_from_trusted_device() {
        let mut collection = MobileTrustedCollection::new();
        collection
            .register_device("device-001", "CN=TestDevice")
            .unwrap();
        let result =
            collection.submit_data("device-001", "test data", 1234567890, vec![0xaa, 0xbb]);
        assert!(result.is_ok());
        assert_eq!(collection.get_submissions().len(), 1);
    }

    #[test]
    fn test_submit_data_from_untrusted_device() {
        let mut collection = MobileTrustedCollection::new();
        let result =
            collection.submit_data("device-001", "test data", 1234567890, vec![0xaa, 0xbb]);
        assert!(result.is_err());
    }

    #[test]
    fn test_revoke_device() {
        let mut collection = MobileTrustedCollection::new();
        collection
            .register_device("device-001", "CN=TestDevice")
            .unwrap();
        collection.revoke_device("device-001").unwrap();
        assert!(!collection.is_device_trusted("device-001"));
    }

    #[test]
    fn test_device_submission_timestamp() {
        let mut collection = MobileTrustedCollection::new();
        collection
            .register_device("device-001", "CN=TestDevice")
            .unwrap();
        let submission = collection
            .submit_data("device-001", "data", 1234567890, vec![])
            .unwrap();
        assert!(submission.received_at >= submission.device_timestamp);
    }
}
