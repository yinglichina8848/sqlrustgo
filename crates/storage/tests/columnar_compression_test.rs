use sqlrustgo_storage::columnar::{
    auto_select_compression, CompressionConfig, CompressionLevel, CompressionType,
};
use sqlrustgo_storage::engine::ColumnDefinition;

/// Test that ColumnDefinition can hold compression config
#[test]
fn test_column_definition_with_compression() {
    let col = ColumnDefinition {
        name: "data".to_string(),
        data_type: "TEXT".to_string(),
        nullable: false,
        is_unique: false,
        is_primary_key: false,
        references: None,
        auto_increment: false,
        compression: Some(CompressionConfig {
            level: CompressionLevel::Best,
            auto_select: false,
        }),
    };

    assert!(col.compression.is_some());
    assert_eq!(col.compression.unwrap().level, CompressionLevel::Best);
}

/// Test ColumnDefinition with auto-select compression
#[test]
fn test_column_definition_auto_select_compression() {
    let col = ColumnDefinition {
        name: "id".to_string(),
        data_type: "INTEGER".to_string(),
        nullable: false,
        is_unique: false,
        is_primary_key: true,
        references: None,
        auto_increment: true,
        compression: Some(CompressionConfig::default()),
    };

    assert!(col.compression.is_some());
    let config = col.compression.unwrap();
    assert_eq!(config.level, CompressionLevel::Default);
    assert!(config.auto_select);
}

/// Test ColumnDefinition without compression (None = auto)
#[test]
fn test_column_definition_no_compression() {
    let col = ColumnDefinition {
        name: "name".to_string(),
        data_type: "VARCHAR(100)".to_string(),
        nullable: true,
        is_unique: false,
        is_primary_key: false,
        references: None,
        auto_increment: false,
        compression: None,
    };

    assert!(col.compression.is_none());
}

/// Test CompressionLevel mapping
#[test]
fn test_compression_level_lz4_mapping() {
    assert_eq!(CompressionLevel::Fastest.lz4_level(), 0);
    assert_eq!(CompressionLevel::Default.lz4_level(), 12);
    assert_eq!(CompressionLevel::Best.lz4_level(), 17);
    assert_eq!(CompressionLevel::Custom(10).lz4_level(), 10);
}

#[test]
fn test_compression_level_zstd_mapping() {
    assert_eq!(CompressionLevel::Fastest.zstd_level(), 1);
    assert_eq!(CompressionLevel::Default.zstd_level(), 3);
    assert_eq!(CompressionLevel::Best.zstd_level(), 19);
    assert_eq!(CompressionLevel::Custom(15).zstd_level(), 15);
    // Test clamping
    assert_eq!(CompressionLevel::Custom(100).zstd_level(), 22);
    assert_eq!(CompressionLevel::Custom(-5).zstd_level(), 1);
}

/// Test CompressionConfig default
#[test]
fn test_compression_config_default() {
    let config = CompressionConfig::default();
    assert_eq!(config.level, CompressionLevel::Default);
    assert!(config.auto_select);
}

/// Test auto_select_compression function
#[test]
fn test_auto_select_integer() {
    let (algo, level) = auto_select_compression("INTEGER", CompressionLevel::Default);
    assert_eq!(algo, CompressionType::Lz4);
    assert_eq!(level, CompressionLevel::Default);
}

#[test]
fn test_auto_select_text() {
    let (algo, _) = auto_select_compression("TEXT", CompressionLevel::Default);
    assert_eq!(algo, CompressionType::Zstd);
}

#[test]
fn test_auto_select_boolean() {
    let (algo, level) = auto_select_compression("BOOLEAN", CompressionLevel::Default);
    assert_eq!(algo, CompressionType::None);
    assert_eq!(level, CompressionLevel::Default);
}

#[test]
fn test_auto_select_varchar_fastest() {
    let (algo, _) = auto_select_compression("VARCHAR", CompressionLevel::Fastest);
    assert_eq!(algo, CompressionType::Snappy);
}
