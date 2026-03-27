//! Data distribution implementations

pub mod uniform;
pub mod zipfian;

use rand::rngs::SmallRng;
use std::ops::Range;

/// Trait for random distribution generators
pub trait RandomDistribution: Send + Sync {
    /// Generate a random ID within the given range
    fn next_id(&self, rng: &mut SmallRng, range: Range<u64>) -> u64;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_uniform_distribution() {
        let uniform = uniform::UniformDistribution;
        let mut rng = SmallRng::seed_from_u64(42);

        // Test uniform distribution returns values within range
        let range = 100..200;
        for _ in 0..1000 {
            let id = uniform.next_id(&mut rng, range.clone());
            assert!(id >= range.start && id < range.end, "ID {} out of range", id);
        }
    }

    #[test]
    fn test_zipfian_distribution() {
        let num_items = 1000;
        let zipfian = zipfian::ZipfianDistribution::new(num_items, 0.9);
        let mut rng = SmallRng::seed_from_u64(42);

        let range = 0..num_items;

        // Test zipfian distribution - should have lower values (hot data) more frequently
        let mut low_count = 0;
        let mut high_count = 0;
        let mid_point = num_items / 2;

        for _ in 0..10000 {
            let id = zipfian.next_id(&mut rng, range.clone());
            assert!(id >= range.start && id < range.end, "ID {} out of range", id);

            if id < mid_point {
                low_count += 1;
            } else {
                high_count += 1;
            }
        }

        // Zipfian should generate more low-valued IDs (hot data)
        assert!(
            low_count > high_count,
            "Zipfian should favor lower IDs (hot data), got low={}, high={}",
            low_count,
            high_count
        );
    }
}
