use rusty_chain::core::types::Transaction;

#[test]
fn test_priority_flow_control_assignment() {
    let mut tx = Transaction::default();
    tx.priority_level_id = Some("high-priority".to_string());
    tx.flow_control_id = Some("throttled".to_string());
    tx.execution_tier_id = Some("tier-1".to_string());

    assert_eq!(tx.priority_level_id.unwrap(), "high-priority");
    assert_eq!(tx.flow_control_id.unwrap(), "throttled");
    assert_eq!(tx.execution_tier_id.unwrap(), "tier-1");
}

#[test]
fn test_priority_flow_control_validation() {
    let mut tx = Transaction::default();
    tx.from = "A".to_string();
    tx.to = "B".to_string();
    tx.amount = 10;
    
    // Empty priority_level_id should fail
    tx.priority_level_id = Some("  ".to_string());
    assert!(tx.validate_basic().is_err());
    
    tx.priority_level_id = Some("standard".to_string());
    assert!(tx.validate_basic().is_ok());

    // Empty flow_control_id should fail
    tx.flow_control_id = Some("".to_string());
    assert!(tx.validate_basic().is_err());

    tx.flow_control_id = Some("bypass".to_string());
    assert!(tx.validate_basic().is_ok());
}
