//! Zipfian distribution generator

use crate::distribution::{RandomDistribution, Range};
use rand::distributions::Distribution;
use rand::rngs::SmallRng;
use zipf::ZipfDistribution;

/// Zipfian distribution - power-law distribution for hot data access
pub struct ZipfianDistribution {
    inner: ZipfDistribution,
    num_items: u64,
}

impl ZipfianDistribution {
    pub fn new(num_items: u64, theta: f64) -> Self {
        let inner = ZipfDistribution::new(num_items as usize, theta).unwrap_or_else(|_| {
            // Fallback to uniform if parameters are invalid
            ZipfDistribution::new(1, 0.0).expect("Failed to create ZipfDistribution")
        });
        Self {
            inner,
            num_items,
        }
    }
}

impl RandomDistribution for ZipfianDistribution {
    fn next_id(&self, rng: &mut SmallRng, range: Range<u64>) -> u64 {
        // Generate zipfian rank (0-based), then offset by range.start
        let rank = self.inner.sample(rng) as u64;
        let offset = rank.min(range.end - range.start - 1);
        range.start + offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_zipfian_creation() {
        let zipf = ZipfianDistribution::new(1000, 0.9);
        assert_eq!(zipf.num_items, 1000);
    }

    #[test]
    fn test_zipfian_in_range() {
        let zipf = ZipfianDistribution::new(100, 0.9);
        let mut rng = SmallRng::seed_from_u64(42);
        let range = 50..150;

        for _ in 0..1000 {
            let id = zipf.next_id(&mut rng, range.clone());
            assert!(id >= range.start && id < range.end, "ID {} out of range", id);
        }
    }
}
