use crate::core::types::{Block, Transaction};
use serde::{Deserialize, Serialize};

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
