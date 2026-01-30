use clap::{Parser, Subcommand};

mod core;

use core::chain::Chain;

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

    /// Mine and append an empty block (demo PoW)
    Mine {
        /// Path for chain JSON (will be created if missing)
        #[arg(long)]
        path: Option<String>,

        /// PoW difficulty (leading '0' hex chars)
        #[arg(long, default_value_t = 3)]
        difficulty: usize,
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
            println!("height={} tip={}", chain.height(), chain.tip_hash());
        }
        Commands::Validate { path } => {
            let p = chain_path(path);
            let chain = load_chain(&p)?;
            chain.validate()?;
            println!("OK: chain is valid (height={})", chain.height());
        }
        Commands::Mine { path, difficulty } => {
            let p = chain_path(path);
            let mut chain = load_or_genesis(&p)?;

            let mined = chain.mine_empty_block(difficulty)?;
            chain.save(&p)?;

            println!("Mined block at height={}", chain.height());
            println!("nonce={} tip={}", mined.header.nonce, chain.tip_hash());
        }
    }

    Ok(())
}
