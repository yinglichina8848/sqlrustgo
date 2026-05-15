use crate::error::SpillResult;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct PartitionManager {
    partitions: HashMap<usize, PathBuf>,
    next_partition_id: usize,
}

impl PartitionManager {
    pub fn new() -> Self {
        Self {
            partitions: HashMap::new(),
            next_partition_id: 0,
        }
    }

    pub fn create_partition(&mut self) -> SpillResult<usize> {
        let id = self.next_partition_id;
        self.next_partition_id += 1;
        Ok(id)
    }

    pub fn num_partitions(&self) -> usize {
        self.partitions.len()
    }
}

impl Default for PartitionManager {
    fn default() -> Self {
        Self::new()
    }
}
