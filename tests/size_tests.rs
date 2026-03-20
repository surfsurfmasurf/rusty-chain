use crate::core::types::{Transaction, Block};

#[test]
fn test_transaction_size() {
    let tx = Transaction::new("A", "B", 100, 1);
    let size = tx.size();
    assert!(size > 0);
}

#[test]
fn test_block_size() {
    let tx = Transaction::new("A", "B", 100, 1);
    let block = Block {
        header: crate::core::types::BlockHeader {
            prev_hash: "abc".to_string(),
            merkle_root: "def".to_string(),
            timestamp_ms: 1000,
            nonce: 1,
        },
        txs: vec![tx],
    };
    let size = block.size();
    assert!(size > 0);
}
