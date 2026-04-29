use rusty_chain::core::types::Transaction;

#[test]
fn test_transaction_multi_layer_networking() {
    let mut tx = Transaction::new("A", "B", 100, 0);
    tx.cell_id = Some("cell-1".to_string());
    tx.area_id = Some("area-51".to_string());
    tx.fabric_id = Some("fabric-0".to_string());
    
    assert_eq!(tx.cell_id.as_deref(), Some("cell-1"));
    assert_eq!(tx.area_id.as_deref(), Some("area-51"));
    assert_eq!(tx.fabric_id.as_deref(), Some("fabric-0"));
    
    // Verify it passes basic validation
    assert!(tx.validate_basic().is_ok());
}

#[test]
fn test_transaction_cell_id_validation() {
    let mut tx = Transaction::new("A", "B", 100, 0);
    tx.cell_id = Some("  ".to_string());
    assert!(tx.validate_basic().is_err());
}
