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
    let coinbase = Transaction {
        from: "SYSTEM".to_string(),
        to: "alice".to_string(),
        amount: 50,
        nonce: 0, // coinbase nonce doesn't matter for now
        pubkey_hex: None,
        signature_b64: None,
    };
    
    c.mine_block(vec![coinbase], 1, None).unwrap();
    
    let state = c.compute_state().unwrap();
    assert_eq!(state.get_balance("alice"), 50);
}

#[test]
fn transfer_tx_updates_balances() {
    let mut c = Chain::new_genesis();
    
    // 1. Mine coinbase to Alice
    let coinbase = Transaction {
        from: "SYSTEM".to_string(),
        to: "alice".to_string(),
        amount: 50,
        nonce: 0,
        pubkey_hex: None,
        signature_b64: None,
    };
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
    c.mine_block(vec![tx], 1, None).unwrap();
    
    // validate should fail
    let err = c.validate().unwrap_err();
    assert!(format!("{:?}", err).contains("Insufficient balance"), "got error: {:?}", err);
}

#[test]
fn invalid_nonce_makes_chain_invalid() {
    let mut c = Chain::new_genesis();
    
    // Fund Alice
    let coinbase = Transaction {
        from: "SYSTEM".to_string(),
        to: "alice".to_string(),
        amount: 50,
        nonce: 0,
        pubkey_hex: None,
        signature_b64: None,
    };
    c.mine_block(vec![coinbase], 1, None).unwrap();
    
    // Alice sends with nonce 5 (expected 0)
    let tx = Transaction::new("alice", "bob", 10, 5);
    c.mine_block(vec![tx], 1, None).unwrap();
    
    let err = c.validate().unwrap_err();
    assert!(format!("{:?}", err).contains("Invalid nonce"), "got error: {:?}", err);
}
