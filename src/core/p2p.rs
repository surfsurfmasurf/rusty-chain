use crate::core::network::Message;
use anyhow::Context;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

/// Shared node state for concurrent peer handling
pub struct NodeState {
    pub peers: Vec<SocketAddr>,
}

pub struct P2PNode {
    pub addr: SocketAddr,
    pub state: Arc<Mutex<NodeState>>,
}

impl P2PNode {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            state: Arc::new(Mutex::new(NodeState { peers: Vec::new() })),
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.addr)
            .await
            .context("Failed to bind P2P listener")?;
        println!("P2P server listening on {}", self.addr);

        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    println!("New inbound connection from {}", peer_addr);
                    let state = Arc::clone(&self.state);
                    tokio::spawn(async move {
                        if let Err(e) = handle_peer(stream, peer_addr, state).await {
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
        let mut stream = TcpStream::connect(target)
            .await
            .context("Failed to connect to peer")?;
        println!("Connected to outbound peer {}", target);

        // Send initial Handshake
        Message::Handshake {
            version: 1,
            best_height,
        }
        .send_async(&mut stream)
        .await?;

        let state = Arc::clone(&self.state);
        tokio::spawn(async move {
            if let Err(e) = handle_peer(stream, target, state).await {
                eprintln!("Error handling peer {}: {}", target, e);
            }
        });

        Ok(())
    }
}

async fn handle_peer(
    mut stream: TcpStream,
    addr: SocketAddr,
    state: Arc<Mutex<NodeState>>,
) -> anyhow::Result<()> {
    // Add to peer list
    {
        let mut s = state.lock().await;
        if !s.peers.contains(&addr) {
            s.peers.push(addr);
        }
    }

    println!("Starting message loop for {}", addr);
    let res = peer_loop(&mut stream, addr).await;

    // Remove from peer list
    {
        let mut s = state.lock().await;
        s.peers.retain(|&p| p != addr);
    }

    res
}

async fn peer_loop(stream: &mut TcpStream, addr: SocketAddr) -> anyhow::Result<()> {
    loop {
        let msg = Message::decode_async(stream)
            .await
            .context("Failed to decode peer message")?;
        println!("Received message from {}: {:?}", addr, msg);

        match msg {
            Message::Ping => {
                println!("Responding to Ping from {}", addr);
                Message::Pong.send_async(stream).await?;
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
            _ => {
                println!("Received unhandled message from {}: {:?}", addr, msg);
            }
        }
    }
}
