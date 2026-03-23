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
    
    // append_block triggers auto-checkpoint every 10 blocks
    for _i in 1..=21 {
        let prev = chain.blocks.last().unwrap();
        let prev_hash = rusty_chain::core::chain::hash_block(prev);
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
            let h = rusty_chain::core::chain::hash_block(&block);
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
    let genesis_hash = rusty_chain::core::chain::hash_block(&chain.blocks[0]);
    
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
