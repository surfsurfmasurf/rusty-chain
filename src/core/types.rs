use serde::{Deserialize, Serialize};

/// Basic block header (minimal, demo-oriented).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeader {
    pub prev_hash: String,
    pub timestamp_ms: u64,
    pub nonce: u64,
    pub merkle_root: String,
}

/// A minimal transaction (placeholder). Week 2 will add signatures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
}

/// Block = header + transactions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<Transaction>,
}
