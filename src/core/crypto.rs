use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;

/// Generate a fresh ed25519 keypair.
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    (sk, vk)
}

pub fn verifying_key_to_hex(vk: &VerifyingKey) -> String {
    hex::encode(vk.to_bytes())
}

pub fn verifying_key_from_hex(s: &str) -> anyhow::Result<VerifyingKey> {
    let bytes = hex::decode(s)?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("verifying key must be 32 bytes (hex len=64)"))?;
    Ok(VerifyingKey::from_bytes(&arr)?)
}

pub fn signing_key_to_base64(sk: &SigningKey) -> String {
    B64.encode(sk.to_bytes())
}

pub fn signing_key_from_base64(s: &str) -> anyhow::Result<SigningKey> {
    let bytes = B64.decode(s)?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("signing key must be 32 bytes (base64)"))?;
    Ok(SigningKey::from_bytes(&arr))
}

pub fn sign_bytes(sk: &SigningKey, msg: &[u8]) -> String {
    let sig: Signature = sk.sign(msg);
    B64.encode(sig.to_bytes())
}

pub fn verify_bytes(vk: &VerifyingKey, msg: &[u8], sig_b64: &str) -> anyhow::Result<()> {
    let sig_bytes = B64.decode(sig_b64)?;
    let sig = Signature::from_slice(&sig_bytes)?;
    vk.verify_strict(msg, &sig)?;
    Ok(())
}
