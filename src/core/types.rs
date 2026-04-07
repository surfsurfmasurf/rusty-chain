use serde::{Deserialize, Serialize};

/// Basic block header (minimal, demo-oriented).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeader {
    pub prev_hash: String,
    pub timestamp_ms: u64,
    pub nonce: u64,
    pub merkle_root: String,
}

impl BlockHeader {
    pub fn hash(&self) -> String {
        crate::core::hash::header_hash(self)
    }

    /// Stateless header verification (PoW check).
    pub fn verify_pow(&self, difficulty: u32) -> anyhow::Result<()> {
        let hash = self.hash();
        let target = "0".repeat(difficulty as usize);
        if !hash.starts_with(&target) {
            anyhow::bail!("invalid PoW: hash={} difficulty={}", hash, difficulty);
        }
        Ok(())
    }
}

/// A minimal transaction (Week 2: add optional signatures).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    #[serde(default)]
    pub fee: u64,
    pub nonce: u64,

    /// Optional ed25519 public key (hex) used to verify `signature_b64`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubkey_hex: Option<String>,

    /// Optional ed25519 signature (base64) over the signing payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_b64: Option<String>,

    /// Optional comment/metadata for the transaction (limit: 128 chars)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Optional sequence number for the transaction (future-proofing)
    #[serde(default)]
    pub sequence: u32,

    /// Optional timestamp for when the transaction was created (Unix epoch ms)
    #[serde(default)]
    pub timestamp_ms: u64,

    /// Optional locktime (block height). If set, the transaction is invalid until the chain reaches this height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locktime: Option<u64>,

    /// Optional expiry (block height). If set, the transaction is invalid after the chain reaches this height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry: Option<u64>,

    /// Optional priority level (0-255). Used for mempool ordering and processing.
    #[serde(default)]
    pub priority: u8,

    /// Optional time-to-live (milliseconds) for mempool duration.
    #[serde(default)]
    pub ttl_ms: u64,

    /// UNIQUE: Unique identifier for the transaction (UUID v4), used for tracking through P2P and mempool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce_id: Option<String>,

    /// Optional P2P message ID to handle P2P-level deduplication.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,

    /// Version number for the transaction format.
    #[serde(default = "default_tx_version")]
    pub version: u32,
}

fn default_tx_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxSignPayload {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(default)]
    pub sequence: u32,
    #[serde(default)]
    pub timestamp_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locktime: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry: Option<u64>,
    #[serde(default)]
    pub priority: u8,
    #[serde(default)]
    pub ttl_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub version: u32,
}

impl Transaction {
    pub fn new(from: impl Into<String>, to: impl Into<String>, amount: u64, nonce: u64) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            amount,
            fee: 0,
            nonce,
            pubkey_hex: None,
            signature_b64: None,
            memo: None,
            sequence: 0,
            timestamp_ms: crate::core::time::now_ms(),
            locktime: None,
            expiry: None,
            priority: 0,
            ttl_ms: 0,
            nonce_id: None,
            message_id: None,
            version: 1,
        }
    }

    pub fn new_with_sequence(
        from: impl Into<String>,
        to: impl Into<String>,
        amount: u64,
        nonce: u64,
        sequence: u32,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            amount,
            fee: 0,
            nonce,
            pubkey_hex: None,
            signature_b64: None,
            memo: None,
            sequence,
            timestamp_ms: crate::core::time::now_ms(),
            locktime: None,
            expiry: None,
            priority: 0,
            ttl_ms: 0,
            nonce_id: None,
            message_id: None,
            version: 1,
        }
    }

    pub fn new_with_locktime(
        from: impl Into<String>,
        to: impl Into<String>,
        amount: u64,
        nonce: u64,
        locktime: u64,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            amount,
            fee: 0,
            nonce,
            pubkey_hex: None,
            signature_b64: None,
            memo: None,
            sequence: 0,
            timestamp_ms: crate::core::time::now_ms(),
            locktime: Some(locktime),
            expiry: None,
            priority: 0,
            ttl_ms: 0,
            nonce_id: None,
            message_id: None,
            version: 1,
        }
    }

    pub fn new_with_fee(
        from: impl Into<String>,
        to: impl Into<String>,
        amount: u64,
        fee: u64,
        nonce: u64,
        sequence: u32,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            amount,
            fee,
            nonce,
            pubkey_hex: None,
            signature_b64: None,
            memo: None,
            sequence,
            timestamp_ms: crate::core::time::now_ms(),
            locktime: None,
            expiry: None,
            priority: 0,
            ttl_ms: 0,
            nonce_id: None,
            message_id: None,
            version: 1,
        }
    }

    pub fn signing_payload(&self) -> TxSignPayload {
        TxSignPayload {
            from: self.from.clone(),
            to: self.to.clone(),
            amount: self.amount,
            fee: self.fee,
            nonce: self.nonce,
            memo: self.memo.clone(),
            sequence: self.sequence,
            timestamp_ms: self.timestamp_ms,
            locktime: self.locktime,
            expiry: self.expiry,
            priority: self.priority,
            ttl_ms: self.ttl_ms,
            nonce_id: self.nonce_id.clone(),
            message_id: self.message_id.clone(),
            version: self.version,
        }
    }

    pub fn signing_bytes(&self) -> Vec<u8> {
        // JSON keeps this demo-friendly; if we need canonical encoding later, we can swap it.
        serde_json::to_vec(&self.signing_payload()).expect("serialize signing payload")
    }

    /// Transaction ID (hash)
    pub fn id(&self) -> String {
        crate::core::hash::tx_hash(self)
    }

    /// Get size.
    pub fn size(&self) -> usize {
        serde_json::to_vec(self).unwrap_or_default().len()
    }

    /// Check if the transaction is a coinbase (reward) transaction.
    pub fn is_coinbase(&self) -> bool {
        self.from == "SYSTEM"
    }

    /// Basic sanity checks (Week 1/early Week 2 demo).
    ///
    /// Note: signatures/balances/nonces will be enforced later.
    pub fn validate_basic(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.from.trim().is_empty(), "tx.from must be non-empty");
        anyhow::ensure!(!self.to.trim().is_empty(), "tx.to must be non-empty");
        anyhow::ensure!(self.from != self.to, "tx.from and tx.to must differ");
        // Minimum amount of 1 unit (prevents dust/negative amounts)
        anyhow::ensure!(self.amount > 0, "tx.amount must be > 0");
        // Sequence should be non-negative (u32 handles this, but let's ensure it's not wrapped or used incorrectly)
        // Add more sequence validation if rules emerge.

        if let Some(memo) = &self.memo {
            anyhow::ensure!(memo.len() <= 128, "memo must be <= 128 characters");
        }

        anyhow::ensure!(self.version > 0, "tx.version must be > 0");

        Ok(())
    }

    /// Basic tx validation for accepting into the mempool or a block.
    pub fn validate_accept(&self) -> anyhow::Result<()> {
        self.validate_basic()?;
        self.verify_signature_if_present()?;
        Ok(())
    }

    /// Verify signature if present.
    ///
    /// Rules (for now):
    /// - If both `pubkey_hex` and `signature_b64` are present, verify strictly.
    /// - If neither is present, treat as unsigned and accept.
    /// - If only one is present, reject.
    pub fn verify_signature_if_present(&self) -> anyhow::Result<()> {
        match (&self.pubkey_hex, &self.signature_b64) {
            (None, None) => Ok(()),
            (Some(_), None) | (None, Some(_)) => {
                anyhow::bail!("tx signature fields must be both present or both absent")
            }
            (Some(pk_hex), Some(sig_b64)) => {
                anyhow::ensure!(
                    self.from == *pk_hex,
                    "signed tx must use from=<pubkey_hex> (from={} pubkey_hex={})",
                    self.from,
                    pk_hex
                );

                let vk = crate::core::crypto::verifying_key_from_hex(pk_hex)?;
                crate::core::crypto::verify_bytes(&vk, &self.signing_bytes(), sig_b64)?;
                Ok(())
            }
        }
    }
}

/// Block = header + transactions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<Transaction>,
}

impl Block {
    pub fn is_coinbase(&self) -> bool {
        self.txs.first().is_some_and(|tx| tx.is_coinbase())
    }

    pub fn total_reward(&self) -> u64 {
        let block_reward = 50;
        let fees: u64 = self
            .txs
            .iter()
            .filter(|tx| !tx.is_coinbase())
            .map(|tx| tx.fee)
            .sum();
        block_reward + fees
    }

    /// Calculate the size of the block in bytes when serialized.
    pub fn size(&self) -> usize {
        serde_json::to_vec(self).unwrap_or_default().len()
    }

    /// Returns true if the block's header satisfies the given PoW difficulty.
    pub fn is_valid_pow(&self, difficulty: u32) -> bool {
        self.header.verify_pow(difficulty).is_ok()
    }

    /// Basic block validation against a previous header.
    pub fn validate_with_prev(
        &self,
        prev_header: &BlockHeader,
        difficulty: u32,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.header.prev_hash == prev_header.hash(),
            "invalid prev_hash: {} (expected {})",
            self.header.prev_hash,
            prev_header.hash()
        );
        anyhow::ensure!(
            self.header.timestamp_ms >= prev_header.timestamp_ms,
            "timestamp cannot go backward: {} (prev: {})",
            self.header.timestamp_ms,
            prev_header.timestamp_ms
        );
        self.header.verify_pow(difficulty)?;
        Ok(())
    }
}
