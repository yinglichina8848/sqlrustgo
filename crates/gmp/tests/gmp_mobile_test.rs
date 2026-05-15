use sqlrustgo_gmp::mobile::{
    collection::{CollectionStatus, MobileCollection, MobileCollectionRecord},
    device::{DeviceStatus, MobileDevice},
    trust::{verify_certificate_fingerprint, verify_device_signature, verify_device_trust},
};

#[test]
fn test_collection_status_as_str() {
    assert_eq!(CollectionStatus::Pending.as_str(), "PENDING");
    assert_eq!(CollectionStatus::Verified.as_str(), "VERIFIED");
    assert_eq!(CollectionStatus::Invalid.as_str(), "INVALID");
}

#[test]
fn test_collection_status_parse() {
    assert_eq!(
        CollectionStatus::parse_status("PENDING"),
        Some(CollectionStatus::Pending)
    );
    assert_eq!(
        CollectionStatus::parse_status("pending"),
        Some(CollectionStatus::Pending)
    );
    assert_eq!(CollectionStatus::parse_status("PENDING "), None);
    assert_eq!(
        CollectionStatus::parse_status("VERIFIED"),
        Some(CollectionStatus::Verified)
    );
    assert_eq!(
        CollectionStatus::parse_status("INVALID"),
        Some(CollectionStatus::Invalid)
    );
    assert_eq!(CollectionStatus::parse_status("UNKNOWN"), None);
    assert_eq!(CollectionStatus::parse_status(""), None);
}

#[test]
fn test_mobile_collection_record_creation() {
    let record = MobileCollectionRecord::new(
        "col-001".to_string(),
        "device-001".to_string(),
        "hash123".to_string(),
        "sig123".to_string(),
        1234567890i64,
    );

    assert_eq!(record.collection_id, "col-001");
    assert_eq!(record.device_id, "device-001");
    assert_eq!(record.data_hash, "hash123");
    assert_eq!(record.device_signature, "sig123");
    assert_eq!(record.trusted_timestamp, 1234567890i64);
    assert_eq!(record.status, CollectionStatus::Pending);
    assert!(record.correlation_id.is_none());
}

#[test]
fn test_mobile_collection_record_with_correlation_id() {
    let record = MobileCollectionRecord::new(
        "col-001".to_string(),
        "device-001".to_string(),
        "hash123".to_string(),
        "sig123".to_string(),
        1234567890i64,
    )
    .with_correlation_id("corr-001".to_string());

    assert_eq!(record.correlation_id, Some("corr-001".to_string()));
}

#[test]
fn test_mobile_collection_table_name() {
    assert_eq!(MobileCollection::TABLE_NAME, "gmp_mobile_collection_audit");
}

#[test]
fn test_device_status_as_str() {
    assert_eq!(DeviceStatus::Registered.as_str(), "REGISTERED");
    assert_eq!(DeviceStatus::Suspended.as_str(), "SUSPENDED");
    assert_eq!(DeviceStatus::Revoked.as_str(), "REVOKED");
}

#[test]
fn test_device_status_parse() {
    assert_eq!(
        DeviceStatus::parse_status("REGISTERED"),
        Some(DeviceStatus::Registered)
    );
    assert_eq!(
        DeviceStatus::parse_status("registered"),
        Some(DeviceStatus::Registered)
    );
    assert_eq!(
        DeviceStatus::parse_status("SUSPENDED"),
        Some(DeviceStatus::Suspended)
    );
    assert_eq!(
        DeviceStatus::parse_status("REVOKED"),
        Some(DeviceStatus::Revoked)
    );
    assert_eq!(DeviceStatus::parse_status("UNKNOWN"), None);
    assert_eq!(DeviceStatus::parse_status(""), None);
}

#[test]
fn test_mobile_device_creation() {
    let device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint123".to_string(),
        "publickey456".to_string(),
    );

    assert_eq!(device.device_id, "device-001");
    assert_eq!(device.certificate_fingerprint, "fingerprint123");
    assert_eq!(device.public_key, "publickey456");
    assert_eq!(device.status, DeviceStatus::Registered);
    assert!(device.registered_at > 0);
    assert!(device.updated_at > 0);
    assert!(device.metadata.is_none());
}

#[test]
fn test_mobile_device_is_trusted() {
    let device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint123".to_string(),
        "publickey456".to_string(),
    );
    assert!(device.is_trusted());
}

#[test]
fn test_mobile_device_can_collect() {
    let device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint123".to_string(),
        "publickey456".to_string(),
    );
    assert!(device.can_collect());
}

#[test]
fn test_verify_device_signature_registered() {
    let device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint123".to_string(),
        "publickey456".to_string(),
    );

    let result = verify_device_signature(&device, b"test data", b"signature");
    assert!(result.is_trusted);
    assert!(result.error_message.is_none());
}

#[test]
fn test_verify_device_trust_registered() {
    let device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint123".to_string(),
        "publickey456".to_string(),
    );

    let result = verify_device_trust(&device);
    assert!(result.is_trusted);
    assert!(result.error_message.is_none());
}

#[test]
fn test_verify_device_trust_suspended() {
    let device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint123".to_string(),
        "publickey456".to_string(),
    );

    let suspended_device = MobileDevice {
        device_id: device.device_id.clone(),
        certificate_fingerprint: device.certificate_fingerprint.clone(),
        public_key: device.public_key.clone(),
        status: DeviceStatus::Suspended,
        registered_at: device.registered_at,
        updated_at: chrono_timestamp(),
        metadata: device.metadata.clone(),
    };

    let result = verify_device_trust(&suspended_device);
    assert!(!result.is_trusted);
    assert_eq!(
        result.error_message,
        Some("Device is suspended".to_string())
    );
}

#[test]
fn test_verify_device_trust_revoked() {
    let revoked_device = MobileDevice {
        device_id: "device-001".to_string(),
        certificate_fingerprint: "fingerprint123".to_string(),
        public_key: "publickey456".to_string(),
        status: DeviceStatus::Revoked,
        registered_at: 1234567890i64,
        updated_at: chrono_timestamp(),
        metadata: None,
    };

    let result = verify_device_trust(&revoked_device);
    assert!(!result.is_trusted);
    assert_eq!(result.error_message, Some("Device is revoked".to_string()));
}

#[test]
fn test_verify_certificate_fingerprint_match() {
    let device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint123".to_string(),
        "publickey456".to_string(),
    );

    let result = verify_certificate_fingerprint(&device, "fingerprint123");
    assert!(result.is_trusted);
    assert!(result.error_message.is_none());
}

#[test]
fn test_verify_certificate_fingerprint_mismatch() {
    let device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint123".to_string(),
        "publickey456".to_string(),
    );

    let result = verify_certificate_fingerprint(&device, "wrong_fingerprint");
    assert!(!result.is_trusted);
    assert_eq!(
        result.error_message,
        Some("Certificate fingerprint mismatch".to_string())
    );
}

fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
