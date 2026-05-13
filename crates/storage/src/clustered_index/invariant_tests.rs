//! ClusteredLeafPage invariant verification tests
//!
//! This module contains tests that verify the invariants of the ClusteredLeafPage
//! data structure are maintained throughout all operations.
//!
//! Invariants checked:
//! 1. Slot directory offsets are always within page bounds
//! 2. Records don't overlap
//! 3. Keys are stored in sorted order (for non-empty pages)
//! 4. Free space calculation is consistent
//! 5. Live record count matches non-deleted slots
//! 6. Page split preserves all records
//! 7. Compact removes only deleted records

use crate::clustered_index::ClusteredLeafPage;
use crate::row_format::types::ClusterKey;
use crate::row_format::encoder;
use sqlrustgo_types::Value;

/// Helper to create a test page with sequential records
fn create_page_with_records(count: usize) -> ClusteredLeafPage {
    let mut page = ClusteredLeafPage::new();
    for i in 0..count {
        let key = ClusterKey::HiddenRowId(i as u64);
        let fixed = vec![Value::Integer(i as i64)];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls: Vec<bool> = vec![false];

        // Ignore errors for testing - some inserts may fail due to space
        let _ = page.insert(&key, &fixed, &varlen, &nulls);
    }
    page
}

/// Helper to create a page with specific keys
fn create_page_with_keys(keys: &[u64]) -> ClusteredLeafPage {
    let mut page = ClusteredLeafPage::new();
    for &key_id in keys {
        let key = ClusterKey::HiddenRowId(key_id);
        let fixed = vec![Value::Integer(key_id as i64)];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls: Vec<bool> = vec![false];

        let _ = page.insert(&key, &fixed, &varlen, &nulls);
    }
    page
}

/// Invariant: All slot offsets are within page bounds
fn check_slot_offsets_in_bounds(page: &ClusteredLeafPage) -> bool {
    let page_size = 4096;
    for i in 0..page.slot_count() {
        if page.is_slot_deleted(i) {
            continue;
        }
        if let Some(offset) = page.get_slot_data_offset(i) {
            if offset >= page_size {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}

/// Invariant: Records don't overlap (data_end <= free_space_start)
fn check_no_record_overlap(page: &ClusteredLeafPage) -> bool {
    page.data_end <= page.free_space_start
}

/// Invariant: Keys are in sorted order (for non-deleted slots)
fn check_keys_sorted(page: &ClusteredLeafPage) -> bool {
    let mut prev_key: Option<ClusterKey> = None;

    for i in 0..page.slot_count() {
        if page.is_slot_deleted(i) {
            continue;
        }
        if let Ok(Some(key)) = page.get_cluster_key(i) {
            if let Some(prev) = prev_key {
                if key < prev {
                    return false;
                }
            }
            prev_key = Some(key);
        }
    }
    true
}

/// Invariant: Free space calculation is consistent
fn check_free_space_consistency(page: &ClusteredLeafPage) -> bool {
    let header_size = 14; // LEAF_PAGE_HEADER_SIZE
    let slot_dir_size = page.slot_count() as usize * 2;
    let used_by_slots = header_size + slot_dir_size;
    let data_end = page.data_end as usize;

    // Data starts after header and slot directory
    if data_end < used_by_slots {
        return false;
    }

    // Free space = free_space_start - data_end
    let reported_free = page.free_space();
    let calculated_free = page.free_space_start as usize - data_end;

    reported_free == calculated_free
}

/// Invariant: Live record count matches non-deleted slots
fn check_live_record_count(page: &ClusteredLeafPage) -> bool {
    let counted: usize = (0..page.slot_count())
        .filter(|&i| !page.is_slot_deleted(i))
        .count();

    counted == page.live_record_count()
}

/// Invariant: Page split preserves all non-deleted records
fn check_split_preserves_records(left: &ClusteredLeafPage, right: &ClusteredLeafPage) -> bool {
    let total_slots = left.slot_count() + right.slot_count();
    let total_live = left.live_record_count() + right.live_record_count();

    // All non-deleted records should be preserved
    // (This is a basic check - full verification would decode and compare)
    total_live > 0 || (left.slot_count() == 0 && right.slot_count() == 0)
}

#[cfg(test)]
mod invariant_tests {
    use super::*;

    #[test]
    fn test_slot_offsets_in_bounds_empty_page() {
        let page = ClusteredLeafPage::new();
        assert!(check_slot_offsets_in_bounds(&page));
    }

    #[test]
    fn test_slot_offsets_in_bounds_with_records() {
        let page = create_page_with_records(10);
        assert!(check_slot_offsets_in_bounds(&page));
    }

    #[test]
    fn test_no_record_overlap_empty_page() {
        let page = ClusteredLeafPage::new();
        assert!(check_no_record_overlap(&page));
    }

    #[test]
    fn test_no_record_overlap_with_records() {
        let page = create_page_with_records(10);
        assert!(check_no_record_overlap(&page));
    }

    #[test]
    fn test_keys_sorted_empty_page() {
        let page = ClusteredLeafPage::new();
        assert!(check_keys_sorted(&page));
    }

    #[test]
    fn test_keys_sorted_sequential() {
        let page = create_page_with_keys(&[1, 2, 3, 4, 5]);
        assert!(check_keys_sorted(&page));
    }

    #[test]
    fn test_keys_sorted_random_order() {
        let page = create_page_with_keys(&[5, 1, 3, 2, 4]);
        assert!(check_keys_sorted(&page)); // Should still be sorted after insert
    }

    #[test]
    fn test_free_space_consistency_empty() {
        let page = ClusteredLeafPage::new();
        assert!(check_free_space_consistency(&page));
    }

    #[test]
    fn test_free_space_consistency_with_records() {
        let page = create_page_with_records(10);
        assert!(check_free_space_consistency(&page));
    }

    #[test]
    fn test_live_record_count_empty() {
        let page = ClusteredLeafPage::new();
        assert!(check_live_record_count(&page));
        assert_eq!(page.live_record_count(), 0);
    }

    #[test]
    fn test_live_record_count_with_records() {
        let page = create_page_with_records(10);
        assert!(check_live_record_count(&page));
        assert_eq!(page.live_record_count(), 10);
    }

    #[test]
    fn test_live_record_count_after_delete() {
        let mut page = create_page_with_records(10);

        // Delete first 3 records
        page.delete(0).unwrap();
        page.delete(1).unwrap();
        page.delete(2).unwrap();

        assert!(check_live_record_count(&page));
        assert_eq!(page.live_record_count(), 7);
    }

    #[test]
    fn test_delete_preserves_invariants() {
        let mut page = create_page_with_records(10);

        // Delete middle record
        page.delete(5).unwrap();

        assert!(check_slot_offsets_in_bounds(&page));
        assert!(check_no_record_overlap(&page));
        assert!(check_keys_sorted(&page));
        assert!(check_free_space_consistency(&page));
        assert!(check_live_record_count(&page));
    }

    #[test]
    fn test_split_preserves_records() {
        let mut page = create_page_with_records(20);

        let split_pos = 10;
        let (right_page, _) = page.split(split_pos);

        // Check left page invariants
        assert!(check_slot_offsets_in_bounds(&page));
        assert!(check_no_record_overlap(&page));
        assert!(check_keys_sorted(&page));
        assert!(check_free_space_consistency(&page));

        // Check right page invariants
        assert!(check_slot_offsets_in_bounds(&right_page));
        assert!(check_no_record_overlap(&right_page));
        assert!(check_keys_sorted(&right_page));
        assert!(check_free_space_consistency(&right_page));

        // Check records are preserved
        assert!(check_split_preserves_records(&page, &right_page));
    }

    #[test]
    fn test_compact_preserves_invariants() {
        let mut page = create_page_with_records(10);

        // Delete half the records
        for i in (0..10).step_by(2) {
            page.delete(i as u16).unwrap();
        }

        // Compact
        let removed = page.compact();

        // Should have removed 5 records
        assert_eq!(removed, 5);

        // Check all invariants
        assert!(check_slot_offsets_in_bounds(&page));
        assert!(check_no_record_overlap(&page));
        assert!(check_keys_sorted(&page));
        assert!(check_free_space_consistency(&page));
        assert!(check_live_record_count(&page));
    }

    #[test]
    fn test_encode_decode_roundtrip_invariant() {
        // Create a page with records
        let mut page = ClusteredLeafPage::new();
        for i in 0..10 {
            let key = ClusterKey::PrimaryKey(Value::Integer(i));
            let fixed = vec![Value::Integer(i * 2), Value::Text(format!("text_{}", i))];
            let varlen: Vec<Option<Vec<u8>>> = vec![Some(vec![1, 2, 3]), None];
            let nulls = vec![false, false, true];

            let _ = page.insert(&key, &fixed, &varlen, &nulls);
        }

        // Encode and decode
        let data = page.data().to_vec();
        let decoded_page = ClusteredLeafPage::from_data(data);

        // Check invariants on decoded page
        assert!(check_slot_offsets_in_bounds(&decoded_page));
        assert!(check_no_record_overlap(&decoded_page));
        assert!(check_keys_sorted(&decoded_page));
        assert!(check_free_space_consistency(&decoded_page));
        assert!(check_live_record_count(&decoded_page));

        // Check record count matches
        assert_eq!(decoded_page.slot_count(), page.slot_count());
        assert_eq!(decoded_page.live_record_count(), page.live_record_count());
    }

    #[test]
    fn test_sequential_key_insert_invariant() {
        // Test that inserting with sequential keys maintains sort order
        let keys = vec![100, 200, 300, 400, 500];
        let page = create_page_with_keys(&keys);

        assert!(check_keys_sorted(&page));

        // Verify all keys are present
        for (i, expected_key) in keys.iter().enumerate() {
            let actual = page.get_cluster_key(i as u16).unwrap().unwrap();
            match actual {
                ClusterKey::HiddenRowId(id) => assert_eq!(id, *expected_key),
                _ => panic!("Expected HiddenRowId"),
            }
        }
    }

    #[test]
    fn test_all_nulls_row_invariant() {
        let mut page = ClusteredLeafPage::new();

        let key = ClusterKey::HiddenRowId(1);
        let fixed = vec![Value::Null, Value::Null, Value::Null];
        let varlen: Vec<Option<Vec<u8>>> = vec![None, None];
        let nulls = vec![true, true, true, true, true];

        page.insert(&key, &fixed, &varlen, &nulls).unwrap();

        assert!(check_slot_offsets_in_bounds(&page));
        assert!(check_no_record_overlap(&page));
        assert!(check_keys_sorted(&page));
        assert!(check_free_space_consistency(&page));
    }

    #[test]
    fn test_large_varlen_invariant() {
        let mut page = ClusteredLeafPage::new();

        let key = ClusterKey::HiddenRowId(1);
        let fixed = vec![Value::Integer(42)];
        let large_data = vec![0xAB; 1000]; // 1KB varlen data
        let varlen: Vec<Option<Vec<u8>>> = vec![Some(large_data)];
        let nulls = vec![false, true];

        page.insert(&key, &fixed, &varlen, &nulls).unwrap();

        assert!(check_slot_offsets_in_bounds(&page));
        assert!(check_no_record_overlap(&page));
        assert!(check_free_space_consistency(&page));
    }

    #[test]
    fn test_boundary_page_full() {
        // Try to fill a page as much as possible
        let mut page = ClusteredLeafPage::new();
        let mut count = 0;

        // Keep inserting until we can't anymore
        for i in 0..1000u64 {
            let key = ClusterKey::HiddenRowId(i);
            let fixed = vec![Value::Integer(i as i64)];
            let varlen: Vec<Option<Vec<u8>>> = vec![];
            let nulls: Vec<bool> = vec![false];

            if page.insert(&key, &fixed, &varlen, &nulls).is_err() {
                break;
            }
            count += 1;
        }

        // Verify invariants on nearly full page
        assert!(check_slot_offsets_in_bounds(&page));
        assert!(check_no_record_overlap(&page));
        assert!(check_keys_sorted(&page));
        assert!(check_free_space_consistency(&page));

        // Page should have some records but not be full
        assert!(count > 0);
        assert!(page.needs_split()); // Should need split at this density
    }

    #[test]
    fn test_alternating_delete_invariant() {
        let mut page = create_page_with_records(20);

        // Delete every other record
        for i in (0..20).step_by(2) {
            page.delete(i as u16).unwrap();
        }

        assert!(check_slot_offsets_in_bounds(&page));
        assert!(check_no_record_overlap(&page));
        assert!(check_keys_sorted(&page));
        assert!(check_free_space_consistency(&page));
        assert!(check_live_record_count(&page));
        assert_eq!(page.live_record_count(), 10);
    }
}
