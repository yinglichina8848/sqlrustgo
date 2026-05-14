//! GIS-specific error types

use thiserror::Error;

/// GIS-related errors
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum GisError {
    /// Invalid WKT format
    #[error("Invalid WKT format: {0}")]
    InvalidWkt(String),

    /// Invalid WKB format
    #[error("Invalid WKB format: {0}")]
    InvalidWkb(String),

    /// Unsupported geometry type
    #[error("Unsupported geometry type: {0}")]
    UnsupportedGeometryType(String),

    /// Parse error during WKT/WKB parsing
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Invalid coordinate value
    #[error("Invalid coordinate: {0}")]
    InvalidCoordinate(String),

    /// Empty geometry
    #[error("Empty geometry not allowed")]
    EmptyGeometry,

    /// Invalid number of coordinates
    #[error("Invalid number of coordinates: expected {expected}, got {actual}")]
    InvalidCoordinateCount { expected: String, actual: usize },

    /// Ring must have at least 4 points (3 + closing point)
    #[error("Ring must have at least 4 points")]
    RingTooShort,

    /// Polygon ring must be closed (first point equals last point)
    #[error("Polygon ring must be closed")]
    RingNotClosed,

    /// Multi-geometry cannot be empty
    #[error("Multi-geometry cannot be empty")]
    EmptyMultiGeometry,

    /// Geometry collection cannot be empty
    #[error("Geometry collection cannot be empty")]
    EmptyGeometryCollection,
}

impl GisError {
    /// Create an invalid WKT error with context
    pub fn invalid_wkt(msg: impl Into<String>) -> Self {
        GisError::InvalidWkt(msg.into())
    }

    /// Create an invalid WKB error with context
    pub fn invalid_wkb(msg: impl Into<String>) -> Self {
        GisError::InvalidWkb(msg.into())
    }

    /// Create a parse error
    pub fn parse_error(msg: impl Into<String>) -> Self {
        GisError::ParseError(msg.into())
    }
}
