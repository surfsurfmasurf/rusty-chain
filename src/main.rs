use anyhow::Context;
use clap::{Parser, Subcommand};

use rusty_chain::core::chain::Chain;
use rusty_chain::core::hash::tx_hash;
use rusty_chain::core::mempool::Mempool;
use rusty_chain::core::types::Transaction;

use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(name = "rusty-chain")]
#[command(about = "A mini blockchain built in Rust (30-day build).", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new chain (writes genesis to disk)
    Init {
        /// Output path for chain JSON
        #[arg(long)]
        path: Option<String>,
    },

    /// Print current chain status
    Status {
        /// Input path for chain JSON
        #[arg(long)]
        path: Option<String>,

        /// Optional path for mempool JSON
        #[arg(long)]
        mempool: Option<String>,
    },

    /// Validate chain invariants (genesis + linkage)
    Validate {
        /// Input path for chain JSON
        #[arg(long)]
        path: Option<String>,
    },

    /// Mine and append a block (uses mempool txs if available)
    Mine {
        /// Path for chain JSON (will be created if missing)
        #[arg(long)]
        path: Option<String>,

        /// Optional path for mempool JSON
        #[arg(long)]
        mempool: Option<String>,

        /// PoW difficulty (leading '0' hex chars)
        #[arg(long, default_value_t = 3)]
        difficulty: usize,
    },

    /// Add a transaction to the mempool
    TxAdd {
        /// Optional path for chain JSON (used for nonce enforcement)
        #[arg(long)]
        chain: Option<String>,

        #[arg(long)]
        from: String,

        #[arg(long)]
        to: String,

        #[arg(long)]
        amount: u64,

        /// Tx nonce (per-sender). If omitted, it will be auto-filled from chain+mempool.
        #[arg(long)]
        nonce: Option<u64>,

        /// Optional path for mempool JSON
        #[arg(long)]
        mempool: Option<String>,
    },

    /// List mempool transactions
    TxList {
        /// Optional path for mempool JSON
        #[arg(long)]
        mempool: Option<String>,
    },
}

fn chain_path(path: Option<String>) -> std::path::PathBuf {
    path.map(std::path::PathBuf::from)
        .unwrap_or_else(Chain::default_path)
}

fn load_chain(path: &std::path::Path) -> anyhow::Result<Chain> {
    anyhow::ensure!(
        path.exists(),
        "chain file does not exist: {}",
        path.display()
    );
    Chain::load(path)
}

fn mempool_path(path: Option<String>) -> std::path::PathBuf {
    path.map(std::path::PathBuf::from)
        .unwrap_or_else(Mempool::default_path)
}

fn load_or_genesis(path: &std::path::Path) -> anyhow::Result<Chain> {
    if path.exists() {
        Chain::load(path)
    } else {
        Ok(Chain::new_genesis())
    }
}

fn validate_nonce_sequence(chain: &Chain, txs: &[Transaction]) -> anyhow::Result<()> {
    // Enforce simple per-sender nonces: expected = chain.next_nonce_for(sender) + index
    // within this tx list.
    let mut expected: HashMap<String, u64> = HashMap::new();
    for (i, tx) in txs.iter().enumerate() {
        let entry = expected
            .entry(tx.from.clone())
            .or_insert_with(|| chain.next_nonce_for(&tx.from));
        anyhow::ensure!(
            tx.nonce == *entry,
            "invalid nonce in mempool tx #{i} sender={} (expected={} got={})",
            tx.from,
            *entry,
            tx.nonce
        );
        *entry = entry.saturating_add(1);
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            let p = chain_path(path);
            let chain = Chain::new_genesis();
            chain.save(&p)?;
            println!("Initialized chain at {}", p.display());
            println!("height={} tip={}", chain.height(), chain.tip_hash());
        }
        Commands::Status { path, mempool } => {
            let p = chain_path(path);
            let chain = load_chain(&p)?;

            let mp_path = mempool_path(mempool);
            let mp_count = if mp_path.exists() {
                Mempool::load(&mp_path)?.txs.len()
            } else {
                0
            };

            println!("chain: {}", p.display());
            println!(
                "height={} tip={} difficulty={} chain_txs={} mempool_txs={}",
                chain.height(),
                chain.tip_hash(),
                chain.pow_difficulty,
                chain.tx_count(),
                mp_count
            );
        }
        Commands::Validate { path } => {
            let p = chain_path(path);
            let chain = load_chain(&p)?;
            chain.validate()?;
            println!("OK: chain is valid (height={})", chain.height());
        }
        Commands::Mine {
            path,
            mempool,
            difficulty,
        } => {
            let p = chain_path(path);
            let mut chain = load_or_genesis(&p)?;

            let mp_path = mempool_path(mempool);
            let mut mp = if mp_path.exists() {
                Mempool::load(&mp_path)?
            } else {
                Mempool::default()
            };

            // Validate mempool txs before draining so we don't lose them on failure.
            for (i, tx) in mp.txs.iter().enumerate() {
                tx.validate_basic()
                    .with_context(|| format!("invalid mempool tx #{i}"))?;
            }

            validate_nonce_sequence(&chain, &mp.txs)?;

            let txs = mp.drain();
            let mined = chain.mine_block(txs, difficulty)?;
            chain.save(&p)?;
            mp.save(&mp_path)?;

            println!("Mined block at height={}", chain.height());
            println!(
                "nonce={} tip={} difficulty={} txs={}",
                mined.header.nonce,
                chain.tip_hash(),
                chain.pow_difficulty,
                mined.txs.len()
            );
        }
        Commands::TxAdd {
            chain,
            from,
            to,
            amount,
            nonce,
            mempool,
        } => {
            let chain_path = chain_path(chain);
            let chain = load_or_genesis(&chain_path)?;
            let base_nonce = chain.next_nonce_for(&from);

            let mp_path = mempool_path(mempool);
            let mut mp = if mp_path.exists() {
                Mempool::load(&mp_path)?
            } else {
                Mempool::default()
            };

            let filled_nonce = nonce.unwrap_or_else(|| mp.next_nonce_for(&from, base_nonce));

            let tx = Transaction {
                from,
                to,
                amount,
                nonce: filled_nonce,
            };
            let h = tx_hash(&tx);
            mp.add_tx_checked(tx, base_nonce)?;
            mp.save(&mp_path)?;
            println!("Added tx to mempool: {}", mp_path.display());
            println!("tx_hash={}", h);
            println!("tx_hash_short={}", h.get(..8).unwrap_or(&h));
            println!("mempool size={}", mp.txs.len());
        }
        Commands::TxList { mempool } => {
            let mp_path = mempool_path(mempool);
            if !mp_path.exists() {
                println!("mempool: {} (empty)", mp_path.display());
                return Ok(());
            }
            let mp = Mempool::load(&mp_path)?;
            println!("mempool: {}", mp_path.display());
            println!("count={}", mp.txs.len());
            for (i, tx) in mp.txs.iter().enumerate() {
                let h = tx_hash(tx);
                let short = h.get(..8).unwrap_or(&h);
                println!(
                    "{i}: {short} {} -> {} amount={} nonce={}",
                    tx.from, tx.to, tx.amount, tx.nonce
                );
            }
        }
    }

    Ok(())
}
