use rusty_chain::core::chain::Chain;
use rusty_chain::core::mempool::Mempool;
use rusty_chain::core::p2p::{P2PNode, P2PNodeHandle};
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::test]
async fn test_peer_whitelisting_bypasses_reputation() {
    let chain = Chain::new_genesis();
    let mempool = Mempool::new();
    let node = P2PNode::new(
        "127.0.0.1:9000".parse().unwrap(),
        chain,
        mempool,
        None,
        None,
    );
    let handle = P2PNodeHandle {
        state: Arc::clone(&node.state),
    };

    let peer_addr: SocketAddr = "1.2.3.4:5678".parse().unwrap();

    // 1. Initially reputation is 0
    assert_eq!(handle.get_reputation(peer_addr).await, 0);

    // 2. Update reputation normally
    handle.update_reputation(peer_addr, -10).await;
    assert_eq!(handle.get_reputation(peer_addr).await, -10);

    // 3. Whitelist the peer
    handle.whitelist_peer(peer_addr).await;

    // 4. Reputation should be reset (or at least ignored)
    // In our implementation we remove it from the map
    assert_eq!(handle.get_reputation(peer_addr).await, 0);

    // 5. Further negative updates should be ignored
    handle.update_reputation(peer_addr, -50).await;
    assert_eq!(handle.get_reputation(peer_addr).await, 0);

    // 6. Verify whitelisted status
    let state = node.state.lock().await;
    assert!(state.whitelisted_peers.contains(&peer_addr));
}
