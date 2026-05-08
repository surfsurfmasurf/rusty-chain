use rusty_chain::core::types::Transaction;

#[test]
fn test_resource_scheduling_field_assignment() {
    let mut tx = Transaction::default();
    tx.from = "A".to_string();
    tx.to = "B".to_string();
    tx.amount = 100;
    
    tx.compute_units_id = Some("high-cpu".to_string());
    tx.memory_limit_id = Some("4g".to_string());
    tx.storage_tier_id = Some("nvme".to_string());
    
    assert_eq!(tx.compute_units_id.unwrap(), "high-cpu");
    assert_eq!(tx.memory_limit_id.unwrap(), "4g");
    assert_eq!(tx.storage_tier_id.unwrap(), "nvme");
}

#[test]
fn test_resource_scheduling_validation() {
    let mut tx = Transaction::default();
    tx.from = "A".to_string();
    tx.to = "B".to_string();
    tx.amount = 10;
    
    // Empty compute_units_id should fail
    tx.compute_units_id = Some("  ".to_string());
    assert!(tx.validate_basic().is_err());
    
    tx.compute_units_id = Some("cpu-1".to_string());
    assert!(tx.validate_basic().is_ok());

    // Empty memory_limit_id should fail
    tx.memory_limit_id = Some("".to_string());
    assert!(tx.validate_basic().is_err());
    
    tx.memory_limit_id = Some("mem-high".to_string());
    assert!(tx.validate_basic().is_ok());
}
