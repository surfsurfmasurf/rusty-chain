use crate::core::hash::sha256_hex;
use crate::core::time::now_ms;
use crate::core::types::{Block, BlockHeader, Transaction};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    pub blocks: Vec<Block>,
}

impl Chain {
    pub fn new_genesis() -> Self {
        let header = BlockHeader {
            prev_hash: "0".repeat(64),
            timestamp_ms: now_ms(),
            nonce: 0,
            merkle_root: merkle_root(&[]),
        };
        let genesis = Block {
            header,
            txs: vec![],
        };
        Self {
            blocks: vec![genesis],
        }
    }

    pub fn height(&self) -> usize {
        self.blocks.len().saturating_sub(1)
    }

    pub fn tip_hash(&self) -> String {
        let tip = self.blocks.last().expect("genesis exists");
        hash_block(tip)
    }

    pub fn default_path() -> PathBuf {
        PathBuf::from("data/chain.json")
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let s = fs::read_to_string(path)?;
        let c: Self = serde_json::from_str(&s)?;
        Ok(c)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let s = serde_json::to_string_pretty(self)?;
        fs::write(path, s)?;
        Ok(())
    }

    /// Mine and append an empty block (demo PoW).
    pub fn mine_empty_block(&mut self, difficulty: usize) -> anyhow::Result<Block> {
        let prev = self.blocks.last().expect("genesis exists");
        let prev_hash = hash_block(prev);

        let txs: Vec<Transaction> = vec![];
        let mut nonce = 0_u64;

        loop {
            let header = BlockHeader {
                prev_hash: prev_hash.clone(),
                timestamp_ms: now_ms(),
                nonce,
                merkle_root: merkle_root(&txs),
            };
            let candidate = Block {
                header,
                txs: txs.clone(),
            };
            let h = hash_block(&candidate);
            if pow_ok(&h, difficulty) {
                self.blocks.push(candidate.clone());
                return Ok(candidate);
            }
            nonce = nonce.wrapping_add(1);
        }
    }

    /// Basic chain validation (linkage + merkle placeholder).
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.blocks.is_empty(), "chain has no blocks");

        let genesis = &self.blocks[0];
        anyhow::ensure!(
            genesis.header.prev_hash == "0".repeat(64),
            "genesis prev_hash must be 64 zeros"
        );
        anyhow::ensure!(
            genesis.header.merkle_root == merkle_root(&genesis.txs),
            "genesis merkle_root mismatch"
        );

        for i in 1..self.blocks.len() {
            let prev = &self.blocks[i - 1];
            let cur = &self.blocks[i];

            anyhow::ensure!(
                cur.header.prev_hash == hash_block(prev),
                "block {i} prev_hash does not match previous block hash"
            );
            anyhow::ensure!(
                cur.header.merkle_root == merkle_root(&cur.txs),
                "block {i} merkle_root mismatch"
            );
        }

        Ok(())
    }
}

pub fn hash_block(block: &Block) -> String {
    // Stable hashing: serialize header + txs as JSON (demo-friendly).
    let bytes = serde_json::to_vec(block).expect("serialize block");
    sha256_hex(&bytes)
}

pub fn merkle_root(txs: &[Transaction]) -> String {
    // Day 2 placeholder: simple hash of concatenated tx JSON.
    let bytes = serde_json::to_vec(txs).expect("serialize txs");
    sha256_hex(&bytes)
}

/// Very small PoW: block hash must start with N '0' hex chars.
pub fn pow_ok(block_hash: &str, difficulty: usize) -> bool {
    block_hash.chars().take(difficulty).all(|c| c == '0')
}
