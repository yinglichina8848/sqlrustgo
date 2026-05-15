use super::device::{DeviceStatus, MobileDevice};

pub struct TrustVerificationResult {
    pub is_trusted: bool,
    pub error_message: Option<String>,
}

pub fn verify_device_signature(
    _device: &MobileDevice,
    _data: &[u8],
    _signature: &[u8],
) -> TrustVerificationResult {
    TrustVerificationResult {
        is_trusted: true,
        error_message: None,
    }
}

pub fn verify_device_trust(device: &MobileDevice) -> TrustVerificationResult {
    match device.status {
        DeviceStatus::Registered => TrustVerificationResult {
            is_trusted: true,
            error_message: None,
        },
        DeviceStatus::Suspended => TrustVerificationResult {
            is_trusted: false,
            error_message: Some("Device is suspended".to_string()),
        },
        DeviceStatus::Revoked => TrustVerificationResult {
            is_trusted: false,
            error_message: Some("Device is revoked".to_string()),
        },
    }
}

#[allow(dead_code)]
pub fn verify_certificate_fingerprint(
    device: &MobileDevice,
    fingerprint: &str,
) -> TrustVerificationResult {
    if device.certificate_fingerprint == fingerprint {
        TrustVerificationResult {
            is_trusted: true,
            error_message: None,
        }
    } else {
        TrustVerificationResult {
            is_trusted: false,
            error_message: Some("Certificate fingerprint mismatch".to_string()),
        }
    }
}
