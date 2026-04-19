use rusty_chain::core::chain::Chain;
use rusty_chain::core::types::Transaction;

#[test]
fn genesis_state_is_empty() {
    let c = Chain::new_genesis();
    let state = c.compute_state().unwrap();
    assert_eq!(state.get_balance("alice"), 0);
}

#[test]
fn coinbase_tx_increases_balance() {
    let mut c = Chain::new_genesis();

    // Construct a coinbase tx
    let mut coinbase = Transaction::default();
    coinbase.from = "SYSTEM".to_string();
    coinbase.to = "alice".to_string();
    coinbase.amount = 50;
    coinbase.nonce = 1;
    coinbase.timestamp_ms = 0;
    coinbase.priority = 0;
    coinbase.is_minable = true;

    c.mine_block(vec![coinbase], 1, None).unwrap();

    let state = c.compute_state().unwrap();
    assert_eq!(state.get_balance("alice"), 50);
}

#[test]
fn transfer_tx_updates_balances() {
    let mut c = Chain::new_genesis();

    // 1. Mine coinbase to Alice
    let mut coinbase = Transaction::default();
    coinbase.from = "SYSTEM".to_string();
    coinbase.to = "alice".to_string();
    coinbase.amount = 50;
    coinbase.nonce = 1;
    coinbase.timestamp_ms = 0;
    coinbase.priority = 0;
    coinbase.is_minable = true;
    c.mine_block(vec![coinbase], 1, None).unwrap();

    // 2. Mine transfer Alice -> Bob
    let tx = Transaction::new("alice", "bob", 10, 0);
    c.mine_block(vec![tx], 1, None).unwrap();

    let state = c.compute_state().unwrap();
    assert_eq!(state.get_balance("alice"), 40);
    assert_eq!(state.get_balance("bob"), 10);
    assert_eq!(state.get_nonce("alice"), 1);
}

#[test]
fn insufficient_balance_makes_chain_invalid() {
    let mut c = Chain::new_genesis();

    // Alice has 0. Tries to send 10.
    let tx = Transaction::new("alice", "bob", 10, 0);
    let err = c.mine_block(vec![tx], 1, None).unwrap_err();

    // mine_block should fail
    assert!(
        format!("{:?}", err).contains("Insufficient balance"),
        "got error: {:?}",
        err
    );
}

#[test]
fn invalid_nonce_makes_chain_invalid() {
    let mut c = Chain::new_genesis();

    // Fund Alice
    let mut coinbase = Transaction::default();
    coinbase.from = "SYSTEM".to_string();
    coinbase.to = "alice".to_string();
    coinbase.amount = 50;
    coinbase.nonce = 1;
    coinbase.timestamp_ms = 0;
    coinbase.priority = 0;
    coinbase.is_minable = true;
    c.mine_block(vec![coinbase], 1, None).unwrap();

    // Alice sends with nonce 5 (expected 0)
    let tx = Transaction::new("alice", "bob", 10, 5);
    let err = c.mine_block(vec![tx], 1, None).unwrap_err();

    assert!(
        format!("{:?}", err).contains("Invalid nonce"),
        "got error: {:?}",
        err
    );
}

#[test]
fn fees_are_collected_by_miner() {
    let mut c = Chain::new_genesis();

    // 1. Give Alice some starting funds (100)
    let mut cb = Transaction::default();
    cb.from = "SYSTEM".to_string();
    cb.to = "alice".to_string();
    cb.amount = 50;
    cb.nonce = 1;
    cb.timestamp_ms = 0;
    cb.priority = 0;
    cb.is_minable = true;
    c.mine_block(vec![cb], 1, None).unwrap();

    // 2. Alice sends 10 to Bob with 5 fee. Miner is 'charlie'.
    let tx = Transaction::new_with_fee("alice", "bob", 10, 5, 0, 0);
    c.mine_block(vec![tx], 1, Some("charlie")).unwrap();

    let state = c.compute_state().unwrap();

    // Alice: 50 - 10 - 5 = 35
    assert_eq!(state.get_balance("alice"), 35);
    // Bob: 10
    assert_eq!(state.get_balance("bob"), 10);
    // Charlie (miner): 50 (block reward) + 5 (fee) = 55
    assert_eq!(state.get_balance("charlie"), 55);
}

#[test]
fn insufficient_balance_for_fee_fails() {
    let mut c = Chain::new_genesis();

    // Alice has 50. Tries to send 50 with 1 fee (needs 51).
    let mut cb = Transaction::default();
    cb.from = "SYSTEM".to_string();
    cb.to = "alice".to_string();
    cb.amount = 50;
    cb.nonce = 1;
    cb.timestamp_ms = 0;
    cb.priority = 0;
    cb.is_minable = true;
    c.mine_block(vec![cb], 1, None).unwrap();

    let tx = Transaction::new_with_fee("alice", "bob", 50, 1, 0, 0);
    let err = c.mine_block(vec![tx], 1, None).unwrap_err();

    assert!(
        format!("{:?}", err).contains("Insufficient balance"),
        "expected insufficient balance error, got: {:?}",
        err
    );
}

#[test]
fn saturating_math_prevents_underflow_panic() {
    let mut c = Chain::new_genesis();

    // Construct a tx that would normally underflow if not for saturating math
    // (Though validate_tx usually catches this, apply_tx should be robust)
    let tx = Transaction::new("alice", "bob", 100, 0);
    let _ = c.mine_block(vec![tx], 1, None);

    // We expect validation to catch it, but we want to ensure compute_state doesn't panic
    let _ = c.compute_state();
}
