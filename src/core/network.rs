use crate::core::types::{Block, Transaction};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Message {
    Ping,
    Pong,
    GetStatus,
    Status { height: u64, tip_hash: String },
    GetBlocks { start_height: u64 },
    Blocks(Vec<Block>),
    NewTransaction(Transaction),
    NewBlock(Block),
    RequestStatus,
    ResponseStatus { height: u64, tip_hash: String },
    Inventory { tx_hashes: Vec<String>, block_hashes: Vec<String> },
    GetMempool,
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
        10 * 1024 * 1024 // 10MB
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
        buf.extend_from_slice(&(11 * 1024 * 1024u32).to_be_bytes());
        let res = Message::decode(Cursor::new(buf));
        assert!(res.is_err());
    }
}
