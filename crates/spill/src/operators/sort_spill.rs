use crate::error::SpillResult;
use crate::memory_tracker::AdaptiveMemoryTracker;
use crate::partition_manager::PartitionManager;
use crate::r#trait::SpillingIterator;
use std::cmp::Ordering;
use std::sync::Arc;

pub struct SortSpillOperator<T: Clone + serde::Serialize + serde::de::DeserializeOwned> {
    tracker: Arc<AdaptiveMemoryTracker>,
    partition_manager: PartitionManager,
    current_partition: Vec<T>,
    spilled_runs: Vec<usize>,
    comparator: fn(&T, &T) -> Ordering,
}

impl<T: Clone + serde::Serialize + serde::de::DeserializeOwned> SortSpillOperator<T> {
    pub fn new(
        tracker: Arc<AdaptiveMemoryTracker>,
        comparator: fn(&T, &T) -> Ordering,
    ) -> SpillResult<Self> {
        Ok(Self {
            tracker,
            partition_manager: PartitionManager::new()?,
            current_partition: Vec::new(),
            spilled_runs: Vec::new(),
            comparator,
        })
    }

    pub fn add(&mut self, item: T) -> SpillResult<()> {
        if self.tracker.should_spill() {
            self.start_spill()?;
        }

        let item_size = std::mem::size_of::<T>();
        if !self.tracker.allocate(item_size as u64) {
            self.start_spill()?;
            if !self.tracker.allocate(item_size as u64) {
                return Err(crate::error::SpillError::OutOfDiskSpace {
                    available: 0,
                    required: item_size as u64,
                });
            }
        }

        self.current_partition.push(item);
        Ok(())
    }

    pub fn calculate_partition_size(element_size: usize, available_memory: usize) -> usize {
        let rows_per_partition = available_memory / element_size;
        (rows_per_partition * 9) / 10
    }

    fn sort_current_partition(&mut self) -> SpillResult<()> {
        self.current_partition.sort_by(self.comparator);
        let partition_id = self
            .partition_manager
            .write_partition(&self.current_partition)?;
        self.spilled_runs.push(partition_id);

        let size = std::mem::size_of::<T>() as u64;
        self.tracker
            .deallocate(size * self.current_partition.len() as u64);
        self.current_partition.clear();
        Ok(())
    }
}

impl<T: Clone + serde::Serialize + serde::de::DeserializeOwned> Iterator for SortSpillOperator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_partition.pop()
    }
}

impl<T: Clone + serde::Serialize + serde::de::DeserializeOwned> SpillingIterator
    for SortSpillOperator<T>
{
    fn start_spill(&mut self) -> SpillResult<()> {
        if self.current_partition.is_empty() {
            return Ok(());
        }
        self.sort_current_partition()
    }

    fn is_spilling(&self) -> bool {
        !self.spilled_runs.is_empty()
    }

    fn num_partitions(&self) -> usize {
        self.spilled_runs.len()
    }

    fn finish_spill(&mut self) {
        self.current_partition.clear();
        self.spilled_runs.clear();
        self.partition_manager.cleanup();
    }
}
