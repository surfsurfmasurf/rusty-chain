use rusty_chain::core::mempool::Mempool;
use rusty_chain::core::types::Transaction;

#[test]
fn test_day75_telemetry_fields() {
    let mut tx = Transaction::new("A", "B", 100, 0);
    tx.stream_id_v2 = Some("stream-456".to_string());
    tx.event_correlation_id = Some("event-789".to_string());
    tx.telemetry_context_id = Some("context-abc".to_string());
    tx.lifecycle_stage = Some("processing".to_string());

    assert!(tx.validate_basic().is_ok());
    assert_eq!(tx.stream_id_v2.unwrap(), "stream-456");
    assert_eq!(tx.lifecycle_stage.unwrap(), "processing");
}

#[test]
fn test_day76_analytics_fields() {
    let mut tx = Transaction::new("A", "B", 100, 0);
    tx.analytics_id = Some("analytics-1".to_string());
    tx.report_id = Some("report-A".to_string());
    tx.metric_context_id = Some("metric-ctx".to_string());

    assert!(tx.validate_basic().is_ok());
    assert_eq!(tx.signing_payload().analytics_id, Some("analytics-1".to_string()));
}

#[test]
fn test_day75_validation_rejection() {
    let mut tx = Transaction::new("A", "B", 100, 0);

    // Empty stream_id_v2 should fail validation if present
    tx.stream_id_v2 = Some("  ".to_string());
    assert!(tx.validate_basic().is_err());

    tx.stream_id_v2 = Some("valid".to_string());
    tx.event_correlation_id = Some("".to_string());
    assert!(tx.validate_basic().is_err());
}

#[test]
fn test_day75_mempool_lookups() {
    let mut mempool = Mempool::new();
    let mut tx1 = Transaction::new("A", "B", 10, 0);
    tx1.lifecycle_stage = Some("stage-1".to_string());
    tx1.event_correlation_id = Some("corr-1".to_string());

    let mut tx2 = Transaction::new("B", "C", 20, 0);
    tx2.lifecycle_stage = Some("stage-1".to_string());
    tx2.event_correlation_id = Some("corr-2".to_string());

    mempool.add_tx(tx1).unwrap();
    mempool.add_tx(tx2).unwrap();

    let stage1_txs = mempool.get_txs_by_lifecycle_stage("stage-1");
    assert_eq!(stage1_txs.len(), 2);

    let corr1_txs = mempool.get_txs_by_event_correlation_id("corr-1");
    assert_eq!(corr1_txs.len(), 1);
}
