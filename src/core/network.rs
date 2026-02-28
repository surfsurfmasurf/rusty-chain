use crate::core::types::{Block, BlockHeader, Transaction};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Message {
    Ping,
    Pong,
    GetStatus,
    Status {
        height: u64,
        tip_hash: String,
    },
    GetBlocks {
        start_height: u64,
    },
    Blocks(Vec<Block>),
    NewTransaction(Transaction),
    NewBlock(Block),
    Inventory {
        tx_hashes: Vec<String>,
        block_hashes: Vec<String>,
    },
    GetMempool,
    Handshake {
        version: u32,
        best_height: u64,
    },
    GetHeaders {
        start_height: u64,
        limit: u32,
    },
    Headers(Vec<BlockHeader>),
    GetData {
        block_hashes: Vec<String>,
    },
    Addr {
        addrs: Vec<SocketAddr>,
    },
    GetAddr,
}

impl Message {
    pub fn encode(&self) -> anyhow::Result<Vec<u8>> {
        let json = serde_json::to_vec(self)?;
        let len = (json.len() as u32).to_be_bytes();
        let mut buf = Vec::with_capacity(4 + json.len());
        buf.extend_from_slice(&len);
        buf.extend_from_slice(&json);
        Ok(buf)
    }

    pub fn decode<R: Read>(mut reader: R) -> anyhow::Result<Self> {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        // Sanity check: limit message size to 10MB
        if len > 10 * 1024 * 1024 {
            return Err(anyhow::anyhow!("Message too large: {} bytes", len));
        }

        let mut json_buf = vec![0u8; len];
        reader.read_exact(&mut json_buf)?;
        let msg = serde_json::from_slice(&json_buf)?;
        Ok(msg)
    }

    pub fn send(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let buf = self.encode()?;
        stream.write_all(&buf)?;
        stream.flush()?;
        Ok(())
    }

    pub async fn send_async<W: tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> anyhow::Result<()> {
        use tokio::io::AsyncWriteExt;
        let buf = self.encode()?;
        writer.write_all(&buf).await?;
        writer.flush().await?;
        Ok(())
    }

    pub fn size_limit() -> usize {
        15 * 1024 * 1024 // 15MB
    }

    pub fn is_gossip(&self) -> bool {
        matches!(self, Message::NewTransaction(_) | Message::NewBlock(_))
    }

    /// Returns the unique ID for gossip messages to prevent loops.
    pub fn gossip_id(&self) -> Option<String> {
        match self {
            Message::NewTransaction(tx) => Some(tx.id()),
            Message::NewBlock(block) => Some(block.header.hash()),
            _ => None,
        }
    }

    pub fn get_type_name(&self) -> &'static str {
        match self {
            Message::Ping => "Ping",
            Message::Pong => "Pong",
            Message::GetStatus => "GetStatus",
            Message::Status { .. } => "Status",
            Message::GetBlocks { .. } => "GetBlocks",
            Message::Blocks(_) => "Blocks",
            Message::NewTransaction(_) => "NewTransaction",
            Message::NewBlock(_) => "NewBlock",
            Message::Inventory { .. } => "Inventory",
            Message::GetMempool => "GetMempool",
            Message::Handshake { .. } => "Handshake",
            Message::GetHeaders { .. } => "GetHeaders",
            Message::Headers(_) => "Headers",
            Message::GetData { .. } => "GetData",
            Message::Addr { .. } => "Addr",
            Message::GetAddr => "GetAddr",
        }
    }

    pub async fn decode_async<R: tokio::io::AsyncRead + Unpin>(
        mut reader: R,
    ) -> anyhow::Result<Self> {
        use tokio::io::AsyncReadExt;
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        // Sanity check: limit message size to 10MB
        if len > 10 * 1024 * 1024 {
            return Err(anyhow::anyhow!("Message too large: {} bytes", len));
        }

        let mut json_buf = vec![0u8; len];
        reader.read_exact(&mut json_buf).await?;
        let msg = serde_json::from_slice(&json_buf)?;
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_message_roundtrip() {
        let msg = Message::Ping;
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(Cursor::new(encoded)).unwrap();
        assert_eq!(msg, decoded);
    }

    #[tokio::test]
    async fn test_message_async_roundtrip() {
        let msg = Message::Status {
            height: 10,
            tip_hash: "abcd".to_string(),
        };
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode_async(Cursor::new(encoded)).await.unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_message_too_large() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(20 * 1024 * 1024u32).to_be_bytes());
        let res = Message::decode(Cursor::new(buf));
        assert!(res.is_err());
    }

    #[test]
    fn test_message_inventory_empty() {
        let msg = Message::Inventory {
            tx_hashes: vec![],
            block_hashes: vec![],
        };
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(Cursor::new(encoded)).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_message_handshake() {
        let msg = Message::Handshake {
            version: 1,
            best_height: 123,
        };
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(Cursor::new(encoded)).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_message_is_gossip() {
        assert!(Message::NewTransaction(Transaction::new("a", "b", 10, 0)).is_gossip());
        assert!(
            Message::NewBlock(Block {
                header: crate::core::types::BlockHeader {
                    prev_hash: "".to_string(),
                    merkle_root: "".to_string(),
                    timestamp_ms: 0,
                    nonce: 0,
                },
                txs: vec![],
            })
            .is_gossip()
        );
        assert!(!Message::Ping.is_gossip());
    }

    #[test]
    fn test_message_get_headers_roundtrip() {
        let msg = Message::GetHeaders {
            start_height: 10,
            limit: 100,
        };
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(std::io::Cursor::new(encoded)).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_message_type_name() {
        assert_eq!(Message::Ping.get_type_name(), "Ping");
        assert_eq!(Message::GetMempool.get_type_name(), "GetMempool");
        assert_eq!(
            Message::NewTransaction(Transaction::new("a", "b", 10, 0)).get_type_name(),
            "NewTransaction"
        );
    }

    #[test]
    fn test_message_addr_getaddr() {
        let msg = Message::Addr {
            addrs: vec!["127.0.0.1:8080".parse().unwrap()],
        };
        let encoded = msg.encode().unwrap();
        let decoded = Message::decode(Cursor::new(encoded)).unwrap();
        assert_eq!(msg, decoded);

        let msg2 = Message::GetAddr;
        let encoded2 = msg2.encode().unwrap();
        let decoded2 = Message::decode(Cursor::new(encoded2)).unwrap();
        assert_eq!(msg2, decoded2);
    }

    #[test]
    fn test_memo_limit_128() {
        let mut tx = Transaction::new("A", "B", 100, 0);
        tx.memo = Some("a".repeat(128));
        assert!(tx.validate_basic().is_ok());

        tx.memo = Some("a".repeat(129));
        assert!(tx.validate_basic().is_err());
    }
}
