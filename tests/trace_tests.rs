use rusty_chain::core::types::Transaction;

#[test]
fn test_transaction_trace_session_id() {
    let mut tx = Transaction::default();
    tx.from = "A".to_string();
    tx.to = "B".to_string();
    tx.amount = 100;
    tx.version = 1;

    // Default is None
    assert!(tx.trace_session_id.is_none());
    assert!(tx.validate_basic().is_ok());

    // Valid trace_session_id
    tx.trace_session_id = Some("trace-123".to_string());
    assert!(tx.validate_basic().is_ok());

    // Empty trace_session_id should fail
    tx.trace_session_id = Some(" ".to_string());
    assert!(tx.validate_basic().is_err());
}

#[test]
fn test_transaction_trace_session_signature_coverage() {
    let mut tx = Transaction::default();
    tx.trace_session_id = Some("trace-xyz".to_string());

    let payload = tx.signing_payload();
    assert_eq!(payload.trace_session_id, Some("trace-xyz".to_string()));
}
