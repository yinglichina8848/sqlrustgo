use crate::error::{SpillError, SpillResult};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use tempfile::TempDir;

pub struct PartitionManager {
    spill_dir: TempDir,
    partitions: Vec<PartitionFile>,
}

struct PartitionFile {
    path: PathBuf,
    size_bytes: u64,
}

impl PartitionManager {
    pub fn new() -> SpillResult<Self> {
        let spill_dir = tempfile::tempdir()
            .map_err(|e| SpillError::IoError(e))?;
        Ok(Self {
            spill_dir,
            partitions: Vec::new(),
        })
    }

    pub fn with_dir(path: PathBuf) -> SpillResult<Self> {
        fs::create_dir_all(&path)
            .map_err(|e| SpillError::IoError(e))?;
        let spill_dir = TempDir::new_in(&path)
            .map_err(|e| SpillError::IoError(e))?;
        Ok(Self {
            spill_dir,
            partitions: Vec::new(),
        })
    }

    pub fn write_partition<T: serde::Serialize>(
        &mut self,
        data: &[T],
    ) -> SpillResult<usize> {
        let partition_id = self.partitions.len();
        let path = self.spill_dir.path().join(format!("partition_{}.bin", partition_id));

        let file = File::create(&path)
            .map_err(|e| SpillError::IoError(e))?;
        let mut writer = BufWriter::new(file);

        for item in data {
            let bytes = bincode::serialize(item)
                .map_err(|e| SpillError::PartitionError(e.to_string()))?;
            writer.write_all(&bytes)
                .map_err(|e| SpillError::IoError(e))?;
        }

        writer.flush()
            .map_err(|e| SpillError::IoError(e))?;

        let size_bytes = fs::metadata(&path)
            .map_err(|e| SpillError::IoError(e))?
            .len();

        let partition = PartitionFile {
            path,
            size_bytes,
        };
        self.partitions.push(partition);

        Ok(partition_id)
    }

    pub fn read_partition<T: serde::de::DeserializeOwned>(
        &self,
        partition_id: usize,
    ) -> SpillResult<Vec<T>> {
        let partition = self.partitions.get(partition_id)
            .ok_or_else(|| SpillError::PartitionError(format!("Invalid partition {}", partition_id)))?;

        let bytes = fs::read(&partition.path)
            .map_err(|e| SpillError::IoError(e))?;

        let items: Vec<T> = bincode::deserialize(&bytes)
            .map_err(|e| SpillError::PartitionError(e.to_string()))?;

        Ok(items)
    }

    pub fn num_partitions(&self) -> usize {
        self.partitions.len()
    }

    pub fn total_bytes_spilled(&self) -> u64 {
        self.partitions.iter().map(|p| p.size_bytes).sum()
    }

    pub fn cleanup(&mut self) {
        self.partitions.clear();
    }
}

impl Default for PartitionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create tempdir")
    }
}

