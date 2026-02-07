use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::core::types::{Block, Transaction};

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
        // 1. Verify all transactions against current state (read-only check)
        for (i, tx) in block.txs.iter().enumerate() {
            if i > 0 && tx.is_coinbase() {
                 anyhow::bail!("Coinbase tx at index {} invalid (only index 0 allowed)", i);
            }
            self.validate_tx(tx)?;
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
            anyhow::bail!("Invalid nonce for {}: expected {}, got {}", tx.from, sender.nonce, tx.nonce);
        }

        // Balance check
        // TODO: add fee?
        if sender.balance < tx.amount {
            anyhow::bail!("Insufficient balance for {}: has {}, needs {}", tx.from, sender.balance, tx.amount);
        }

        Ok(())
    }

    fn apply_tx(&mut self, tx: &Transaction) {
        if !tx.is_coinbase() {
            // Deduct from sender
            let sender = self.accounts.entry(tx.from.clone()).or_default();
            sender.balance -= tx.amount;
            sender.nonce += 1;
        }

        // Add to receiver
        let receiver = self.accounts.entry(tx.to.clone()).or_default();
        receiver.balance += tx.amount;
    }
}
