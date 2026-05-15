use sqlrustgo_gmp::sop::{
    SopStatus, StandardOperatingProcedure, BindingStatus, SOPBinding,
    TrainingRecord, TrainingStatus, GmpOperation, SopTrainingBinding,
    TABLE_SOP, TABLE_TRAINING_RECORDS, TABLE_SOP_BINDINGS,
};

#[test]
fn test_sop_status_as_str() {
    assert_eq!(SopStatus::Active.as_str(), "ACTIVE");
    assert_eq!(SopStatus::Inactive.as_str(), "INACTIVE");
    assert_eq!(SopStatus::Superseded.as_str(), "SUPERSEDED");
}

#[test]
fn test_sop_status_parse() {
    assert_eq!(SopStatus::parse_status("ACTIVE"), Some(SopStatus::Active));
    assert_eq!(SopStatus::parse_status("active"), Some(SopStatus::Active));
    assert_eq!(SopStatus::parse_status("Active"), Some(SopStatus::Active));
    assert_eq!(SopStatus::parse_status("INACTIVE"), Some(SopStatus::Inactive));
    assert_eq!(SopStatus::parse_status("SUPERSEDED"), Some(SopStatus::Superseded));
    assert_eq!(SopStatus::parse_status("INVALID"), None);
    assert_eq!(SopStatus::parse_status(""), None);
}

#[test]
fn test_sop_creation() {
    let sop = StandardOperatingProcedure::new(
        "Test SOP".to_string(),
        "1.0".to_string(),
        "Test description".to_string(),
    );

    assert!(!sop.sop_id.is_empty());
    assert_eq!(sop.name, "Test SOP");
    assert_eq!(sop.version, "1.0");
    assert_eq!(sop.description, "Test description");
    assert_eq!(sop.status, SopStatus::Active);
    assert!(sop.qualification_requirements.is_empty());
    assert!(sop.previous_version_id.is_none());
}

#[test]
fn test_sop_with_qualification_requirements() {
    let sop = StandardOperatingProcedure::new(
        "Training SOP".to_string(),
        "1.0".to_string(),
        "Description".to_string(),
    )
    .with_qualification_requirements(vec![
        "GMP certification".to_string(),
        "Safety training".to_string(),
    ]);

    assert_eq!(sop.qualification_requirements.len(), 2);
    assert_eq!(sop.qualification_requirements[0], "GMP certification");
    assert_eq!(sop.qualification_requirements[1], "Safety training");
}

#[test]
fn test_sop_is_active() {
    let sop = StandardOperatingProcedure::new(
        "Test SOP".to_string(),
        "1.0".to_string(),
        "Description".to_string(),
    );
    assert!(sop.is_active());
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
}

#[test]
fn test_sop_binding_is_active() {
    let binding = SOPBinding::new(
        "workflow-step-001".to_string(),
        "sop-001".to_string(),
        "1.0".to_string(),
    );
    assert!(binding.is_active());
}

#[test]
fn test_training_record_creation() {
    let record = TrainingRecord::new(
        "user-001".to_string(),
        "sop-001".to_string(),
        "1.0".to_string(),
        1234567890i64,
        "trainer-001".to_string(),
    );

    assert!(!record.record_id.is_empty());
    assert_eq!(record.operator_id, "user-001");
    assert_eq!(record.sop_id, "sop-001");
    assert_eq!(record.sop_version, "1.0");
    assert_eq!(record.completion_date, 1234567890i64);
    assert_eq!(record.trainer_signature, "trainer-001");
    assert_eq!(record.status, TrainingStatus::Valid);
}

#[test]
fn test_training_record_with_expiry_date() {
    let record = TrainingRecord::new(
        "user-001".to_string(),
        "sop-001".to_string(),
        "1.0".to_string(),
        1234567890i64,
        "trainer-001".to_string(),
    )
    .with_expiry_date(9999999999i64);

    assert!(record.expiry_date.is_some());
    assert_eq!(record.expiry_date.unwrap(), 9999999999i64);
}

#[test]
fn test_training_record_is_valid() {
    let record = TrainingRecord::new(
        "user-001".to_string(),
        "sop-001".to_string(),
        "1.0".to_string(),
        1234567890i64,
        "trainer-001".to_string(),
    );
    assert!(record.is_valid());

    let expired_record = TrainingRecord::new(
        "user-001".to_string(),
        "sop-001".to_string(),
        "1.0".to_string(),
        1234567890i64,
        "trainer-001".to_string(),
    )
    .with_expiry_date(1i64);
    assert!(!expired_record.is_valid());
}

#[test]
fn test_training_status_as_str() {
    assert_eq!(TrainingStatus::Valid.as_str(), "VALID");
    assert_eq!(TrainingStatus::Expired.as_str(), "EXPIRED");
    assert_eq!(TrainingStatus::Superseded.as_str(), "SUPERSEDED");
}

#[test]
fn test_training_status_parse() {
    assert_eq!(TrainingStatus::parse_status("VALID"), Some(TrainingStatus::Valid));
    assert_eq!(TrainingStatus::parse_status("EXPIRED"), Some(TrainingStatus::Expired));
    assert_eq!(TrainingStatus::parse_status("SUPERSEDED"), Some(TrainingStatus::Superseded));
    assert_eq!(TrainingStatus::parse_status("INVALID"), None);
}

#[test]
fn test_gmp_operation_creation() {
    let op = GmpOperation::new("op-001", "CALIBRATION", vec!["SOP-001".to_string()]);

    assert_eq!(op.operation_id, "op-001");
    assert_eq!(op.operation_type, "CALIBRATION");
    assert_eq!(op.required_sops.len(), 1);
    assert!(op.requires_training_verification);
}

#[test]
fn test_sop_training_binding_record_training() {
    let mut binding = SopTrainingBinding::new();
    let record = binding.record_training("user-001", "sop-001", "cert-001");

    assert_eq!(record.operator_id, "user-001");
    assert_eq!(record.sop_id, "sop-001");
    assert!(record.completed_at > 0);
}

#[test]
fn test_sop_training_binding_register_operation() {
    let mut binding = SopTrainingBinding::new();
    let op = GmpOperation::new("op-001", "CALIBRATION", vec!["SOP-001".to_string()]);
    binding.register_operation(op);

    let has_training = binding.operator_has_training("user-001", "sop-001");
    assert!(!has_training);
}

#[test]
fn test_sop_training_binding_verify_training_success() {
    let mut binding = SopTrainingBinding::new();
    binding.record_training("user-001", "sop-001", "cert-001");
    binding.register_operation(GmpOperation::new("op-001", "CALIBRATION", vec!["sop-001".to_string()]));

    let result = binding.verify_training("user-001", "op-001");
    assert!(result.is_verified);
    assert!(result.missing_sops.is_empty());
    assert!(result.expired_sops.is_empty());
}

#[test]
fn test_sop_training_binding_verify_training_missing() {
    let mut binding = SopTrainingBinding::new();
    binding.record_training("user-001", "sop-001", "cert-001");
    binding.register_operation(GmpOperation::new("op-001", "CALIBRATION", vec!["sop-001".to_string(), "sop-002".to_string()]));

    let result = binding.verify_training("user-001", "op-001");
    assert!(!result.is_verified);
    assert!(result.missing_sops.contains(&"sop-002".to_string()));
}

#[test]
fn test_sop_training_binding_operator_has_training() {
    let mut binding = SopTrainingBinding::new();
    binding.record_training("user-001", "sop-001", "cert-001");

    assert!(binding.operator_has_training("user-001", "sop-001"));
    assert!(!binding.operator_has_training("user-002", "sop-001"));
}

#[test]
fn test_sop_training_binding_get_operator_sops() {
    let mut binding = SopTrainingBinding::new();
    binding.record_training("user-001", "sop-001", "cert-001");
    binding.record_training("user-001", "sop-002", "cert-002");

    let sops = binding.get_operator_sops("user-001");
    assert_eq!(sops.len(), 2);
}

#[test]
fn test_sop_training_binding_verify_nonexistent_operation() {
    let binding = SopTrainingBinding::new();
    let result = binding.verify_training("user-001", "nonexistent");

    assert!(!result.is_verified);
}

#[test]
fn test_table_constants() {
    assert_eq!(TABLE_SOP, "gmp_standard_operating_procedures");
    assert_eq!(TABLE_TRAINING_RECORDS, "gmp_training_records");
    assert_eq!(TABLE_SOP_BINDINGS, "gmp_sop_bindings");
}
