use rusty_chain::core::chain::Chain;

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
