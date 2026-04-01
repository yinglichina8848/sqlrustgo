//! Uniform distribution generator

use crate::distribution::{RandomDistribution, Range};
use rand::rngs::SmallRng;
use rand::Rng;

/// Uniform distribution - equal probability for all values
pub struct UniformDistribution;

impl UniformDistribution {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UniformDistribution {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomDistribution for UniformDistribution {
    fn next_id(&self, rng: &mut SmallRng, range: Range<u64>) -> u64 {
        rng.gen_range(range)
    }
}
