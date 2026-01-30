use rusty_chain::core::chain::{Chain, hash_block, pow_ok};

#[test]
fn genesis_has_height_zero() {
    let c = Chain::new_genesis();
    assert_eq!(c.height(), 0);
    assert_eq!(c.blocks.len(), 1);
}

#[test]
fn save_then_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("chain.json");

    let c = Chain::new_genesis();
    c.save(&path).unwrap();

    let loaded = Chain::load(&path).unwrap();
    assert_eq!(loaded.height(), 0);
    assert_eq!(loaded.tip_hash(), c.tip_hash());
}

#[test]
fn validate_accepts_genesis() {
    let c = Chain::new_genesis();
    c.validate().unwrap();
}

#[test]
fn validate_rejects_broken_prev_hash_linkage() {
    let mut c = Chain::new_genesis();
    c.blocks.push(c.blocks[0].clone());

    // Tamper with linkage.
    c.blocks[1].header.prev_hash = "deadbeef".to_string();

    let err = c.validate().unwrap_err().to_string();
    assert!(err.contains("prev_hash"), "unexpected error: {err}");
}

#[test]
fn mine_produces_pow_ok_hash() {
    let mut c = Chain::new_genesis();
    let difficulty = 2;

    let mined = c.mine_empty_block(difficulty).unwrap();
    c.validate().unwrap();

    let h = hash_block(&mined);
    assert!(pow_ok(&h, difficulty), "expected pow_ok for hash={h}");
}
