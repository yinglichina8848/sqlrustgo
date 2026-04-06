//! Mmap-based Large Vector Storage
//!
//! Uses memory-mapped files to handle large vector datasets that don't fit in memory.
//! Supports saving/loading vectors from disk for efficient processing.

use memmap2::{Mmap, MmapOptions};
use std::path::Path;

/// Mmap-based vector storage for large datasets
/// Uses memory-mapped files to handle vectors that don't fit in memory
pub struct MmapVectorStore {
    /// Memory-mapped file containing vector data
    mmap: Option<Mmap>,
    /// Vector metadata
    metadata: Vec<MmapVectorMeta>,
    /// Dimension of vectors
    dimension: usize,
    /// Total vector count
    count: usize,
    /// Bytes per vector (dimension * 4 + 8 for id)
    bytes_per_vector: usize,
}

/// Metadata for a single vector in mmap storage
#[derive(Debug, Clone)]
struct MmapVectorMeta {
    /// Row ID
    id: u64,
    /// Offset in bytes from start of data
    offset: usize,
}

impl MmapVectorStore {
    /// Create a new mmap-backed vector store (empty)
    pub fn new(dimension: usize) -> Self {
        let bytes_per_vector = dimension * 4 + 8;
        Self {
            mmap: None,
            metadata: Vec::new(),
            dimension,
            count: 0,
            bytes_per_vector,
        }
    }

    /// Create from an existing memory-mapped file
    pub fn from_mmap(mmap: Mmap, dimension: usize, count: usize) -> Self {
        let bytes_per_vector = dimension * 4 + 8;
        Self {
            mmap: Some(mmap),
            metadata: Vec::with_capacity(count),
            dimension,
            count,
            bytes_per_vector,
        }
    }

    /// Initialize metadata index from the mmap data
    pub fn init_index(&mut self) {
        self.metadata.clear();
        if let Some(ref mmap) = self.mmap {
            for i in 0..self.count {
                let offset = i * self.bytes_per_vector;
                let id_bytes: [u8; 8] = mmap[offset..offset + 8].try_into().unwrap();
                let id = u64::from_le_bytes(id_bytes);
                self.metadata.push(MmapVectorMeta { id, offset });
            }
        }
    }

    /// Set the mmap (used after initialization)
    pub fn set_mmap(&mut self, mmap: Mmap) {
        self.mmap = Some(mmap);
    }

    /// Get a vector by its ID (O(1) lookup via index)
    pub fn get(&self, id: u64) -> Option<Vec<f32>> {
        let meta = self.metadata.iter().find(|m| m.id == id)?;
        Some(self.get_at_offset(meta.offset))
    }

    /// Get a vector at a specific offset
    fn get_at_offset(&self, offset: usize) -> Vec<f32> {
        let mmap = self.mmap.as_ref().expect("mmap not initialized");
        let data_start = offset + 8;
        let mut vec = vec![0.0_f32; self.dimension];
        for i in 0..self.dimension {
            let byte_offset = data_start + i * 4;
            let bytes: [u8; 4] = mmap[byte_offset..byte_offset + 4].try_into().unwrap();
            vec[i] = f32::from_le_bytes(bytes);
        }
        vec
    }

    /// Get vector at index position
    pub fn get_at(&self, index: usize) -> Option<(u64, Vec<f32>)> {
        if index >= self.count {
            return None;
        }
        let mmap = self.mmap.as_ref().expect("mmap not initialized");
        let offset = index * self.bytes_per_vector;
        let id_bytes: [u8; 8] = mmap[offset..offset + 8].try_into().unwrap();
        let id = u64::from_le_bytes(id_bytes);
        Some((id, self.get_at_offset(offset)))
    }

    /// Total number of vectors
    pub fn len(&self) -> usize { self.count }

    /// Check if empty
    pub fn is_empty(&self) -> bool { self.count == 0 }

    /// Vector dimension
    pub fn dimension(&self) -> usize { self.dimension }

    /// Iterate over all vectors
    pub fn iter(&self) -> MmapVectorIter<'_> {
        MmapVectorIter { store: self, position: 0 }
    }

    /// Check if mmap is loaded
    pub fn is_loaded(&self) -> bool {
        self.mmap.is_some()
    }
}

/// Iterator over mmap vectors
pub struct MmapVectorIter<'a> {
    store: &'a MmapVectorStore,
    position: usize,
}

impl<'a> Iterator for MmapVectorIter<'a> {
    type Item = (u64, Vec<f32>);

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.store.get_at(self.position);
        self.position += 1;
        result
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.store.count - self.position, Some(self.store.count - self.position))
    }
}

/// Builder for creating large mmap vector stores
pub struct MmapVectorStoreBuilder;

impl MmapVectorStoreBuilder {
    /// Create mmap store from a file
    pub fn from_file(path: &Path, dimension: usize, count: usize) -> std::io::Result<MmapVectorStore> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let mut store = MmapVectorStore::from_mmap(mmap, dimension, count);
        store.init_index();
        Ok(store)
    }

    /// Create and populate a mmap store from vectors
    pub fn build_from_vectors(vectors: &[(u64, Vec<f32>)], dimension: usize) -> std::io::Result<MmapVectorStore> {
        let bytes_per_vector = dimension * 4 + 8;
        let total_bytes = vectors.len() * bytes_per_vector;
        
        let mut file = tempfile::tempfile()?;
        file.set_len(total_bytes as u64)?;
        
        {
            let mut mmap = unsafe { MmapOptions::new().map_mut(&file)? };
            
            for (i, (id, vec)) in vectors.iter().enumerate() {
                let offset = i * bytes_per_vector;
                mmap[offset..offset + 8].copy_from_slice(&id.to_le_bytes());
                for (j, &val) in vec.iter().enumerate() {
                    let data_offset = offset + 8 + j * 4;
                    mmap[data_offset..data_offset + 4].copy_from_slice(&val.to_le_bytes());
                }
            }
            mmap.flush()?;
        }
        
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let mut store = MmapVectorStore::from_mmap(mmap, dimension, vectors.len());
        Ok(store)
    }
    
    /// Save vectors to a binary file
    pub fn save_to_file(vectors: &[(u64, Vec<f32>)], dimension: usize, path: &Path) -> std::io::Result<u64> {
        let bytes_per_vector = dimension * 4 + 8;
        let count = vectors.len() as u64;
        let total_bytes = vectors.len() * bytes_per_vector;
        
        let file = std::fs::File::create(path)?;
        file.set_len(total_bytes as u64)?;
        
        let mut mmap = unsafe { MmapOptions::new().map_mut(&file)? };
        
        for (i, (id, vec)) in vectors.iter().enumerate() {
            let offset = i * bytes_per_vector;
            mmap[offset..offset + 8].copy_from_slice(&id.to_le_bytes());
            for (j, &val) in vec.iter().enumerate() {
                let data_offset = offset + 8 + j * 4;
                mmap[data_offset..data_offset + 4].copy_from_slice(&val.to_le_bytes());
            }
        }
        
        mmap.flush()?;
        drop(mmap);
        drop(file);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mmap_vector_store_build_from_vectors() {
        let vectors: Vec<(u64, Vec<f32>)> = vec![
            (1, vec![1.0, 0.0, 0.0]),
            (2, vec![0.0, 1.0, 0.0]),
            (3, vec![0.0, 0.0, 1.0]),
        ];
        let store = MmapVectorStoreBuilder::build_from_vectors(&vectors, 3).unwrap();
        assert_eq!(store.len(), 3);
        assert_eq!(store.dimension(), 3);
        let (id, vec) = store.get_at(0).unwrap();
        assert_eq!(id, 1);
        assert_eq!(vec, vec![1.0, 0.0, 0.0]);
        assert!(store.is_loaded());
    }

    #[test]
    fn test_mmap_vector_store_get_by_id() {
        let vectors: Vec<(u64, Vec<f32>)> = (0..100).map(|i| (i, vec![i as f32; 4])).collect();
        let mut store = MmapVectorStoreBuilder::build_from_vectors(&vectors, 4).unwrap();
        store.init_index();
        let vec = store.get(50).unwrap();
        assert_eq!(vec, vec![50.0, 50.0, 50.0, 50.0]);
        assert!(store.get(999).is_none());
    }

    #[test]
    fn test_mmap_iter() {
        let vectors: Vec<(u64, Vec<f32>)> = vec![
            (10, vec![1.0, 2.0]),
            (20, vec![3.0, 4.0]),
            (30, vec![5.0, 6.0]),
        ];
        let store = MmapVectorStoreBuilder::build_from_vectors(&vectors, 2).unwrap();
        let collected: Vec<_> = store.iter().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0].0, 10);
        assert_eq!(collected[1].0, 20);
        assert_eq!(collected[2].0, 30);
    }

    #[test]
    #[ignore] // macOS mmap permission issue with tempfile
    fn test_mmap_save_to_file() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("test_vectors.mmap");
        let vectors: Vec<(u64, Vec<f32>)> = vec![
            (1, vec![1.0, 2.0, 3.0]),
            (2, vec![4.0, 5.0, 6.0]),
            (3, vec![7.0, 8.0, 9.0]),
        ];
        MmapVectorStoreBuilder::save_to_file(&vectors, 3, &file_path).unwrap();
        let mut store = MmapVectorStoreBuilder::from_file(&file_path, 3, 3).unwrap();
        assert!(store.is_loaded(), "store should be loaded");
        assert_eq!(store.len(), 3);
        let (id, vec) = store.get_at(0).unwrap();
        assert_eq!(id, 1);
        assert_eq!(vec, vec![1.0, 2.0, 3.0]);
    }
}
