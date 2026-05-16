use sqlrustgo_gmp::calibration::{
    device::{CalibrationDevice, CalibrationInterval},
    record::{CalibrationMeasurement, CalibrationRecord, CalibrationResult},
    status::CalibrationStatus,
};

#[test]
fn test_calibration_interval_new() {
    let interval = CalibrationInterval::new(30);
    assert_eq!(interval.days, 30);
    assert_eq!(interval.hours, 0);
}

#[test]
fn test_calibration_interval_from_days() {
    let interval = CalibrationInterval::from_days(90);
    assert_eq!(interval.days, 90);
    assert_eq!(interval.hours, 0);
}

#[test]
fn test_calibration_interval_to_seconds() {
    let interval = CalibrationInterval::new(1);
    assert_eq!(interval.to_seconds(), 86400);

    let interval2 = CalibrationInterval { days: 1, hours: 12 };
    assert_eq!(interval2.to_seconds(), 129600);
}

#[test]
fn test_calibration_status_as_str() {
    assert_eq!(CalibrationStatus::Current.as_str(), "CURRENT");
    assert_eq!(CalibrationStatus::Due.as_str(), "DUE");
    assert_eq!(CalibrationStatus::Expired.as_str(), "EXPIRED");
}

#[test]
fn test_calibration_status_parse() {
    assert_eq!(
        CalibrationStatus::parse_status("CURRENT"),
        Some(CalibrationStatus::Current)
    );
    assert_eq!(
        CalibrationStatus::parse_status("current"),
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
    assert_eq!(CalibrationStatus::parse_status("UNKNOWN"), None);
    assert_eq!(CalibrationStatus::parse_status(""), None);
}

#[test]
fn test_calibration_result_as_str() {
    assert_eq!(CalibrationResult::Pass.as_str(), "PASS");
    assert_eq!(CalibrationResult::Fail.as_str(), "FAIL");
}

#[test]
fn test_calibration_result_parse() {
    assert_eq!(
        CalibrationResult::parse_status("PASS"),
        Some(CalibrationResult::Pass)
    );
    assert_eq!(
        CalibrationResult::parse_status("pass"),
        Some(CalibrationResult::Pass)
    );
    assert_eq!(
        CalibrationResult::parse_status("FAIL"),
        Some(CalibrationResult::Fail)
    );
    assert_eq!(CalibrationResult::parse_status("UNKNOWN"), None);
}

#[test]
fn test_calibration_device_creation() {
    let device = CalibrationDevice::new(
        "dev-001".to_string(),
        "thermometer".to_string(),
        90,
        "±0.1°C".to_string(),
    );

    assert_eq!(device.device_id, "dev-001");
    assert_eq!(device.device_type, "thermometer");
    assert_eq!(device.calibration_interval.days, 90);
    assert_eq!(device.tolerance_criteria, "±0.1°C");
    assert_eq!(device.status, CalibrationStatus::Current);
    assert!(device.last_calibration_at.is_none());
    assert!(device.next_calibration_at.is_some());
}

#[test]
fn test_calibration_device_is_calibration_due() {
    let device = CalibrationDevice::new(
        "dev-001".to_string(),
        "thermometer".to_string(),
        0,
        "±0.1°C".to_string(),
    );
    assert!(device.is_calibration_due());
}

#[test]
fn test_calibration_device_is_not_expired_initially() {
    let device = CalibrationDevice::new(
        "dev-001".to_string(),
        "thermometer".to_string(),
        365,
        "±0.1°C".to_string(),
    );
    assert!(!device.is_calibration_expired());
}

#[test]
fn test_calibration_device_update_status_current() {
    let device = CalibrationDevice::new(
        "dev-001".to_string(),
        "thermometer".to_string(),
        365,
        "±0.1°C".to_string(),
    );

    assert_eq!(device.status, CalibrationStatus::Current);
}

#[test]
fn test_calibration_device_update_status_due() {
    let mut device = CalibrationDevice::new(
        "dev-001".to_string(),
        "thermometer".to_string(),
        0,
        "±0.1°C".to_string(),
    );
    device.update_status();
    assert_eq!(device.status, CalibrationStatus::Due);
}

#[test]
fn test_calibration_record_creation() {
    let record = CalibrationRecord::new(
        "dev-001".to_string(),
        1234567890i64,
        CalibrationResult::Pass,
        "tech-signature".to_string(),
    );

    assert!(!record.record_id.is_empty());
    assert_eq!(record.device_id, "dev-001");
    assert_eq!(record.calibration_date, 1234567890i64);
    assert_eq!(record.result, CalibrationResult::Pass);
    assert!(record.measured_values.is_empty());
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
            measured: 51.0,
            tolerance: 1.0,
            passed: true,
        },
    ];

    let record = CalibrationRecord::new(
        "dev-001".to_string(),
        1234567890i64,
        CalibrationResult::Pass,
        "tech-signature".to_string(),
    )
    .with_measurements(measurements);

    assert_eq!(record.measured_values.len(), 2);
}

#[test]
fn test_calibration_record_all_measurements_passed() {
    let measurements = vec![
        CalibrationMeasurement {
            parameter: "temp".to_string(),
            expected: 25.0,
            measured: 25.05,
            tolerance: 0.1,
            passed: true,
        },
        CalibrationMeasurement {
            parameter: "humidity".to_string(),
            expected: 50.0,
            measured: 51.0,
            tolerance: 1.0,
            passed: true,
        },
    ];

    let record = CalibrationRecord::new(
        "dev-001".to_string(),
        1234567890i64,
        CalibrationResult::Pass,
        "tech-signature".to_string(),
    )
    .with_measurements(measurements);

    assert!(record.all_measurements_passed());
}

#[test]
fn test_calibration_record_some_measurements_failed() {
    let measurements = vec![
        CalibrationMeasurement {
            parameter: "temp".to_string(),
            expected: 25.0,
            measured: 25.05,
            tolerance: 0.1,
            passed: true,
        },
        CalibrationMeasurement {
            parameter: "humidity".to_string(),
            expected: 50.0,
            measured: 60.0,
            tolerance: 1.0,
            passed: false,
        },
    ];

    let record = CalibrationRecord::new(
        "dev-001".to_string(),
        1234567890i64,
        CalibrationResult::Pass,
        "tech-signature".to_string(),
    )
    .with_measurements(measurements);

    assert!(!record.all_measurements_passed());
}
