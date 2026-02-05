use crate::core::crypto::{
    generate_keypair, signing_key_from_base64, signing_key_to_base64, verifying_key_to_hex,
};
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyFile {
    /// ed25519 secret key (32 bytes), base64-encoded.
    pub signing_key_b64: String,

    /// ed25519 public key (32 bytes), hex-encoded.
    pub verifying_key_hex: String,
}

impl KeyFile {
    pub fn keys_dir() -> PathBuf {
        PathBuf::from("data/keys")
    }

    pub fn path_for(name: &str) -> PathBuf {
        Self::keys_dir().join(format!("{name}.json"))
    }

    pub fn generate() -> (Self, SigningKey, VerifyingKey) {
        let (sk, vk) = generate_keypair();
        let file = Self {
            signing_key_b64: signing_key_to_base64(&sk),
            verifying_key_hex: verifying_key_to_hex(&vk),
        };
        (file, sk, vk)
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let s = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&s)?)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let s = serde_json::to_string_pretty(self)?;
        fs::write(path, s)?;
        Ok(())
    }

    pub fn signing_key(&self) -> anyhow::Result<SigningKey> {
        signing_key_from_base64(&self.signing_key_b64)
    }
}
