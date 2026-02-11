use rusty_chain::core::chain::Chain;
use rusty_chain::core::mempool::Mempool;
use rusty_chain::core::types::Transaction;

#[test]
fn chain_next_nonce_for_starts_at_zero() {
    let c = Chain::new_genesis();
    assert_eq!(c.next_nonce_for("alice"), 0);
}

#[test]
fn chain_next_nonce_for_is_max_plus_one() {
    let mut c = Chain::new_genesis();

    // Fund alice
    let cb = Transaction {
        from: "SYSTEM".to_string(),
        to: "alice".to_string(),
        amount: 50,
        fee: 0,
        nonce: 1,
        pubkey_hex: None,
        signature_b64: None,
        memo: None,
    };
    c.mine_block(vec![cb], 0, None).unwrap();

    let tx1 = Transaction::new("alice", "bob", 1, 0);
    c.mine_block(vec![tx1], 0, None).unwrap();

    let tx2 = Transaction::new("alice", "bob", 1, 1);
    c.mine_block(vec![tx2], 0, None).unwrap();

    assert_eq!(c.next_nonce_for("alice"), 2);
    assert_eq!(c.next_nonce_for("bob"), 0);
}

#[test]
fn mempool_add_tx_checked_enforces_sequential_nonces() {
    let mut mp = Mempool::default();
    let base = 0;

    let tx0 = Transaction::new("alice", "bob", 1, 0);
    mp.add_tx_checked(tx0, base).unwrap();

    // Gap should fail (expected nonce=1).
    let tx2 = Transaction::new("alice", "bob", 1, 2);
    let err = mp.add_tx_checked(tx2, base).unwrap_err().to_string();
    assert!(err.contains("invalid nonce"), "unexpected error: {err}");

    // Next sequential nonce should succeed.
    let tx1 = Transaction::new("alice", "bob", 1, 1);
    mp.add_tx_checked(tx1, base).unwrap();

    assert_eq!(mp.txs.len(), 2);
}

#[test]
fn mempool_next_nonce_for_includes_pending_count() {
    let mut mp = Mempool::default();
    let base = 10;

    assert_eq!(mp.next_nonce_for("alice", base), 10);

    mp.add_tx_checked(Transaction::new("alice", "bob", 1, 10), base)
        .unwrap();

    mp.add_tx_checked(Transaction::new("alice", "bob", 1, 11), base)
        .unwrap();

    assert_eq!(mp.next_nonce_for("alice", base), 12);
    assert_eq!(mp.next_nonce_for("bob", base), 10);
}
