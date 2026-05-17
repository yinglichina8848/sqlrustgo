use sqlrustgo_gmp::evidence_export::{
    AuditChainRecord, EvidenceRecord, JsonExporter, PackageBuilder, PdfExporter,
    SignerEd25519,
};

#[test]
fn test_json_exporter_records() {
    let records = vec![
        AuditChainRecord {
            action: "INSERT".to_string(),
            block_height: 1,
            hash: "abc123def456789012".to_string(),
            timestamp: 1000,
        },
        AuditChainRecord {
            action: "UPDATE".to_string(),
            block_height: 2,
            hash: "def456789abc123456".to_string(),
            timestamp: 2000,
        },
    ];

    let json = JsonExporter::export_records(&records).unwrap();
    let json_str = String::from_utf8(json).unwrap();
    assert!(json_str.contains("INSERT"));
    assert!(json_str.contains("UPDATE"));
}

#[test]
fn test_json_exporter_evidence() {
    let evidence = vec![
        EvidenceRecord {
            operation: "create_table".to_string(),
            hash: "hash1".to_string(),
            timestamp: 1000,
        },
    ];

    let json = JsonExporter::export_evidence(&evidence).unwrap();
    let json_str = String::from_utf8(json).unwrap();
    assert!(json_str.contains("create_table"));
}

#[test]
fn test_signer_ed25519() {
    let signer = SignerEd25519::new();
    let data = b"test data to sign";

    let signature = signer.sign(data);
    assert_eq!(signature.len(), 64);

    let public_key = signer.public_key();
    assert_eq!(public_key.len(), 32);
}

#[test]
fn test_signer_from_secret_key() {
    let secret_key = [0u8; 32];
    let signer = SignerEd25519::from_secret_key(&secret_key).unwrap();

    let data = b"test";
    let signature = signer.sign(data);
    assert_eq!(signature.len(), 64);
}

#[test]
fn test_signature_verification() {
    use sqlrustgo_gmp::evidence_export::verify_signature;

    let signer = SignerEd25519::new();
    let data = b"test data for verification";

    let signature = signer.sign(data);
    let public_key = signer.public_key();

    let result = verify_signature(data, &signature, &public_key).unwrap();
    assert!(result);
}

#[test]
fn test_signature_verification_fails_with_wrong_data() {
    use sqlrustgo_gmp::evidence_export::verify_signature;

    let signer = SignerEd25519::new();
    let data = b"original data";
    let wrong_data = b"tampered data";

    let signature = signer.sign(data);
    let public_key = signer.public_key();

    let result = verify_signature(wrong_data, &signature, &public_key).unwrap();
    assert!(!result);
}

#[test]
fn test_pdf_exporter_generates_pdf() {
    let records = vec![
        AuditChainRecord {
            action: "INSERT".to_string(),
            block_height: 1,
            hash: "abc123def456789012".to_string(),
            timestamp: 1000,
        },
    ];

    let evidence = vec![
        EvidenceRecord {
            operation: "create_table".to_string(),
            hash: "xyz789abc123456789".to_string(),
            timestamp: 1000,
        },
    ];

    let pdf = PdfExporter::generate_compliance_report("Test Report", &records, &evidence).unwrap();
    assert!(!pdf.is_empty());
    assert!(pdf.starts_with(&[0x25, 0x50, 0x44, 0x46])); // %PDF
}

#[test]
fn test_package_builder_build_and_verify() {
    let temp_dir = std::env::temp_dir().join("evidence_package_test");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let records = vec![
        AuditChainRecord {
            action: "INSERT".to_string(),
            block_height: 1,
            hash: "abc123def456789012".to_string(),
            timestamp: 1000,
        },
    ];

    let evidence = vec![
        EvidenceRecord {
            operation: "create_table".to_string(),
            hash: "def456789abc123456".to_string(),
            timestamp: 1000,
        },
    ];

    let signer = SignerEd25519::new();

    let package_path = PackageBuilder::new()
        .with_records(records.clone())
        .with_evidence(evidence.clone())
        .with_timestamp_range(0, 1000)
        .with_signer(signer)
        .build(&temp_dir)
        .unwrap();

    assert!(package_path.manifest.exists());
    assert!(temp_dir.join("records.json").exists());
    assert!(temp_dir.join("evidence.json").exists());
    assert!(temp_dir.join("report.pdf").exists());
    assert!(temp_dir.join("signature.bin").exists());
    assert!(temp_dir.join("public_key.bin").exists());

    let report = PackageBuilder::verify(&temp_dir).unwrap();
    assert!(report.is_valid);
    assert!(report.manifest_valid);

    let _ = std::fs::remove_dir_all(&temp_dir);
}