use crate::core::network::Message;
use anyhow::Context;
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
            })),
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
}

async fn handle_peer(
    mut stream: TcpStream,
    addr: SocketAddr,
    state: Arc<Mutex<NodeState>>,
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
