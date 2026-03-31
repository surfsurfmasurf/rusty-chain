use crate::core::types::Transaction;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Mempool {
    pub txs: Vec<Transaction>,

    /// Transaction ID to index mapping for fast O(1) lookups.
    #[serde(skip, default)]
    pub tx_index: std::collections::HashMap<String, usize>,
}

impl Mempool {
    pub fn default_path() -> PathBuf {
        PathBuf::from("data/mempool.json")
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let s = fs::read_to_string(path)?;
        let mut m: Self = serde_json::from_str(&s)?;

        // Rebuild tx index
        for (i, tx) in m.txs.iter().enumerate() {
            m.tx_index.insert(tx.id(), i);
        }

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
        anyhow::ensure!(!self.tx_index.contains_key(&h), "duplicate tx (hash={h})");
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

        // If a transaction with the same sender and nonce exists, we check if the new tx
        // is an RBF (Replace-By-Fee) candidate.
        // Rule: same sender, same nonce, higher fee OR higher sequence.
        if let Some(pos) = self
            .txs
            .iter()
            .position(|t| t.from == tx.from && t.nonce == tx.nonce)
        {
            let existing = &self.txs[pos];

            // RBF Rule:
            // 1. Fee must be strictly higher (to prevent spam).
            // 2. Sequence must be higher than the existing transaction's sequence.
            anyhow::ensure!(
                tx.fee > existing.fee,
                "replacement tx must have a strictly higher fee (existing={} new={})",
                existing.fee,
                tx.fee
            );
            anyhow::ensure!(
                tx.sequence > existing.sequence,
                "replacement tx must have a higher sequence number (existing={} new={})",
                existing.sequence,
                tx.sequence
            );

            // Replace the existing transaction
            self.tx_index.remove(&existing.id());
            let id = tx.id();
            self.txs[pos] = tx;
            self.tx_index.insert(id, pos);
            return Ok(());
        }

        let expected = self.next_nonce_for(&tx.from, base_nonce);
        anyhow::ensure!(
            tx.nonce == expected,
            "invalid nonce for sender={} (expected={} got={})",
            tx.from,
            expected,
            tx.nonce
        );

        self.ensure_unique_hash(&tx)?;

        let id = tx.id();
        let pos = self.txs.len();
        self.txs.push(tx);
        self.tx_index.insert(id, pos);
        Ok(())
    }

    pub fn add_tx(&mut self, tx: Transaction) -> anyhow::Result<()> {
        tx.validate_accept()?;

        self.ensure_unique_hash(&tx)?;

        let id = tx.id();
        let pos = self.txs.len();
        self.txs.push(tx);
        self.tx_index.insert(id, pos);
        Ok(())
    }

    pub fn drain(&mut self) -> Vec<Transaction> {
        let mut out = Vec::new();
        std::mem::swap(&mut self.txs, &mut out);
        self.tx_index.clear();
        out
    }

    /// Returns a transaction by its ID if it exists in the mempool.
    pub fn get_tx_by_id(&self, tx_id: &str) -> Option<&Transaction> {
        self.tx_index.get(tx_id).and_then(|&idx| self.txs.get(idx))
    }

    /// Returns true if the mempool contains a transaction with the given ID.
    pub fn contains_tx(&self, tx_id: &str) -> bool {
        self.tx_index.contains_key(tx_id)
    }

    /// Returns the number of transactions in the mempool.
    pub fn len(&self) -> usize {
        self.txs.len()
    }

    /// Returns true if the mempool is empty.
    pub fn is_empty(&self) -> bool {
        self.txs.is_empty()
    }

    /// Truncates the mempool to a maximum size, removing lowest fee transactions.
    pub fn truncate(&mut self, max_size: usize) {
        if self.txs.len() > max_size {
            self.sort_by_fee();
            self.txs.truncate(max_size);
            self.rebuild_index();
        }
    }

    /// Removes a transaction from the mempool by its ID.
    pub fn remove_tx(&mut self, tx_id: &str) {
        if let Some(pos) = self.tx_index.remove(tx_id) {
            self.txs.remove(pos);
            // Rebuild index after removal because positions shifted
            self.rebuild_index();
        }
    }

    /// Sorts transactions in the mempool by fee (descending).
    pub fn sort_by_fee(&mut self) {
        self.txs.sort_by(|a, b| b.fee.cmp(&a.fee));
        self.rebuild_index();
    }

    /// Removes all transactions from the mempool that are included in the given slice.
    pub fn remove_included(&mut self, txs: &[Transaction]) {
        let ids: HashSet<String> = txs.iter().map(|t| t.id()).collect();
        self.txs.retain(|t| !ids.contains(&t.id()));
        self.rebuild_index();
    }

    /// Evicts transactions from the mempool that have exceeded the time-to-live (TTL).
    /// Returns the number of evicted transactions.
    pub fn evict_expired(&mut self, ttl_ms: u64, now_ms: u64) -> usize {
        let count_before = self.txs.len();
        self.txs.retain(|t| {
            if t.timestamp_ms == 0 {
                // If timestamp is not set (legacy or internal), keep it for now
                // or we could treat it as expired.
                return true;
            }
            now_ms < t.timestamp_ms.saturating_add(ttl_ms)
        });
        let evicted = count_before - self.txs.len();
        if evicted > 0 {
            self.rebuild_index();
        }
        evicted
    }

    fn rebuild_index(&mut self) {
        self.tx_index.clear();
        for (i, tx) in self.txs.iter().enumerate() {
            self.tx_index.insert(tx.id(), i);
        }
    }
}

#[cfg(test)]
mod mempool_index_tests {
    use super::*;
    use crate::core::types::Transaction;

    #[test]
    fn test_mempool_index_consistency() {
        let mut mempool = Mempool::new();
        let tx1 = Transaction::new("A", "B", 10, 0);
        let tx2 = Transaction::new("A", "C", 20, 1);
        let id1 = tx1.id();
        let id2 = tx2.id();

        mempool.add_tx(tx1).unwrap();
        mempool.add_tx(tx2).unwrap();

        assert_eq!(mempool.tx_index.len(), 2);
        assert_eq!(mempool.tx_index.get(&id1), Some(&0));
        assert_eq!(mempool.tx_index.get(&id2), Some(&1));

        mempool.remove_tx(&id1);
        assert_eq!(mempool.tx_index.len(), 1);
        assert_eq!(mempool.tx_index.get(&id2), Some(&0)); // Shifted
    }

    #[test]
    fn test_mempool_sort_by_fee() {
        let mut mempool = Mempool::new();
        let mut tx_low = Transaction::new("A", "B", 10, 0);
        tx_low.fee = 1;
        let mut tx_high = Transaction::new("A", "C", 20, 1);
        tx_high.fee = 10;

        let id_high = tx_high.id();

        mempool.add_tx(tx_low).unwrap();
        mempool.add_tx(tx_high).unwrap();

        assert_eq!(mempool.txs[0].fee, 1);

        mempool.sort_by_fee();

        assert_eq!(mempool.txs[0].fee, 10);
        assert_eq!(mempool.tx_index.get(&id_high), Some(&0));
    }
}
