use crate::core::types::{Block, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Account {
    pub balance: u64,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State {
    pub accounts: HashMap<String, Account>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        self.accounts.get(address).map(|a| a.balance).unwrap_or(0)
    }

    pub fn get_nonce(&self, address: &str) -> u64 {
        self.accounts.get(address).map(|a| a.nonce).unwrap_or(0)
    }

    /// Apply a block to the state.
    ///
    /// If any transaction is invalid (e.g. insufficient balance), returns an error
    /// and the state remains unchanged (atomic application is the caller's responsibility
    /// if they modify `self` directly, but here we clone inside or assume sequential checks).
    ///
    /// For this simple implementation, we'll check everything before mutating.
    pub fn apply_block(&mut self, block: &Block) -> anyhow::Result<()> {
        use anyhow::Context;

        // 1. Verify all transactions against current state (read-only check)
        for (i, tx) in block.txs.iter().enumerate() {
            if i > 0 && tx.is_coinbase() {
                anyhow::bail!("Coinbase tx at index {} invalid (only index 0 allowed)", i);
            }
            self.validate_tx(tx).with_context(|| format!("tx index={}", i))?;
        }

        // 2. Apply transactions (mutate)
        for tx in &block.txs {
            self.apply_tx(tx);
        }

        Ok(())
    }

    fn validate_tx(&self, tx: &Transaction) -> anyhow::Result<()> {
        if tx.is_coinbase() {
            // Coinbase validation rules:
            // - Must be the first tx in block (checked by apply_block loop index if we pass it, but here we just check validity)
            // - Amount logic (checked by consensus, not state?)
            // For now, assume it's valid if it's a coinbase.
            return Ok(());
        }

        let sender = self.accounts.get(&tx.from).cloned().unwrap_or_default();

        // Nonce check
        if tx.nonce != sender.nonce {
            anyhow::bail!(
                "Invalid nonce for {}: expected {}, got {}",
                tx.from,
                sender.nonce,
                tx.nonce
            );
        }

        // Balance check
        let total_needed = tx.amount.checked_add(tx.fee).ok_or_else(|| {
            anyhow::anyhow!("Amount + Fee overflow for {}", tx.from)
        })?;

        if sender.balance < total_needed {
            anyhow::bail!(
                "Insufficient balance for {}: has {}, needs {} (amount={} fee={})",
                tx.from,
                sender.balance,
                total_needed,
                tx.amount,
                tx.fee
            );
        }

        Ok(())
    }

    fn apply_tx(&mut self, tx: &Transaction) {
        if !tx.is_coinbase() {
            // Deduct from sender (amount + fee)
            let sender = self.accounts.entry(tx.from.clone()).or_default();
            sender.balance -= tx.amount + tx.fee;
            sender.nonce += 1;
        }

        // Add to receiver (amount only; fees are already collected by the miner via coinbase)
        let receiver = self.accounts.entry(tx.to.clone()).or_default();
        receiver.balance += tx.amount;
    }
}
