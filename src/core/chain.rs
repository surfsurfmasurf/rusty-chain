use crate::core::hash::sha256_hex;
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
            timestamp_ms: 0,
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
