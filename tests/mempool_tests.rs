use rusty_chain::core::mempool::Mempool;
use rusty_chain::core::types::Transaction;

#[test]
fn test_mempool_get_tx_by_id() {
    let mut mempool = Mempool::new();
    let tx = Transaction::new("A", "B", 100, 0);
    let id = tx.id();

    mempool.add_tx(tx.clone()).unwrap();

    let retrieved = mempool.get_tx_by_id(&id).unwrap();
    assert_eq!(retrieved.from, "A");
    assert_eq!(retrieved.amount, 100);
}

#[test]
fn test_mempool_contains_tx() {
    let mut mempool = Mempool::new();
    let tx = Transaction::new("A", "B", 100, 0);
    let id = tx.id();

    assert!(!mempool.contains_tx(&id));
    mempool.add_tx(tx).unwrap();
    assert!(mempool.contains_tx(&id));
}

#[test]
fn test_mempool_stats() {
    let mut mempool = Mempool::new();
    let tx1 = Transaction::new_with_fee("A", "B", 100, 10, 0, 0);
    let tx2 = Transaction::new_with_fee("C", "D", 200, 20, 0, 0);

    mempool.add_tx(tx1).unwrap();
    mempool.add_tx(tx2).unwrap();

    let count = mempool.txs.len();
    let total_size: usize = mempool.txs.iter().map(|t| t.size()).sum();
    let min_fee = mempool.txs.iter().map(|t| t.fee).min().unwrap();
    let max_fee = mempool.txs.iter().map(|t| t.fee).max().unwrap();

    assert_eq!(count, 2);
    assert!(total_size > 0);
    assert_eq!(min_fee, 10);
    assert_eq!(max_fee, 20);
}
