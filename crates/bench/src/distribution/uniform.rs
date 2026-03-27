//! Uniform distribution generator

/// Uniform distribution - equal probability for all values
pub struct Uniform;

impl Uniform {
    pub fn new() -> Self {
        Self
    }

    pub fn next(&mut self, _min: u64, _max: u64) -> u64 {
        // TODO: Implement uniform distribution
        0
    }
}
