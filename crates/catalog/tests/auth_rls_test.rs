use sqlrustgo_catalog::auth_rls::RowLevelSecurity;

#[test]
fn test_rls_predicate_generation() {
    let mut rls = RowLevelSecurity::new();
    rls.add_policy(1, "orders", "region = '华北'");

    let predicate = rls.get_predicate(1, "orders");
    assert_eq!(predicate, Some("region = '华北'".to_string()));
}

#[test]
fn test_rls_no_policy() {
    let rls = RowLevelSecurity::new();
    let predicate = rls.get_predicate(999, "orders");
    assert_eq!(predicate, None);
}

#[test]
fn test_rls_multiple_tables() {
    let mut rls = RowLevelSecurity::new();
    rls.add_policy(1, "orders", "region = '华北'");
    rls.add_policy(1, "products", "category = '电子产品'");

    assert_eq!(
        rls.get_predicate(1, "orders"),
        Some("region = '华北'".to_string())
    );
    assert_eq!(
        rls.get_predicate(1, "products"),
        Some("category = '电子产品'".to_string())
    );
}
