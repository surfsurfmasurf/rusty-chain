use crate::core::chain::Chain;
use crate::core::mempool::Mempool;
use crate::core::network::{Message, PeerInfo};
use crate::core::types::{Block, BlockHeader, Transaction};
use anyhow::Context;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};

/// Commands that can be sent to the peer handler
#[derive(Debug, Clone)]
pub enum PeerCmd {
    SendMessage(Message),
    Disconnect,
}

/// Shared node state for concurrent peer handling
pub struct NodeState {
    pub known_addrs: HashSet<SocketAddr>,
    pub peer_senders: HashMap<SocketAddr, mpsc::UnboundedSender<PeerCmd>>,
    pub seen_messages: HashSet<String>,
    pub peer_reputation: HashMap<SocketAddr, i32>,
    pub outgoing_conns: HashSet<SocketAddr>,
    pub chain: Chain,
    pub mempool: Mempool,
    pub peer_list_path: Option<String>,
    pub whitelist_path: Option<String>,
    pub banned_peers: HashSet<SocketAddr>,
    pub whitelisted_peers: HashSet<SocketAddr>,
}

pub struct P2PNode {
    pub addr: SocketAddr,
    pub state: Arc<Mutex<NodeState>>,
}

impl P2PNode {
    pub fn new(
        addr: SocketAddr,
        chain: Chain,
        mempool: Mempool,
        peer_list_path: Option<String>,
        whitelist_path: Option<String>,
    ) -> Self {
        let mut known_addrs = HashSet::new();
        known_addrs.insert(addr);

        // Load existing peers if path provided
        #[allow(clippy::collapsible_if)]
        if let Some(ref path) = peer_list_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(addrs) = serde_json::from_str::<HashSet<SocketAddr>>(&content) {
                    println!("Loaded {} known addresses from {}", addrs.len(), path);
                    known_addrs.extend(addrs);
                }
            }
        }

        let mut whitelisted_peers = HashSet::new();
        #[allow(clippy::collapsible_if)]
        if let Some(ref path) = whitelist_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(addrs) = serde_json::from_str::<HashSet<SocketAddr>>(&content) {
                    println!("Loaded {} whitelisted addresses from {}", addrs.len(), path);
                    whitelisted_peers.extend(addrs);
                }
            }
        }

        Self {
            addr,
            state: Arc::new(Mutex::new(NodeState {
                known_addrs,
                peer_senders: HashMap::new(),
                seen_messages: HashSet::new(),
                peer_reputation: HashMap::new(),
                outgoing_conns: HashSet::new(),
                chain,
                mempool,
                peer_list_path,
                whitelist_path,
                banned_peers: HashSet::new(),
                whitelisted_peers,
            })),
        }
    }

    pub async fn start(&self, agent: String) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.addr)
            .await
            .context("Failed to bind P2P listener")?;
        println!("P2P server listening on {} (agent={})", self.addr, agent);

        let node_state = Arc::clone(&self.state);
        let save_state = Arc::clone(&self.state);
        let save_whitelist = Arc::clone(&self.state);
        let evict_state = Arc::clone(&self.state);
        let reputation_gossip = Arc::clone(&self.state);
        let reconnection_task = Arc::clone(&self.state);
        let agent_for_recon = agent.clone();

        // Background peer saver
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                let state = save_state.lock().await;
                #[allow(clippy::collapsible_if)]
                if let Some(ref path) = state.peer_list_path {
                    if let Ok(content) = serde_json::to_string_pretty(&state.known_addrs) {
                        let _ = std::fs::write(path, content);
                    }
                }
            }
        });

        // Background reconnection task
        let node_for_recon = P2PNode {
            addr: self.addr,
            state: Arc::clone(&reconnection_task),
        };
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                let (height, targets) = {
                    let state = reconnection_task.lock().await;
                    let height = state.chain.height() as u64;
                    let targets: Vec<SocketAddr> = state
                        .known_addrs
                        .iter()
                        .filter(|addr| {
                            !state.peer_senders.contains_key(addr) && **addr != node_for_recon.addr
                        })
                        .cloned()
                        .collect();
                    (height, targets)
                };

                for target in targets {
                    let _ = node_for_recon
                        .connect(target, height, agent_for_recon.clone())
                        .await;
                }
            }
        });

        // Background whitelist saver
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                let state = save_whitelist.lock().await;
                #[allow(clippy::collapsible_if)]
                if let Some(ref path) = state.whitelist_path {
                    if let Ok(content) = serde_json::to_string_pretty(&state.whitelisted_peers) {
                        let _ = std::fs::write(path, content);
                    }
                }
            }
        });

        // Background mempool evictor
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(300)).await; // 5 minutes
                let mut state = evict_state.lock().await;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                // TTL: 24 hours (86,400,000 ms)
                let evicted = state.mempool.evict_expired(86_400_000, now);
                if evicted > 0 {
                    println!(
                        "Background evictor: removed {} expired transactions",
                        evicted
                    );
                }
            }
        });

        // Background reputation gossip
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(600)).await; // 10 minutes
                let state = reputation_gossip.lock().await;
                println!(
                    "Requesting reputation snapshots from {} peers",
                    state.peer_senders.len()
                );
                for tx in state.peer_senders.values() {
                    let _ = tx.send(PeerCmd::SendMessage(Message::GetReputation));
                }
            }
        });

        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    let is_banned = {
                        let state = node_state.lock().await;
                        state.banned_peers.contains(&peer_addr)
                    };

                    if is_banned {
                        println!("Rejecting connection from banned peer {}", peer_addr);
                        continue;
                    }

                    println!("New inbound connection from {}", peer_addr);
                    let state = Arc::clone(&node_state);
                    let node_handle = P2PNodeHandle {
                        state: Arc::clone(&state),
                    };
                    let agent_clone = agent.clone();
                    tokio::spawn(async move {
                        if let Err(e) =
                            handle_peer(stream, peer_addr, state, node_handle, agent_clone).await
                        {
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

    pub async fn connect(
        &self,
        target: SocketAddr,
        best_height: u64,
        agent: String,
    ) -> anyhow::Result<()> {
        println!("Connecting to {}...", target);
        let stream = match TcpStream::connect(target).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to connect to {}: {}", target, e);
                return Err(e.into());
            }
        };
        println!("Connected to outbound peer {}", target);

        // Add to known addrs
        {
            let mut s = self.state.lock().await;
            s.known_addrs.insert(target);
            s.outgoing_conns.insert(target);
        }

        let mut stream = stream;

        // Send initial Handshake
        Message::Handshake {
            version: 1,
            best_height,
            agent: agent.clone(),
        }
        .send_async(&mut stream)
        .await?;

        let state = Arc::clone(&self.state);
        let node_handle = P2PNodeHandle {
            state: Arc::clone(&state),
        };
        let agent_clone = agent.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_peer(stream, target, state, node_handle, agent_clone).await {
                eprintln!("Error handling peer {}: {}", target, e);
            }
        });

        Ok(())
    }
}

/// A lightweight handle to the P2PNode to avoid circular Arc or complex lifetimes in handlers
#[derive(Clone)]
pub struct P2PNodeHandle {
    pub state: Arc<Mutex<NodeState>>,
}

impl P2PNodeHandle {
    #[allow(clippy::collapsible_if)]
    pub async fn broadcast_except(&self, msg: Message, except: SocketAddr) -> anyhow::Result<()> {
        let state = self.state.lock().await;
        for (&addr, tx) in &state.peer_senders {
            if addr != except {
                let _ = tx.send(PeerCmd::SendMessage(msg.clone()));
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

    pub async fn get_peer_count(&self) -> usize {
        let state = self.state.lock().await;
        state.peer_senders.len()
    }

    pub async fn update_reputation(&self, peer: SocketAddr, delta: i32) {
        let mut state = self.state.lock().await;

        if state.whitelisted_peers.contains(&peer) {
            println!(
                "Skipping reputation update for whitelisted peer {} (delta: {})",
                peer, delta
            );
            return;
        }

        let score = state.peer_reputation.entry(peer).or_insert(0);
        *score = score.saturating_add(delta);
        println!(
            "Peer {} reputation updated: {} (delta: {})",
            peer, *score, delta
        );

        // Auto-ban threshold: -100
        if *score <= -100 {
            println!(
                "Peer {} reached ban threshold ({}), banning...",
                peer, *score
            );
            state.banned_peers.insert(peer);
            if let Some(tx) = state.peer_senders.get(&peer) {
                let _ = tx.send(PeerCmd::SendMessage(Message::Reject {
                    code: 403,
                    reason: "Banned due to low reputation".to_string(),
                    message_type: "Handshake".to_string(),
                }));
                let _ = tx.send(PeerCmd::Disconnect);
            }
        }
    }

    pub async fn get_reputation(&self, peer: SocketAddr) -> i32 {
        let state = self.state.lock().await;
        *state.peer_reputation.get(&peer).unwrap_or(&0)
    }

    pub async fn whitelist_peer(&self, peer: SocketAddr) {
        let mut state = self.state.lock().await;
        state.whitelisted_peers.insert(peer);
        state.peer_reputation.remove(&peer);
        state.banned_peers.remove(&peer);
        println!("Peer {} has been whitelisted", peer);
    }

    pub async fn ban_peer(&self, peer: SocketAddr) {
        let mut state = self.state.lock().await;
        if state.whitelisted_peers.contains(&peer) {
            println!("Cannot ban whitelisted peer {}", peer);
            return;
        }
        state.banned_peers.insert(peer);
        state.peer_reputation.insert(peer, -100);
        if let Some(tx) = state.peer_senders.get(&peer) {
            let _ = tx.send(PeerCmd::Disconnect);
        }
        println!("Peer {} has been manually banned", peer);
    }

    pub async fn unban_peer(&self, peer: SocketAddr) {
        let mut state = self.state.lock().await;
        state.banned_peers.remove(&peer);
        state.peer_reputation.insert(peer, 0);
        println!("Peer {} has been unbanned", peer);
    }

    pub async fn unwhitelist_peer(&self, peer: SocketAddr) {
        let mut state = self.state.lock().await;
        state.whitelisted_peers.remove(&peer);
        println!("Peer {} has been removed from whitelist", peer);
    }

    pub async fn get_whitelisted_peers(&self) -> HashSet<SocketAddr> {
        let state = self.state.lock().await;
        state.whitelisted_peers.clone()
    }

    pub async fn get_banned_peers(&self) -> HashSet<SocketAddr> {
        let state = self.state.lock().await;
        state.banned_peers.clone()
    }

    pub async fn get_mempool_info(&self) -> (usize, usize, u64, u64) {
        let state = self.state.lock().await;
        let count = state.mempool.txs.len();
        let total_size = state.mempool.txs.iter().map(|tx| tx.size()).sum();
        let min_fee = state.mempool.txs.iter().map(|tx| tx.fee).min().unwrap_or(0);
        let max_fee = state.mempool.txs.iter().map(|tx| tx.fee).max().unwrap_or(0);
        (count, total_size, min_fee, max_fee)
    }

    pub async fn merge_reputation(&self, incoming: Vec<(SocketAddr, i32)>) {
        let mut state = self.state.lock().await;
        for (addr, incoming_score) in incoming {
            if state.whitelisted_peers.contains(&addr) {
                continue;
            }
            let current = state.peer_reputation.entry(addr).or_insert(0);
            // Weighted average: 50% current, 50% incoming
            *current = (*current + incoming_score) / 2;

            // Check for auto-ban after merge
            if *current <= -100 {
                state.banned_peers.insert(addr);
                // If we are currently connected to this peer, disconnect them
                if let Some(tx) = state.peer_senders.get(&addr) {
                    let _ = tx.send(PeerCmd::SendMessage(Message::Reject {
                        code: 403,
                        reason: "Banned due to low merged reputation".to_string(),
                        message_type: "Reputation".to_string(),
                    }));
                    let _ = tx.send(PeerCmd::Disconnect);
                }
            }
        }
    }

    #[allow(clippy::collapsible_if)]
    pub async fn send_to(&self, target: SocketAddr, msg: Message) -> anyhow::Result<()> {
        let state = self.state.lock().await;
        if let Some(tx) = state.peer_senders.get(&target) {
            let _ = tx.send(PeerCmd::SendMessage(msg));
        } else {
            eprintln!(
                "Failed to send {}: Peer {} not found",
                msg.get_type_name(),
                target
            );
        }
        Ok(())
    }

    pub async fn broadcast(&self, msg: Message) -> anyhow::Result<()> {
        let state = self.state.lock().await;
        for tx in state.peer_senders.values() {
            let _ = tx.send(PeerCmd::SendMessage(msg.clone()));
        }
        Ok(())
    }

    pub async fn get_headers(&self, start_height: u64, limit: u32) -> Vec<BlockHeader> {
        let state = self.state.lock().await;
        state
            .chain
            .blocks
            .iter()
            .skip(start_height as usize)
            .take(limit as usize)
            .map(|b| b.header.clone())
            .collect()
    }

    pub async fn get_blocks_by_hash(&self, hashes: Vec<String>) -> Vec<Block> {
        let state = self.state.lock().await;
        let mut results = Vec::new();
        for hash in hashes {
            if let Some(&height) = state.chain.block_index.get(&hash) {
                if let Some(block) = state.chain.blocks.get(height) {
                    results.push(block.clone());
                }
            }
        }
        results
    }

    async fn process_new_block(&self, block: Block, from: SocketAddr) -> anyhow::Result<()> {
        let blk_id = block.header.hash();
        if self.mark_seen(blk_id.clone()).await {
            println!("Gossip: New Block {} from {}", blk_id, from);

            // 1. Initial validation against local chain (lightweight clone)
            let chain_copy = {
                let state = self.state.lock().await;
                // Basic duplicate check against local chain
                if state.chain.block_index.contains_key(&blk_id) {
                    return Ok(());
                }
                state.chain.clone()
            };

            if let Err(e) = chain_copy.validate_block(&block) {
                println!("Invalid block {} from {}: {}", blk_id, from, e);
                self.update_reputation(from, -50).await;
                return Ok(());
            }

            // 2. Final append under lock
            {
                let mut state = self.state.lock().await;
                // Re-verify linkage in case tip changed during validation
                if block.header.prev_hash != state.chain.tip_hash() {
                    println!(
                        "Gossip block {} from {} rejected: prev_hash mismatch during lock",
                        blk_id, from
                    );
                    return Ok(());
                }

                if let Err(e) = state.chain.append_block(block.clone()) {
                    println!(
                        "Failed to append validated block {} to chain: {}",
                        blk_id, e
                    );
                    return Ok(());
                }
                // 3. Clear mempool txs
                state.mempool.remove_included(&block.txs);
            }

            // 4. Update reputation
            self.update_reputation(from, 10).await;
            // 5. Re-gossip
            self.broadcast_except(Message::NewBlock(block), from)
                .await?;
        }
        Ok(())
    }

    async fn process_new_transaction(
        &self,
        tx: Transaction,
        from: SocketAddr,
    ) -> anyhow::Result<()> {
        let tx_id = tx.id();
        let gossip_id = format!("{}_{}", tx_id, tx.fee);
        if self.mark_seen(gossip_id).await {
            println!(
                "Gossip: New Transaction {} (fee={}) from {}",
                tx_id, tx.fee, from
            );
            // 1. Validate tx
            let mut state = self.state.lock().await;
            if let Err(e) = state.chain.validate_transaction(&tx) {
                println!("Invalid transaction {} from {}: {}", tx_id, from, e);
                drop(state);
                self.update_reputation(from, -10).await;
                return Ok(());
            }
            // 2. Add to mempool
            let base_nonce = state.chain.next_nonce_for(&tx.from);
            if let Err(e) = state.mempool.add_tx_checked(tx.clone(), base_nonce) {
                println!("Failed to add tx {} from {} to mempool: {}", tx_id, from, e);
                // Send rejection message for invalid RBF attempt or nonce gap
                let _ = self
                    .send_to(
                        from,
                        Message::Reject {
                            code: 1,
                            reason: e.to_string(),
                            message_type: "NewTransaction".to_string(),
                        },
                    )
                    .await;
                return Ok(());
            }
            drop(state);
            self.update_reputation(from, 1).await;

            // 3. Re-gossip
            self.broadcast_except(Message::NewTransaction(tx), from)
                .await?;
        }
        Ok(())
    }

    pub async fn process_message(&self, msg: Message, from: SocketAddr) -> anyhow::Result<()> {
        println!("Processing {} message from {}", msg.get_type_name(), from);
        match msg {
            Message::Ping => {
                self.send_to(from, Message::Pong).await?;
            }
            Message::Handshake {
                version,
                best_height,
                agent,
            } => {
                println!(
                    "Handshake from {}: version={}, height={}, agent={}",
                    from, version, best_height, agent
                );
                // Version check: simple exact match for this demo
                if version != 1 {
                    println!("Incompatible version from {}: {}", from, version);
                    self.send_to(
                        from,
                        Message::Reject {
                            code: 400,
                            reason: format!("Incompatible version: {}", version),
                            message_type: "Handshake".to_string(),
                        },
                    )
                    .await?;
                    // Disconnect is a PeerCmd, not a Message.
                    // We need a way to trigger disconnection from process_message.
                    // For now, we'll just return an error or let it time out,
                    // but better is to send a Disconnect command to the peer.
                    // NodeState doesn't have direct access to the mpsc sender here except via PeerCmd.
                    let state = self.state.lock().await;
                    if let Some(tx) = state.peer_senders.get(&from) {
                        let _ = tx.send(PeerCmd::Disconnect);
                    }
                    return Ok(());
                }

                // Request mempool transactions upon connection
                self.send_to(from, Message::GetMempoolTxs).await?;

                // Sync logic: if they are ahead, request headers
                let our_height = {
                    let state = self.state.lock().await;
                    state.chain.blocks.len() as u64
                };

                if best_height > our_height {
                    println!(
                        "Peer {} is ahead ({} > {}), requesting checkpoints...",
                        from, best_height, our_height
                    );
                    self.send_to(from, Message::GetCheckpoints).await?;
                }

                // Request addresses during handshake
                self.send_to(from, Message::GetAddr).await?;
            }
            Message::NewTransaction(tx) => {
                self.process_new_transaction(tx, from).await?;
            }
            Message::NewBlock(block) => {
                self.process_new_block(block, from).await?;
            }
            Message::GetHeaders {
                start_height,
                limit,
            } => {
                let headers = self.get_headers(start_height, limit).await;
                self.send_to(from, Message::Headers(headers)).await?;
            }
            Message::Headers(headers) => {
                if !headers.is_empty() {
                    println!("Received {} headers from {}", headers.len(), from);
                    // Request blocks for these headers
                    let hashes = headers.iter().map(|h| h.hash()).collect();
                    self.send_to(
                        from,
                        Message::GetData {
                            block_hashes: hashes,
                        },
                    )
                    .await?;

                    // Batching: if we likely received a full batch, request more.
                    // Default limit in GetHeaders is usually around 100-2000.
                    // For now we use 100 as the trigger.
                    if headers.len() >= 100 {
                        let current_height = {
                            let state = self.state.lock().await;
                            state.chain.height() as u64
                        };
                        let next_start = current_height + 1;
                        println!(
                            "Requesting next batch of headers starting at {}",
                            next_start
                        );
                        self.send_to(
                            from,
                            Message::GetHeaders {
                                start_height: next_start,
                                limit: 100,
                            },
                        )
                        .await?;
                    }
                }
            }
            Message::Blocks(blocks) => {
                println!("Received {} blocks from {}", blocks.len(), from);
                for block in blocks {
                    self.process_new_block(block, from).await?;
                }
            }
            Message::GetData { block_hashes } => {
                let blocks = self.get_blocks_by_hash(block_hashes).await;
                self.send_to(from, Message::Blocks(blocks)).await?;
            }
            Message::GetFeeEstimate { tx_size } => {
                let rate = {
                    let state = self.state.lock().await;
                    state.chain.estimate_fee_rate(10) // Window of 10 blocks
                };
                let fee_per_byte = rate.ceil() as u64;
                let estimated_total = (tx_size as f64 * rate).ceil() as u64;
                self.send_to(
                    from,
                    Message::FeeEstimate {
                        fee_per_byte,
                        estimated_total,
                    },
                )
                .await?;
            }
            Message::FeeEstimate {
                fee_per_byte,
                estimated_total,
            } => {
                println!(
                    "Received fee estimate from {}: {} units (rate: {}/byte)",
                    from, estimated_total, fee_per_byte
                );
            }
            Message::GetAddr => {
                let addrs = {
                    let state = self.state.lock().await;
                    state.peer_senders.keys().cloned().collect::<Vec<_>>()
                };
                self.send_to(from, Message::Addr { addrs }).await?;
            }
            Message::GetAllAddr => {
                let addrs = {
                    let state = self.state.lock().await;
                    state.known_addrs.iter().cloned().collect::<Vec<_>>()
                };
                self.send_to(from, Message::Addr { addrs }).await?;
            }
            Message::GetPeers => {
                let peers = {
                    let state = self.state.lock().await;
                    state
                        .peer_reputation
                        .iter()
                        .map(|(addr, &reputation)| PeerInfo {
                            addr: *addr,
                            reputation,
                            is_banned: state.banned_peers.contains(addr),
                        })
                        .collect::<Vec<_>>()
                };
                self.send_to(from, Message::Peers(peers)).await?;
            }
            Message::Whitelist(addr) => {
                println!("Received Whitelist request for {} from {}", addr, from);
                // In a real scenario, this would require admin auth or be local-only.
                // For this demo, we allow it.
                self.whitelist_peer(addr).await;
            }
            Message::Ban(addr) => {
                println!("Received Ban request for {} from {}", addr, from);
                self.ban_peer(addr).await;
            }
            Message::Unban(addr) => {
                println!("Received Unban request for {} from {}", addr, from);
                self.unban_peer(addr).await;
            }
            Message::GetBanned => {
                let banned = self.get_banned_peers().await.into_iter().collect();
                self.send_to(from, Message::Banned(banned)).await?;
            }
            Message::GetWhitelisted => {
                let whitelisted = self.get_whitelisted_peers().await.into_iter().collect();
                self.send_to(from, Message::Whitelisted(whitelisted))
                    .await?;
            }
            Message::Unwhitelist(addr) => {
                println!("Received Unwhitelist request for {} from {}", addr, from);
                self.unwhitelist_peer(addr).await;
            }
            Message::Peers(peers) => {
                println!("Received reputation data for {} peers", peers.len());
                for p in peers {
                    println!(
                        "  - {}: reputation={} (banned: {})",
                        p.addr, p.reputation, p.is_banned
                    );
                }
            }
            Message::Reject {
                code,
                reason,
                message_type,
            } => {
                println!(
                    "Peer {} rejected our {}: [{}] {}",
                    from, message_type, code, reason
                );
            }
            Message::Addr { addrs } => {
                let mut new_addrs = Vec::new();
                {
                    let mut state = self.state.lock().await;
                    for addr in addrs {
                        if state.known_addrs.insert(addr) {
                            new_addrs.push(addr);
                        }
                    }
                }
                if !new_addrs.is_empty() {
                    println!("Received {} new addresses from {}", new_addrs.len(), from);
                    // Gossip new addresses
                    self.broadcast_except(Message::Addr { addrs: new_addrs }, from)
                        .await?;
                }
            }
            Message::GetReputation => {
                let scores = {
                    let state = self.state.lock().await;
                    state
                        .peer_reputation
                        .iter()
                        .map(|(&addr, &score)| (addr, score))
                        .collect()
                };
                self.send_to(from, Message::Reputation(scores)).await?;
            }
            Message::Reputation(scores) => {
                println!(
                    "Received reputation snapshot with {} entries from {}",
                    scores.len(),
                    from
                );
                self.merge_reputation(scores).await;
            }
            Message::GetCheckpoints => {
                let checkpoints = {
                    let state = self.state.lock().await;
                    state.chain.checkpoints.clone()
                };
                self.send_to(from, Message::Checkpoints(checkpoints))
                    .await?;
            }
            Message::Checkpoints(checkpoints) => {
                println!("Received {} checkpoints from {}", checkpoints.len(), from);
                if !checkpoints.is_empty() {
                    let state = self.state.lock().await;
                    let our_height = state.chain.height();

                    // Find the highest common checkpoint
                    let mut highest_checkpoint: Option<(usize, String)> = None;
                    for (height, hash) in checkpoints {
                        if height <= our_height {
                            if let Some(our_hash) = state.chain.get_checkpoint_at(height) {
                                if our_hash == hash {
                                    if highest_checkpoint
                                        .as_ref()
                                        .map_or(true, |(h, _)| height > *h)
                                    {
                                        highest_checkpoint = Some((height, hash));
                                    }
                                }
                            }
                        } else {
                            // Peer is ahead, we can use their checkpoints for future validation
                            if highest_checkpoint
                                .as_ref()
                                .map_or(true, |(h, _)| height > *h)
                            {
                                highest_checkpoint = Some((height, hash));
                            }
                        }
                    }

                    if let Some((h, _)) = highest_checkpoint {
                        if h as u64 > our_height as u64 {
                            println!(
                                "Peer {} has a newer checkpoint at {}, requesting headers...",
                                from, h
                            );
                            let _ = self
                                .send_to(
                                    from,
                                    Message::GetHeaders {
                                        start_height: our_height as u64 + 1,
                                        limit: 100,
                                    },
                                )
                                .await;
                        }
                    }
                }
            }
            Message::GetMempoolTxs => {
                let txs = {
                    let state = self.state.lock().await;
                    state.mempool.txs.clone()
                };
                self.send_to(from, Message::MempoolTxs(txs)).await?;
            }
            Message::MempoolTxs(txs) => {
                println!("Received {} transactions from {} mempool", txs.len(), from);
                for tx in txs {
                    self.process_new_transaction(tx, from).await?;
                }
            }
            Message::BroadcastTransaction(tx) => {
                println!(
                    "Received request to broadcast transaction {} from {}",
                    tx.id(),
                    from
                );
                self.process_new_transaction(tx, from).await?;
            }
            Message::GetMempoolInfo => {
                let (count, total_size, min_fee, max_fee) = self.get_mempool_info().await;
                self.send_to(
                    from,
                    Message::MempoolInfo {
                        count,
                        total_size,
                        min_fee,
                        max_fee,
                    },
                )
                .await?;
            }
            Message::MempoolInfo {
                count,
                total_size,
                min_fee,
                max_fee,
            } => {
                println!(
                    "Received mempool info from {}: count={}, size={} bytes, fees={}-{}",
                    from, count, total_size, min_fee, max_fee
                );
            }
            _ => {
                println!("Received unhandled message from {}: {:?}", from, msg);
            }
        }
        Ok(())
    }
}

async fn handle_peer(
    stream: TcpStream,
    addr: SocketAddr,
    state: Arc<Mutex<NodeState>>,
    node: P2PNodeHandle,
    agent: String,
) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<PeerCmd>();

    // Add to peer list
    {
        let mut s = state.lock().await;
        s.known_addrs.insert(addr);
        s.peer_senders.insert(addr, tx);
    }

    println!("Starting message loop for {}", addr);
    let (reader, writer) = stream.into_split();
    let mut reader = reader;
    let writer = Arc::new(Mutex::new(writer));

    let writer_clone = Arc::clone(&writer);
    let state_for_reader = Arc::clone(&state);
    let peer_reader = async move {
        // Send initial Handshake upon connection (for both inbound and outbound)
        {
            let best_height = {
                let s = state_for_reader.lock().await;
                s.chain.height() as u64
            };
            let mut w = writer_clone.lock().await;
            Message::Handshake {
                version: 1,
                best_height,
                agent,
            }
            .send_async(&mut *w)
            .await?;
        }

        loop {
            let msg = Message::decode_async(&mut reader)
                .await
                .context("Failed to decode peer message")?;
            println!("Received message from {}: {:?}", addr, msg);

            match msg {
                Message::Pong => {
                    println!("Received Pong from {}", addr);
                }
                m => {
                    node.process_message(m, addr).await?;
                }
            }
        }
    };

    let peer_writer = async move {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                PeerCmd::SendMessage(msg) => {
                    let mut w = writer.lock().await;
                    msg.send_async(&mut *w).await?;
                }
                PeerCmd::Disconnect => {
                    break;
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
        s.peer_senders.remove(&addr);
        s.outgoing_conns.remove(&addr);
    }

    res
}
