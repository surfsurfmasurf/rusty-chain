use crate::core::types::Transaction;
use sha2::{Digest, Sha256};

/// SHA-256 hex digest.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    hex::encode(out)
}

/// Stable transaction id (demo-friendly): SHA-256 of JSON-serialized tx.
pub fn tx_hash(tx: &Transaction) -> String {
    let bytes = serde_json::to_vec(tx).expect("serialize tx");
    sha256_hex(&bytes)
}
