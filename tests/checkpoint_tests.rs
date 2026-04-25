use rusty_chain::core::chain::Chain;

#[test]
fn test_checkpoints_genesis() {
    let chain = Chain::new_genesis();
    assert_eq!(chain.checkpoints.len(), 1);
    assert!(chain.checkpoints.contains_key(&0));
    assert!(chain.validate_checkpoints().is_ok());
}

#[test]
fn test_manual_checkpointing() {
    let mut chain = Chain::new_genesis();

    // Mine a few blocks
    for _ in 0..5 {
        chain.mine_empty_block(3).unwrap();
    }

    let height = chain.height();
    chain.add_checkpoint();

    assert_eq!(chain.checkpoints.len(), 2);
    assert!(chain.checkpoints.contains_key(&height));
    assert!(chain.validate_checkpoints().is_ok());
}

#[test]
fn test_checkpoint_failure() {
    let mut chain = Chain::new_genesis();
    chain.add_checkpoint();

    // Manually corrupt a block
    if let Some(block) = chain.blocks.get_mut(0) {
        block.header.nonce = 999;
    }

    assert!(chain.validate_checkpoints().is_err());
}

#[test]
fn test_automatic_checkpointing() {
    let mut chain = Chain::new_genesis();
    chain.pow_difficulty = 3;

    // append_block triggers auto-checkpoint every 10 blocks
    for _i in 1..=21 {
        let prev = chain.blocks.last().unwrap();
        let prev_hash = prev.header.hash();
        let header = rusty_chain::core::types::BlockHeader {
            prev_hash,
            timestamp_ms: rusty_chain::core::time::now_ms(),
            nonce: 0,
            merkle_root: rusty_chain::core::chain::merkle_root(&[]),
        };
        let mut block = rusty_chain::core::types::Block {
            header,
            txs: vec![],
        };

        // Find valid PoW for difficulty 3
        let mut n = 0;
        loop {
            block.header.nonce = n;
            let h = block.header.hash();
            if rusty_chain::core::chain::pow_ok(&h, 3) {
                break;
            }
            n += 1;
        }

        chain.append_block(block).unwrap();
    }

    // Checkpoints at 0, 10, 20
    assert_eq!(chain.checkpoints.len(), 3);
    assert!(chain.checkpoints.contains_key(&0));
    assert!(chain.checkpoints.contains_key(&10));
    assert!(chain.checkpoints.contains_key(&20));
    assert!(chain.validate_checkpoints().is_ok());
}

#[test]
fn test_get_checkpoint_helpers() {
    let mut chain = Chain::new_genesis();
    let genesis_hash = chain.blocks[0].header.hash();

    assert_eq!(chain.get_checkpoint_at(0), Some(genesis_hash.clone()));
    assert_eq!(chain.get_last_checkpoint(), Some((0, genesis_hash)));

    // Add checkpoint
    for _ in 0..5 {
        chain.mine_empty_block(3).unwrap();
    }
    let height = chain.height();
    let hash = chain.tip_hash();
    chain.add_checkpoint();

    assert_eq!(chain.get_checkpoint_at(height), Some(hash.clone()));
    assert_eq!(chain.get_last_checkpoint(), Some((height, hash)));
}

#[test]
fn test_header_stateless_verification() {
    let mut chain = Chain::new_genesis();
    chain.pow_difficulty = 3;
    let _header = &chain.blocks[0].header;
    // Genesis header might not have valid PoW for difficulty 3 if created with different difficulty
    // Let's mine one block to be sure
    let block = chain.mine_empty_block(3).unwrap();
    assert!(block.header.verify_pow(3).is_ok());
    // Should fail for impossible difficulty
    assert!(block.header.verify_pow(64).is_err());
}

#[test]
fn test_block_stateless_verification() {
    let mut chain = Chain::new_genesis();
    let prev_header = chain.blocks[0].header.clone();
    let block = chain.mine_empty_block(3).unwrap();

    // Valid block against its actual parent
    assert!(block.validate_with_prev(&prev_header, 3).is_ok());

    // Fail due to wrong parent
    let mut wrong_prev = prev_header.clone();
    wrong_prev.nonce = 12345;
    assert!(block.validate_with_prev(&wrong_prev, 3).is_err());

    // Fail due to difficulty mismatch
    assert!(block.validate_with_prev(&prev_header, 64).is_err());
}
