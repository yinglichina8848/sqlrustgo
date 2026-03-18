//! Heap Storage - page-based table storage
//!
//! Heap storage organizes table data in pages, supporting:
//! - Row storage in data pages
//! - Page allocation and deallocation
//! - Sequential and random access
//! - Integration with buffer pool

use crate::page::{Page, PAGE_SIZE};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

/// Heap page header size
const HEAP_PAGE_HEADER_SIZE: usize = 48;
/// Heap magic number
#[allow(dead_code)]
const HEAP_MAGIC: u32 = 0x48454150; // "HEAP" in hex

/// Row identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RowId {
    /// Page number
    pub page_id: u32,
    /// Slot index within page
    pub slot: u16,
}

impl RowId {
    pub fn new(page_id: u32, slot: u16) -> Self {
        Self { page_id, slot }
    }
}

/// Heap table metadata
#[derive(Debug, Clone)]
pub struct HeapMeta {
    /// Table ID
    pub table_id: u64,
    /// Table name
    pub name: String,
    /// First data page
    pub first_page: u32,
    /// Last data page
    pub last_page: u32,
    /// Number of rows
    pub row_count: u64,
    /// Number of pages
    pub page_count: u32,
    /// Free pages list
    pub free_pages: Vec<u32>,
}

impl HeapMeta {
    pub fn new(table_id: u64, name: String) -> Self {
        Self {
            table_id,
            name,
            first_page: 0,
            last_page: 0,
            row_count: 0,
            page_count: 0,
            free_pages: Vec::new(),
        }
    }
}

/// Heap storage - page-based table storage
pub struct HeapStorage {
    /// Path to heap file
    file_path: PathBuf,
    /// Table metadata
    meta: HeapMeta,
}

impl HeapStorage {
    /// Create a new heap storage
    pub fn new(file_path: PathBuf, table_id: u64, table_name: String) -> std::io::Result<Self> {
        let meta = HeapMeta::new(table_id, table_name);

        // Create or open the file
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&file_path)?;

        drop(file);

        // Get initial page count
        let page_count = Self::get_page_count(&file_path);

        let mut heap = Self { file_path, meta };

        // Initialize with first page if empty
        if page_count == 0 {
            heap.allocate_page()?;
        } else {
            heap.meta.page_count = page_count;
            heap.meta.first_page = 1;
            heap.meta.last_page = page_count - 1;
        }

        Ok(heap)
    }

    /// Get page count
    fn get_page_count(file_path: &PathBuf) -> u32 {
        if let Ok(file) = File::open(file_path) {
            if let Ok(metadata) = file.metadata() {
                return (metadata.len() / PAGE_SIZE as u64) as u32;
            }
        }
        0
    }

    /// Read a page from disk
    fn read_page(&self, page_id: u32) -> std::io::Result<Vec<u8>> {
        let mut file = File::open(&self.file_path)?;
        let mut buffer = vec![0u8; PAGE_SIZE];
        file.seek(SeekFrom::Start((page_id as u64) * (PAGE_SIZE as u64)))?;
        file.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    /// Write a page to disk
    fn write_page(&self, page_id: u32, buffer: &[u8]) -> std::io::Result<()> {
        let mut file = OpenOptions::new().write(true).open(&self.file_path)?;
        file.seek(SeekFrom::Start((page_id as u64) * (PAGE_SIZE as u64)))?;
        file.write_all(buffer)?;
        file.flush()?;
        Ok(())
    }

    /// Allocate a new page
    fn allocate_page(&mut self) -> std::io::Result<u32> {
        let page_id = self.meta.page_count;
        let page = Page::new_data(page_id, self.meta.table_id);

        // Write page to disk
        self.write_page(page_id, &page.to_bytes())?;

        // Update metadata
        if self.meta.first_page == 0 {
            self.meta.first_page = page_id;
        }
        self.meta.last_page = page_id;
        self.meta.page_count += 1;

        Ok(page_id)
    }

    /// Insert a Row into the heap
    pub fn insert(&mut self, data: &[u8]) -> std::io::Result<RowId> {
        // Try to find a page with enough space
        for page_id in 0..self.meta.page_count {
            let mut buffer = self.read_page(page_id)?;

            // Check free space (offset 44-47 in page header)
            let free_space =
                u32::from_le_bytes([buffer[44], buffer[45], buffer[46], buffer[47]]) as usize;

            if free_space >= 4 + data.len() {
                // Get row count (offset 40-43)
                let row_count =
                    u32::from_le_bytes([buffer[40], buffer[41], buffer[42], buffer[43]]);
                let slot = row_count as u16;

                // Calculate data offset
                let data_offset = HEAP_PAGE_HEADER_SIZE + (slot as usize) * (4 + data.len());

                // Write size (4 bytes) + data
                buffer[data_offset..data_offset + 4]
                    .copy_from_slice(&(data.len() as u32).to_le_bytes());
                buffer[data_offset + 4..data_offset + 4 + data.len()].copy_from_slice(data);

                // Update row count
                buffer[40..44].copy_from_slice(&(row_count + 1).to_le_bytes());

                // Update free space
                let new_free_space = free_space - 4 - data.len();
                buffer[44..48].copy_from_slice(&(new_free_space as u32).to_le_bytes());

                // Write page back
                self.write_page(page_id, &buffer)?;

                self.meta.row_count += 1;
                return Ok(RowId::new(page_id, slot));
            }
        }

        // Need to allocate new page
        let new_page_id = self.allocate_page()?;
        let mut buffer = self.read_page(new_page_id)?;

        // Write first row at header end
        let data_offset = HEAP_PAGE_HEADER_SIZE;
        buffer[data_offset..data_offset + 4].copy_from_slice(&(data.len() as u32).to_le_bytes());
        buffer[data_offset + 4..data_offset + 4 + data.len()].copy_from_slice(data);

        // Update row count to 1
        buffer[40..44].copy_from_slice(&1u32.to_le_bytes());

        // Update free space
        let free_space = PAGE_SIZE - HEAP_PAGE_HEADER_SIZE - 4 - data.len();
        buffer[44..48].copy_from_slice(&(free_space as u32).to_le_bytes());

        // Write page
        self.write_page(new_page_id, &buffer)?;

        self.meta.row_count += 1;
        Ok(RowId::new(new_page_id, 0))
    }

    /// Get a Row by RowId
    pub fn get(&self, row_id: RowId) -> std::io::Result<Option<Vec<u8>>> {
        let buffer = self.read_page(row_id.page_id)?;

        // Get row count
        let row_count = u32::from_le_bytes([buffer[40], buffer[41], buffer[42], buffer[43]]);

        if row_id.slot as u32 >= row_count {
            return Ok(None);
        }

        // Calculate offset
        let mut offset = HEAP_PAGE_HEADER_SIZE;
        for _ in 0..row_id.slot {
            let size = u32::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]) as usize;
            offset += 4 + size;
        }

        // Read the Row
        let size = u32::from_le_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]) as usize;

        if size == 0 || offset + 4 + size > PAGE_SIZE {
            return Ok(None);
        }

        Ok(Some(buffer[offset + 4..offset + 4 + size].to_vec()))
    }

    /// Scan all rows
    pub fn scan(&self) -> std::io::Result<Vec<Vec<u8>>> {
        let mut rows = Vec::new();

        for page_id in 0..self.meta.page_count {
            let buffer = self.read_page(page_id)?;

            let row_count = u32::from_le_bytes([buffer[40], buffer[41], buffer[42], buffer[43]]);

            let mut offset = HEAP_PAGE_HEADER_SIZE;
            for _ in 0..row_count {
                let size = u32::from_le_bytes([
                    buffer[offset],
                    buffer[offset + 1],
                    buffer[offset + 2],
                    buffer[offset + 3],
                ]) as usize;

                if size > 0 && offset + 4 + size <= PAGE_SIZE {
                    rows.push(buffer[offset + 4..offset + 4 + size].to_vec());
                }
                offset += 4 + size;

                if offset >= PAGE_SIZE {
                    break;
                }
            }
        }

        Ok(rows)
    }

    /// Get row count
    pub fn row_count(&self) -> u64 {
        self.meta.row_count
    }

    /// Get page count
    pub fn page_count(&self) -> u32 {
        self.meta.page_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_heap_storage_creation() {
        let dir = tempdir().unwrap();
        let heap_path = dir.path().join("test_heap.dat");

        let heap = HeapStorage::new(heap_path, 1, "test".to_string()).unwrap();

        assert_eq!(heap.row_count(), 0);
        assert!(heap.page_count() >= 1);
    }

    #[test]
    fn test_heap_insert_and_get() {
        let dir = tempdir().unwrap();
        let heap_path = dir.path().join("test_heap.dat");

        let mut heap = HeapStorage::new(heap_path, 1, "test".to_string()).unwrap();

        let data = vec![1, 2, 3, 4];
        let row_id = heap.insert(&data).unwrap();

        assert_eq!(row_id.page_id, 1);
        assert_eq!(row_id.slot, 0);

        let retrieved = heap.get(row_id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), data);
    }

    #[test]
    fn test_heap_multiple_inserts() {
        let dir = tempdir().unwrap();
        let heap_path = dir.path().join("test_heap.dat");

        let mut heap = HeapStorage::new(heap_path, 1, "test".to_string()).unwrap();

        // Insert multiple rows
        for i in 0..10 {
            let data = vec![i as u8];
            heap.insert(&data).unwrap();
        }

        assert_eq!(heap.row_count(), 10);

        let rows = heap.scan().unwrap();
        assert_eq!(rows.len(), 10);
    }

    #[test]
    fn test_heap_scan() {
        let dir = tempdir().unwrap();
        let heap_path = dir.path().join("test_heap.dat");

        let mut heap = HeapStorage::new(heap_path, 1, "test".to_string()).unwrap();

        heap.insert(&vec![10, 20]).unwrap();
        heap.insert(&vec![30, 40]).unwrap();
        heap.insert(&vec![50, 60]).unwrap();

        let rows = heap.scan().unwrap();
        assert_eq!(rows.len(), 3);
    }
}
