use rusty_chain::core::types::Transaction;

#[test]
fn test_workload_isolation_fields() {
    let mut tx = Transaction::new("A", "B", 100, 1);
    tx.workload_id = Some("compute-intensive".to_string());
    tx.stack_id = Some("v1.2.3".to_string());
    tx.isolate_id = Some("iso-456".to_string());
    
    assert!(tx.validate_basic().is_ok());
    assert_eq!(tx.workload_id.unwrap(), "compute-intensive");
}

#[test]
fn test_workload_validation_rejects_empty() {
    let mut tx = Transaction::new("A", "B", 100, 1);
    tx.workload_id = Some("  ".to_string());
    assert!(tx.validate_basic().is_err());
    
    tx.workload_id = Some("valid".to_string());
    tx.stack_id = Some("".to_string());
    assert!(tx.validate_basic().is_err());
}
