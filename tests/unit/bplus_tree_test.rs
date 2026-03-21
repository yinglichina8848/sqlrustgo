// B+Tree Index Tests
use sqlrustgo_storage::BPlusTree;

#[test]
fn test_bplus_tree_new() {
    let tree: BPlusTree = BPlusTree::new();
    assert!(tree.is_empty());
}

#[test]
fn test_bplus_tree_insert_and_get() {
    let mut tree: BPlusTree = BPlusTree::new();
    
    tree.insert(1, 100);
    tree.insert(2, 200);
    
    assert!(!tree.is_empty());
    
    let val = tree.search(1);
    assert_eq!(val, Some(100));
}

#[test]
fn test_bplus_tree_search_missing() {
    let tree: BPlusTree = BPlusTree::new();
    
    let result = tree.search(999);
    assert!(result.is_none());
}

#[test]
fn test_bplus_tree_remove() {
    let mut tree: BPlusTree = BPlusTree::new();
    
    tree.insert(1, 100);
    assert!(!tree.is_empty());
    
    tree.remove(1);
    assert!(tree.is_empty());
}

#[test]
fn test_bplus_tree_len() {
    let mut tree: BPlusTree = BPlusTree::new();
    
    assert_eq!(tree.len(), 0);
    
    tree.insert(1, 100);
    assert_eq!(tree.len(), 1);
    
    tree.insert(2, 200);
    assert_eq!(tree.len(), 2);
}

#[test]
fn test_bplus_tree_range_query() {
    let mut tree: BPlusTree = BPlusTree::new();
    
    tree.insert(1, 100);
    tree.insert(2, 200);
    tree.insert(3, 300);
    tree.insert(4, 400);
    tree.insert(5, 500);
    
    let results = tree.range_query(2, 4);
    assert_eq!(results.len(), 3);
}

#[test]
fn test_bplus_tree_keys() {
    let mut tree: BPlusTree = BPlusTree::new();
    
    tree.insert(3, 300);
    tree.insert(1, 100);
    tree.insert(2, 200);
    
    let keys = tree.keys();
    assert_eq!(keys, vec![1, 2, 3]);
}
