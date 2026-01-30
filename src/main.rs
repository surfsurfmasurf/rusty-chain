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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            let p = path
                .map(std::path::PathBuf::from)
                .unwrap_or_else(Chain::default_path);
            let chain = Chain::new_genesis();
            chain.save(&p)?;
            println!("Initialized chain at {}", p.display());
            println!("height={} tip={}", chain.height(), chain.tip_hash());
        }
        Commands::Status { path } => {
            let p = path
                .map(std::path::PathBuf::from)
                .unwrap_or_else(Chain::default_path);
            let chain = Chain::load(&p)?;
            println!("chain: {}", p.display());
            println!("height={} tip={}", chain.height(), chain.tip_hash());
        }
    }

    Ok(())
}
