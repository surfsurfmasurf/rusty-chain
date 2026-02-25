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

Next:
- Implement block and transaction sync (GetBlocks / GetHeaders) for new nodes.
- Peer discovery (DNS seeds or addr gossip).
- Better logging and error reporting in the P2P loop.

## Day 21
- P2P: Added `GetHeaders` and `GetData` messages to the network protocol.
- P2P: Implemented server-side handlers for `GetHeaders` (retrieving block headers) and `GetData` (retrieving full blocks by hash).
- P2P: Enhanced `P2PNodeHandle` with `get_headers` and `get_blocks_by_hash` thread-safe helpers.
- Network: Added unit tests for new message types and P2P roundtrip logic.
- Pushed progress to main branch (Day 21 of 30).

## Day 26
- P2P: Major refactor of peer message handling to simplify the async loop.
- P2P: Consolidated all message processing logic into `process_message` for consistency.
- P2P: Moved `Ping`/`Pong` response logic into `process_message`.
- P2P: Simplified `P2PNode` and `handle_peer` by removing redundant handles and clones.
- Network: Removed unused `RequestStatus` and `ResponseStatus` message variants.
- Cleanup: Fixed clippy warnings and applied standard formatting.
- Tests: Verified all core, state, and networking tests pass.
- Pushed progress to main branch (Day 26 of 30).


