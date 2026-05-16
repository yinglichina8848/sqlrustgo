//! GMP Statement Parser Tests
//!
//! Tests for GMP-10, GMP-11, and GMP-12 SQL statement parsing.

use sqlrustgo_parser::parse;
use sqlrustgo_parser::Statement;

#[test]
fn test_parse_register_device_basic() {
    let sql = "REGISTER DEVICE device001 sensor001 Thermometer";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse REGISTER DEVICE: {:?}",
        result
    );
    match result.unwrap() {
        Statement::RegisterDevice(_) => {}
        _ => panic!("Expected RegisterDevice statement"),
    }
}

#[test]
fn test_parse_register_device_with_fingerprint() {
    let sql = "REGISTER DEVICE device001 sensor001 Thermometer CERTIFICATE FINGERPRINT abc123";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse REGISTER DEVICE with fingerprint: {:?}",
        result
    );
    match result.unwrap() {
        Statement::RegisterDevice(_) => {}
        _ => panic!("Expected RegisterDevice statement"),
    }
}

#[test]
fn test_parse_device_heartbeat() {
    let sql = "DEVICE HEARTBEAT device001";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DEVICE HEARTBEAT: {:?}",
        result
    );
    match result.unwrap() {
        Statement::DeviceHeartbeat(_) => {}
        _ => panic!("Expected DeviceHeartbeat statement"),
    }
}

#[test]
fn test_parse_device_collect() {
    let sql = "DEVICE COLLECT DATA device001 temperature_reading";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DEVICE COLLECT: {:?}",
        result
    );
    match result.unwrap() {
        Statement::CollectData(_) => {}
        _ => panic!("Expected CollectData statement"),
    }
}

#[test]
fn test_parse_create_sop_basic() {
    let sql = "CREATE SOP SOP001 v1 Description";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CREATE SOP: {:?}", result);
    match result.unwrap() {
        Statement::CreateSOP(_) => {}
        _ => panic!("Expected CreateSOP statement"),
    }
}

#[test]
fn test_parse_create_sop_with_version_keyword() {
    let sql = "CREATE SOP SOP001 VERSION v2 MySOP description";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE SOP with VERSION keyword: {:?}",
        result
    );
    match result.unwrap() {
        Statement::CreateSOP(_) => {}
        _ => panic!("Expected CreateSOP statement"),
    }
}

#[test]
fn test_parse_record_training() {
    let sql = "RECORD TRAINING sop001 operator001";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse RECORD TRAINING: {:?}",
        result
    );
    match result.unwrap() {
        Statement::RecordTraining(_) => {}
        _ => panic!("Expected RecordTraining statement"),
    }
}

#[test]
fn test_parse_bind_sop() {
    let sql = "BIND SOP workflow001 step001 sop001";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse BIND SOP: {:?}", result);
    match result.unwrap() {
        Statement::BindSOP(_) => {}
        _ => panic!("Expected BindSOP statement"),
    }
}

#[test]
fn test_parse_register_calibration_device() {
    let sql = "CALIBRATION DEVICE cal001 Thermometer";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CALIBRATION DEVICE: {:?}",
        result
    );
    match result.unwrap() {
        Statement::RegisterCalibrationDevice(_) => {}
        _ => panic!("Expected RegisterCalibrationDevice statement"),
    }
}

#[test]
fn test_parse_register_calibration_device_with_interval() {
    let sql = "CALIBRATION DEVICE cal001 Thermometer INTERVAL 90";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CALIBRATION DEVICE with INTERVAL: {:?}",
        result
    );
    match result.unwrap() {
        Statement::RegisterCalibrationDevice(_) => {}
        _ => panic!("Expected RegisterCalibrationDevice statement"),
    }
}

#[test]
fn test_parse_record_calibration_pass() {
    let sql = "RECORD CALIBRATION cal001 PASS";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse RECORD CALIBRATION PASS: {:?}",
        result
    );
    match result.unwrap() {
        Statement::RecordCalibration(_) => {}
        _ => panic!("Expected RecordCalibration statement"),
    }
}

#[test]
fn test_parse_record_calibration_fail() {
    let sql = "RECORD CALIBRATION cal001 FAIL";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse RECORD CALIBRATION FAIL: {:?}",
        result
    );
    match result.unwrap() {
        Statement::RecordCalibration(_) => {}
        _ => panic!("Expected RecordCalibration statement"),
    }
}

#[test]
fn test_parse_multiple_gmp_statements() {
    let statements = vec![
        "REGISTER DEVICE d1 sensor1 Thermometer",
        "DEVICE HEARTBEAT d1",
        "DEVICE COLLECT DATA d1 temp",
        "CREATE SOP SOP001 v1 Description",
        "RECORD TRAINING sop001 user001",
        "BIND SOP wf1 step1 sop001",
        "CALIBRATION DEVICE cal001 Gauge",
        "RECORD CALIBRATION cal001 PASS",
    ];

    for sql in statements {
        let result = parse(sql);
        assert!(result.is_ok(), "Failed to parse: {}", sql);
    }
}
