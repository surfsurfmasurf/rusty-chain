pub mod chain;
pub mod crypto;
pub mod hash;
pub mod keys;
pub mod mempool;
pub mod network;
pub mod state;
pub mod time;
pub mod types;

use crate::core::network::Message;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub struct P2PNode {
    port: u16,
    peers: Arc<Mutex<HashSet<String>>>,
}

impl P2PNode {
    pub fn new(port: u16, peer_addrs: Vec<String>) -> Self {
        let mut peers = HashSet::new();
        for addr in peer_addrs {
            peers.insert(addr);
        }
        Self {
            port,
            peers: Arc::new(Mutex::from(peers)),
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        println!("Listening on P2P port: {}", self.port);

        loop {
            let (stream, addr) = listener.accept().await?;
            println!("New peer connected: {}", addr);
            tokio::spawn(async move {
                if let Err(e) = handle_peer(stream).await {
                    eprintln!("Peer handler error: {}", e);
                }
            });
        }
    }
}

async fn handle_peer(mut stream: TcpStream) -> anyhow::Result<()> {
    let (mut reader, mut writer) = stream.split();
    loop {
        let msg = Message::decode_async(&mut reader).await?;
        println!("Received message: {:?}", msg);

        match msg {
            Message::Ping => {
                Message::Pong.send_async(&mut writer).await?;
            }
            _ => {
                // Ignore other messages for now
            }
        }
    }
}
