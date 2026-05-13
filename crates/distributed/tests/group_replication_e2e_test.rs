use sqlrustgo_distributed::{
    group_membership::{GroupMembership, MemberRole, MemberState},
    group_replication::{CertificationResult, GroupReplication, GroupReplicationConfig},
};
use std::collections::HashSet;

fn create_group(num_nodes: u64) -> GroupMembership {
    let mut gm = GroupMembership::new(1);

    for j in 2..=num_nodes {
        gm.add_member(j, format!("node-{}", j)).unwrap();
    }

    for j in 1..=num_nodes {
        gm.update_member_state(j, MemberState::Online).unwrap();
    }

    gm
}

#[test]
fn test_single_primary_write() {
    let mut gm = create_group(3);
    let _ = gm.set_member_role(1, MemberRole::Primary);
    let _ = gm.set_member_role(2, MemberRole::Secondary);
    let _ = gm.set_member_role(3, MemberRole::Secondary);

    let config = GroupReplicationConfig {
        single_primary: true,
        auto_rejoin: true,
        certification_window_size: 1000,
        max_memory_size: 128 * 1024 * 1024,
    };

    let mut replication = GroupReplication::new(gm, config);
    replication.set_primary(Some(1));

    assert!(replication.is_primary());
    assert!(replication.can_accept_writes());

    let mut write_set = HashSet::new();
    write_set.insert(b"key1".to_vec());

    let result = replication.replicate(write_set);
    assert_eq!(result, CertificationResult::Certified);
}

#[test]
fn test_multi_primary_concurrent_writes() {
    let mut gm = create_group(3);
    let _ = gm.set_member_role(1, MemberRole::Primary);
    let _ = gm.set_member_role(2, MemberRole::Primary);
    let _ = gm.set_member_role(3, MemberRole::Secondary);

    let config = GroupReplicationConfig {
        single_primary: false,
        auto_rejoin: true,
        certification_window_size: 1000,
        max_memory_size: 128 * 1024 * 1024,
    };

    let mut replication = GroupReplication::new(gm, config);

    assert!(replication.is_primary_member());
    assert!(replication.can_accept_writes());

    let mut write_set = HashSet::new();
    write_set.insert(b"key1".to_vec());

    let result = replication.replicate(write_set);
    assert_eq!(result, CertificationResult::Certified);

    assert_eq!(replication.certified_count(), 1);
}

#[test]
fn test_certification_prunes_old_entries() {
    let gm = create_group(1);

    let small_config = GroupReplicationConfig {
        single_primary: true,
        auto_rejoin: true,
        certification_window_size: 3,
        max_memory_size: 128 * 1024 * 1024,
    };

    let mut replication = GroupReplication::new(gm, small_config);

    for i in 0..5 {
        let mut write_set = HashSet::new();
        write_set.insert(format!("key{}", i).into_bytes());
        replication.replicate(write_set);
    }

    assert!(replication.window_size() <= 3);
}

#[test]
fn test_write_after_recovery_mode() {
    let gm = GroupMembership::new(1);
    let mut replication = GroupReplication::with_defaults(gm);
    replication.set_primary(Some(1));

    assert!(replication.can_accept_writes());

    replication.set_recovery_mode(true);
    assert!(!replication.can_accept_writes());

    replication.set_recovery_mode(false);
    assert!(replication.can_accept_writes());
}

#[test]
fn test_all_nodes_online_in_group() {
    let gm = create_group(5);

    assert_eq!(gm.get_online_members().len(), 5);
}