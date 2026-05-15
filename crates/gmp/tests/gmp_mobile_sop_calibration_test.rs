//! GMP Mobile, SOP, and Calibration Module Tests
//!
//! Tests for GMP-10 (Mobile Device Management),
//! GMP-11 (SOP and Training), and GMP-12 (Calibration Management).

use sqlrustgo_gmp::calibration::{
    CalibrationDevice, CalibrationInterval, CalibrationMeasurement, CalibrationRecord,
    CalibrationResult, CalibrationStatus, TABLE_CALIBRATION_DEVICES, TABLE_CALIBRATION_RECORDS,
};
use sqlrustgo_gmp::mobile::{
    verify_device_signature, verify_device_trust, CollectionStatus, DeviceStatus,
    MobileCollectionRecord, MobileDevice, TrustVerificationResult, TABLE_MOBILE_COLLECTIONS,
    TABLE_MOBILE_DEVICES,
};
use sqlrustgo_gmp::sop::{
    BindingStatus, SOPBinding, SopStatus, StandardOperatingProcedure, TrainingRecord,
    TrainingStatus, TABLE_SOP, TABLE_SOP_BINDINGS, TABLE_TRAINING_RECORDS,
};

// =============================================================================
// GMP-10: Mobile Device Management Tests
// =============================================================================

#[test]
fn test_device_status_parse() {
    assert_eq!(
        DeviceStatus::parse_status("REGISTERED"),
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
    assert_eq!(
        DeviceStatus::parse_status("registered"),
        Some(DeviceStatus::Registered)
    );
    assert_eq!(DeviceStatus::parse_status("unknown"), None);
}

#[test]
fn test_device_status_as_str() {
    assert_eq!(DeviceStatus::Registered.as_str(), "REGISTERED");
    assert_eq!(DeviceStatus::Suspended.as_str(), "SUSPENDED");
    assert_eq!(DeviceStatus::Revoked.as_str(), "REVOKED");
}

#[test]
fn test_mobile_device_creation() {
    let device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint-abc".to_string(),
        "public-key-xyz".to_string(),
    );

    assert_eq!(device.device_id, "device-001");
    assert_eq!(device.certificate_fingerprint, "fingerprint-abc");
    assert_eq!(device.public_key, "public-key-xyz");
    assert_eq!(device.status, DeviceStatus::Registered);
    assert!(device.registered_at > 0);
    assert!(device.updated_at > 0);
    assert!(device.metadata.is_none());
}

#[test]
fn test_mobile_device_is_trusted() {
    let device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint".to_string(),
        "public-key".to_string(),
    );
    assert!(device.is_trusted());

    let mut suspended_device = device.clone();
    suspended_device.status = DeviceStatus::Suspended;
    assert!(!suspended_device.is_trusted());
}

#[test]
fn test_mobile_device_can_collect() {
    let device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint".to_string(),
        "public-key".to_string(),
    );
    assert!(device.can_collect());

    let mut revoked_device = device.clone();
    revoked_device.status = DeviceStatus::Revoked;
    assert!(!revoked_device.can_collect());
}

#[test]
fn test_verify_device_trust_registered() {
    let device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint".to_string(),
        "public-key".to_string(),
    );
    let result = verify_device_trust(&device);
    assert!(result.is_trusted);
    assert!(result.error_message.is_none());
}

#[test]
fn test_verify_device_trust_suspended() {
    let mut device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint".to_string(),
        "public-key".to_string(),
    );
    device.status = DeviceStatus::Suspended;
    let result = verify_device_trust(&device);
    assert!(!result.is_trusted);
    assert!(result.error_message.is_some());
    assert_eq!(result.error_message.unwrap(), "Device is suspended");
}

#[test]
fn test_verify_device_trust_revoked() {
    let mut device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint".to_string(),
        "public-key".to_string(),
    );
    device.status = DeviceStatus::Revoked;
    let result = verify_device_trust(&device);
    assert!(!result.is_trusted);
    assert!(result.error_message.is_some());
    assert_eq!(result.error_message.unwrap(), "Device is revoked");
}

#[test]
fn test_verify_device_signature() {
    let device = MobileDevice::new(
        "device-001".to_string(),
        "fingerprint".to_string(),
        "public-key".to_string(),
    );
    let data = b"test data";
    let signature = b"test signature";
    let result = verify_device_signature(&device, data, signature);
    // Currently returns true (stub implementation)
    assert!(result.is_trusted);
}

#[test]
fn test_collection_status_parse() {
    assert_eq!(
        CollectionStatus::parse_status("PENDING"),
        Some(CollectionStatus::Pending)
    );
    assert_eq!(
        CollectionStatus::parse_status("VERIFIED"),
        Some(CollectionStatus::Verified)
    );
    assert_eq!(
        CollectionStatus::parse_status("INVALID"),
        Some(CollectionStatus::Invalid)
    );
    assert_eq!(
        CollectionStatus::parse_status("pending"),
        Some(CollectionStatus::Pending)
    );
    assert_eq!(CollectionStatus::parse_status("unknown"), None);
}

#[test]
fn test_collection_status_as_str() {
    assert_eq!(CollectionStatus::Pending.as_str(), "PENDING");
    assert_eq!(CollectionStatus::Verified.as_str(), "VERIFIED");
    assert_eq!(CollectionStatus::Invalid.as_str(), "INVALID");
}

#[test]
fn test_mobile_collection_record_creation() {
    let record = MobileCollectionRecord::new(
        "collection-001".to_string(),
        "device-001".to_string(),
        "data-hash-abc".to_string(),
        "device-sig-xyz".to_string(),
        1234567890,
    );

    assert_eq!(record.collection_id, "collection-001");
    assert_eq!(record.device_id, "device-001");
    assert_eq!(record.data_hash, "data-hash-abc");
    assert_eq!(record.device_signature, "device-sig-xyz");
    assert_eq!(record.trusted_timestamp, 1234567890);
    assert_eq!(record.status, CollectionStatus::Pending);
    assert!(record.correlation_id.is_none());
    assert!(record.collected_at > 0);
}

#[test]
fn test_mobile_collection_record_with_correlation_id() {
    let record = MobileCollectionRecord::new(
        "collection-001".to_string(),
        "device-001".to_string(),
        "data-hash".to_string(),
        "signature".to_string(),
        1234567890,
    )
    .with_correlation_id("corr-123".to_string());

    assert_eq!(record.correlation_id, Some("corr-123".to_string()));
}

#[test]
fn test_mobile_table_constants() {
    assert_eq!(TABLE_MOBILE_DEVICES, "gmp_mobile_devices");
    assert_eq!(TABLE_MOBILE_COLLECTIONS, "gmp_mobile_collection_audit");
}

// =============================================================================
// GMP-11: SOP and Training Tests
// =============================================================================

#[test]
fn test_sop_status_parse() {
    assert_eq!(SopStatus::parse_status("ACTIVE"), Some(SopStatus::Active));
    assert_eq!(
        SopStatus::parse_status("INACTIVE"),
        Some(SopStatus::Inactive)
    );
    assert_eq!(
        SopStatus::parse_status("SUPERSEDED"),
        Some(SopStatus::Superseded)
    );
    assert_eq!(SopStatus::parse_status("active"), Some(SopStatus::Active));
    assert_eq!(SopStatus::parse_status("unknown"), None);
}

#[test]
fn test_sop_status_as_str() {
    assert_eq!(SopStatus::Active.as_str(), "ACTIVE");
    assert_eq!(SopStatus::Inactive.as_str(), "INACTIVE");
    assert_eq!(SopStatus::Superseded.as_str(), "SUPERSEDED");
}

#[test]
fn test_standard_operating_procedure_creation() {
    let sop = StandardOperatingProcedure::new(
        "SOP-001".to_string(),
        "1.0".to_string(),
        "Standard operating procedure description".to_string(),
    );

    assert!(!sop.sop_id.is_empty());
    assert_eq!(sop.name, "SOP-001");
    assert_eq!(sop.version, "1.0");
    assert_eq!(sop.description, "Standard operating procedure description");
    assert!(sop.qualification_requirements.is_empty());
    assert_eq!(sop.status, SopStatus::Active);
    assert!(sop.created_at > 0);
    assert!(sop.updated_at > 0);
    assert!(sop.previous_version_id.is_none());
}

#[test]
fn test_sop_with_qualification_requirements() {
    let sop = StandardOperatingProcedure::new(
        "SOP-001".to_string(),
        "1.0".to_string(),
        "Description".to_string(),
    )
    .with_qualification_requirements(vec![
        "Training completed".to_string(),
        "Certification obtained".to_string(),
    ]);

    assert_eq!(sop.qualification_requirements.len(), 2);
    assert_eq!(sop.qualification_requirements[0], "Training completed");
}

#[test]
fn test_sop_is_active() {
    let sop = StandardOperatingProcedure::new(
        "SOP-001".to_string(),
        "1.0".to_string(),
        "Description".to_string(),
    );
    assert!(sop.is_active());

    let mut inactive_sop = sop.clone();
    inactive_sop.status = SopStatus::Inactive;
    assert!(!inactive_sop.is_active());
}

#[test]
fn test_training_status_parse() {
    assert_eq!(
        TrainingStatus::parse_status("VALID"),
        Some(TrainingStatus::Valid)
    );
    assert_eq!(
        TrainingStatus::parse_status("EXPIRED"),
        Some(TrainingStatus::Expired)
    );
    assert_eq!(
        TrainingStatus::parse_status("SUPERSEDED"),
        Some(TrainingStatus::Superseded)
    );
    assert_eq!(
        TrainingStatus::parse_status("valid"),
        Some(TrainingStatus::Valid)
    );
    assert_eq!(TrainingStatus::parse_status("unknown"), None);
}

#[test]
fn test_training_status_as_str() {
    assert_eq!(TrainingStatus::Valid.as_str(), "VALID");
    assert_eq!(TrainingStatus::Expired.as_str(), "EXPIRED");
    assert_eq!(TrainingStatus::Superseded.as_str(), "SUPERSEDED");
}

#[test]
fn test_training_record_creation() {
    let record = TrainingRecord::new(
        "operator-001".to_string(),
        "sop-001".to_string(),
        "1.0".to_string(),
        1234567890,
        "trainer-signature".to_string(),
    );

    assert!(!record.record_id.is_empty());
    assert_eq!(record.operator_id, "operator-001");
    assert_eq!(record.sop_id, "sop-001");
    assert_eq!(record.sop_version, "1.0");
    assert_eq!(record.completion_date, 1234567890);
    assert!(record.expiry_date.is_none());
    assert_eq!(record.trainer_signature, "trainer-signature");
    assert_eq!(record.status, TrainingStatus::Valid);
    assert!(record.superseded_by.is_none());
}

#[test]
fn test_training_record_with_expiry() {
    let record = TrainingRecord::new(
        "operator-001".to_string(),
        "sop-001".to_string(),
        "1.0".to_string(),
        1234567890,
        "signature".to_string(),
    )
    .with_expiry_date(9999999999);

    assert!(record.expiry_date.is_some());
    assert_eq!(record.expiry_date.unwrap(), 9999999999);
}

#[test]
fn test_training_record_is_valid() {
    let record = TrainingRecord::new(
        "operator-001".to_string(),
        "sop-001".to_string(),
        "1.0".to_string(),
        1234567890,
        "signature".to_string(),
    )
    .with_expiry_date(9999999999); // Far future

    assert!(record.is_valid());
}

#[test]
fn test_training_record_superseded_not_valid() {
    let mut record = TrainingRecord::new(
        "operator-001".to_string(),
        "sop-001".to_string(),
        "1.0".to_string(),
        1234567890,
        "signature".to_string(),
    );
    record.status = TrainingStatus::Superseded;
    assert!(!record.is_valid());
}

#[test]
fn test_binding_status_as_str() {
    assert_eq!(BindingStatus::Active.as_str(), "ACTIVE");
    assert_eq!(BindingStatus::Inactive.as_str(), "INACTIVE");
}

#[test]
fn test_sop_binding_creation() {
    let binding = SOPBinding::new(
        "workflow-step-001".to_string(),
        "sop-001".to_string(),
        "1.0".to_string(),
    );

    assert!(!binding.binding_id.is_empty());
    assert_eq!(binding.workflow_step_id, "workflow-step-001");
    assert_eq!(binding.sop_id, "sop-001");
    assert_eq!(binding.sop_version, "1.0");
    assert!(binding.required_training);
    assert_eq!(binding.status, BindingStatus::Active);
    assert!(binding.created_at > 0);
}

#[test]
fn test_sop_binding_is_active() {
    let binding = SOPBinding::new(
        "workflow-step-001".to_string(),
        "sop-001".to_string(),
        "1.0".to_string(),
    );
    assert!(binding.is_active());

    let mut inactive_binding = binding.clone();
    inactive_binding.status = BindingStatus::Inactive;
    assert!(!inactive_binding.is_active());
}

#[test]
fn test_sop_table_constants() {
    assert_eq!(TABLE_SOP, "gmp_standard_operating_procedures");
    assert_eq!(TABLE_TRAINING_RECORDS, "gmp_training_records");
    assert_eq!(TABLE_SOP_BINDINGS, "gmp_sop_bindings");
}

// =============================================================================
// GMP-12: Calibration Management Tests
// =============================================================================

#[test]
fn test_calibration_interval_new() {
    let interval = CalibrationInterval::new(30);
    assert_eq!(interval.days, 30);
    assert_eq!(interval.hours, 0);
}

#[test]
fn test_calibration_interval_to_seconds() {
    let interval = CalibrationInterval::new(1); // 1 day
    assert_eq!(interval.to_seconds(), 86400); // 24 * 60 * 60

    let interval2 = CalibrationInterval { days: 1, hours: 12 };
    assert_eq!(interval2.to_seconds(), 129600); // (24 + 12) * 60 * 60
}

#[test]
fn test_calibration_status_parse() {
    assert_eq!(
        CalibrationStatus::parse_status("CURRENT"),
        Some(CalibrationStatus::Current)
    );
    assert_eq!(
        CalibrationStatus::parse_status("DUE"),
        Some(CalibrationStatus::Due)
    );
    assert_eq!(
        CalibrationStatus::parse_status("EXPIRED"),
        Some(CalibrationStatus::Expired)
    );
    assert_eq!(
        CalibrationStatus::parse_status("current"),
        Some(CalibrationStatus::Current)
    );
    assert_eq!(CalibrationStatus::parse_status("unknown"), None);
}

#[test]
fn test_calibration_status_as_str() {
    assert_eq!(CalibrationStatus::Current.as_str(), "CURRENT");
    assert_eq!(CalibrationStatus::Due.as_str(), "DUE");
    assert_eq!(CalibrationStatus::Expired.as_str(), "EXPIRED");
}

#[test]
fn test_calibration_device_creation() {
    let device = CalibrationDevice::new(
        "calib-device-001".to_string(),
        "Thermometer".to_string(),
        30, // 30 days
        "±0.1°C".to_string(),
    );

    assert_eq!(device.device_id, "calib-device-001");
    assert_eq!(device.device_type, "Thermometer");
    assert_eq!(device.calibration_interval.days, 30);
    assert_eq!(device.tolerance_criteria, "±0.1°C");
    assert_eq!(device.status, CalibrationStatus::Current);
    assert!(device.last_calibration_at.is_none());
    assert!(device.next_calibration_at.is_some());
    assert!(device.registered_at > 0);
}

#[test]
fn test_calibration_device_is_calibration_due() {
    let device = CalibrationDevice::new(
        "calib-device-001".to_string(),
        "Thermometer".to_string(),
        30,
        "±0.1°C".to_string(),
    );
    // New device should not be due
    assert!(!device.is_calibration_due());
}

#[test]
fn test_calibration_result_parse() {
    assert_eq!(
        CalibrationResult::parse_status("PASS"),
        Some(CalibrationResult::Pass)
    );
    assert_eq!(
        CalibrationResult::parse_status("FAIL"),
        Some(CalibrationResult::Fail)
    );
    assert_eq!(
        CalibrationResult::parse_status("pass"),
        Some(CalibrationResult::Pass)
    );
    assert_eq!(CalibrationResult::parse_status("unknown"), None);
}

#[test]
fn test_calibration_result_as_str() {
    assert_eq!(CalibrationResult::Pass.as_str(), "PASS");
    assert_eq!(CalibrationResult::Fail.as_str(), "FAIL");
}

#[test]
fn test_calibration_record_creation() {
    let record = CalibrationRecord::new(
        "calib-device-001".to_string(),
        1234567890,
        CalibrationResult::Pass,
        "technician-signature".to_string(),
    );

    assert!(!record.record_id.is_empty());
    assert_eq!(record.device_id, "calib-device-001");
    assert_eq!(record.calibration_date, 1234567890);
    assert_eq!(record.result, CalibrationResult::Pass);
    assert!(record.measured_values.is_empty());
    assert_eq!(record.technician_signature, "technician-signature");
    assert!(record.created_at > 0);
}

#[test]
fn test_calibration_record_with_measurements() {
    let measurements = vec![
        CalibrationMeasurement {
            parameter: "temperature".to_string(),
            expected: 25.0,
            measured: 25.05,
            tolerance: 0.1,
            passed: true,
        },
        CalibrationMeasurement {
            parameter: "humidity".to_string(),
            expected: 50.0,
            measured: 50.1,
            tolerance: 0.5,
            passed: true,
        },
    ];

    let record = CalibrationRecord::new(
        "calib-device-001".to_string(),
        1234567890,
        CalibrationResult::Pass,
        "signature".to_string(),
    )
    .with_measurements(measurements);

    assert_eq!(record.measured_values.len(), 2);
    assert!(record.all_measurements_passed());
}

#[test]
fn test_calibration_record_all_measurements_passed() {
    let measurements = vec![CalibrationMeasurement {
        parameter: "temp".to_string(),
        expected: 25.0,
        measured: 25.5, // Outside tolerance
        tolerance: 0.1,
        passed: false,
    }];

    let record = CalibrationRecord::new(
        "calib-device-001".to_string(),
        1234567890,
        CalibrationResult::Fail,
        "signature".to_string(),
    )
    .with_measurements(measurements);

    assert!(!record.all_measurements_passed());
}

#[test]
fn test_calibration_table_constants() {
    assert_eq!(TABLE_CALIBRATION_DEVICES, "gmp_calibration_devices");
    assert_eq!(TABLE_CALIBRATION_RECORDS, "gmp_calibration_records");
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn test_trust_verification_result_struct() {
    let trusted_result = TrustVerificationResult {
        is_trusted: true,
        error_message: None,
    };
    assert!(trusted_result.is_trusted);
    assert!(trusted_result.error_message.is_none());

    let untrusted_result = TrustVerificationResult {
        is_trusted: false,
        error_message: Some("Test error".to_string()),
    };
    assert!(!untrusted_result.is_trusted);
    assert!(untrusted_result.error_message.is_some());
}

#[test]
fn test_calibration_measurement_struct() {
    let measurement = CalibrationMeasurement {
        parameter: "temperature".to_string(),
        expected: 100.0,
        measured: 100.05,
        tolerance: 0.1,
        passed: false,
    };

    assert_eq!(measurement.parameter, "temperature");
    assert!((measurement.expected - 100.0).abs() < 0.001);
    assert!((measurement.measured - 100.05).abs() < 0.001);
    assert!((measurement.tolerance - 0.1).abs() < 0.001);
    assert!(!measurement.passed);
}
