use clap::{Parser, Subcommand};

mod core;

use core::chain::Chain;
use core::mempool::Mempool;
use core::types::Transaction;

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
        #[arg(long)]
        from: String,

        #[arg(long)]
        to: String,

        #[arg(long)]
        amount: u64,

        #[arg(long, default_value_t = 0)]
        nonce: u64,

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
        Commands::Status { path } => {
            let p = chain_path(path);
            let chain = load_chain(&p)?;
            println!("chain: {}", p.display());
            println!(
                "height={} tip={} difficulty={} txs={}",
                chain.height(),
                chain.tip_hash(),
                chain.pow_difficulty,
                chain.tx_count()
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
            from,
            to,
            amount,
            nonce,
            mempool,
        } => {
            let mp_path = mempool_path(mempool);
            let mut mp = if mp_path.exists() {
                Mempool::load(&mp_path)?
            } else {
                Mempool::default()
            };

            let tx = Transaction {
                from,
                to,
                amount,
                nonce,
            };
            mp.add_tx(tx);
            mp.save(&mp_path)?;
            println!("Added tx to mempool: {}", mp_path.display());
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
                println!("{i}: {} -> {} amount={} nonce={}", tx.from, tx.to, tx.amount, tx.nonce);
            }
        }
    }

    Ok(())
}
