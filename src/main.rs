use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "rusty-chain")]
#[command(about = "A mini blockchain built in Rust (30-day build).", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new chain (creates a genesis placeholder)
    Init,

    /// Print current chain status
    Status,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            // Day 1 placeholder; real genesis + storage comes next.
            println!("Initialized (placeholder). Next: write genesis block to disk.");
        }
        Commands::Status => {
            // Day 1 placeholder; real height/hash reading comes next.
            println!("Status (placeholder). Next: show height + tip hash.");
        }
    }
}
