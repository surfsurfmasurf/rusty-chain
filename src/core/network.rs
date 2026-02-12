use crate::core::types::{Block, Transaction};
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
}
