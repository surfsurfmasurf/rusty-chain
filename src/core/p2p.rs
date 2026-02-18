use crate::core::network::Message;
use anyhow::Context;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};

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
}

pub struct P2PNode {
    pub addr: SocketAddr,
    pub state: Arc<Mutex<NodeState>>,
}

impl P2PNode {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            state: Arc::new(Mutex::new(NodeState {
                peers: Vec::new(),
                peer_senders: Vec::new(),
                seen_messages: HashSet::new(),
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
}

async fn handle_peer(
    mut stream: TcpStream,
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
    let (mut reader, mut writer) = stream.split();

    let peer_reader = async {
        loop {
            let msg = Message::decode_async(&mut reader)
                .await
                .context("Failed to decode peer message")?;
            println!("Received message from {}: {:?}", addr, msg);

            match msg {
                Message::Ping => {
                    println!("Responding to Ping from {}", addr);
                    Message::Pong.send_async(&mut writer).await?;
                }
                Message::Pong => {
                    println!("Received Pong from {}", addr);
                }
                Message::Handshake {
                    version,
                    best_height,
                } => {
                    println!(
                        "Handshake from {}: version={}, height={}",
                        addr, version, best_height
                    );
                }
                Message::NewTransaction(tx) => {
                    let tx_id = tx.id();
                    if node.mark_seen(tx_id.clone()).await {
                        println!("Gossip: Transaction {} from {}", tx_id, addr);
                        node.broadcast_except(Message::NewTransaction(tx), addr)
                            .await?;
                    }
                }
                Message::NewBlock(block) => {
                    let blk_id = block.header.hash();
                    if node.mark_seen(blk_id.clone()).await {
                        println!("Gossip: Block {} from {}", blk_id, addr);
                        node.broadcast_except(Message::NewBlock(block), addr)
                            .await?;
                    }
                }
                _ => {
                    println!("Received unhandled message from {}: {:?}", addr, msg);
                }
            }
        }
    };

    let peer_writer = async {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                PeerCmd::SendMessage(msg) => {
                    msg.send_async(&mut writer).await?;
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
