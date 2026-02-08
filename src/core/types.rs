use serde::{Deserialize, Serialize};

/// Basic block header (minimal, demo-oriented).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeader {
    pub prev_hash: String,
    pub timestamp_ms: u64,
    pub nonce: u64,
    pub merkle_root: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxSignPayload {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
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
        }
    }

    pub fn new_with_fee(
        from: impl Into<String>,
        to: impl Into<String>,
        amount: u64,
        fee: u64,
        nonce: u64,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            amount,
            fee,
            nonce,
            pubkey_hex: None,
            signature_b64: None,
        }
    }

    pub fn signing_payload(&self) -> TxSignPayload {
        TxSignPayload {
            from: self.from.clone(),
            to: self.to.clone(),
            amount: self.amount,
            fee: self.fee,
            nonce: self.nonce,
        }
    }

    pub fn signing_bytes(&self) -> Vec<u8> {
        // JSON keeps this demo-friendly; if we need canonical encoding later, we can swap it.
        serde_json::to_vec(&self.signing_payload()).expect("serialize signing payload")
    }

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
        anyhow::ensure!(self.amount > 0, "tx.amount must be > 0");
        
        if self.is_coinbase() {
             // Coinbase rules: no signature required (for now), but maybe nonce should be block height?
             // For simplicity, we just allow it. The state application logic will ensure it's only valid as the first tx in a block.
        }
        
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
