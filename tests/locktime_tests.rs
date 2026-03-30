use rusty_chain::core::chain::Chain;
use rusty_chain::core::types::Transaction;

#[test]
fn test_locktime_validation() {
    let mut chain = Chain::new_genesis();
    let alice = "alice";
    let bob = "bob";

    // 0. Give Alice some coins
    // Mine a block where Alice is the miner to get reward (50 coins)
    chain.mine_block(vec![], 1, Some(alice)).unwrap(); // height 1
    chain.mine_empty_block(1).unwrap(); // height 2
    chain.mine_empty_block(1).unwrap(); // height 3
    
    // Alice now has 50 coins. Nonce for Alice is 0 because reward txs don't count towards sender nonces.
    let alice_nonce = chain.compute_state().unwrap().get_nonce(alice);
    assert_eq!(alice_nonce, 0);

    // 1. Transaction locked in the future (height 6)
    let mut tx = Transaction::new(alice, bob, 10, 0);
    tx.locktime = Some(6);

    // Should fail validation at current height (3)
    let res = chain.validate_transaction(&tx);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("Transaction locked until height 6"));

    // 2. Mine blocks to reach height 6
    chain.mine_empty_block(1).unwrap(); // height 4
    chain.mine_empty_block(1).unwrap(); // height 5
    chain.mine_empty_block(1).unwrap(); // height 6

    // Alice now has balance and we are at height 6 >= locktime 6.
    chain.validate_transaction(&tx).expect("Should be valid now");

    // 3. Try a transaction locked at height 8 (we are at height 6)
    // Nonce must be 0 because we haven't applied tx yet (validate_transaction is read-only)
    let mut tx2 = Transaction::new(alice, bob, 5, 0);
    tx2.locktime = Some(8);
    let res = chain.validate_transaction(&tx2);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("Transaction locked until height 8"));

    // Mine another block to reach height 8
    chain.mine_empty_block(1).unwrap(); // height 7
    chain.mine_empty_block(1).unwrap(); // height 8
    chain.validate_transaction(&tx2).expect("Should be valid at height 8");
}
