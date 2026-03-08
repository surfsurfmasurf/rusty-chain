use rusty_chain::core::mempool::Mempool;
use rusty_chain::core::types::Transaction;

#[test]
fn rbf_replacement_requires_higher_fee_and_sequence() {
    let mut mp = Mempool::default();
    let base = 0;

    // 1. Initial transaction (nonce=0, fee=0, sequence=0)
    let tx1 = Transaction::new("alice", "bob", 10, 0);
    mp.add_tx_checked(tx1, base).expect("Initial tx should be added");

    // 2. Replacement with higher fee but SAME sequence (should fail)
    let mut tx2 = Transaction::new_with_fee("alice", "bob", 10, 10, 0);
    tx2.sequence = 0;
    let err = mp.add_tx_checked(tx2.clone(), base).unwrap_err().to_string();
    assert!(err.contains("higher sequence number"), "Error should mention sequence: {err}");

    // 3. Replacement with SAME fee but higher sequence (should fail)
    let mut tx3 = Transaction::new("alice", "bob", 10, 0);
    tx3.sequence = 1;
    tx3.fee = 0;
    let err = mp.add_tx_checked(tx3, base).unwrap_err().to_string();
    assert!(err.contains("strictly higher fee"), "Error should mention fee: {err}");

    // 4. Replacement with higher fee AND higher sequence (should succeed)
    let mut tx4 = Transaction::new_with_fee("alice", "bob", 10, 10, 0);
    tx4.sequence = 1;
    mp.add_tx_checked(tx4.clone(), base).expect("RBF should succeed with higher fee and sequence");

    assert_eq!(mp.txs.len(), 1, "Should still only have 1 tx in mempool after replacement");
    assert_eq!(mp.txs[0].fee, 10);
    assert_eq!(mp.txs[0].sequence, 1);
}

#[test]
fn rbf_replaces_correct_transaction() {
    let mut mp = Mempool::default();
    let base = 0;

    // Add two txs from alice
    mp.add_tx_checked(Transaction::new("alice", "bob", 1, 0), base).unwrap();
    mp.add_tx_checked(Transaction::new("alice", "bob", 1, 1), base).unwrap();

    // Replace the first one (nonce=0)
    let mut rbf = Transaction::new_with_fee("alice", "bob", 5, 10, 0);
    rbf.sequence = 1;
    mp.add_tx_checked(rbf, base).unwrap();

    assert_eq!(mp.txs.len(), 2);
    assert_eq!(mp.txs[0].nonce, 0);
    assert_eq!(mp.txs[0].fee, 10, "First tx should be replaced");
    assert_eq!(mp.txs[1].nonce, 1);
    assert_eq!(mp.txs[1].fee, 0, "Second tx should be untouched");
}
