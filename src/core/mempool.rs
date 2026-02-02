use crate::core::types::Transaction;
use serde::{Deserialize, Serialize};
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

    pub fn add_tx(&mut self, tx: Transaction) {
        self.txs.push(tx);
    }

    pub fn drain(&mut self) -> Vec<Transaction> {
        let mut out = Vec::new();
        std::mem::swap(&mut self.txs, &mut out);
        out
    }
}
