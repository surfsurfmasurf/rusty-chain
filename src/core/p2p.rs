use crate::core::network::Message;
use anyhow::Context;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

pub struct P2PNode {
    pub addr: SocketAddr,
}

impl P2PNode {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
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
                    tokio::spawn(async move {
                        if let Err(e) = handle_peer(stream, peer_addr).await {
                            eprintln!("Peer {} disconnected with error: {:?}", peer_addr, e);
                        } else {
                            println!("Peer {} disconnected gracefully", peer_addr);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                    // Add a small delay to prevent tight loop on persistent errors
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

        tokio::spawn(async move {
            if let Err(e) = handle_peer(stream, target).await {
                eprintln!("Error handling peer {}: {}", target, e);
            }
        });

        Ok(())
    }
}

async fn handle_peer(mut stream: TcpStream, addr: SocketAddr) -> anyhow::Result<()> {
    println!("Starting message loop for {}", addr);
    loop {
        let msg = Message::decode_async(&mut stream)
            .await
            .context("Failed to decode peer message")?;
        println!("Received message from {}: {:?}", addr, msg);

        match msg {
            Message::Ping => {
                println!("Responding to Ping from {}", addr);
                Message::Pong.send_async(&mut stream).await?;
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
