use rusty_chain::core::mempool::Mempool;
use rusty_chain::core::types::Transaction;

#[test]
fn validate_basic_accepts_ok_tx() {
    let tx = Transaction::new("alice", "bob", 1, 0);

    tx.validate_basic().unwrap();
}

#[test]
fn validate_basic_rejects_empty_from() {
    let tx = Transaction::new("", "bob", 1, 0);

    let err = tx.validate_basic().unwrap_err().to_string();
    assert!(err.contains("from"), "unexpected error: {err}");
}

#[test]
fn validate_basic_rejects_zero_amount() {
    let tx = Transaction::new("alice", "bob", 0, 0);

    let err = tx.validate_basic().unwrap_err().to_string();
    assert!(err.contains("amount"), "unexpected error: {err}");
}

#[test]
fn mempool_add_rejects_invalid_tx() {
    let mut mp = Mempool::default();
    let tx = Transaction::new("alice", "alice", 1, 0);

    let err = mp.add_tx(tx).unwrap_err().to_string();
    assert!(err.contains("must differ"), "unexpected error: {err}");
    assert_eq!(mp.txs.len(), 0);
}

#[test]
fn mempool_add_rejects_duplicate_tx() {
    let mut mp = Mempool::default();
    let tx = Transaction::new("alice", "bob", 1, 0);

    mp.add_tx(tx.clone()).unwrap();

    let err = mp.add_tx(tx).unwrap_err().to_string();
    assert!(err.contains("duplicate tx"), "unexpected error: {err}");
    assert_eq!(mp.txs.len(), 1);
}

#[test]
fn mempool_add_rejects_signed_tx_with_from_mismatch() {
    use rusty_chain::core::crypto::{generate_keypair, sign_bytes, verifying_key_to_hex};

    let (sk, vk) = generate_keypair();

    let mut tx = Transaction::new("alice", "bob", 1, 0);
    let sig = sign_bytes(&sk, &tx.signing_bytes());
    tx.pubkey_hex = Some(verifying_key_to_hex(&vk));
    tx.signature_b64 = Some(sig);

    let mut mp = Mempool::default();
    let err = mp.add_tx(tx).unwrap_err().to_string();
    assert!(err.contains("from=<pubkey_hex>"), "unexpected error: {err}");
    assert_eq!(mp.txs.len(), 0);
}
