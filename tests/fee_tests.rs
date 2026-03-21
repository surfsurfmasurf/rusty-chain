use rusty_chain::core::chain::Chain;
use rusty_chain::core::types::Transaction;

#[test]
fn test_fee_rate_estimation_with_history() {
    let mut chain = Chain::new_genesis();

    // Fund TEST_SENDER via block rewards. Each reward is 50.
    for _ in 0..100 {
        chain.mine_block(vec![], 0, Some("TEST_SENDER")).unwrap();
    }

    // Create 5 blocks. Total funds = 5000.
    for i in 1..=5 {
        let mut txs = Vec::new();
        let next_nonce = chain.next_nonce_for("TEST_SENDER");
        let mut tx = Transaction::new(
            "TEST_SENDER".to_string(),
            format!("receiver_{i}"),
            10,
            next_nonce,
        );
        tx.fee = 50;
        txs.push(tx);

        chain.mine_block(txs, 0, None).unwrap();
    }

    let rate = chain.estimate_fee_rate(10);
    assert!(rate > 0.0, "Rate should be positive");
}

#[test]
fn test_fee_rate_estimation_empty_chain() {
    let chain = Chain::new_genesis();
    let rate = chain.estimate_fee_rate(10);
    assert_eq!(rate, 1.0, "Should return default 1.0 for empty chain");
}
