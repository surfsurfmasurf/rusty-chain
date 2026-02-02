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

impl Transaction {
    /// Basic sanity checks (Week 1/early Week 2 demo).
    ///
    /// Note: signatures/balances/nonces will be enforced later.
    pub fn validate_basic(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.from.trim().is_empty(), "tx.from must be non-empty");
        anyhow::ensure!(!self.to.trim().is_empty(), "tx.to must be non-empty");
        anyhow::ensure!(self.from != self.to, "tx.from and tx.to must differ");
        anyhow::ensure!(self.amount > 0, "tx.amount must be > 0");
        Ok(())
    }
}

/// Block = header + transactions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<Transaction>,
}
