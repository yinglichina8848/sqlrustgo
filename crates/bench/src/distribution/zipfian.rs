//! Zipfian distribution generator

/// Zipfian distribution - power-law distribution
pub struct Zipfian;

impl Zipfian {
    pub fn new() -> Self {
        Self
    }

    pub fn next(&mut self, _n: u64) -> u64 {
        // TODO: Implement zipfian distribution
        0
    }
}
