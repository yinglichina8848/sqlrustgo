//! SQL data types for catalog definitions
//!
//! This module defines the data types that can be used in column definitions.
//! These are the logical types used in the catalog, separate from the
//! physical storage types in the Value enum.

use serde::{Deserialize, Serialize};
use std::fmt;

/// SQL data types supported by the database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DataType {
    /// NULL type
    Null,
    /// Boolean type (TRUE/FALSE)
    Boolean,
    /// 64-bit signed integer
    Integer,
    /// 64-bit floating point
    Float,
    /// Variable-length text
    #[default]
    Text,
    /// Binary large object
    Blob,
    /// Date type (days since epoch)
    Date,
    /// Timestamp type (microseconds since epoch)
    Timestamp,
    /// UUID type (128-bit unique identifier)
    Uuid,
    /// Array type (variable-length array of elements)
    Array,
    /// Enum type (enumeration with allowed values)
    Enum,
    /// Hidden rowid type (auto-incrementing integer for tables without explicit PK)
    /// This is an internal type, not exposed to users via CREATE TABLE
    RowId,
    /// Base geometry type (superclass for all geometry types)
    Geometry,
    /// Point geometry (single point in 2D space)
    Point,
    /// LineString geometry (sequence of points forming a line)
    LineString,
    /// Polygon geometry (closed ring with optional holes)
    Polygon,
    /// MultiPoint geometry (collection of points)
    MultiPoint,
    /// MultiLineString geometry (collection of line strings)
    MultiLineString,
    /// MultiPolygon geometry (collection of polygons)
    MultiPolygon,
    /// GeometryCollection (collection of arbitrary geometry types)
    GeometryCollection,
}

impl DataType {
    /// Get the SQL name for this data type
    pub fn sql_name(&self) -> &'static str {
        match self {
            DataType::Null => "NULL",
            DataType::Boolean => "BOOLEAN",
            DataType::Integer => "INTEGER",
            DataType::Float => "FLOAT",
            DataType::Text => "TEXT",
            DataType::Blob => "BLOB",
            DataType::Date => "DATE",
            DataType::Timestamp => "TIMESTAMP",
            DataType::Uuid => "UUID",
            DataType::Array => "ARRAY",
            DataType::Enum => "ENUM",
            DataType::RowId => "ROWID",
            DataType::Geometry => "GEOMETRY",
            DataType::Point => "POINT",
            DataType::LineString => "LINESTRING",
            DataType::Polygon => "POLYGON",
            DataType::MultiPoint => "MULTIPOINT",
            DataType::MultiLineString => "MULTILINESTRING",
            DataType::MultiPolygon => "MULTIPOLYGON",
            DataType::GeometryCollection => "GEOMETRYCOLLECTION",
        }
    }

    /// Parse a SQL type name into a DataType
    /// Note: ARRAY<T> and ENUM(...) are handled specially in the parser
    pub fn parse_sql_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "NULL" => Some(DataType::Null),
            "BOOLEAN" | "BOOL" => Some(DataType::Boolean),
            "INTEGER" | "INT" | "INT64" | "BIGINT" => Some(DataType::Integer),
            "FLOAT" | "DOUBLE" | "REAL" => Some(DataType::Float),
            "TEXT" | "VARCHAR" | "CHAR" | "STRING" => Some(DataType::Text),
            "BLOB" | "BINARY" | "VARBINARY" => Some(DataType::Blob),
            "DATE" => Some(DataType::Date),
            "TIMESTAMP" | "DATETIME" => Some(DataType::Timestamp),
            "UUID" => Some(DataType::Uuid),
            "ARRAY" => Some(DataType::Array),
            "ENUM" => Some(DataType::Enum),
            "GEOMETRY" => Some(DataType::Geometry),
            "POINT" => Some(DataType::Point),
            "LINESTRING" => Some(DataType::LineString),
            "POLYGON" => Some(DataType::Polygon),
            "MULTIPOINT" => Some(DataType::MultiPoint),
            "MULTILINESTRING" => Some(DataType::MultiLineString),
            "MULTIPOLYGON" => Some(DataType::MultiPolygon),
            "GEOMETRYCOLLECTION" => Some(DataType::GeometryCollection),
            _ => None,
        }
    }

    /// Check if this type can be used in a primary key
    pub fn is_valid_for_primary_key(&self) -> bool {
        matches!(
            self,
            DataType::Integer
                | DataType::Text
                | DataType::Boolean
                | DataType::Date
                | DataType::Uuid
                | DataType::RowId
                | DataType::Geometry
                | DataType::Point
                | DataType::LineString
                | DataType::Polygon
                | DataType::MultiPoint
                | DataType::MultiLineString
                | DataType::MultiPolygon
                | DataType::GeometryCollection
        )
    }

    /// Check if this type supports ordering comparisons
    pub fn is_orderable(&self) -> bool {
        !matches!(
            self,
            DataType::Blob
                | DataType::Array
                | DataType::Enum
                | DataType::Geometry
                | DataType::Point
                | DataType::LineString
                | DataType::Polygon
                | DataType::MultiPoint
                | DataType::MultiLineString
                | DataType::MultiPolygon
                | DataType::GeometryCollection
        )
    }

    /// Check if this type supports equality comparisons
    pub fn is_equatable(&self) -> bool {
        true
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.sql_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_sql_name() {
        assert_eq!(DataType::Integer.sql_name(), "INTEGER");
        assert_eq!(DataType::Text.sql_name(), "TEXT");
        assert_eq!(DataType::Boolean.sql_name(), "BOOLEAN");
    }

    #[test]
    fn test_parse_sql_name() {
        assert_eq!(DataType::parse_sql_name("INTEGER"), Some(DataType::Integer));
        assert_eq!(DataType::parse_sql_name("INT"), Some(DataType::Integer));
        assert_eq!(DataType::parse_sql_name("VARCHAR"), Some(DataType::Text));
        assert_eq!(DataType::parse_sql_name("BLOB"), Some(DataType::Blob));
        assert_eq!(DataType::parse_sql_name("UNKNOWN"), None);
    }

    #[test]
    fn test_valid_for_primary_key() {
        assert!(DataType::Integer.is_valid_for_primary_key());
        assert!(DataType::Text.is_valid_for_primary_key());
        assert!(!DataType::Blob.is_valid_for_primary_key());
        assert!(!DataType::Float.is_valid_for_primary_key());
    }

    #[test]
    fn test_is_orderable() {
        assert!(DataType::Integer.is_orderable());
        assert!(DataType::Text.is_orderable());
        assert!(!DataType::Blob.is_orderable());
    }

    #[test]
    fn test_default() {
        assert_eq!(DataType::default(), DataType::Text);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", DataType::Integer), "INTEGER");
        assert_eq!(format!("{}", DataType::Text), "TEXT");
    }

    #[test]
    fn test_data_type_sql_name_all() {
        assert_eq!(DataType::Null.sql_name(), "NULL");
        assert_eq!(DataType::Boolean.sql_name(), "BOOLEAN");
        assert_eq!(DataType::Integer.sql_name(), "INTEGER");
        assert_eq!(DataType::Float.sql_name(), "FLOAT");
        assert_eq!(DataType::Text.sql_name(), "TEXT");
        assert_eq!(DataType::Blob.sql_name(), "BLOB");
        assert_eq!(DataType::Date.sql_name(), "DATE");
        assert_eq!(DataType::Timestamp.sql_name(), "TIMESTAMP");
        assert_eq!(DataType::Uuid.sql_name(), "UUID");
        assert_eq!(DataType::Array.sql_name(), "ARRAY");
        assert_eq!(DataType::Enum.sql_name(), "ENUM");
        assert_eq!(DataType::Geometry.sql_name(), "GEOMETRY");
        assert_eq!(DataType::Point.sql_name(), "POINT");
        assert_eq!(DataType::LineString.sql_name(), "LINESTRING");
        assert_eq!(DataType::Polygon.sql_name(), "POLYGON");
        assert_eq!(DataType::MultiPoint.sql_name(), "MULTIPOINT");
        assert_eq!(DataType::MultiLineString.sql_name(), "MULTILINESTRING");
        assert_eq!(DataType::MultiPolygon.sql_name(), "MULTIPOLYGON");
        assert_eq!(
            DataType::GeometryCollection.sql_name(),
            "GEOMETRYCOLLECTION"
        );
    }

    #[test]
    fn test_parse_sql_name_all() {
        assert_eq!(DataType::parse_sql_name("NULL"), Some(DataType::Null));
        assert_eq!(DataType::parse_sql_name("BOOLEAN"), Some(DataType::Boolean));
        assert_eq!(DataType::parse_sql_name("BOOL"), Some(DataType::Boolean));
        assert_eq!(DataType::parse_sql_name("INT64"), Some(DataType::Integer));
        assert_eq!(DataType::parse_sql_name("BIGINT"), Some(DataType::Integer));
        assert_eq!(DataType::parse_sql_name("DOUBLE"), Some(DataType::Float));
        assert_eq!(DataType::parse_sql_name("REAL"), Some(DataType::Float));
        assert_eq!(DataType::parse_sql_name("CHAR"), Some(DataType::Text));
        assert_eq!(DataType::parse_sql_name("STRING"), Some(DataType::Text));
        assert_eq!(DataType::parse_sql_name("BINARY"), Some(DataType::Blob));
        assert_eq!(DataType::parse_sql_name("VARBINARY"), Some(DataType::Blob));
        assert_eq!(
            DataType::parse_sql_name("DATETIME"),
            Some(DataType::Timestamp)
        );
        assert_eq!(DataType::parse_sql_name("UUID"), Some(DataType::Uuid));
        assert_eq!(DataType::parse_sql_name("ARRAY"), Some(DataType::Array));
        assert_eq!(DataType::parse_sql_name("ENUM"), Some(DataType::Enum));
        assert_eq!(
            DataType::parse_sql_name("GEOMETRY"),
            Some(DataType::Geometry)
        );
        assert_eq!(DataType::parse_sql_name("POINT"), Some(DataType::Point));
        assert_eq!(
            DataType::parse_sql_name("LINESTRING"),
            Some(DataType::LineString)
        );
        assert_eq!(DataType::parse_sql_name("POLYGON"), Some(DataType::Polygon));
        assert_eq!(
            DataType::parse_sql_name("MULTIPOINT"),
            Some(DataType::MultiPoint)
        );
        assert_eq!(
            DataType::parse_sql_name("MULTILINESTRING"),
            Some(DataType::MultiLineString)
        );
        assert_eq!(
            DataType::parse_sql_name("MULTIPOLYGON"),
            Some(DataType::MultiPolygon)
        );
        assert_eq!(
            DataType::parse_sql_name("GEOMETRYCOLLECTION"),
            Some(DataType::GeometryCollection)
        );
        assert_eq!(DataType::parse_sql_name("INVALID"), None);
    }

    #[test]
    fn test_valid_for_primary_key_all() {
        assert!(DataType::Boolean.is_valid_for_primary_key());
        assert!(DataType::Date.is_valid_for_primary_key());
        assert!(DataType::Uuid.is_valid_for_primary_key());
        assert!(!DataType::Float.is_valid_for_primary_key());
        assert!(!DataType::Timestamp.is_valid_for_primary_key());
        assert!(!DataType::Array.is_valid_for_primary_key());
        assert!(!DataType::Enum.is_valid_for_primary_key());
    }

    #[test]
    fn test_is_orderable_all() {
        assert!(DataType::Null.is_orderable());
        assert!(DataType::Boolean.is_orderable());
        assert!(DataType::Integer.is_orderable());
        assert!(DataType::Float.is_orderable());
        assert!(DataType::Text.is_orderable());
        assert!(DataType::Date.is_orderable());
        assert!(DataType::Timestamp.is_orderable());
        assert!(DataType::Uuid.is_orderable());
        assert!(!DataType::Blob.is_orderable());
        assert!(!DataType::Array.is_orderable());
        assert!(!DataType::Enum.is_orderable());
        assert!(!DataType::Geometry.is_orderable());
        assert!(!DataType::Point.is_orderable());
        assert!(!DataType::LineString.is_orderable());
        assert!(!DataType::Polygon.is_orderable());
        assert!(!DataType::MultiPoint.is_orderable());
        assert!(!DataType::MultiLineString.is_orderable());
        assert!(!DataType::MultiPolygon.is_orderable());
        assert!(!DataType::GeometryCollection.is_orderable());
    }

    #[test]
    fn test_is_equatable_all() {
        for dt in [
            DataType::Null,
            DataType::Boolean,
            DataType::Integer,
            DataType::Float,
            DataType::Text,
            DataType::Blob,
            DataType::Date,
            DataType::Timestamp,
            DataType::Uuid,
            DataType::Array,
            DataType::Enum,
            DataType::Geometry,
            DataType::Point,
            DataType::LineString,
            DataType::Polygon,
            DataType::MultiPoint,
            DataType::MultiLineString,
            DataType::MultiPolygon,
            DataType::GeometryCollection,
        ] {
            assert!(dt.is_equatable(), "DataType {:?} should be equatable", dt);
        }
    }

    #[test]
    fn test_data_type_clone() {
        let dt = DataType::Integer;
        let cloned = dt;
        assert_eq!(dt, cloned);
    }

    #[test]
    fn test_data_type_copy() {
        let dt = DataType::Text;
        let copied = dt;
        assert_eq!(dt, copied);
    }

    #[test]
    fn test_data_type_debug() {
        let dt = DataType::Integer;
        let debug = format!("{:?}", dt);
        assert!(debug.contains("Integer"));
    }

    #[test]
    fn test_data_type_partial_eq_all() {
        let types = [
            DataType::Null,
            DataType::Boolean,
            DataType::Integer,
            DataType::Float,
            DataType::Text,
            DataType::Blob,
            DataType::Date,
            DataType::Timestamp,
            DataType::Uuid,
            DataType::Array,
            DataType::Enum,
        ];
        for dt in &types {
            assert_eq!(*dt, *dt);
        }
        assert_ne!(DataType::Integer, DataType::Float);
    }
}
