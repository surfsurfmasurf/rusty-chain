use rusty_chain::core::crypto::{generate_keypair, sign_bytes, verifying_key_to_hex};
use rusty_chain::core::types::Transaction;

#[test]
fn signed_tx_verifies() {
    let (sk, vk) = generate_keypair();

    let mut tx = Transaction::new("alice", "bob", 10, 0);
    let sig = sign_bytes(&sk, &tx.signing_bytes());

    tx.pubkey_hex = Some(verifying_key_to_hex(&vk));
    tx.signature_b64 = Some(sig);

    tx.verify_signature_if_present().unwrap();
}

#[test]
fn signed_tx_rejects_tampering() {
    let (sk, vk) = generate_keypair();

    let mut tx = Transaction::new("alice", "bob", 10, 0);
    let sig = sign_bytes(&sk, &tx.signing_bytes());

    tx.pubkey_hex = Some(verifying_key_to_hex(&vk));
    tx.signature_b64 = Some(sig);

    // Tamper after signing.
    tx.amount = 999;

    let err = tx.verify_signature_if_present().unwrap_err().to_string();
    assert!(
        err.contains("signature") || err.contains("Verification"),
        "err={err}"
    );
}

#[test]
fn tx_signature_requires_both_fields() {
    let tx = Transaction {
        pubkey_hex: Some("00".repeat(32)),
        ..Transaction::new("alice", "bob", 1, 0)
    };

    let err = tx.verify_signature_if_present().unwrap_err().to_string();
    assert!(err.contains("both"), "err={err}");
}
