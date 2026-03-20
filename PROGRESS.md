# Progress log

## Day 1
- Repo bootstrap
- Rust project scaffolding
- CI + docs baseline

Next:
- Define core data structures: `BlockHeader`, `Block`, `Chain`
- Add hashing + serialization

## Day 2
- Added core types: BlockHeader/Block/Transaction
- Added hashing helper (sha256 hex)
- Added Chain storage: genesis + JSON load/save
- CLI: init/status now write/read chain.json
- Tests: genesis + save/load roundtrip

## Day 3
- Genesis now uses real timestamp (ms)
- Added `validate` command (genesis invariants + prev_hash linkage)
- Added minimal PoW + `mine` command (leading '0' hex chars)
- Tests: validation + mining PoW checks

## Day 4
- Store PoW difficulty in `chain.json` (`Chain.pow_difficulty`) with serde defaults for backward compatibility
- `mine` persists the chosen difficulty, `status` prints it
- `validate` now enforces PoW for all non-genesis blocks
- Tests: PoW failure detection + JSON defaulting when the field is missing

Day 5
- Added mempool (file-backed) with tx add/list CLI
- Mining now includes mempool txs when present
- Merkle root now hashes tx hashes (order-sensitive)
- Added tests for merkle root + tx hash stability

## Day 6
- Status now prints total tx count in-chain (and optional mempool tx count)
- Added basic tx validation (non-empty from/to, from!=to, amount>0)
- `mine` refuses to mine invalid mempool txs (sanity check)
- Tests: tx validation + mempool insert rejection

## Day 7
- CLI: `tx-list` now prints a short tx hash prefix for easier demo/debug
- Mempool: reject duplicate txs by tx hash
- CLI: `tx-add` prints the tx hash
- Tests: added coverage for duplicate rejection

Next:
- Start Week 2: signatures / account model / nonce enforcement
- Nonce enforcement rules (per-sender monotonic nonces)
- Better tx display formatting (fees? signature preview?)

## Day 8
- Added chain helper to compute per-sender next nonce (max+1)
- Mempool: added nonce-aware tx insertion (`add_tx_checked`) and expected-nonce helper
- CLI: `tx-add` now auto-fills nonce when omitted, and enforces per-sender nonces using chain+mempool
- CLI: `mine` validates mempool nonce sequence before draining/mining (prevents accidental tx loss)
- Tests: added nonce coverage (chain + mempool)

## Day 9
- Added ed25519 crypto helpers + key storage (`data/keys/*.json`)
- CLI: `keygen` + `addr` for local demo keypairs
- Transactions now have an explicit signing payload, with optional signature fields
- Tests: signature verification + tamper detection

## Day 10
- Mempool now validates tx signatures (unsigned allowed, signed must verify)
- Signed txs are now bound to `from=<pubkey_hex>` (prevents "sign as A, send as B")
- CLI: `tx-add --signer <name>` binds `from` to signer pubkey hex automatically
- Tests: coverage for from/pubkey binding and mempool rejection

Next:
- Decide account model (UTXO vs account) and enforce balances during `mine`/`validate`
- Improve tx display formatting (fees? signature preview?)

## Day 11
- Added `Account` and `State` structures (balance/nonce tracking)
- Implemented state validation: check balances and nonces when validating blocks
- Added coinbase transaction support (System -> Miner, +50 coins)
- CLI: `mine` now accepts `--miner <address>` to earn block rewards
- Tests: added state transition tests (insufficient funds, invalid nonces, balance updates)

Day 12
- Added `fee` field to `Transaction` and `TxSignPayload` (default: 0)
- Implemented fee deduction from sender balance during state application
- Implemented fee collection: miner reward = block reward (50) + sum(tx fees)
- CLI: `tx-add` now supports optional `--fee`
- CLI: `tx-list` now displays fees
- Tests: added coverage for fee deduction, miner collection, and insufficient balance for fees

## Day 13
- Added `Transaction::id()` helper for unified hash access across core and CLI
- Refactored `Mempool`, `Chain`, and `CLI` to use `tx.id()` instead of direct hashing
- Robustness: used saturating math for balance updates in `State` application
- Polish: removed `expect` from `now_ms` (now uses `unwrap_or` for safety)
- Added robustness tests for state transitions

## Day 14
- Refactored coinbase transactions: nonce now matches block height for easier indexing
- Enhanced state validation: enforce coinbase nonce and reward amount (block reward + fees)
- Improved mining safety: pre-validate mempool transactions against ledger state before committing to PoW
- Added `Block` and `Chain` helpers (`is_coinbase`, `total_reward`, `tip_header`)
- Fixed and updated test suite to comply with new consensus and state rules

Next:
- Start Week 3: P2P networking baseline (libp2p or simple tokio tcp)
- Block propagation

## ## Day 16
- Expanded P2P `Message` enum with `RequestStatus`, `ResponseStatus`, `Inventory`, and `GetMempool`
- Refactored message size limits into a dedicated `size_limit()` helper
- Added `node` subcommand to CLI for future P2P background loop
- Improved P2P unit tests with `Inventory` roundtrip coverage
- Pushed progress to main branch (Day 16 of 30)

## Day 18
- P2P: Implemented baseline P2P background server using `tokio`.
- P2P: Added `P2PNode` with support for both inbound listeners and outbound connection attempts.
- P2P: Added initial `Handshake` message for P2P versioning and height exchange.
- P2P: Implemented basic `Ping`/`Pong` message handling in the async peer loop.
- P2P: Improved peer handler with robust disconnection and error handling.
- CLI: Integrated `node` command to launch the P2P listener and connect to peers.
- Tests: Added P2P message handshake unit tests.

## Day 19
- P2P: Implemented thread-safe `NodeState` with active peer tracking and `peer_senders` list.
- P2P: Added `mpsc` channel-based communication for each peer to handle concurrent reads/writes.
- P2P: Implemented `broadcast` and `broadcast_except` for efficient message dissemination.
- P2P: Implemented a basic gossip protocol for `NewTransaction` and `NewBlock` propagation.
- P2P: Added `seen_messages` (HashSet) cache to prevent infinite gossip loops.
- P2P: Introduced `P2PNodeHandle` for lightweight, shared access to node state from peer handlers.
- Refactor: Split `handle_peer` into reader/writer loops using `tokio::select!`.

## Day 20
- Validation: Implemented `validate_transaction` and `validate_block` in `Chain` to unify state checks across CLI and P2P.
- P2P: Implemented `process_message` to handle incoming gossip (TXs and Blocks) with full validation.
- P2P: Node now tracks `Chain` and `Mempool` state, enabling real-time validation of gossiped data.
- P2P: Automatic mempool clearing when blocks are accepted via P2P.
- CLI: Updated `node` command to load/initialize local chain and mempool before starting the P2P server.
- Tests: Added unit tests for P2P gossip identification.

## Day 27
- P2P Sync: Implemented initial blockchain sync logic (Handshake -> GetHeaders -> GetData -> Blocks).
- P2P Sync: Added support for multi-batch header synchronization (100 headers per batch).
- P2P Sync: Integrated peer height check during handshake to trigger sync when behind.
- P2P Sync: Optimized block processing with duplicate checks to prevent redundant validation.
- Polish: Increased transaction memo character limit from 64 to 128 for more flexibility.
- Tests: Added unit tests for memo length constraints and message roundtrips.
- Refactor: Cleaned up P2P message handlers and fixed clippy warnings.
- Pushed progress to main branch (Day 27 of 30).

## Day 28
- P2P Logging: Added `Message::get_type_name()` to simplify and unify P2P message logging.
- P2P: Improved `process_message` logging to automatically include the message type.
- P2P: Added error logging in `send_to` when a target peer is not found in the state.
- Refactor: Cleaned up redundant ping/pong logging in peer handlers.
- Tests: Added unit tests for new message type name helper.
- Pushed progress to main branch (Day 28 of 30).

## Day 29
- P2P: Added `GetAddr` and `Addr` messages for peer discovery.
- P2P: Implemented `known_addrs` tracking in `NodeState`.
- P2P: Integrated peer discovery into the handshake process.
- P2P: Implemented gossip for `Addr` messages to spread discovery across the network.
- P2P: Added gossip loop prevention for `Addr` messages using address set hashing.
- Tests: Added unit tests for `Addr` and `GetAddr` message serialization.
- Pushed progress to main branch (Day 29 of 30).

## Day 30
- P2P: Implemented peer list persistence to disk (`--peers-file`).
- P2P: Added background task to periodically save known addresses to JSON.
- P2P: Automatic loading of peers from file on node startup.
- Refactor: Replaced `Vec` with `HashMap` for peer tracking in `NodeState` for O(1) lookups and removal.
- Polish: Final clippy and format pass for the 30-day challenge completion.
- Pushed progress to main branch (Day 30 of 30).

## Day 31
- Added `sequence` field to `Transaction` and `TxSignPayload` for future-proofing (e.g. fee bumping, replace-by-fee).
- Updated test suite (`state_tests.rs`, `nonce_tests.rs`) to include mandatory `sequence` in manual `Transaction` initializers.
- Refactored `validate_basic` in `Transaction` for cleaner error handling and removed redundant coinbase-specific checks.
- Improved P2P message documentation in `network.rs` with doc comments.
- Polished P2P message handlers and unified validation logic.
- Pushed 4 commits as part of the Day 31 maintenance and enhancement session.

## Day 32
- Implemented Replace-By-Fee (RBF) logic in the mempool.
- Added `Transaction::new_with_sequence` helper for unified transaction creation.
- Enhanced `add_tx_checked` to enforce both strictly higher fees and higher sequence numbers for replacements.
- Added comprehensive unit tests for RBF logic (`tests/rbf_tests.rs`).
- Refactored `validate_basic` to include initial sequence-related checks.
- Pushed 8 commits as part of the Day 32 work session.

## Day 33
- P2P: Implemented rejection messages for invalid RBF attempts in `process_message`.
- CLI: Added `--sequence` flag to `tx-add` for manual RBF control.
- CLI: `tx-list` now displays the sequence number for each transaction.
- Tests: Added `rbf_rejects_lower_fee_even_with_higher_sequence` unit test.
- Docs: Updated README with RBF and CLI enhancement details.
- Pushed 5 commits (Day 33 of 30+).

## Day 34
- Core: Added `timestamp_ms` to `Transaction` and `TxSignPayload` for time-aware mempool.
- Mempool: Implemented `evict_expired` to support TTL-based transaction eviction.
- CLI: Added `tx-evict` command to manually prune expired transactions from mempool.
- CLI: `tx-list` and `tx-add` now support and display transaction timestamps.
- Tests: Added `eviction_tests` to verify TTL logic in mempool.
- Pushed 8 commits (Day 34 of 30+).

## Day 35
- P2P: Implemented peer reputation scoring system.
- P2P: Peers gain reputation for valid blocks (+10) and transactions (+1).
- P2P: Peers lose reputation for invalid blocks (-50) and transactions (-10).
- P2P: Added background mempool evictor to remove expired transactions every 5 minutes.
- Refactor: Unified logging for transaction rejection with sender address inclusion.
- Tests: Verified core reputation update logic and background task stability.
- Pushed 8 commits as part of the Day 35 work session.

## Day 36
- P2P: Implemented automatic peer banning for reputation scores below -100.
- P2P: Added `Message::Reject` handling in peer message loop for better feedback.
- P2P: Implemented `PeerCmd::Disconnect` for explicit and clean peer disconnection.
- Refactor: Optimized `update_reputation` to trigger immediate disconnection on ban.
- Docs: Added documentation for reputation-based banning logic.
- Pushed 8 commits as part of the Day 36 work session.

## Day 37
- P2P: Implemented peer whitelisting to prevent accidental banning of trusted nodes.
- P2P: Added `Message::Whitelist` to the network protocol for peer-to-peer whitelist requests.
- P2P: Updated `NodeState` to track `whitelisted_peers` and bypass reputation logic for them.
- P2P: Implemented automatic reputation reset and ban removal upon whitelisting.
- Tests: Added comprehensive P2P whitelisting tests (`tests/p2p_tests.rs`).
- Refactor: Cleaned up P2P reputation update logic with whitelisting checks.
- Pushed 5 commits (Day 37 of 30+).

## Day 38
- P2P: Implemented `peers` CLI command to query reputation scores from a running node.
- P2P: Implemented persistent whitelist storage to disk (`data/whitelist.json`).
- P2P: Added background whitelist saver task (periodic save every 60s).
- P2P: Automatic loading of whitelisted peers from disk on node startup.
- Refactor: Updated `P2PNode::new` and CLI to support the new whitelist persistence logic.
- Tests: Updated `tests/p2p_tests.rs` to comply with the new `P2PNode` signature.
- Pushed 8 commits (Day 38 of 30+).

## Day 39
- P2P: Implemented manual `ban` and `unban` CLI commands for better network control.
- P2P: Implemented `banned`, `whitelisted`, and `unwhitelist` CLI commands.
- P2P: Expanded network protocol with `Ban`, `Unban`, `GetBanned`, `Banned`, `GetWhitelisted`, `Whitelisted`, and `Unwhitelist` messages.
- P2P: Added administration logic to `NodeState` to handle manual peer management.
- Refactor: Improved CLI output for peer lists with better formatting.
- Fixed: Resolved move-after-borrow issues in CLI message handling.
- Pushed 8 commits (Day 39 of 30+).

## Day 40
- P2P: Implemented P2P version negotiation (enforcing `version: 1`).
- P2P: Added `agent` string to `Handshake` for client identification (e.g. `rusty-chain/0.1.0`).
- P2P: Node now broadcasts its own handshake to both inbound and outbound peers.
- CLI: Added optional `--agent` flag to `node` command to customize node identification.
- Refactor: Unified handshake logic in peer handlers and improved disconnection handling for version mismatches.
- Tests: Updated P2P handshake tests to cover new `agent` field.
- Pushed 8 commits as part of the Day 40 work session.

## Day 41
- P2P: Implemented `GetReputation` and `Reputation` messages for peer-to-peer reputation sharing.
- P2P: Added `NodeState::get_reputation_snapshot()` to generate a mapping of `PeerAddr` to `ReputationScore`.
- P2P: Implemented reputation request handler in `process_message` (on `GetReputation`, send `Reputation`).
- P2P: Integrated reputation merging logic: incoming reputation data updates local scores using a weighted average (50/50).
- P2P: Added a background gossip task to periodically request reputation snapshots from connected peers every 10 minutes.
- Refactor: Cleaned up reputation data structures and added unified `ReputationScore` type for clarity.
- Tests: Added unit tests for reputation snapshot serialization and merging.
- Pushed 8 commits as part of the Day 41 work session.

## Day 42
- P2P: Added `GetAllAddr` message to the network protocol for complete peer discovery.
- P2P: Narrowed `GetAddr` to return only active peers, while `GetAllAddr` returns all known addresses.
- P2P: Implemented background reconnection task to automatically restore lost connections.
- CLI: Added `known-addrs` command to query all seen addresses from a remote node.
- Core: Improved P2P node state to track outgoing connections for better management.
- Tests: Added unit test for new `GetAllAddr` message roundtrip.
- Polish: Unified P2P message handlers and improved connection stability.
- Pushed 8 commits as part of the Day 42 work session.

## Day 43
- Core: Added `size()` helper to `Transaction` and `Block` for serialization size (used for fee estimation).
- P2P: Added `GetFeeEstimate` and `FeeEstimate` messages to the network protocol.
- Refactor: Improved `total_reward` to correctly handle coinbase vs normal transaction fees.
- Tests: Added `size_tests.rs` and updated `network.rs` with fee estimate roundtrip tests.
- Pushed 8+ commits for Day 43 (maintaining the streak).

Next:
- Implement a basic fee estimation algorithm based on recent block fee rates.
- Add UPnP support for better connectivity.
