//! Graph Engine Integration Tests
//!
//! Tests the complete Graph Engine with GMP traceability use case.

use sqlrustgo_graph::*;

#[test]
fn test_gmp_traceability_chain() {
    // Create a new graph store
    let mut store = InMemoryGraphStore::new();

    // GMP traceability: Batch -> Device -> SOP -> Step

    // 1. Create Batch node
    let mut batch_props = PropertyMap::new();
    batch_props.insert("id", "BATCH-2024-001");
    batch_props.insert("product", "Vaccine-A");
    batch_props.insert("quantity", 1000i64);
    let batch_id = store.create_node("Batch", batch_props);
    assert!(store.get_node(batch_id).is_some());

    // 2. Create Device node
    let mut device_props = PropertyMap::new();
    device_props.insert("id", "DEVICE-001");
    device_props.insert("model", "GMP-2000");
    device_props.insert("status", "operational");
    let device_id = store.create_node("Device", device_props);

    // 3. Create SOP node
    let mut sop_props = PropertyMap::new();
    sop_props.insert("id", "SOP-PROD-001");
    sop_props.insert("title", "Production Procedure A");
    sop_props.insert("version", "2.1");
    let sop_id = store.create_node("SOP", sop_props);

    // 4. Create Regulation node
    let mut reg_props = PropertyMap::new();
    reg_props.insert("id", "FDA-2024-001");
    reg_props.insert("title", "GMP Guidelines 2024");
    let regulation_id = store.create_node("Regulation", reg_props);

    // 5. Create edges for traceability
    store
        .create_edge(batch_id, device_id, "produced_by", PropertyMap::new())
        .unwrap();
    store
        .create_edge(device_id, sop_id, "follows", PropertyMap::new())
        .unwrap();
    store
        .create_edge(batch_id, regulation_id, "governed_by", PropertyMap::new())
        .unwrap();

    // Verify node counts
    assert_eq!(store.node_count(), 4);
    assert_eq!(store.edge_count(), 3);

    // Verify BFS traversal from batch
    let mut visited = Vec::new();
    store.bfs(batch_id, |node| {
        visited.push(node);
        true
    });
    assert!(visited.contains(&batch_id));
    assert!(visited.contains(&device_id));
    assert!(visited.contains(&sop_id));
    assert!(visited.contains(&regulation_id));

    // Verify neighbors by edge label
    let produced_by = store.neighbors_by_edge_label(batch_id, "produced_by");
    assert_eq!(produced_by.len(), 1);
    assert_eq!(produced_by[0], device_id);
}

#[test]
fn test_node_crud_operations() {
    let mut store = InMemoryGraphStore::new();

    // Create
    let node_id = store.create_node("Batch", PropertyMap::new());
    assert!(store.get_node(node_id).is_some());

    // Read
    let node = store.get_node(node_id).unwrap();
    assert_eq!(node.label, store.label_registry().get("Batch").unwrap());

    // Update
    let mut new_props = PropertyMap::new();
    new_props.insert("updated", true);
    store.update_node(node_id, new_props).unwrap();

    let updated_node = store.get_node(node_id).unwrap();
    assert!(updated_node.properties.get("updated").is_some());

    // Delete
    store.delete_node(node_id).unwrap();
    assert!(store.get_node(node_id).is_none());
}

#[test]
fn test_edge_crud_operations() {
    let mut store = InMemoryGraphStore::new();

    // Create nodes
    let node_a = store.create_node("Batch", PropertyMap::new());
    let node_b = store.create_node("Device", PropertyMap::new());

    // Create edge
    let edge_id = store
        .create_edge(node_a, node_b, "produced_by", PropertyMap::new())
        .unwrap();
    assert!(store.get_edge(edge_id).is_some());

    // Verify edge data
    let edge = store.get_edge(edge_id).unwrap();
    assert_eq!(edge.from, node_a);
    assert_eq!(edge.to, node_b);

    // Delete edge
    store.delete_edge(edge_id).unwrap();
    assert!(store.get_edge(edge_id).is_none());
}

#[test]
fn test_bfs_traversal() {
    let mut store = InMemoryGraphStore::new();

    // Create chain: 1 -> 2 -> 3 -> 4
    let n1 = store.create_node("Batch", PropertyMap::new());
    let n2 = store.create_node("Device", PropertyMap::new());
    let n3 = store.create_node("SOP", PropertyMap::new());
    let n4 = store.create_node("Step", PropertyMap::new());

    store
        .create_edge(n1, n2, "link", PropertyMap::new())
        .unwrap();
    store
        .create_edge(n2, n3, "link", PropertyMap::new())
        .unwrap();
    store
        .create_edge(n3, n4, "link", PropertyMap::new())
        .unwrap();

    // BFS from n1 should visit all nodes
    let mut visited = Vec::new();
    store.bfs(n1, |node| {
        visited.push(node);
        true
    });

    assert!(visited.contains(&n1));
    assert!(visited.contains(&n2));
    assert!(visited.contains(&n3));
    assert!(visited.contains(&n4));
}

#[test]
fn test_dfs_traversal() {
    let mut store = InMemoryGraphStore::new();

    // Create graph: 1 -> 2, 1 -> 3, 2 -> 4
    let n1 = store.create_node("Batch", PropertyMap::new());
    let n2 = store.create_node("Device", PropertyMap::new());
    let n3 = store.create_node("SOP", PropertyMap::new());
    let n4 = store.create_node("Step", PropertyMap::new());

    store
        .create_edge(n1, n2, "link", PropertyMap::new())
        .unwrap();
    store
        .create_edge(n1, n3, "link", PropertyMap::new())
        .unwrap();
    store
        .create_edge(n2, n4, "link", PropertyMap::new())
        .unwrap();

    // DFS from n1 should visit all nodes
    let mut visited = Vec::new();
    store.dfs(n1, |node| {
        visited.push(node);
        true
    });

    assert!(visited.contains(&n1));
    assert!(visited.contains(&n2));
    assert!(visited.contains(&n3));
    assert!(visited.contains(&n4));
}

#[test]
fn test_label_registry() {
    let store = InMemoryGraphStore::new();
    let registry = store.label_registry();

    // GMP labels should be pre-registered
    assert!(registry.contains("Batch"));
    assert!(registry.contains("Device"));
    assert!(registry.contains("SOP"));
    assert!(registry.contains("Regulation"));

    // Custom label should be registered on use
    let mut store2 = InMemoryGraphStore::new();
    store2.create_node("CustomLabel", PropertyMap::new());
    assert!(store2.label_registry().contains("CustomLabel"));
}

#[test]
fn test_nodes_by_label() {
    let mut store = InMemoryGraphStore::new();

    // Create multiple batches
    store.create_node("Batch", PropertyMap::new());
    store.create_node("Batch", PropertyMap::new());
    store.create_node("Batch", PropertyMap::new());
    store.create_node("Device", PropertyMap::new());

    let batches = store.nodes_by_label("Batch");
    assert_eq!(batches.len(), 3);

    let devices = store.nodes_by_label("Device");
    assert_eq!(devices.len(), 1);
}

#[test]
fn test_neighbors_operations() {
    let mut store = InMemoryGraphStore::new();

    // Create: A -> B, A -> C, D -> A
    let node_a = store.create_node("Batch", PropertyMap::new());
    let node_b = store.create_node("Device", PropertyMap::new());
    let node_c = store.create_node("SOP", PropertyMap::new());
    let node_d = store.create_node("Operator", PropertyMap::new());

    store
        .create_edge(node_a, node_b, "uses", PropertyMap::new())
        .unwrap();
    store
        .create_edge(node_a, node_c, "follows", PropertyMap::new())
        .unwrap();
    store
        .create_edge(node_d, node_a, "operates", PropertyMap::new())
        .unwrap();

    // Outgoing neighbors of A
    let outgoing = store.outgoing_neighbors(node_a);
    assert_eq!(outgoing.len(), 2);
    assert!(outgoing.contains(&node_b) || outgoing.contains(&node_c));

    // Incoming neighbors of A
    let incoming = store.incoming_neighbors(node_a);
    assert_eq!(incoming.len(), 1);
    assert_eq!(incoming[0], node_d);

    // Neighbors by edge label
    let uses_neighbors = store.neighbors_by_edge_label(node_a, "uses");
    assert_eq!(uses_neighbors.len(), 1);
    assert_eq!(uses_neighbors[0], node_b);
}

#[test]
fn test_error_handling() {
    let mut store = InMemoryGraphStore::new();

    // Create a node
    let node_id = store.create_node("Batch", PropertyMap::new());

    // Try to create edge with non-existent target
    let result = store.create_edge(node_id, NodeId(9999), "link", PropertyMap::new());
    assert!(result.is_err());

    // Try to delete non-existent edge
    let result = store.delete_edge(EdgeId(9999));
    assert!(result.is_err());

    // Try to delete non-existent node
    let result = store.delete_node(NodeId(9999));
    assert!(result.is_err());
}

#[test]
fn test_property_types() {
    let mut store = InMemoryGraphStore::new();

    let mut props = PropertyMap::new();
    props.insert("string_val", "hello");
    props.insert("int_val", 42i64);
    props.insert("float_val", 3.14f64);
    props.insert("bool_val", true);
    props.insert(
        "uuid_val",
        PropertyValue::Uuid("550e8400-e29b-41d4-a716-446655440000".to_string()),
    );

    let node_id = store.create_node("Batch", props);
    let node = store.get_node(node_id).unwrap();

    assert_eq!(
        node.properties.get("string_val").unwrap().as_string(),
        Some(&"hello".to_string())
    );
    assert_eq!(node.properties.get("int_val").unwrap().as_int(), Some(42));
    assert_eq!(
        node.properties.get("float_val").unwrap().as_float(),
        Some(3.14)
    );
    assert_eq!(
        node.properties.get("bool_val").unwrap().as_bool(),
        Some(true)
    );
}

#[test]
fn test_gmp_full_scenario() {
    // Simulate a full GMP traceability scenario

    let mut store = InMemoryGraphStore::new();

    // 1. Create Materials
    let material_a = {
        let mut props = PropertyMap::new();
        props.insert("id", "MAT-001");
        props.insert("name", "Active Ingredient A");
        props.insert("supplier", "ChemCorp");
        store.create_node("Material", props)
    };

    let material_b = {
        let mut props = PropertyMap::new();
        props.insert("id", "MAT-002");
        props.insert("name", "Excipient B");
        props.insert("supplier", "PharmaChem");
        store.create_node("Material", props)
    };

    // 2. Create Batch using Materials
    let batch = {
        let mut props = PropertyMap::new();
        props.insert("id", "BATCH-2024-001");
        props.insert("product", "Tablet-100mg");
        props.insert("quantity", 50000i64);
        props.insert("manufacture_date", "2024-01-15");
        store.create_node("Batch", props)
    };

    // 3. Create Device
    let device = {
        let mut props = PropertyMap::new();
        props.insert("id", "DEV-MILL-001");
        props.insert("type", "Tablet Milling Machine");
        props.insert("status", "validated");
        store.create_node("Device", props)
    };

    // 4. Create Operator
    let operator = {
        let mut props = PropertyMap::new();
        props.insert("id", "OP-001");
        props.insert("name", "John Smith");
        props.insert("certification", "GMP-2024");
        store.create_node("Operator", props)
    };

    // 5. Create QA Record
    let qa_record = {
        let mut props = PropertyMap::new();
        props.insert("id", "QA-2024-001");
        props.insert("result", "approved");
        props.insert("inspector", "Dr. Jane Doe");
        store.create_node("QA", props)
    };

    // 6. Create Regulation
    let regulation = {
        let mut props = PropertyMap::new();
        props.insert("id", "FDA-21CFR-Part11");
        props.insert("title", "Electronic Records");
        store.create_node("Regulation", props)
    };

    // 7. Create relationships
    // Batch uses Materials
    store
        .create_edge(batch, material_a, "uses_material", PropertyMap::new())
        .unwrap();
    store
        .create_edge(batch, material_b, "uses_material", PropertyMap::new())
        .unwrap();

    // Batch produced by Device
    store
        .create_edge(batch, device, "produced_by", PropertyMap::new())
        .unwrap();

    // Batch operated by Operator
    store
        .create_edge(batch, operator, "operated_by", PropertyMap::new())
        .unwrap();

    // Batch inspected by QA
    store
        .create_edge(batch, qa_record, "inspected_by", PropertyMap::new())
        .unwrap();

    // Batch governed by Regulation
    store
        .create_edge(batch, regulation, "governed_by", PropertyMap::new())
        .unwrap();

    // 8. Verify traceability - BFS from batch
    let mut traceability = Vec::new();
    store.bfs(batch, |node| {
        traceability.push(node);
        true
    });

    assert!(traceability.contains(&batch));
    assert!(traceability.contains(&material_a));
    assert!(traceability.contains(&material_b));
    assert!(traceability.contains(&device));
    assert!(traceability.contains(&operator));
    assert!(traceability.contains(&qa_record));
    assert!(traceability.contains(&regulation));

    // 9. Verify all nodes have correct types
    assert_eq!(store.nodes_by_label("Material").len(), 2);
    assert_eq!(store.nodes_by_label("Batch").len(), 1);
    assert_eq!(store.nodes_by_label("Device").len(), 1);
    assert_eq!(store.nodes_by_label("Operator").len(), 1);
    assert_eq!(store.nodes_by_label("QA").len(), 1);
    assert_eq!(store.nodes_by_label("Regulation").len(), 1);

    // 10. Verify edges
    let batch_node = store.get_node(batch).unwrap();
    let outgoing = store.outgoing_neighbors(batch);
    // Should have 5 outgoing edges (2 materials + device + operator + qa + regulation = 6, but we also have incoming... let me recount)
    // uses_material (2) + produced_by (1) + operated_by (1) + inspected_by (1) + governed_by (1) = 6
    assert_eq!(outgoing.len(), 6);
}
