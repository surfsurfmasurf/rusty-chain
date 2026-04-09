use rusty_chain::core::types::Transaction;

#[test]
fn test_transaction_tag_persistence() {
    let mut tx = Transaction::new("A", "B", 100, 0);
    tx.tag = Some("test-tag".to_string());

    let payload = tx.signing_payload();
    assert_eq!(payload.tag, Some("test-tag".to_string()));

    let json = serde_json::to_string(&tx).unwrap();
    assert!(json.contains("\"tag\":\"test-tag\""));

    let tx2: Transaction = serde_json::from_str(&json).unwrap();
    assert_eq!(tx2.tag, Some("test-tag".to_string()));
}

#[test]
fn test_transaction_tag_optional() {
    let tx = Transaction::new("A", "B", 100, 0);
    assert!(tx.tag.is_none());

    let json = serde_json::to_string(&tx).unwrap();
    assert!(!json.contains("\"tag\""));
}
