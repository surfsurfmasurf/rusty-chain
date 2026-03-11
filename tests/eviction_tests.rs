use rusty_chain::core::mempool::Mempool;
use rusty_chain::core::types::Transaction;

#[test]
fn test_mempool_eviction() {
    let mut mp = Mempool::new();

    // Add txs with different timestamps
    let mut tx1 = Transaction::new("alice", "bob", 10, 0);
    tx1.timestamp_ms = 1000;

    let mut tx2 = Transaction::new("alice", "bob", 10, 1);
    tx2.timestamp_ms = 2000;

    let mut tx3 = Transaction::new("alice", "bob", 10, 2);
    tx3.timestamp_ms = 3000;

    mp.add_tx(tx1).unwrap();
    mp.add_tx(tx2).unwrap();
    mp.add_tx(tx3).unwrap();

    assert_eq!(mp.txs.len(), 3);

    // Evict with TTL 1500 at now=3000
    // tx1: 1000 + 1500 = 2500 < 3000 (expired)
    // tx2: 2000 + 1500 = 3500 > 3000 (ok)
    // tx3: 3000 + 1500 = 4500 > 3000 (ok)
    let evicted = mp.evict_expired(1500, 3000);

    assert_eq!(evicted, 1);
    assert_eq!(mp.txs.len(), 2);
    assert_eq!(mp.txs[0].timestamp_ms, 2000);
    assert_eq!(mp.txs[1].timestamp_ms, 3000);
}
