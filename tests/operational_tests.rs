use rusty_chain::core::types::Transaction;

#[test]
fn test_operational_context_fields() {
    let mut tx = Transaction::default();
    tx.from = "A".to_string();
    tx.to = "B".to_string();
    tx.amount = 10;
    tx.context_id = Some("ctx-123".to_string());
    tx.operation_id = Some("op-456".to_string());
    tx.controller_ref = Some("ctrl-789".to_string());

    assert!(tx.validate_basic().is_ok());

    let payload = tx.signing_payload();
    assert_eq!(payload.context_id, Some("ctx-123".to_string()));
    assert_eq!(payload.operation_id, Some("op-456".to_string()));
    assert_eq!(payload.controller_ref, Some("ctrl-789".to_string()));
}

#[test]
fn test_operational_context_validation() {
    let mut tx = Transaction::default();
    tx.from = "A".to_string();
    tx.to = "B".to_string();
    tx.amount = 10;

    tx.context_id = Some("  ".to_string());
    assert!(tx.validate_basic().is_err());

    tx.context_id = Some("valid".to_string());
    tx.operation_id = Some("".to_string());
    assert!(tx.validate_basic().is_err());
}
