use crate::core::hash::sha256_hex;
use crate::core::state::State;
use crate::core::time::now_ms;
use crate::core::types::{Block, BlockHeader, Transaction};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    /// Chain-wide PoW difficulty (leading '0' hex chars).
    ///
    /// Stored in the chain file so `validate` can check PoW without CLI flags.
    #[serde(default = "default_pow_difficulty")]
    pub pow_difficulty: usize,

    pub blocks: Vec<Block>,

    /// Checkpoints for pruning and fast synchronization.
    /// Maps block height to block hash.
    #[serde(default)]
    pub checkpoints: std::collections::HashMap<usize, String>,

    /// Block hash to Block mapping for fast O(1) block retrieval by hash.
    /// Used primarily in P2P sync (GetData handling).
    #[serde(skip, default)]
    pub block_index: std::collections::HashMap<String, usize>,
}

fn default_pow_difficulty() -> usize {
    3
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
            header: header.clone(),
            txs: vec![],
        };
        let genesis_hash = header.hash();
        let mut checkpoints = std::collections::HashMap::new();
        checkpoints.insert(0, genesis_hash.clone());

        let mut block_index = std::collections::HashMap::new();
        block_index.insert(genesis_hash, 0);

        Self {
            pow_difficulty: default_pow_difficulty(),
            blocks: vec![genesis],
            checkpoints,
            block_index,
        }
    }

    pub fn height(&self) -> usize {
        self.blocks.len().saturating_sub(1)
    }

    pub fn tip_hash(&self) -> String {
        let tip = self.blocks.last().expect("genesis exists");
        tip.header.hash()
    }

    pub fn tx_count(&self) -> usize {
        self.blocks.iter().map(|b| b.txs.len()).sum()
    }

    /// Compute the next expected nonce for a given sender based on transactions already in-chain.
    ///
    /// Nonce enforcement is kept intentionally simple for Week 2 demos:
    /// - Per-sender monotonically increasing u64 starting at 0.
    /// - This does NOT check balances or signatures (yet).
    pub fn next_nonce_for(&self, sender: &str) -> u64 {
        let mut max_nonce: Option<u64> = None;
        for b in &self.blocks {
            for tx in &b.txs {
                if tx.from == sender {
                    max_nonce = Some(max_nonce.map_or(tx.nonce, |m| m.max(tx.nonce)));
                }
            }
        }
        max_nonce.map_or(0, |m| m.saturating_add(1))
    }

    pub fn default_path() -> PathBuf {
        PathBuf::from("data/chain.json")
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let s = fs::read_to_string(path)?;
        let mut c: Self = serde_json::from_str(&s)?;

        // Rebuild block index
        c.rebuild_block_index();

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

    pub fn tip_header(&self) -> &BlockHeader {
        &self.blocks.last().expect("genesis exists").header
    }

    /// Mine and append a block with provided transactions.
    ///
    /// If `miner_address` is provided, a coinbase transaction (50 coins + fees) is prepended.
    pub fn mine_block(
        &mut self,
        mut txs: Vec<Transaction>,
        new_difficulty: usize,
        miner_address: Option<&str>,
    ) -> anyhow::Result<Block> {
        let block_height = self.height() as u64 + 1;

        // Persist difficulty so later `validate` has the right context.
        self.pow_difficulty = new_difficulty;
        let difficulty = self.pow_difficulty;

        // Prepend coinbase if miner specified
        if let Some(miner) = miner_address {
            let total_fees: u64 = txs.iter().map(|tx| tx.fee).sum();
            let coinbase = Transaction {
                from: "SYSTEM".to_string(),
                to: miner.to_string(),
                amount: 50 + total_fees,
                fee: 0,
                nonce: block_height,
                pubkey_hex: None,
                signature_b64: None,
                memo: Some(format!("Block {block_height} Reward")),
                sequence: 0,
                timestamp_ms: now_ms(),
                locktime: None,
                expiry: None,
                priority: 255, // Max priority for coinbase
                ttl_ms: 0,
                nonce_id: Some(format!("coinbase-{}", block_height)),
                expiration_ms: 0,
                message_id: None,
                tag: None,
                unique_id: None,
                weight: 0,
                is_private: false,
                session_id: None,
                payload_checksum: None,
                version: 1,
            };
            txs.insert(0, coinbase);
        }

        // Validate state transitions (balances, nonces) before mining.
        // We create a temporary state, apply the new transactions, and see if it holds.
        let mut state = self.compute_state()?;
        state
            .apply_block_txs(&txs, block_height as usize)
            .context("mempool transactions failed state application")?;

        let prev = self.blocks.last().expect("genesis exists");
        let prev_hash = prev.header.hash();

        let merkle_root = merkle_root(&txs);
        let timestamp_ms = now_ms();
        let mut nonce = 0_u64;

        loop {
            let header = BlockHeader {
                prev_hash: prev_hash.clone(),
                timestamp_ms,
                nonce,
                merkle_root: merkle_root.clone(),
            };
            let h = header.hash();
            if pow_ok(&h, difficulty) {
                let candidate = Block {
                    header,
                    txs: txs.clone(),
                };
                let hash = candidate.header.hash();
                let height = self.blocks.len();
                self.blocks.push(candidate.clone());
                self.block_index.insert(hash, height);

                // Auto-checkpoint every 10 blocks
                if height > 0 && height.is_multiple_of(10) {
                    self.checkpoints.insert(height, candidate.header.hash());
                }

                return Ok(candidate);
            }
            nonce = nonce.wrapping_add(1);
        }
    }

    /// Mine and append an empty block (demo PoW).
    pub fn mine_empty_block(&mut self, new_difficulty: usize) -> anyhow::Result<Block> {
        self.mine_block(vec![], new_difficulty, None)
    }

    pub fn compute_state(&self) -> anyhow::Result<State> {
        let mut state = State::new();
        for (i, block) in self.blocks.iter().enumerate() {
            state
                .apply_block(block, i)
                .with_context(|| format!("block {}", i))?;
        }
        Ok(state)
    }

    /// Validates a single transaction against the current ledger state.
    pub fn validate_transaction(&self, tx: &Transaction) -> anyhow::Result<()> {
        tx.validate_accept()
            .context("TX baseline validation failed")?;

        // Versioning check (future-proofing)
        anyhow::ensure!(tx.version == 1, "only transaction version 1 is supported");

        let state = self.compute_state()?;
        state.validate_transaction(tx, self.height() + 1)?;
        Ok(())
    }

    /// Validates a block's structure, PoW, and state transitions.
    pub fn validate_block(&self, block: &Block) -> anyhow::Result<()> {
        let prev_block = self.blocks.last().expect("genesis exists");
        block.validate_with_prev(&prev_block.header, self.pow_difficulty as u32)?;

        let merkle = merkle_root(&block.txs);
        anyhow::ensure!(
            block.header.merkle_root == merkle,
            "merkle mismatch: expected {} got {}",
            merkle,
            block.header.merkle_root
        );

        // 2. State transition
        let mut state = self.compute_state()?;
        state
            .apply_block(block, self.height() + 1)
            .context("state transition failed for block")?;

        Ok(())
    }

    /// Appends a validated block to the chain.
    pub fn append_block(&mut self, block: Block) -> anyhow::Result<()> {
        let hash = block.header.hash();
        let height = self.blocks.len();

        self.validate_block(&block)?;

        self.blocks.push(block);
        self.block_index.insert(hash, height);

        // Auto-checkpoint every 10 blocks
        let current_height = self.height();
        if current_height > 0 && current_height.is_multiple_of(10) {
            self.add_checkpoint();
        }

        Ok(())
    }

    /// Adds a checkpoint at the current height.
    pub fn add_checkpoint(&mut self) {
        let height = self.height();
        let hash = self.tip_hash();
        self.checkpoints.insert(height, hash);
    }

    /// Gets a checkpoint at a specific height if it exists.
    pub fn get_checkpoint_at(&self, height: usize) -> Option<String> {
        self.checkpoints.get(&height).cloned()
    }

    /// Returns the highest checkpoint currently known.
    pub fn get_last_checkpoint(&self) -> Option<(usize, String)> {
        self.checkpoints
            .iter()
            .max_by_key(|&(&h, _)| h)
            .map(|(&h, hash)| (h, hash.clone()))
    }

    /// Validates the chain against its checkpoints.
    pub fn validate_checkpoints(&self) -> anyhow::Result<()> {
        for (&height, expected_hash) in &self.checkpoints {
            if height < self.blocks.len() {
                let actual_hash = self.blocks[height].header.hash();
                anyhow::ensure!(
                    actual_hash == *expected_hash,
                    "checkpoint mismatch at height {}: expected {}, got {}",
                    height,
                    expected_hash,
                    actual_hash
                );
            }
        }
        Ok(())
    }

    /// Calculates the average fee rate (fee/size) of recent blocks.
    pub fn estimate_fee_rate(&self, window: usize) -> f64 {
        let n = self.blocks.len();
        if n <= 1 {
            return 1.0; // Default: 1 unit per byte
        }

        let start = n.saturating_sub(window).max(1); // Skip genesis
        let recent = &self.blocks[start..];

        let mut total_fee = 0.0;
        let mut total_size = 0.0;

        for block in recent {
            for tx in &block.txs {
                // Skip SYSTEM transactions (e.g., rewards)
                if tx.from == "SYSTEM" {
                    continue;
                }
                total_fee += tx.fee as f64;
                total_size += tx.size() as f64;
            }
        }

        if total_size == 0.0 {
            // If no transactions in the window, try to find the last non-coinbase fee
            for block in self.blocks.iter().rev() {
                for tx in &block.txs {
                    if tx.from != "SYSTEM" && tx.size() > 0 {
                        return tx.fee as f64 / tx.size() as f64;
                    }
                }
            }
            1.0 // Default fallback
        } else {
            total_fee / total_size
        }
    }

    /// Basic chain validation (linkage + merkle placeholder).
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

        // Checkpoints validation
        self.validate_checkpoints()?;

        // Validate state transitions (balances, nonces)
        // This ensures every block in the chain is valid according to the state rules.
        self.compute_state().context("state validation failed")?;

        for i in 1..self.blocks.len() {
            let prev = &self.blocks[i - 1];
            let cur = &self.blocks[i];

            for (j, tx) in cur.txs.iter().enumerate() {
                tx.validate_accept()
                    .with_context(|| format!("invalid tx in block={i} index={j}"))?;
            }

            cur.validate_with_prev(&prev.header, self.pow_difficulty as u32)
                .with_context(|| format!("block {} linkage/PoW fail", i))?;

            let expected_merkle = merkle_root(&cur.txs);
            anyhow::ensure!(
                cur.header.merkle_root == expected_merkle,
                "block {i} merkle_root mismatch (expected={expected_merkle} got={})",
                cur.header.merkle_root
            );
        }

        Ok(())
    }

    /// Rebuilds the block index from scratch.
    pub fn rebuild_block_index(&mut self) {
        self.block_index.clear();
        for (i, block) in self.blocks.iter().enumerate() {
            self.block_index.insert(block.header.hash(), i);
        }
    }
}

pub fn hash_block(block: &Block) -> String {
    // Stable hashing: serialize header + txs as JSON (demo-friendly).
    let bytes = serde_json::to_vec(block).expect("serialize block");
    sha256_hex(&bytes)
}

pub fn merkle_root(txs: &[Transaction]) -> String {
    // Simple demo merkle: hash of concatenated tx hashes.
    if txs.is_empty() {
        return sha256_hex(&[]);
    }

    let joined = txs.iter().map(|t| t.id()).collect::<Vec<_>>().join("");

    sha256_hex(joined.as_bytes())
}

/// Very small PoW: block hash must start with N '0' hex chars.
pub fn pow_ok(block_hash: &str, difficulty: usize) -> bool {
    block_hash.chars().take(difficulty).all(|c| c == '0')
}
