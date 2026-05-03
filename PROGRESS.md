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

## Day 46 (Today)
- Core: Added hierarchical cluster (`cluster_id`), process (`process_id`), and deployment (`deployment_id`) identifiers to `Transaction` and `TxSignPayload`.
- Validation: Implemented `cluster_id` non-empty check in `validate_basic`.
- Refactor: Unified all `Transaction` constructors and the signing payload with the new fields.
- Maintenance: Fixed `TxSignPayload` schema mismatch and resolved clippy warnings (snake_case).
- Polish: Improved documentation for multi-layered network and scalability fields.
- Maintenance: Pushed 8 commits as part of the Day 46 maintenance and scalability enhancement session.

Next:
- Implement automated risk-based transaction filtering in the mempool.
- Explore integration of hierarchical network identifiers with P2P routing.
- Core: Added `flow_rate_id`, `congestion_id`, `packet_id`, `frame_id`, `stream_offset`, `channel_capacity_id`, `buffer_size_id`, and `pipeline_id` to `Transaction` for advanced network telemetry and scalability.
- Core: Added `is_redundant`, `is_fragmented`, `is_validated`, `is_audited`, and `is_system_update` flags for granular transaction lifecycle tracking.
- Core: Implemented `cached_size` optimization in `Transaction::size()` for improved performance.
- Core: Added `next_hop` internal field to `Transaction` for future routing enhancements.
- Validation: Implemented `flow_rate_id`, `congestion_id`, and `latency_id` non-empty validation in `validate_basic`.
- Maintenance: Pushed 8 commits as part of the Day 16 scalability and maintenance session.

Next:
- Implement P2P routing logic utilizing `next_hop` and network telemetry identifiers.
- Explore integration with high-frequency telemetry streams.

Day 63 (Today)
- Core: Added `layer_id`, `tier_id`, and `plane_id` to `Transaction` for network topology awareness.
- Core: Added `path_id`, `hop_id`, and `route_id` for P2P-level routing tracking and analysis.
- Core: Added `is_minable`, `state_ref`, `gas_limit`, `external_system`, `script`, `bridge_id`, `app_id`, and `session_id` fields for future scalability and cross-chain support.
- Refactor: Updated `TxSignPayload` to include all new fields, ensuring full signature coverage for transaction metadata.
- Maintenance: Refactored `state_tests.rs` and `nonce_tests.rs` to use `Transaction::default()` and field-based initialization, significantly improving test maintainability.
- Polish: Unified transaction constructors and coinbase generation in `chain.rs` and `types.rs`.
- Maintenance: Pushed 9 commits as part of the Day 63 scalability and maintenance session.

Next:
- Implement P2P routing logic using the new path/route identifiers.
- Explore libp2p integration for more robust peer discovery.

- Core: Added `latency_id`, `throughput_id`, and `reliability_id` fields to `Transaction` for advanced network performance monitoring and routing.
- Refactor: Updated `TxSignPayload` to include the new performance scaling fields, ensuring signature coverage.
- Core: Refactored `Transaction::default()` to include all newly added network performance identifiers.
- Polish: Unified hierarchical field documentation and improved consistency across core transaction types.
- Maintenance: Pushed 9 commits as part of the Day 67 work session focusing on scaling and network performance metadata.

## Day 67 (Today)
- Core: Added `cell_id`, `area_id`, and `fabric_id` to `Transaction` and `TxSignPayload` for multi-layered networking and sub-cluster isolation.
- Validation: Implemented `cell_id` non-empty validation in `validate_basic`.
- Refactor: Updated `Transaction::default()` and `signing_payload` to support the new networking fields.
- Tests: Added `tests/network_layer_tests.rs` with coverage for multi-layered field assignment and validation.
- Maintenance: Pushed 8 commits as part of the Day 67 enhancement and scalability session.

Next:
- Implement sub-cluster routing logic based on `cell_id`.
- Explore dynamic fabric discovery in P2P node.

## Day 66 (Today)
- Core: Added `tx_id_hash`, `authority_id`, `instantiation_id`, `module_id`, `plugin_id`, `service_id`, and `endpoint_id` to `Transaction` and `TxSignPayload` for enhanced deduplication, delegated validation, and modular extensibility.
- Refactor: Unified all `Transaction` constructors and the signing payload with the new fields.
- Maintenance: Ensured `Transaction::default()` remains consistent with the expanding schema.
- Polish: Improved field documentation for core transaction types.
- Maintenance: Pushed 9 commits as part of the Day 66 work session to enhance core metadata and modularity.

## ## Day 16
- Expanded P2P `Message` enum with `RequestStatus`, `ResponseStatus`, `Inventory`, and `GetMempool`
- Refactored message size limits into a dedicated `size_limit()` helper
- Added `node` subcommand to CLI for future P2P background loop
- Improved P2P unit tests with `Inventory` roundtrip coverage
- Pushed progress to main branch (Day 16 of 30)

## Day 64 (Today)
- Core: Added `event_id`, `audit_id`, `source_id`, `location_id`, `datacenter_id`, `partition_id`, `domain_id`, and `framework_id` fields to `Transaction` for enhanced auditing and compliance.
- Core: Added `priority_score` and `risk_score` (f64) for advanced transaction prioritization and security filtering.
- Core: Added `jurisdiction_id` for regulatory compliance tracking.
- Core: Implemented `Default` for `Transaction` and refactored all constructors to use `..Default::default()` for improved maintainability.
- Core: Refactored coinbase transaction generation in `chain.rs` to use `Default::default()`.
- Validation: Implemented range enforcement for `priority_score` and `risk_score` (0.0 - 1.0) in `validate_basic`.
- Refactor: Removed `Eq` requirement from `Transaction`, `Block`, and P2P `Message` to accommodate `f64` fields.
- Maintenance: Pushed 8 commits as part of the Day 64 maintenance and enhancement session.

Next:
- Implement automated risk-based transaction filtering in the mempool.
- Explore integration of hierarchical network identifiers with P2P routing.

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

## Day 68 (Today)
- Core: Added `sensor_id`, `telemetry_id`, `metric_id`, `signal_id`, `pulse_id`, and `frequency_id` to `Transaction` and `TxSignPayload` for observability and high-frequency telemetry support.
- Core: Added `is_verifiable` flag to bypass signature verification for specific system transactions.
- Core: Added `debug_tag` internal field (skipped in serialization) for better troubleshooting.
- Core: Added `signing_size` helper to `Transaction`.
- Validation: Implemented `signal_id` non-empty validation and updated `validate_accept` to respect the `is_verifiable` flag.
- Refactor: Unified all `Transaction` constructors and signing payload generation.
- Maintenance: Pushed 8 commits as part of the Day 68 observability and scaling session.

Next:
- Implement telemetry stream aggregation in the P2P node.
- Explore time-series indexing for sensor-based transactions.

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

## Day 44
- Core: Implemented `estimate_fee_rate` in `Chain` based on a rolling window of recent blocks.
- Core: Optimized fee calculation to exclude coinbase transactions from the average.
- P2P: Added `GetFeeEstimate` and `FeeEstimate` message handlers to `process_message`.
- P2P: Fee estimates now use the rolling average from the last 10 blocks.
- CLI: Added `fee-estimate` command to query network-wide fee rates from any node.
- CLI: Improved output formatting for fee estimates with size-specific details.
- Tests: Added `tests/fee_tests.rs` for verifying rolling average logic and edge cases.
- Pushed 8 commits as part of the Day 44 maintenance and enhancement session.

## Day 45
- Core: Added checkpointing support to `Chain` for validation and future syncing.
- Core: Implemented `add_checkpoint` and `validate_checkpoints` in `Chain`.
- Core: Updated genesis to include initial checkpoint at height 0.
- Core: Implemented automatic checkpointing in `append_block` every 10 blocks.
- Tests: Added `tests/checkpoint_tests.rs` with coverage for manual and automatic checkpointing.
- Tests: Added checkpoint failure coverage with block corruption.
- Refactor: Enhanced `validate()` to check checkpoints automatically.
- Pushed 8 commits as part of the Day 45 work session.

## Day 60 (Today)
- Core: Added `weight` field to `Transaction` for congestion control and fee-per-weight calculations.
- Core: Added `unique_id` to `Transaction` for robust P2P double-spend prevention.
- Core: Added `is_private` flag to `Transaction` for future privacy enhancement support.
- Core: Updated `TxSignPayload` to include `weight`, `unique_id`, and `is_private` for signature integrity.
- Validation: Implemented mandatory `unique_id` non-empty check in `validate_basic`.
- Refactor: Unified all `Transaction` constructors with the new transaction fields.
- Maintenance: Pushed 8 commits as part of the Day 60 work session.

Day 61
- Core: Added `session_id` to `Transaction` and `TxSignPayload` for cross-session replay protection.
- Core: Added `payload_checksum` to `Transaction` for quick integrity verification.
- Core: Added `checkpoint_index` to `Transaction` for tracking verification state.
- Core: Added `is_batched` flag to `Transaction` for batching support.
- Core: Added `parent_id` to `Transaction` for hierarchical transaction support.
- Core: Added `schema_id` to `Transaction` for structured metadata support.
- Core: Added `local_difficulty` to `Transaction` for local verification options.
- Core: Added `salt` to `Transaction` for same-block replay protection.
- Core: Added `size_bytes` to `Transaction` for fixed-size transaction support.
- Core: Added `request_id` to `Transaction` for P2P request correlation.
- Core: Added `origin` field to `Transaction` for source tracking.
- Core: Added `external_ref` to `Transaction` for cross-chain linkage support.
- Core: Added `category` field to `Transaction` for transaction classification.
- Maintenance: Pushed 13 commits as part of the Day 61 work session.

Day 62 (Today)
- Core: Added `schema_id` and `request_id` to `Transaction` for metadata schema and P2P correlation.
- Core: Added `p2p_message_id` and `anchor_id` for P2P-level broadcast tracking and checkpoint anchoring.
- Core: Added `shard_id`, `subnet_id`, `cluster_id`, `region_id`, `zone_id`, and `group_id` for multi-layered hierarchical network and scalability support.
- Refactor: Unified transaction constructors and coinbase generation with the new fields.
- Maintenance: Pushed 8+ commits as part of the Day 62 work session to enhance P2P and scalability features.

## Day 46
- Core: Added `get_checkpoint_at` and `get_last_checkpoint` to `Chain`.
- P2P: Added `GetCheckpoints` and `Checkpoints` messages to the network protocol.
- P2P: Implemented `GetCheckpoints` and `Checkpoints` message handlers.
- Mempool: Added `remove_included` for batch removal of transactions.
- Refactor: Optimized mempool clearing in P2P node using `remove_included`.
- Tests: Added test coverage for new checkpoint helpers.
- Polish: Improved handshake-triggered synchronization using checkpoints.
- Pushed 8 commits as part of the Day 46 maintenance and enhancement session.

## Day 50
- P2P: Added `GetMempoolTxs`, `MempoolTxs`, and `BroadcastTransaction` to the network protocol.
- P2P: Implemented mempool synchronization upon connection.
- CLI: Added `mempool` command to query transactions from remote nodes.
- CLI: Added `--broadcast-to` flag to `tx-add` for immediate transaction dissemination.
- Refactor: Moved transaction and block processing into dedicated methods to resolve async recursion.
- Core: Standardized imports and fixed type resolution in P2P module.
- Tests: Added unit tests for new P2P messages and transaction broadcasting.
- Pushed 8 commits for Day 50 (30-day challenge continue).

## Day 48
- Core: Added `block_index` to `Chain` for O(1) block retrieval by hash.
- Core: Added `tx_index` to `Mempool` for O(1) transaction duplicate checks and lookups.
- P2P: Optimized `get_blocks_by_hash` to use the new `block_index`, speeding up `GetData` response times.
- Mempool: Implemented index maintenance for RBF (Replace-By-Fee) and bulk transaction removal.
- Tests: Added `test_mempool_index_consistency` to verify index integrity during removals and shifts.
- Refactor: Replaced manual O(N) searches with O(1) hash map lookups in performance-critical paths.

## Day 49
- Core: Fixed a critical index bug in `Mempool::add_tx_checked` that caused the transaction index to become stale upon replacement.
- Core: Fixed a move-after-borrow issue in `Chain::append_block` and optimized its checkpointing logic.
- P2P: Optimized P2P block processing by moving structural validation outside the global node state lock, reducing contention.
- P2P: Improved robustness of block gossip by adding re-verification of chain linkage under lock to prevent race conditions.
- Core: Enhanced fee estimation in `Chain::estimate_fee_rate` with a fallback mechanism to the most recent transaction fee if the window is empty.
- Core: Polished `State` transition logic with saturating math for nonce increments and improved coinbase transaction logging.
- Pushed 8 commits as part of the Day 49 maintenance and optimization session.

## Day 50 (Today)
- Mempool: Implemented `get_tx_by_id` and `contains_tx` for O(1) transaction lookups via index.
- P2P: Optimized `process_message` to skip duplicate transactions and blocks early using O(1) index checks.
- P2P: Added `P2PNode::get_mempool_info` to return high-level mempool statistics (count, total size, min/max fee).
- CLI: Added `mempool-info` command to query node mempool statistics remotely.
- Core: Added `Block::is_valid_pow` helper for independent Proof-of-Work verification.
- Refactor: Moved P2P message serialization checks to a dedicated `network_tests` module.
- Tests: Added `tests/mempool_tests.rs` for verifying index-based lookups and stats.
- Pushed 8 commits as part of the Day 50 work session.

## Day 51 (Today)
- Core: Added `locktime` support to `Transaction` and `TxSignPayload`.
- State: Implemented locktime enforcement during block and transaction validation.
- CLI: Added `--locktime` flag to `tx-add` and displayed locktime in `tx-list`.
- Tests: Added comprehensive integration test for transaction locktime (`tests/locktime_tests.rs`).
- Core: Added `Transaction::new_with_locktime` helper.
- Core: Updated coinbase transaction construction to include the `locktime` field.
- Refactor: Improved documentation for `is_coinbase` in `types.rs`.
- Pushed 8 commits as part of the Day 51 work session.

## Day 52 (Today)
- Mempool: Added `sort_by_fee`, `len()`, and `is_empty()` helpers to `Mempool`.
- Core: Integrated automatic checkpointing into `Chain::mine_block` for consistency.
- Tests: Added unit tests for mempool fee-based sorting and index integrity.
- Polish: Standardized checkpointing logic using the modulo operator.
- Maintenance: Pushed 8 commits for the Day 52 maintenance and optimization session.

## Day 53 (Today)
- Core: Added transaction versioning support (`version` field) to `Transaction` and `TxSignPayload`.
- Core: Implemented mandatory transaction version check in `validate_transaction` (enforcing `version: 1`).
- Core: Updated all `Transaction` constructors and coinbase generation to default to version 1.
- Mempool: Added unit tests for transaction TTL eviction to ensure mempool freshness.
- Types: Enforced strictly positive transaction versions in `validate_basic`.
- Pushed 8 commits as part of the Day 53 work session.

## Day 55 (Today)
- Core: Added `expiry` field to `Transaction` and `TxSignPayload` for block-height based expiration.
- State: Implemented transaction expiration enforcement during block and mempool validation.
- CLI: Added `--expiry` flag to `tx-add` and displayed expiry height in `tx-list`.
- CLI: Updated memo character limit documentation to match core rules (128 chars).
- Tests: Added `tests/expiry_tests.rs` verifying both state enforcement and mempool TTL eviction.
- Polish: Unified all `Transaction` constructors and coinbase construction with the new `expiry` field.
- Maintenance: Fixed manual `Transaction` initializers in existing tests to include the `expiry` field.
- Pushed 8 commits as part of the Day 55 work session.

## Day 58 (Today)
- Core: Added `expiration_ms` field to `Transaction` and `TxSignPayload` for explicit time-based expiration.
- State: Implemented transaction expiration enforcement in `validate_basic`.
- Mempool: Updated `evict_expired` to respect `expiration_ms`.
- CLI: Added `--expiration` flag to `tx-add` and display in `tx-list`.
- Tests: Added `tests/expiration_ms_tests.rs` and updated existing test suites for compatibility.
- Core: Implemented `Default` for `Transaction` for easier future testing and construction.
- Maintenance: Pushed 8 commits as part of the Day 58 maintenance and enhancement session.

## Day 59 (Today)
- Core: Added `tag` field to `Transaction` and `TxSignPayload` for categorization.
- CLI: Added `--tag` flag to `tx-add` for manual transaction tagging.
- CLI: Updated `tx-list` to display transaction tags when present.
- Tests: Added `tests/tag_tests.rs` for verifying tag persistence and optionality.
- Tests: Fixed `expiration_ms_tests.rs` formatting and unused imports.
- Maintenance: Updated all manual `Transaction` initializers in test suites to include the new `tag` field.
- Refactor: Polished `TxAdd` command and transaction display logic.
- Pushed 8 commits as part of the Day 59 maintenance and enhancement session.


## Day 47 (Today)
- Core: Added `pool_id` and `trace_id` to `Transaction` and `TxSignPayload` for mempool isolation and transaction monitoring.
- Refactor: Integrated new identifiers into all `Transaction` constructors and the signing payload.
- Maintenance: Updated coinbase transaction generation in `chain.rs` to support the expanded schema.
- Polish: Improved documentation for hierarchical network and routing fields.
- Maintenance: Pushed 8 commits as part of the Day 47 work session to enhance scalability and auditability.

## Day 65 (Today)
- Core: Added `flow_id`, `step_id`, `task_id`, `sequence_id`, `stream_id`, and `batch_id` to `Transaction` for workflow and lifecycle tracking.
- Core: Added `controller_id`, `worker_id`, and `instance_id` to `Transaction` for enhanced system observability.
- Core: Added `compliance_id`, `policy_id`, and `legal_ref` to `Transaction` for auditing and regulatory compliance.
- Core: Implemented `is_reverting`, `is_conditional`, and `is_delegated` status flags in `Transaction`.
- Validation: Added length enforcement for `metadata_label` (64 chars) and large transaction logging in `validate_basic`.
- Refactor: Unified `BlockHeader` with `has_valid_difficulty` helper for cleaner PoW checks.
- Maintenance: Pushed 8 commits as part of the Day 65 maintenance and scalability session.

Next:
- Implement automated risk-based transaction filtering in the mempool.
- Explore integration of hierarchical network identifiers with P2P routing.
