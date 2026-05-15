use crate::SpillResult;

pub trait SpillingIterator: Iterator {
    fn start_spill(&mut self) -> SpillResult<()>;

    fn is_spilling(&self) -> bool;

    fn num_partitions(&self) -> usize;

    fn finish_spill(&mut self);
}

#[derive(Debug, Clone, Default)]
pub struct SpillStats {
    pub spill_count: usize,
    pub bytes_spilled: u64,
    pub partitions_created: usize,
    pub fallback_attempts: usize,
}
