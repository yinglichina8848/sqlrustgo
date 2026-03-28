//! Columnar Storage - ColumnChunk Data Structure
//!
//! Core component for column-oriented storage in SQLRustGo.

use sqlrustgo_types::Value;
use std::cmp::Ordering;

/// Bitmap for null value tracking
#[derive(Debug, Clone)]
pub struct Bitmap {
    pub bits: Vec<u64>,
    pub len: usize,
}

impl Bitmap {
    /// Create a new empty bitmap
    pub fn new() -> Self {
        Self {
            bits: Vec::new(),
            len: 0,
        }
    }

    /// Create a bitmap with capacity for `capacity` elements
    pub fn with_capacity(capacity: usize) -> Self {
        let num_words = (capacity + 63) / 64;
        Self {
            bits: vec![0u64; num_words],
            len: capacity,
        }
    }

    /// Set the bit at index to 1 (indicating non-null)
    pub fn set(&mut self, index: usize) {
        if index >= self.len {
            return;
        }
        self.bits[index / 64] |= 1u64 << (index % 64);
    }

    /// Set the bit at index to 0 (indicating null)
    pub fn set_null(&mut self, index: usize) {
        if index >= self.len {
            return;
        }
        self.bits[index / 64] &= !(1u64 << (index % 64));
    }

    /// Check if the bit at index is set (non-null)
    pub fn is_set(&self, index: usize) -> bool {
        if index >= self.len {
            return false;
        }
        (self.bits[index / 64] & (1u64 << (index % 64))) != 0
    }

    /// Check if the bit at index is null
    pub fn is_null(&self, index: usize) -> bool {
        !self.is_set(index)
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Count the number of null bits
    pub fn null_count(&self) -> usize {
        let mut count = 0;
        for &word in &self.bits {
            count += word.count_zeros() as usize;
        }
        // Adjust for bits beyond len
        let extra_bits = self.bits.len() * 64 - self.len;
        count - extra_bits
    }

    /// Count the number of set bits (non-null)
    pub fn set_count(&self) -> usize {
        let mut count = 0;
        for &word in &self.bits {
            count += word.count_ones() as usize;
        }
        count
    }
}

impl Default for Bitmap {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a column chunk
#[derive(Debug, Clone)]
pub struct ColumnStats {
    pub null_count: usize,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
    pub distinct_count: Option<usize>,
}

impl ColumnStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self {
            null_count: 0,
            min_value: None,
            max_value: None,
            distinct_count: None,
        }
    }

    /// Update stats with a new value
    pub fn update(&mut self, value: &Value, is_null: bool) {
        if is_null {
            self.null_count += 1;
            return;
        }

        // Update min
        match &self.min_value {
            None => self.min_value = Some(value.clone()),
            Some(min) => {
                if value < min {
                    self.min_value = Some(value.clone());
                }
            }
        }

        // Update max
        match &self.max_value {
            None => self.max_value = Some(value.clone()),
            Some(max) => {
                if value > max {
                    self.max_value = Some(value.clone());
                }
            }
        }
    }
}

impl Default for ColumnStats {
    fn default() -> Self {
        Self::new()
    }
}

/// ColumnChunk - column-oriented storage for a single column
#[derive(Debug, Clone)]
pub struct ColumnChunk {
    data: Vec<Value>,
    null_bitmap: Option<Bitmap>,
    stats: ColumnStats,
}

impl ColumnChunk {
    /// Create a new empty ColumnChunk
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            null_bitmap: None,
            stats: ColumnStats::new(),
        }
    }

    /// Create a ColumnChunk with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            null_bitmap: Some(Bitmap::with_capacity(capacity)),
            stats: ColumnStats::new(),
        }
    }

    /// Push a non-null value to the chunk
    pub fn push(&mut self, value: Value) {
        self.push_value(value, false);
    }

    /// Push a null value to the chunk
    pub fn push_null(&mut self) {
        self.push_value(Value::Null, true);
    }

    /// Push a value with null flag
    fn push_value(&mut self, value: Value, is_null: bool) {
        let index = self.data.len();

        // Ensure null_bitmap exists (create even for non-nulls to track positions)
        if self.null_bitmap.is_none() {
            self.null_bitmap = Some(Bitmap::with_capacity(self.data.len()));
        }

        // Update bitmap if it exists
        if let Some(ref mut bitmap) = self.null_bitmap {
            // Extend bitmap if needed
            while bitmap.len <= index {
                bitmap.bits.push(0);
                bitmap.len += 1;
            }
            if is_null {
                bitmap.set_null(index);
            } else {
                bitmap.set(index);
            }
        }

        self.data.push(value.clone());
        self.stats.update(&value, is_null);
    }

    /// Set the value at index to null
    pub fn set_null(&mut self, index: usize) {
        if index >= self.data.len() {
            return;
        }

        if let Some(ref mut bitmap) = self.null_bitmap {
            bitmap.set_null(index);
        }

        self.stats.null_count += 1;
    }

    /// Check if value at index is null
    pub fn is_null(&self, index: usize) -> bool {
        if index >= self.data.len() {
            return true;
        }
        if let Some(ref bitmap) = self.null_bitmap {
            bitmap.is_null(index)
        } else {
            false
        }
    }

    /// Get the value at index
    pub fn get(&self, index: usize) -> Option<&Value> {
        if index >= self.data.len() {
            None
        } else {
            Some(&self.data[index])
        }
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the statistics
    pub fn stats(&self) -> &ColumnStats {
        &self.stats
    }

    /// Get the null bitmap
    pub fn null_bitmap(&self) -> Option<&Bitmap> {
        self.null_bitmap.as_ref()
    }

    /// Get all values (including nulls as Value::Null)
    pub fn values(&self) -> &[Value] {
        &self.data
    }

    /// Iterate over non-null values
    pub fn iter(&self) -> ColumnChunkIter {
        ColumnChunkIter {
            chunk: self,
            index: 0,
        }
    }
}

impl Default for ColumnChunk {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over ColumnChunk
#[derive(Debug)]
pub struct ColumnChunkIter<'a> {
    chunk: &'a ColumnChunk,
    index: usize,
}

impl<'a> Iterator for ColumnChunkIter<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.chunk.data.len() {
            let idx = self.index;
            self.index += 1;
            if !self.chunk.is_null(idx) {
                return Some(&self.chunk.data[idx]);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmap_new() {
        let bitmap = Bitmap::new();
        assert!(bitmap.is_empty());
        assert_eq!(bitmap.len(), 0);
    }

    #[test]
    fn test_bitmap_with_capacity() {
        let bitmap = Bitmap::with_capacity(100);
        assert_eq!(bitmap.len(), 100);
        assert!(!bitmap.is_set(0));
        assert!(!bitmap.is_set(50));
        assert!(!bitmap.is_set(99));
    }

    #[test]
    fn test_bitmap_set_and_check() {
        let mut bitmap = Bitmap::with_capacity(10);

        bitmap.set(0);
        bitmap.set(5);
        bitmap.set(9);

        assert!(bitmap.is_set(0));
        assert!(bitmap.is_set(5));
        assert!(bitmap.is_set(9));
        assert!(!bitmap.is_set(1));
        assert!(!bitmap.is_set(8));
    }

    #[test]
    fn test_bitmap_null() {
        let mut bitmap = Bitmap::with_capacity(10);
        bitmap.set(0);
        bitmap.set(5);

        bitmap.set_null(0);
        assert!(bitmap.is_null(0));
        assert!(!bitmap.is_null(5));
    }

    #[test]
    fn test_bitmap_null_count() {
        let mut bitmap = Bitmap::with_capacity(10);
        bitmap.set(0);
        bitmap.set(1);
        bitmap.set(2);
        // 7 nulls

        assert_eq!(bitmap.null_count(), 7);
        assert_eq!(bitmap.set_count(), 3);
    }

    #[test]
    fn test_column_chunk_new() {
        let chunk = ColumnChunk::new();
        assert!(chunk.is_empty());
        assert_eq!(chunk.len(), 0);
    }

    #[test]
    fn test_column_chunk_push() {
        let mut chunk = ColumnChunk::new();
        chunk.push(Value::Integer(42));
        chunk.push(Value::Integer(100));

        assert_eq!(chunk.len(), 2);
        assert!(!chunk.is_null(0));
        assert!(!chunk.is_null(1));
        assert_eq!(chunk.get(0), Some(&Value::Integer(42)));
    }

    #[test]
    fn test_column_chunk_push_null() {
        let mut chunk = ColumnChunk::new();
        chunk.push(Value::Integer(42));
        chunk.push_null();
        chunk.push(Value::Integer(100));

        assert_eq!(chunk.len(), 3);
        assert!(!chunk.is_null(0));
        assert!(chunk.is_null(1));
        assert!(!chunk.is_null(2));
    }

    #[test]
    fn test_column_chunk_stats() {
        let mut chunk = ColumnChunk::new();
        chunk.push(Value::Integer(10));
        chunk.push(Value::Integer(20));
        chunk.push(Value::Integer(30));
        chunk.push_null();

        let stats = chunk.stats();
        assert_eq!(stats.null_count, 1);
        assert_eq!(stats.min_value, Some(Value::Integer(10)));
        assert_eq!(stats.max_value, Some(Value::Integer(30)));
    }

    #[test]
    fn test_column_chunk_iter() {
        let mut chunk = ColumnChunk::new();
        chunk.push(Value::Integer(10));
        chunk.push_null();
        chunk.push(Value::Integer(20));

        let values: Vec<_> = chunk.iter().collect();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], &Value::Integer(10));
        assert_eq!(values[1], &Value::Integer(20));
    }

    #[test]
    fn test_column_chunk_multiple_types() {
        let mut chunk = ColumnChunk::new();
        chunk.push(Value::Integer(42));
        chunk.push(Value::Float(3.14));
        chunk.push(Value::Text("hello".to_string()));
        chunk.push(Value::Boolean(true));

        assert_eq!(chunk.len(), 4);
        assert_eq!(chunk.get(0), Some(&Value::Integer(42)));
        assert_eq!(chunk.get(1), Some(&Value::Float(3.14)));
        assert_eq!(chunk.get(2), Some(&Value::Text("hello".to_string())));
        assert_eq!(chunk.get(3), Some(&Value::Boolean(true)));
    }

    #[test]
    fn test_column_chunk_with_capacity() {
        let chunk = ColumnChunk::with_capacity(100);
        assert!(chunk.null_bitmap().is_some());
        assert_eq!(chunk.null_bitmap().unwrap().len(), 100);
    }

    #[test]
    fn test_column_chunk_set_null() {
        let mut chunk = ColumnChunk::new();
        chunk.push(Value::Integer(10));
        chunk.push(Value::Integer(20));

        chunk.set_null(0);
        assert!(chunk.is_null(0));
        assert!(!chunk.is_null(1));
    }

    #[test]
    fn test_stats_update() {
        let mut stats = ColumnStats::new();

        stats.update(&Value::Integer(10), false);
        stats.update(&Value::Integer(5), false);
        stats.update(&Value::Integer(20), false);
        stats.update(&Value::Null, true);

        assert_eq!(stats.null_count, 1);
        assert_eq!(stats.min_value, Some(Value::Integer(5)));
        assert_eq!(stats.max_value, Some(Value::Integer(20)));
    }

    #[test]
    fn test_stats_float() {
        let mut stats = ColumnStats::new();

        stats.update(&Value::Float(3.14), false);
        stats.update(&Value::Float(2.71), false);
        stats.update(&Value::Float(1.41), false);

        assert_eq!(stats.min_value, Some(Value::Float(1.41)));
        assert_eq!(stats.max_value, Some(Value::Float(3.14)));
    }

    #[test]
    fn test_stats_text() {
        let mut stats = ColumnStats::new();

        stats.update(&Value::Text("apple".to_string()), false);
        stats.update(&Value::Text("zebra".to_string()), false);
        stats.update(&Value::Text("banana".to_string()), false);

        assert_eq!(stats.min_value, Some(Value::Text("apple".to_string())));
        assert_eq!(stats.max_value, Some(Value::Text("zebra".to_string())));
    }
}
