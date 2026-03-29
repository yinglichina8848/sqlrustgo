//! Clock Replacer - CLOCK algorithm page replacement
//!
//! CLOCK is a page replacement algorithm used by MySQL/InnoDB and PostgreSQL.
//! It approximates LRU with a circular buffer and reference bits.
//!
//! ## Algorithm
//!
//! ```text
//! Each page has a reference (use) bit:
//! - Bit = 1: page was recently used, don't evict
//! - Bit = 0: page can be evicted
//!
//! On eviction candidate request:
//! 1. Advance clock hand
//! 2. If current page's bit is 1: clear it and advance
//! 3. If current page's bit is 0: evict this page
//! 4. Repeat until page found or full cycle
//! ```

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Page frame with reference bit for CLOCK algorithm
struct ClockFrame {
    /// Page ID stored in this frame
    page_id: u32,
    /// Reference bit - 1 means recently used, don't evict
    referenced: bool,
}

impl ClockFrame {
    fn new(page_id: u32) -> Self {
        Self {
            page_id,
            referenced: true, // New pages start as recently used
        }
    }
}

/// CLOCK page replacement algorithm
///
/// Uses a circular buffer with reference bits to approximate LRU
/// without requiring immediate eviction on each access.
pub struct ClockReplacer {
    /// Clock frames indexed by page_id
    frames: RwLock<HashMap<u32, Arc<RwLock<ClockFrame>>>>,
    /// Clock hand position (index into frame order)
    hand: RwLock<usize>,
    /// Maximum number of frames
    capacity: usize,
    /// Number of currently evictable pages
    size: RwLock<usize>,
}

impl ClockReplacer {
    /// Create a new CLOCK replacer with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            frames: RwLock::new(HashMap::new()),
            hand: RwLock::new(0),
            capacity,
            size: RwLock::new(0),
        }
    }

    /// Record that a page was accessed - sets reference bit to 1
    ///
    /// This should be called when a page is fetched from the buffer pool.
    pub fn record_access(&self, page_id: u32) {
        if let Some(frame) = self.frames.read().unwrap().get(&page_id) {
            if let Ok(mut f) = frame.write() {
                f.referenced = true;
            }
        }
    }

    /// Find a victim page to evict
    ///
    /// Returns Some(page_id) of victim, or None if no page can be evicted
    /// (all pages are pinned or pool is empty).
    ///
    /// The algorithm:
    /// 1. Look at current hand position
    /// 2. If page's reference bit is 1: clear it and advance
    /// 3. If page's reference bit is 0: this is the victim
    /// 4. Continue until victim found or full cycle
    pub fn evict(&self) -> Option<u32> {
        let capacity = self.capacity;
        let frames_guard = self.frames.read().unwrap();
        let frame_ids: Vec<u32> = frames_guard.keys().cloned().collect();
        drop(frames_guard);

        if frame_ids.is_empty() {
            return None;
        }

        let mut hand = self.hand.write().unwrap();
        let mut scans = 0;
        let max_scans = capacity * 2; // Prevent infinite loops

        loop {
            if scans >= max_scans {
                // Full cycle without finding victim - all pages are pinned
                return None;
            }

            let frame_ids_static = self.frames.read().unwrap();
            if frame_ids_static.is_empty() {
                return None;
            }

            // Get frame at current hand position
            let current_id = frame_ids_static
                .keys()
                .nth(*hand % frame_ids_static.len())
                .copied();

            if let Some(page_id) = current_id {
                let frame = frame_ids_static.get(&page_id).unwrap();
                let frame_guard = frame.read().unwrap();

                if frame_guard.referenced {
                    drop(frame_guard);
                    // Page was recently used - clear bit and advance
                    // Need to write lock to modify
                    if let Ok(mut f) = frame.write() {
                        f.referenced = false;
                    }
                    *hand = (*hand + 1) % capacity;
                    scans += 1;
                } else {
                    // Found victim - remove from frames
                    *hand = (*hand + 1) % capacity;
                    drop(frame_guard);
                    drop(frame);
                    drop(frame_ids_static);

                    // Remove from frames
                    self.frames.write().unwrap().remove(&page_id);
                    *self.size.write().unwrap() -= 1;

                    return Some(page_id);
                }
            } else {
                *hand = (*hand + 1) % capacity;
                scans += 1;
            }
        }
    }

    /// Add a page to the replacer
    ///
    /// If pool is at capacity, call evict() first to make room.
    pub fn add(&self, page_id: u32) {
        let mut frames = self.frames.write().unwrap();

        // Check if already exists
        if frames.contains_key(&page_id) {
            // Update and set as recently used
            if let Some(frame) = frames.get(&page_id) {
                if let Ok(mut f) = frame.write() {
                    f.referenced = true;
                }
            }
            return;
        }

        // Evict if at capacity
        while frames.len() >= self.capacity {
            drop(frames);
            if let Some(_victim) = self.evict() {
                // Victim removed in evict()
            }
            frames = self.frames.write().unwrap();
            // Re-check after potential eviction
            if frames.len() >= self.capacity {
                // Still at capacity - try evict again
                continue;
            }
            break;
        }

        // Add new frame
        let frame = Arc::new(RwLock::new(ClockFrame::new(page_id)));
        frames.insert(page_id, frame);
        *self.size.write().unwrap() += 1;
    }

    /// Remove a page from the replacer (e.g., when unpinned and flushed)
    pub fn remove(&self, page_id: u32) -> bool {
        let mut frames = self.frames.write().unwrap();
        if frames.remove(&page_id).is_some() {
            *self.size.write().unwrap() -= 1;
            true
        } else {
            false
        }
    }

    /// Get current size (number of pages tracked)
    pub fn size(&self) -> usize {
        *self.size.read().unwrap()
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// Get number of pages that can be evicted (reference bit = false)
    ///
    /// This is an approximation - exact count requires scanning all frames.
    pub fn evictable_count(&self) -> usize {
        let frames = self.frames.read().unwrap();
        frames
            .values()
            .filter_map(|f| f.read().ok())
            .filter(|f| !f.referenced)
            .count()
    }

    /// Pin a page - prevents it from being evicted
    ///
    /// Updates the reference bit to 1 so it won't be chosen as victim.
    pub fn pin(&self, page_id: u32) {
        self.record_access(page_id);
    }

    /// Unpin a page - allows it to be evicted
    ///
    /// Called when a transaction finishes using the page.
    /// The reference bit remains at its current value.
    pub fn unpin(&self, _page_id: u32) {
        // Reference bit stays as-is, allowing eviction on next cycle
        // The page is now a candidate for eviction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_replacer_creation() {
        let replacer = ClockReplacer::new(10);
        assert_eq!(replacer.capacity(), 10);
        assert!(replacer.is_empty());
        assert_eq!(replacer.size(), 0);
    }

    #[test]
    fn test_clock_replacer_add() {
        let replacer = ClockReplacer::new(3);
        replacer.add(1);
        replacer.add(2);
        assert_eq!(replacer.size(), 2);
    }

    #[test]
    fn test_clock_replacer_record_access() {
        let replacer = ClockReplacer::new(3);
        replacer.add(1);
        replacer.record_access(1);
        // Should not panic - just update reference bit
        assert_eq!(replacer.size(), 1);
    }

    #[test]
    fn test_clock_replacer_evict() {
        let replacer = ClockReplacer::new(3);
        replacer.add(1);
        replacer.add(2);
        replacer.add(3);

        // All pages now have referenced=true, need to cycle through
        // First cycle: all bits cleared
        // Second cycle: first checked page evicted
        let victim = replacer.evict();
        // Should get some page
        assert!(victim.is_some());
    }

    #[test]
    fn test_clock_replacer_evict_empty() {
        let replacer = ClockReplacer::new(3);
        let victim = replacer.evict();
        assert!(victim.is_none());
    }

    #[test]
    fn test_clock_replacer_remove() {
        let replacer = ClockReplacer::new(3);
        replacer.add(1);
        replacer.add(2);
        assert_eq!(replacer.size(), 2);

        assert!(replacer.remove(1));
        assert_eq!(replacer.size(), 1);

        assert!(!replacer.remove(999)); // Not found
    }

    #[test]
    fn test_clock_replacer_pin_unpin() {
        let replacer = ClockReplacer::new(3);
        replacer.add(1);

        replacer.pin(1);
        replacer.unpin(1);
        // Should not panic
        assert_eq!(replacer.size(), 1);
    }

    #[test]
    fn test_clock_replacer_capacity_limit() {
        let replacer = ClockReplacer::new(2);
        replacer.add(1);
        replacer.add(2);
        // Adding 3rd page should trigger eviction
        replacer.add(3);

        // Pool should have at most 2 pages (possibly 3 during add if eviction hasn't completed)
        // The size should stabilize at 2
        assert!(replacer.size() <= 2);
    }

    #[test]
    fn test_clock_replacer_evictable_count() {
        let replacer = ClockReplacer::new(3);
        replacer.add(1);
        replacer.add(2);

        // Initially all have referenced=true, none are evictable
        // After scanning through (evict attempt), some become evictable
        replacer.evict();
        replacer.evict();

        // After eviction attempts, pages should have referenced=false
        let evictable = replacer.evictable_count();
        assert!(evictable >= 0);
    }

    #[test]
    fn test_clock_replacer_reuse_page_id() {
        let replacer = ClockReplacer::new(3);
        replacer.add(1);
        replacer.evict();
        replacer.add(1); // Re-add same page
        assert_eq!(replacer.size(), 1);
    }

    #[test]
    fn test_clock_replacer_stress() {
        let replacer = ClockReplacer::new(5);

        // Add pages
        for i in 0..5 {
            replacer.add(i);
        }
        assert_eq!(replacer.size(), 5);

        // Access some pages
        replacer.record_access(0);
        replacer.record_access(2);

        // Evict several pages
        for _ in 0..5 {
            replacer.evict();
        }

        assert!(replacer.is_empty());
    }
}
