// Page Tests
use sqlrustgo_storage::page::{Page, PageType, PageWriter};
use sqlrustgo_types::Value;
use std::io::Cursor;

#[test]
fn test_page_new() {
    let page = Page::new(1);
    assert_eq!(page.page_id(), 1);
}

#[test]
fn test_page_new_data() {
    let page = Page::new_data(1, 100);
    assert_eq!(page.page_id(), 1);
    assert_eq!(page.page_type(), PageType::Data);
}

#[test]
fn test_page_size() {
    let size = Page::size();
    assert!(size > 0);
}

#[test]
fn test_page_type() {
    let page = Page::new(1);
    assert_eq!(page.page_type(), PageType::Free);
}

#[test]
fn test_page_row_count() {
    let page = Page::new(1);
    assert_eq!(page.row_count(), 0);
}

#[test]
fn test_page_free_space() {
    let page = Page::new(1);
    let free = page.free_space();
    assert!(free > 0);
}

#[test]
fn test_page_write_row() {
    let mut page = Page::new(1);
    let values = vec![
        Value::Integer(1),
        Value::Text("test".to_string()),
    ];
    
    let result = page.write_row(&values);
    assert!(result);
    assert_eq!(page.row_count(), 1);
}

#[test]
fn test_page_read_rows() {
    let mut page = Page::new(1);
    let values = vec![
        Value::Integer(1),
        Value::Text("test".to_string()),
    ];
    
    page.write_row(&values);
    let rows = page.read_rows();
    
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], Value::Integer(1));
}

#[test]
fn test_page_to_bytes() {
    let page = Page::new(1);
    let bytes = page.to_bytes();
    assert!(!bytes.is_empty());
}

#[test]
fn test_page_from_bytes() {
    let page = Page::new(1);
    let bytes = page.to_bytes();
    
    let restored = Page::from_bytes(bytes);
    assert!(restored.is_some());
}

#[test]
fn test_value_to_bytes_integer() {
    let value = Value::Integer(42);
    let bytes = sqlrustgo_storage::page::value_to_bytes(&value);
    assert!(!bytes.is_empty());
}

#[test]
fn test_value_to_bytes_text() {
    let value = Value::Text("hello".to_string());
    let bytes = sqlrustgo_storage::page::value_to_bytes(&value);
    assert!(!bytes.is_empty());
}

#[test]
fn test_value_to_bytes_null() {
    let value = Value::Null;
    let bytes = sqlrustgo_storage::page::value_to_bytes(&value);
    assert!(!bytes.is_empty());
}

#[test]
fn test_bytes_to_value_integer() {
    let value = Value::Integer(42);
    let bytes = sqlrustgo_storage::page::value_to_bytes(&value);
    let restored = sqlrustgo_storage::page::bytes_to_value(&bytes);
    assert!(restored.is_some());
}

#[test]
fn test_bytes_to_value_text() {
    let value = Value::Text("hello".to_string());
    let bytes = sqlrustgo_storage::page::value_to_bytes(&value);
    let restored = sqlrustgo_storage::page::bytes_to_value(&bytes);
    assert!(restored.is_some());
}

#[test]
fn test_page_writer() {
    let mut buffer = Vec::new();
    {
        let mut writer = PageWriter::new(&mut buffer);
        let page = Page::new(1);
        let result = writer.write_page(&page);
        assert!(result.is_ok());
    }
    assert!(!buffer.is_empty());
}
