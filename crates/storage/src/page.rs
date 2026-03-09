//! Page management for storage engine
//!
//! # Binary Page Format
//!
//! +----------------+----------------+----------------+----------------+
//! | Page Header (64 bytes)                                      |
//! +----------------+----------------+----------------+----------------+
//! | Magic Number    | Version    | Page Type  | Checksum       |
//! +----------------+----------------+----------------+----------------+
//! | Previous Page   | Next Page   | Row Count  | Free Space     |
//! +----------------+----------------+----------------+----------------+
//! | Table ID (8 bytes)                                           |
//! +----------------+----------------+----------------+----------------+
//! | Reserved (24 bytes)                                          |
//! +----------------+----------------+----------------+----------------+
//! | Data Area (>= 4032 bytes)                                    |
//! +----------------+----------------+----------------+----------------+

use sqlrustgo_types::Value;
use std::io::{Read, Write};

/// Page size constant (4KB)
pub const PAGE_SIZE: usize = 4096;
/// Page header size
pub const PAGE_HEADER_SIZE: usize = 64;
/// Maximum data size per page
pub const PAGE_DATA_SIZE: usize = PAGE_SIZE - PAGE_HEADER_SIZE;

/// Page type enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageType {
    /// Data page containing table rows
    Data,
    /// Index page containing B+Tree nodes
    Index,
    /// Free/empty page
    Free,
    /// Metadata page
    Meta,
}

/// Page magic number for validation
const PAGE_MAGIC: u32 = 0x5051_4753; // "PQGS" in hex
/// Current page format version
const PAGE_VERSION: u16 = 1;

/// Page structure with binary format
#[derive(Debug, Clone)]
pub struct Page {
    pub page_id: u32,
    pub data: Vec<u8>,
    page_type: PageType,
    row_count: u32,
    free_space: u32,
    table_id: u64,
}

impl Page {
    /// Create a new page
    pub fn new(page_id: u32) -> Self {
        Self {
            page_id,
            data: vec![0u8; PAGE_SIZE],
            page_type: PageType::Free,
            row_count: 0,
            free_space: PAGE_DATA_SIZE as u32,
            table_id: 0,
        }
    }

    /// Create a data page
    pub fn new_data(page_id: u32, table_id: u64) -> Self {
        let mut page = Self::new(page_id);
        page.page_type = PageType::Data;
        page.table_id = table_id;
        page.write_header();
        page
    }

    /// Get page ID
    pub fn page_id(&self) -> u32 {
        self.page_id
    }

    /// Get page size
    pub fn size() -> usize {
        PAGE_SIZE
    }

    /// Get page type
    pub fn page_type(&self) -> PageType {
        self.page_type
    }

    /// Get row count
    pub fn row_count(&self) -> u32 {
        self.row_count
    }

    /// Get free space
    pub fn free_space(&self) -> u32 {
        self.free_space
    }

    /// Write page header
    fn write_header(&mut self) {
        let mut offset = 0;

        // Magic number (4 bytes)
        self.data[offset..offset + 4].copy_from_slice(&PAGE_MAGIC.to_le_bytes());
        offset += 4;

        // Page ID (4 bytes)
        self.data[offset..offset + 4].copy_from_slice(&self.page_id.to_le_bytes());
        offset += 4;

        // Version (2 bytes)
        self.data[offset..offset + 2].copy_from_slice(&PAGE_VERSION.to_le_bytes());
        offset += 2;

        // Page type (1 byte)
        self.data[offset] = match self.page_type {
            PageType::Data => 1,
            PageType::Index => 2,
            PageType::Free => 0,
            PageType::Meta => 3,
        };
        offset += 1;

        // Reserved (1 byte)
        offset += 1;

        // Checksum placeholder (4 bytes)
        offset += 4;

        // Previous page (4 bytes)
        offset += 4;

        // Next page (4 bytes)
        offset += 4;

        // Row count (4 bytes)
        self.data[offset..offset + 4].copy_from_slice(&self.row_count.to_le_bytes());
        offset += 4;

        // Free space (4 bytes)
        self.data[offset..offset + 4].copy_from_slice(&self.free_space.to_le_bytes());
        offset += 4;

        // Table ID (8 bytes)
        self.data[offset..offset + 8].copy_from_slice(&self.table_id.to_le_bytes());
    }

    /// Read page header
    fn read_header(&mut self) {
        // Magic number
        let magic = u32::from_le_bytes([self.data[0], self.data[1], self.data[2], self.data[3]]);
        if magic != PAGE_MAGIC {
            return;
        }

        // Page ID
        self.page_id = u32::from_le_bytes([self.data[4], self.data[5], self.data[6], self.data[7]]);

        let mut offset = 8;

        // Version (2 bytes) - skip
        offset += 2;

        // Page type (1 byte)
        self.page_type = match self.data[offset] {
            1 => PageType::Data,
            2 => PageType::Index,
            3 => PageType::Meta,
            _ => PageType::Free,
        };
        offset += 2; // page_type (1) + reserved (1)

        offset += 4; // skip checksum
        offset += 4; // skip prev page
        offset += 4; // skip next page

        // Row count
        self.row_count = u32::from_le_bytes([
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        ]);
        offset += 4;

        // Free space
        self.free_space = u32::from_le_bytes([
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        ]);
        offset += 4;

        // Table ID
        self.table_id = u64::from_le_bytes([
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
            self.data[offset + 4],
            self.data[offset + 5],
            self.data[offset + 6],
            self.data[offset + 7],
        ]);
    }

    /// Write a row to the page
    pub fn write_row(&mut self, values: &[Value]) -> bool {
        let serialized = row_to_bytes(values);
        let row_size = serialized.len() as u32;

        if row_size > self.free_space {
            return false;
        }

        let offset = PAGE_HEADER_SIZE + (PAGE_DATA_SIZE - self.free_space as usize);

        // Write row size (4 bytes) + data
        self.data[offset..offset + 4].copy_from_slice(&row_size.to_le_bytes());
        self.data[offset + 4..offset + 4 + serialized.len()].copy_from_slice(&serialized);

        self.row_count += 1;
        self.free_space -= row_size + 4;
        self.write_header();

        true
    }

    /// Read all rows from page
    pub fn read_rows(&self) -> Vec<Vec<Value>> {
        let mut rows = Vec::new();
        let mut offset = PAGE_HEADER_SIZE;

        while offset < PAGE_SIZE - 4 {
            let row_size = u32::from_le_bytes([
                self.data[offset],
                self.data[offset + 1],
                self.data[offset + 2],
                self.data[offset + 3],
            ]) as usize;

            if row_size == 0 || offset + 4 + row_size > PAGE_SIZE {
                break;
            }

            if let Some(row) = bytes_to_values(&self.data[offset + 4..offset + 4 + row_size]) {
                rows.push(row);
            }

            offset += 4 + row_size;
        }

        rows
    }

    /// Serialize page to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }

    /// Deserialize page from bytes
    pub fn from_bytes(data: Vec<u8>) -> Option<Self> {
        if data.len() != PAGE_SIZE {
            return None;
        }

        let mut page = Self {
            page_id: 0,
            data,
            page_type: PageType::Free,
            row_count: 0,
            free_space: PAGE_DATA_SIZE as u32,
            table_id: 0,
        };

        page.read_header();
        Some(page)
    }
}

/// Convert Value to binary format
pub fn value_to_bytes(value: &Value) -> Vec<u8> {
    match value {
        Value::Null => vec![0x00],
        Value::Integer(i) => {
            let mut bytes = vec![0x01];
            bytes.extend_from_slice(&i.to_le_bytes());
            bytes
        }
        Value::Float(f) => {
            let mut bytes = vec![0x02];
            bytes.extend_from_slice(&f.to_le_bytes());
            bytes
        }
        Value::Text(s) => {
            let mut bytes = vec![0x03];
            let len = (s.len() as u32).to_le_bytes();
            bytes.extend_from_slice(&len);
            bytes.extend_from_slice(s.as_bytes());
            bytes
        }
        Value::Boolean(b) => vec![if *b { 0x04 } else { 0x05 }],
        Value::Blob(blob) => {
            let mut bytes = vec![0x06];
            let len = (blob.len() as u32).to_le_bytes();
            bytes.extend_from_slice(&len);
            bytes.extend_from_slice(blob);
            bytes
        }
    }
}

/// Convert binary format to Value
pub fn bytes_to_value(data: &[u8]) -> Option<Value> {
    if data.is_empty() {
        return None;
    }

    match data[0] {
        0x00 => Some(Value::Null),
        0x01 if data.len() >= 9 => {
            let i = i64::from_le_bytes([
                data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            ]);
            Some(Value::Integer(i))
        }
        0x02 if data.len() >= 9 => {
            let f = f64::from_le_bytes([
                data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            ]);
            Some(Value::Float(f))
        }
        0x03 if data.len() >= 5 => {
            let len = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
            if data.len() >= 5 + len {
                Some(Value::Text(
                    String::from_utf8_lossy(&data[5..5 + len]).to_string(),
                ))
            } else {
                None
            }
        }
        0x04 => Some(Value::Boolean(true)),
        0x05 => Some(Value::Boolean(false)),
        0x06 if data.len() >= 5 => {
            let len = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
            if data.len() >= 5 + len {
                Some(Value::Blob(data[5..5 + len].to_vec()))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Convert row (Vec<Value>) to bytes
fn row_to_bytes(values: &[Value]) -> Vec<u8> {
    let mut result = Vec::new();
    for value in values {
        result.extend_from_slice(&value_to_bytes(value));
    }
    result
}

/// Convert bytes to row (Vec<Value>)
fn bytes_to_values(data: &[u8]) -> Option<Vec<Value>> {
    let mut values = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        if let Some(value) = bytes_to_value(&data[offset..]) {
            values.push(value);
            // Move offset based on value type
            match data[offset] {
                0x00 => offset += 1,
                0x01 => offset += 9,
                0x02 => offset += 9,
                0x03 | 0x06 if offset + 5 <= data.len() => {
                    let len = u32::from_le_bytes([
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                        data[offset + 4],
                    ]) as usize;
                    offset += 5 + len;
                }
                0x04 | 0x05 => offset += 1,
                _ => break,
            }
        } else {
            break;
        }
    }

    Some(values)
}

/// Binary page writer for efficient serialization
pub struct PageWriter<W: Write> {
    writer: W,
}

impl<W: Write> PageWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write_page(&mut self, page: &Page) -> std::io::Result<()> {
        self.writer.write_all(&page.to_bytes())
    }
}

/// Binary page reader for efficient deserialization
pub struct PageReader<R: Read> {
    reader: R,
}

impl<R: Read> PageReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn read_page(&mut self) -> std::io::Result<Page> {
        let mut data = vec![0u8; PAGE_SIZE];
        self.reader.read_exact(&mut data)?;
        Page::from_bytes(data).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid page data")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_creation() {
        let page = Page::new(1);
        assert_eq!(page.page_id(), 1);
    }

    #[test]
    fn test_page_data_access() {
        let mut page = Page::new_data(1, 100);
        page.data[PAGE_HEADER_SIZE] = 0xAB;
        page.data[PAGE_HEADER_SIZE + 1] = 0xCD;
        assert_eq!(page.data[PAGE_HEADER_SIZE], 0xAB);
    }

    #[test]
    fn test_value_to_bytes() {
        assert_eq!(value_to_bytes(&Value::Null), vec![0x00]);
        assert_eq!(
            value_to_bytes(&Value::Integer(42)),
            vec![0x01, 42, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(value_to_bytes(&Value::Boolean(true)), vec![0x04]);
        assert_eq!(value_to_bytes(&Value::Boolean(false)), vec![0x05]);
    }

    #[test]
    fn test_bytes_to_value() {
        assert_eq!(bytes_to_value(&[0x00]), Some(Value::Null));
        assert_eq!(bytes_to_value(&[0x04]), Some(Value::Boolean(true)));
        assert_eq!(bytes_to_value(&[0x05]), Some(Value::Boolean(false)));
    }

    #[test]
    fn test_page_write_read_row() {
        let mut page = Page::new_data(1, 100);
        let row = vec![Value::Integer(1), Value::Text("test".to_string())];

        assert!(page.write_row(&row));
        assert_eq!(page.row_count(), 1);

        let rows = page.read_rows();
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn test_page_to_from_bytes() {
        let mut page = Page::new_data(1, 100);
        page.write_row(&vec![Value::Integer(42)]);

        let bytes = page.to_bytes();
        assert_eq!(bytes.len(), PAGE_SIZE);

        let restored = Page::from_bytes(bytes).unwrap();
        assert_eq!(restored.page_id(), 1);
    }

    #[test]
    fn test_page_constants() {
        assert_eq!(PAGE_SIZE, 4096);
        assert_eq!(PAGE_HEADER_SIZE, 64);
        assert_eq!(PAGE_DATA_SIZE, 4032);
    }

    #[test]
    fn test_page_debug() {
        let page = Page::new(1);
        let debug = format!("{:?}", page);
        assert!(debug.contains("Page"));
    }

    #[test]
    fn test_page_size() {
        assert_eq!(Page::size(), PAGE_SIZE);
    }

    #[test]
    fn test_page_new_data() {
        let page = Page::new_data(1, 100);
        assert_eq!(page.page_id(), 1);
        assert_eq!(page.page_type(), PageType::Data);
        assert_eq!(page.row_count(), 0);
        assert_eq!(page.free_space(), PAGE_DATA_SIZE as u32);
    }

    #[test]
    fn test_page_write_read_row_full() {
        let mut page = Page::new_data(1, 100);

        let row1 = vec![Value::Integer(1), Value::Text("test1".to_string())];
        let row2 = vec![Value::Integer(2), Value::Text("test2".to_string())];

        assert!(page.write_row(&row1));
        assert!(page.write_row(&row2));
        assert_eq!(page.row_count(), 2);

        let rows = page.read_rows();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_page_free_space_after_write() {
        let mut page = Page::new_data(1, 100);

        let row = vec![Value::Integer(42)];
        page.write_row(&row);

        let free = page.free_space();
        assert!(free < PAGE_DATA_SIZE as u32);
    }

    #[test]
    fn test_page_clone() {
        let page = Page::new(1);
        let cloned = page.clone();
        assert_eq!(cloned.page_id(), 1);
    }

    #[test]
    fn test_value_to_bytes_integer() {
        let bytes = value_to_bytes(&Value::Integer(100));
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_value_to_bytes_text() {
        let bytes = value_to_bytes(&Value::Text("hello".to_string()));
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_bytes_to_value_integer() {
        let bytes = value_to_bytes(&Value::Integer(42));
        let value = bytes_to_value(&bytes);
        assert!(value.is_some());
    }

    #[test]
    fn test_bytes_to_value_text() {
        let bytes = value_to_bytes(&Value::Text("test".to_string()));
        let value = bytes_to_value(&bytes);
        assert!(value.is_some());
    }

    #[test]
    fn test_page_type_copy() {
        let pt = PageType::Data;
        let pt2 = pt;
        assert_eq!(pt, pt2);
    }
}
