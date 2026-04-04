use rusty_chain::core::chain::Chain;
use rusty_chain::core::mempool::Mempool;
use rusty_chain::core::types::Transaction;

#[test]
fn test_transaction_expiry_enforcement() {
    let mut chain = Chain::new_genesis();

    // Give ALICE some coins so balance check passes
    let alice_addr = "ALICE";
    chain.mine_block(vec![], 3, Some(alice_addr)).unwrap();

    let state = chain.compute_state().unwrap();
    assert!(state.get_balance(alice_addr) >= 10);

    // Create a transaction that expires at height 1 (current height is 1)
    let mut tx = Transaction::new(alice_addr, "BOB", 10, 0);
    tx.expiry = Some(1);

    // Should fail validation for height 2
    let res = state.validate_transaction(&tx, 2);
    assert!(res.is_err());
    let err_msg = format!("{:?}", res.unwrap_err());
    assert!(err_msg.contains("expired"));

    // Should pass validation for height 1
    state
        .validate_transaction(&tx, 1)
        .expect("valid at height 1");

    // Create a transaction that expires at height 5
    tx.expiry = Some(5);
    state
        .validate_transaction(&tx, 5)
        .expect("valid at height 5");
    let res2 = state.validate_transaction(&tx, 6);
    assert!(res2.is_err());
}

#[test]
fn test_mempool_evict_expired_transactions() {
    let mut mempool = Mempool::new();
    let mut tx1 = Transaction::new("A", "B", 10, 0);
    tx1.timestamp_ms = 1000;

    let mut tx2 = Transaction::new("C", "D", 20, 0);
    tx2.timestamp_ms = 2000;

    mempool.add_tx(tx1).unwrap();
    mempool.add_tx(tx2).unwrap();

    // Evict with 500ms TTL at time 2000
    // tx1 (1000) + 500 = 1500 < 2000 -> Expired
    // tx2 (2000) + 500 = 2500 > 2000 -> OK
    let evicted = mempool.evict_expired(500, 2000);
    assert_eq!(evicted, 1);
    assert_eq!(mempool.len(), 1);
    assert_eq!(mempool.txs[0].amount, 20);
}
