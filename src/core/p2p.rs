use crate::core::chain::Chain;
use crate::core::mempool::Mempool;
use crate::core::network::Message;
use anyhow::Context;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};

/// Commands that can be sent to the peer handler
#[derive(Debug)]
pub enum PeerCmd {
    SendMessage(Message),
}

/// Shared node state for concurrent peer handling
pub struct NodeState {
    pub peers: Vec<SocketAddr>,
    pub peer_senders: Vec<mpsc::UnboundedSender<PeerCmd>>,
    pub seen_messages: HashSet<String>,
    pub chain: Chain,
    pub mempool: Mempool,
}

pub struct P2PNode {
    pub addr: SocketAddr,
    pub state: Arc<Mutex<NodeState>>,
}

impl P2PNode {
    pub fn new(addr: SocketAddr, chain: Chain, mempool: Mempool) -> Self {
        Self {
            addr,
            state: Arc::new(Mutex::new(NodeState {
                peers: Vec::new(),
                peer_senders: Vec::new(),
                seen_messages: HashSet::new(),
                chain,
                mempool,
            })),
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.addr)
            .await
            .context("Failed to bind P2P listener")?;
        println!("P2P server listening on {}", self.addr);

        let node_state = Arc::clone(&self.state);
        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    println!("New inbound connection from {}", peer_addr);
                    let state = Arc::clone(&node_state);
                    let node_handle = self.clone_handle();
                    tokio::spawn(async move {
                        if let Err(e) = handle_peer(stream, peer_addr, state, node_handle).await {
                            eprintln!("Peer {} disconnected with error: {:?}", peer_addr, e);
                        } else {
                            println!("Peer {} disconnected gracefully", peer_addr);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    pub async fn connect(&self, target: SocketAddr, best_height: u64) -> anyhow::Result<()> {
        println!("Connecting to {}...", target);
        let stream = match TcpStream::connect(target).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to connect to {}: {}", target, e);
                return Err(e.into());
            }
        };
        println!("Connected to outbound peer {}", target);

        let mut stream = stream;

        // Send initial Handshake
        Message::Handshake {
            version: 1,
            best_height,
        }
        .send_async(&mut stream)
        .await?;

        let state = Arc::clone(&self.state);
        let node_handle = self.clone_handle();
        tokio::spawn(async move {
            if let Err(e) = handle_peer(stream, target, state, node_handle).await {
                eprintln!("Error handling peer {}: {}", target, e);
            }
        });

        Ok(())
    }

    pub async fn broadcast(&self, msg: Message) -> anyhow::Result<()> {
        let state = self.state.lock().await;
        for tx in &state.peer_senders {
            let _ = tx.send(PeerCmd::SendMessage(msg.clone()));
        }
        Ok(())
    }

    /// Broadcast a message to all peers except the specified one
    #[allow(clippy::collapsible_if)]
    pub async fn broadcast_except(&self, msg: Message, except: SocketAddr) -> anyhow::Result<()> {
        let state = self.state.lock().await;
        for (i, addr) in state.peers.iter().enumerate() {
            if *addr != except {
                if let Some(tx) = state.peer_senders.get(i) {
                    let _ = tx.send(PeerCmd::SendMessage(msg.clone()));
                }
            }
        }
        Ok(())
    }

    fn clone_handle(&self) -> P2PNodeHandle {
        P2PNodeHandle {
            state: Arc::clone(&self.state),
        }
    }
}

/// A lightweight handle to the P2PNode to avoid circular Arc or complex lifetimes in handlers
#[derive(Clone)]
pub struct P2PNodeHandle {
    pub state: Arc<Mutex<NodeState>>,
}

impl P2PNodeHandle {
    #[allow(clippy::collapsible_if)]
    pub async fn broadcast_except(&self, msg: Message, except: SocketAddr) -> anyhow::Result<()> {
        let state = self.state.lock().await;
        for (i, addr) in state.peers.iter().enumerate() {
            if *addr != except {
                if let Some(tx) = state.peer_senders.get(i) {
                    let _ = tx.send(PeerCmd::SendMessage(msg.clone()));
                }
            }
        }
        Ok(())
    }

    pub async fn mark_seen(&self, id: String) -> bool {
        let mut state = self.state.lock().await;
        state.seen_messages.insert(id)
    }

    pub async fn is_seen(&self, id: &str) -> bool {
        let state = self.state.lock().await;
        state.seen_messages.contains(id)
    }

    pub async fn get_peer_count(&self) -> usize {
        let state = self.state.lock().await;
        state.peers.len()
    }

    #[allow(clippy::collapsible_if)]
    pub async fn send_to(&self, target: SocketAddr, msg: Message) -> anyhow::Result<()> {
        let state = self.state.lock().await;
        if let Some(pos) = state.peers.iter().position(|&p| p == target) {
            if let Some(tx) = state.peer_senders.get(pos) {
                let _ = tx.send(PeerCmd::SendMessage(msg));
            }
        }
        Ok(())
    }

    pub async fn get_headers(
        &self,
        start_height: u64,
        limit: u32,
    ) -> Vec<crate::core::types::BlockHeader> {
        let state = self.state.lock().await;
        state
            .chain
            .blocks
            .iter()
            .skip(start_height as usize)
            .take(limit as usize)
            .map(|b| b.header.clone())
            .collect()
    }

    pub async fn get_blocks_by_hash(&self, hashes: Vec<String>) -> Vec<crate::core::types::Block> {
        let state = self.state.lock().await;
        let mut results = Vec::new();
        for hash in hashes {
            if let Some(block) = state
                .chain
                .blocks
                .iter()
                .find(|b| crate::core::chain::hash_block(b) == hash)
            {
                results.push(block.clone());
            }
        }
        results
    }

    pub async fn process_message(&self, msg: Message, from: SocketAddr) -> anyhow::Result<()> {
        match msg {
            Message::Ping => {
                println!("Responding to Ping from {}", from);
                self.send_to(from, Message::Pong).await?;
            }
            Message::Handshake {
                version,
                best_height,
            } => {
                println!(
                    "Handshake from {}: version={}, height={}",
                    from, version, best_height
                );
                // If they are ahead, we might want to sync headers later.
                // For now, just respond with our own status if we were the ones receiving.
                // In a real handshake, both sides exchange their heights.
            }
            Message::NewTransaction(tx) => {
                let tx_id = tx.id();
                if self.mark_seen(tx_id.clone()).await {
                    println!("Gossip: New Transaction {} from {}", tx_id, from);
                    // 1. Validate tx
                    let mut state = self.state.lock().await;
                    if let Err(e) = state.chain.validate_transaction(&tx) {
                        println!("Invalid transaction {} from {}: {}", tx_id, from, e);
                        return Ok(());
                    }
                    // 2. Add to mempool
                    let base_nonce = state.chain.next_nonce_for(&tx.from);
                    if let Err(e) = state.mempool.add_tx_checked(tx.clone(), base_nonce) {
                        println!("Failed to add tx {} to mempool: {}", tx_id, e);
                        return Ok(());
                    }
                    drop(state);

                    // 3. Re-gossip
                    self.broadcast_except(Message::NewTransaction(tx), from)
                        .await?;
                }
            }
            Message::NewBlock(block) => {
                let blk_id = block.header.hash();
                if self.mark_seen(blk_id.clone()).await {
                    println!("Gossip: New Block {} from {}", blk_id, from);
                    // 1. Validate block
                    let mut state = self.state.lock().await;
                    if let Err(e) = state.chain.validate_block(&block) {
                        println!("Invalid block {} from {}: {}", blk_id, from, e);
                        return Ok(());
                    }
                    // 2. Append to chain
                    if let Err(e) = state.chain.append_block(block.clone()) {
                        println!("Failed to append block {} to chain: {}", blk_id, e);
                        return Ok(());
                    }
                    // 3. Clear mempool txs
                    for tx in &block.txs {
                        state.mempool.remove_tx(&tx.id());
                    }
                    // 4. Re-gossip
                    drop(state);
                    self.broadcast_except(Message::NewBlock(block), from)
                        .await?;
                }
            }
            Message::GetHeaders {
                start_height,
                limit,
            } => {
                let headers = self.get_headers(start_height, limit).await;
                self.send_to(from, Message::Headers(headers)).await?;
            }
            Message::GetData { block_hashes } => {
                let blocks = self.get_blocks_by_hash(block_hashes).await;
                self.send_to(from, Message::Blocks(blocks)).await?;
            }
            _ => {
                println!("Received unhandled message from {}: {:?}", from, msg);
            }
        }
        Ok(())
    }
}

async fn handle_peer(
    stream: TcpStream,
    addr: SocketAddr,
    state: Arc<Mutex<NodeState>>,
    node: P2PNodeHandle,
) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<PeerCmd>();

    // Add to peer list
    {
        let mut s = state.lock().await;
        if !s.peers.contains(&addr) {
            s.peers.push(addr);
            s.peer_senders.push(tx);
        }
    }

    println!("Starting message loop for {}", addr);
    let (reader, writer) = stream.into_split();
    let mut reader = reader;
    let writer = Arc::new(Mutex::new(writer));

    let peer_reader = async move {
        loop {
            let msg = Message::decode_async(&mut reader)
                .await
                .context("Failed to decode peer message")?;
            println!("Received message from {}: {:?}", addr, msg);

            match msg {
                Message::Pong => {
                    println!("Received Pong from {}", addr);
                }
                m => {
                    node.process_message(m, addr).await?;
                }
            }
        }
    };

    let peer_writer = async move {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                PeerCmd::SendMessage(msg) => {
                    let mut w = writer.lock().await;
                    msg.send_async(&mut *w).await?;
                }
            }
        }
        anyhow::Ok(())
    };

    let res = tokio::select! {
        r = peer_reader => r,
        w = peer_writer => w,
    };

    // Remove from peer list
    {
        let mut s = state.lock().await;
        if let Some(pos) = s.peers.iter().position(|&p| p == addr) {
            s.peers.remove(pos);
            s.peer_senders.remove(pos);
        }
    }

    res
}
