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
        let listener = TcpListener::bind(self.addr).await.context("Failed to bind P2P listener")?;
        println!("P2P server listening on {}", self.addr);

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            println!("New inbound connection from {}", peer_addr);
            tokio::spawn(async move {
                if let Err(e) = handle_peer(stream, peer_addr).await {
                    eprintln!("Error handling peer {}: {}", peer_addr, e);
                }
            });
        }
    }

    pub async fn connect(&self, target: SocketAddr) -> anyhow::Result<()> {
        println!("Connecting to {}...", target);
        let mut stream = TcpStream::connect(target).await.context("Failed to connect to peer")?;
        println!("Connected to outbound peer {}", target);
        
        // Send initial Ping
        Message::Ping.send_async(&mut stream).await?;

        tokio::spawn(async move {
            if let Err(e) = handle_peer(stream, target).await {
                eprintln!("Error handling peer {}: {}", target, e);
            }
        });

        Ok(())
    }
}

async fn handle_peer(mut stream: TcpStream, addr: SocketAddr) -> anyhow::Result<()> {
    loop {
        let msg = Message::decode_async(&mut stream).await.context("Failed to decode peer message")?;
        println!("Received message from {}: {:?}", addr, msg);

        match msg {
            Message::Ping => {
                Message::Pong.send_async(&mut stream).await?;
            }
            Message::Pong => {
                // Ignore for now
            }
            _ => {
                // To be implemented: sync, inv, etc.
            }
        }
    }
}
