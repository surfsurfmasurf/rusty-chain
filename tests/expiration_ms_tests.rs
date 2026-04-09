use rusty_chain::core::mempool::Mempool;
use rusty_chain::core::types::Transaction;
use std::thread;
use std::time::Duration;

#[test]
fn test_transaction_expiration_validation() {
    let now = rusty_chain::core::time::now_ms();
    let mut tx = Transaction::new("A", "B", 100, 0);

    // Set expiration in the past
    tx.expiration_ms = now - 1000;

    let res = tx.validate_basic();
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("expired"));

    // Set expiration in the future
    tx.expiration_ms = now + 10000;
    assert!(tx.validate_basic().is_ok());
}

#[test]
fn test_mempool_expiration_eviction() {
    let mut mp = Mempool::new();
    let now = rusty_chain::core::time::now_ms();

    let mut tx1 = Transaction::new("A", "B", 100, 0);
    tx1.expiration_ms = now + 500; // Expires soon

    let mut tx2 = Transaction::new("C", "D", 200, 0);
    tx2.expiration_ms = now + 5000; // Expires later

    mp.add_tx(tx1).unwrap();
    mp.add_tx(tx2).unwrap();

    assert_eq!(mp.len(), 2);

    // Evict at now + 1000ms
    let evicted = mp.evict_expired(now + 1000);
    assert_eq!(evicted, 1);
    assert_eq!(mp.len(), 1);
    assert_eq!(mp.txs[0].amount, 200);
}
