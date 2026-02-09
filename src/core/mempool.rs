use crate::core::types::Transaction;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Mempool {
    pub txs: Vec<Transaction>,
}

impl Mempool {
    pub fn default_path() -> PathBuf {
        PathBuf::from("data/mempool.json")
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let s = fs::read_to_string(path)?;
        let m: Self = serde_json::from_str(&s)?;
        Ok(m)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let s = serde_json::to_string_pretty(self)?;
        fs::write(path, s)?;
        Ok(())
    }

    fn ensure_unique_hash(&self, tx: &Transaction) -> anyhow::Result<()> {
        let h = tx.id();
        let existing: HashSet<String> = self.txs.iter().map(|t| t.id()).collect();
        anyhow::ensure!(!existing.contains(&h), "duplicate tx (hash={h})");
        Ok(())
    }

    /// Compute the next expected nonce for `sender` given a base nonce (usually from chain).
    ///
    /// Rule: expected = base + number of pending txs from sender.
    pub fn next_nonce_for(&self, sender: &str, base_nonce: u64) -> u64 {
        let pending = self.txs.iter().filter(|t| t.from == sender).count() as u64;
        base_nonce.saturating_add(pending)
    }

    /// Add a tx enforcing a simple per-sender nonce rule.
    ///
    /// This is intentionally minimal (Week 2 demo): it prevents gaps and duplicates for a sender
    /// within the mempool, using the caller-provided `base_nonce` (from chain).
    pub fn add_tx_checked(&mut self, tx: Transaction, base_nonce: u64) -> anyhow::Result<()> {
        tx.validate_accept()?;

        let expected = self.next_nonce_for(&tx.from, base_nonce);
        anyhow::ensure!(
            tx.nonce == expected,
            "invalid nonce for sender={} (expected={} got={})",
            tx.from,
            expected,
            tx.nonce
        );

        anyhow::ensure!(
            !self
                .txs
                .iter()
                .any(|t| t.from == tx.from && t.nonce == tx.nonce),
            "duplicate nonce for sender={} (nonce={})",
            tx.from,
            tx.nonce
        );

        self.ensure_unique_hash(&tx)?;

        self.txs.push(tx);
        Ok(())
    }

    pub fn add_tx(&mut self, tx: Transaction) -> anyhow::Result<()> {
        tx.validate_accept()?;

        self.ensure_unique_hash(&tx)?;

        self.txs.push(tx);
        Ok(())
    }

    pub fn drain(&mut self) -> Vec<Transaction> {
        let mut out = Vec::new();
        std::mem::swap(&mut self.txs, &mut out);
        out
    }
}
