use rusty_chain::core::types::Transaction;

#[test]
fn test_resource_allocation_fields() {
    let mut tx = Transaction::new("A", "B", 100, 1);
    tx.quota_id = Some("high-priority".to_string());
    tx.budget_id = Some("q2-operations".to_string());
    tx.resource_pool_id = Some("pool-789".to_string());

    assert!(tx.validate_basic().is_ok());
    assert_eq!(tx.quota_id.unwrap(), "high-priority");
}

#[test]
fn test_resource_validation_rejects_empty() {
    let mut tx = Transaction::new("A", "B", 100, 1);
    tx.quota_id = Some("  ".to_string());
    assert!(tx.validate_basic().is_err());

    tx.quota_id = Some("valid".to_string());
    tx.budget_id = Some("".to_string());
    assert!(tx.validate_basic().is_err());
}
