//! Binary Format Module
//!
//! This module provides binary serialization/deserialization interfaces
//! for future distributed storage (v2.0).

use crate::types::Value;

/// BinaryFormat trait - defines interface for binary serialization
///
/// # Why (为什么)
/// In distributed scenarios, data needs to be serialized to binary format
/// for efficient network transmission and storage.
///
/// # How (如何实现)
/// Implement this trait for your data types to enable binary serialization.
pub trait BinaryFormat: Sized {
    /// Serialize to binary bytes
    fn to_bytes(&self) -> Vec<u8>;

    /// Deserialize from binary bytes
    fn from_bytes(data: &[u8]) -> Result<Self, BinaryFormatError>;
}

/// Errors that can occur during binary format operations
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryFormatError {
    /// Not enough data in the buffer
    InsufficientData,
    /// Invalid data format
    InvalidFormat(String),
    /// Data too large
    DataTooLarge(usize),
    /// Unknown error
    Unknown(String),
}

impl std::fmt::Display for BinaryFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryFormatError::InsufficientData => write!(f, "Insufficient data in buffer"),
            BinaryFormatError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            BinaryFormatError::DataTooLarge(size) => write!(f, "Data too large: {} bytes", size),
            BinaryFormatError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for BinaryFormatError {}

/// Helper functions for binary serialization of common types
pub mod helpers {
    use super::*;

    /// Write a u64 to bytes in big-endian format
    pub fn write_u64(value: u64) -> [u8; 8] {
        value.to_be_bytes()
    }

    /// Read a u64 from bytes in big-endian format
    pub fn read_u64(data: &[u8]) -> Result<u64, BinaryFormatError> {
        if data.len() < 8 {
            return Err(BinaryFormatError::InsufficientData);
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&data[..8]);
        Ok(u64::from_be_bytes(bytes))
    }

    /// Write a i64 to bytes in big-endian format
    pub fn write_i64(value: i64) -> [u8; 8] {
        value.to_be_bytes()
    }

    /// Read a i64 from bytes in big-endian format
    pub fn read_i64(data: &[u8]) -> Result<i64, BinaryFormatError> {
        if data.len() < 8 {
            return Err(BinaryFormatError::InsufficientData);
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&data[..8]);
        Ok(i64::from_be_bytes(bytes))
    }

    /// Write a f64 to bytes in big-endian format
    pub fn write_f64(value: f64) -> [u8; 8] {
        value.to_be_bytes()
    }

    /// Read a f64 from bytes in big-endian format
    pub fn read_f64(data: &[u8]) -> Result<f64, BinaryFormatError> {
        if data.len() < 8 {
            return Err(BinaryFormatError::InsufficientData);
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&data[..8]);
        Ok(f64::from_be_bytes(bytes))
    }

    /// Write a string with length prefix
    pub fn write_string(value: &str) -> Vec<u8> {
        let len = value.len() as u64;
        let mut result = write_u64(len).to_vec();
        result.extend_from_slice(value.as_bytes());
        result
    }

    /// Read a string with length prefix
    pub fn read_string(data: &[u8]) -> Result<String, BinaryFormatError> {
        let len = read_u64(data)? as usize;
        if data.len() < 8 + len {
            return Err(BinaryFormatError::InsufficientData);
        }
        let str_data = &data[8..8 + len];
        String::from_utf8(str_data.to_vec())
            .map_err(|e| BinaryFormatError::InvalidFormat(e.to_string()))
    }

    /// Write a boolean as a single byte
    pub fn write_bool(value: bool) -> [u8; 1] {
        [if value { 1 } else { 0 }]
    }

    /// Read a boolean from a single byte
    pub fn read_bool(data: &[u8]) -> Result<bool, BinaryFormatError> {
        if data.is_empty() {
            return Err(BinaryFormatError::InsufficientData);
        }
        Ok(data[0] != 0)
    }
}

impl BinaryFormat for Value {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Value::Null => {
                let result = vec![0u8]; // type indicator
                result
            }
            Value::Integer(n) => {
                let mut result = vec![1u8]; // type indicator
                result.extend_from_slice(&helpers::write_i64(*n));
                result
            }
            Value::Float(f) => {
                let mut result = vec![2u8]; // type indicator
                result.extend_from_slice(&helpers::write_f64(*f));
                result
            }
            Value::Text(s) => {
                let mut result = vec![3u8]; // type indicator
                result.extend_from_slice(&helpers::write_string(s));
                result
            }
            Value::Boolean(b) => {
                let mut result = vec![4u8]; // type indicator
                result.extend_from_slice(&helpers::write_bool(*b));
                result
            }
            Value::Blob(b) => {
                let mut result = vec![5u8]; // type indicator
                                            // Write blob as length-prefixed bytes
                let len = b.len() as u64;
                result.extend_from_slice(&helpers::write_u64(len));
                result.extend_from_slice(b);
                result
            }
        }
    }

    fn from_bytes(data: &[u8]) -> Result<Self, BinaryFormatError> {
        if data.is_empty() {
            return Err(BinaryFormatError::InsufficientData);
        }
        match data[0] {
            0 => Ok(Value::Null),
            1 => Ok(Value::Integer(helpers::read_i64(&data[1..])?)),
            2 => Ok(Value::Float(helpers::read_f64(&data[1..])?)),
            3 => Ok(Value::Text(helpers::read_string(&data[1..])?)),
            4 => Ok(Value::Boolean(helpers::read_bool(&data[1..])?)),
            5 => {
                // Read blob: length (u64) + data
                let len = helpers::read_u64(&data[1..])? as usize;
                if data.len() < 1 + 8 + len {
                    return Err(BinaryFormatError::InsufficientData);
                }
                let blob_data = data[9..9 + len].to_vec();
                Ok(Value::Blob(blob_data))
            }
            _ => Err(BinaryFormatError::InvalidFormat(format!(
                "Unknown type indicator: {}",
                data[0]
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_bytes_integer() {
        let value = Value::Integer(42);
        let bytes = value.to_bytes();
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 1); // type indicator for Integer
    }

    #[test]
    fn test_value_to_bytes_float() {
        #[allow(clippy::approx_constant)]
        let value = Value::Float(3.14);
        let bytes = value.to_bytes();
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 2); // type indicator for Float
    }

    #[test]
    fn test_value_to_bytes_text() {
        let value = Value::Text("hello".to_string());
        let bytes = value.to_bytes();
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 3); // type indicator for Text
    }

    #[test]
    fn test_value_to_bytes_boolean() {
        let value = Value::Boolean(true);
        let bytes = value.to_bytes();
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 4); // type indicator for Boolean
    }

    #[test]
    fn test_value_to_bytes_null() {
        let value = Value::Null;
        let bytes = value.to_bytes();
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 0); // type indicator for Null
    }

    #[test]
    fn test_value_roundtrip_integer() {
        let original = Value::Integer(12345);
        let bytes = original.to_bytes();
        let restored = Value::from_bytes(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_value_roundtrip_float() {
        #[allow(clippy::approx_constant)]
        let original = Value::Float(2.71828);
        let bytes = original.to_bytes();
        let restored = Value::from_bytes(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_value_roundtrip_text() {
        let original = Value::Text("test string".to_string());
        let bytes = original.to_bytes();
        let restored = Value::from_bytes(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_value_roundtrip_boolean() {
        let original = Value::Boolean(false);
        let bytes = original.to_bytes();
        let restored = Value::from_bytes(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_value_roundtrip_null() {
        let original = Value::Null;
        let bytes = original.to_bytes();
        let restored = Value::from_bytes(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_helpers_write_read_u64() {
        let value: u64 = 1234567890;
        let bytes = helpers::write_u64(value);
        let restored = helpers::read_u64(&bytes).unwrap();
        assert_eq!(value, restored);
    }

    #[test]
    fn test_helpers_write_read_string() {
        let value = "Hello, World!";
        let bytes = helpers::write_string(value);
        let restored = helpers::read_string(&bytes).unwrap();
        assert_eq!(value, restored);
    }

    #[test]
    fn test_binary_format_error_display() {
        let err = BinaryFormatError::InsufficientData;
        assert_eq!(format!("{}", err), "Insufficient data in buffer");

        let err = BinaryFormatError::InvalidFormat("test".to_string());
        assert_eq!(format!("{}", err), "Invalid format: test");

        let err = BinaryFormatError::DataTooLarge(100);
        assert_eq!(format!("{}", err), "Data too large: 100 bytes");
    }
}
