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

Next:
- Wire signed txs into CLI (`tx-add --signer <name>`)
- Decide account model (UTXO vs account) and enforce balances during `mine`/`validate`
- Improve tx display (fees/signature preview)
