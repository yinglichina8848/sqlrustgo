use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeviceStatus {
    Registered,
    Suspended,
    Revoked,
}

impl DeviceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceStatus::Registered => "REGISTERED",
            DeviceStatus::Suspended => "SUSPENDED",
            DeviceStatus::Revoked => "REVOKED",
        }
    }

    pub fn parse_status(s: &str) -> Option<DeviceStatus> {
        match s.to_uppercase().as_str() {
            "REGISTERED" => Some(DeviceStatus::Registered),
            "SUSPENDED" => Some(DeviceStatus::Suspended),
            "REVOKED" => Some(DeviceStatus::Revoked),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileDevice {
    pub device_id: String,
    pub certificate_fingerprint: String,
    pub public_key: String,
    pub status: DeviceStatus,
    pub registered_at: i64,
    pub updated_at: i64,
    pub metadata: Option<String>,
}

impl MobileDevice {
    pub fn new(
        device_id: String,
        certificate_fingerprint: String,
        public_key: String,
    ) -> Self {
        let now = chrono_timestamp();
        Self {
            device_id,
            certificate_fingerprint,
            public_key,
            status: DeviceStatus::Registered,
            registered_at: now,
            updated_at: now,
            metadata: None,
        }
    }

    pub fn is_trusted(&self) -> bool {
        self.status == DeviceStatus::Registered
    }

    pub fn can_collect(&self) -> bool {
        self.status == DeviceStatus::Registered
    }
}

fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
