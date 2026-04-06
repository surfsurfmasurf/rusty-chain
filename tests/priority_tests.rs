use rusty_chain::core::mempool::Mempool;
use rusty_chain::core::types::Transaction;

#[test]
fn test_mempool_priority_sorting() {
    let mut mempool = Mempool::new();

    // Same fee, different priority
    let mut tx1 = Transaction::new("A", "B", 10, 0);
    tx1.fee = 10;
    tx1.priority = 10;

    let mut tx2 = Transaction::new("A", "C", 10, 1);
    tx2.fee = 10;
    tx2.priority = 20; // Higher priority

    // Higher fee, lower priority
    let mut tx3 = Transaction::new("A", "D", 10, 2);
    tx3.fee = 20;
    tx3.priority = 0;

    mempool.add_tx(tx1).unwrap();
    mempool.add_tx(tx2).unwrap();
    mempool.add_tx(tx3).unwrap();

    let sorted = mempool.drain_sorted();

    // 1. tx3 (fee 20)
    assert_eq!(sorted[0].from, "A");
    assert_eq!(sorted[0].to, "D");

    // 2. tx2 (fee 10, priority 20)
    assert_eq!(sorted[1].to, "C");

    // 3. tx1 (fee 10, priority 10)
    assert_eq!(sorted[2].to, "B");
}

#[test]
fn test_mempool_priority_limit_size() {
    let mut mempool = Mempool::new();

    let mut tx1 = Transaction::new("A", "B", 10, 0);
    tx1.fee = 10;
    tx1.priority = 50;

    let mut tx2 = Transaction::new("A", "C", 10, 1);
    tx2.fee = 10;
    tx2.priority = 100;

    let size = tx1.size();
    mempool.add_tx(tx1).unwrap();
    mempool.add_tx(tx2).unwrap();

    // Limit to size of 1 tx. Higher priority should stay.
    mempool.limit_size(size + 1);

    assert_eq!(mempool.len(), 1);
    assert_eq!(mempool.txs[0].priority, 100);
}
