use anyhow::Result;
use clap::{Parser, Subcommand};
// Добавляем импорты
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod commands;
pub mod lockfile;
mod utils;

#[derive(Parser)]
#[command(name = "kley")]
#[command(about = "Fast local package manager for npm (JS/TS)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Publish the current package to the local store
    Publish,
    /// Add a package from the store to the current project
    Add {
        name: String,
        #[arg(long)]
        dev: bool,
    },
    /// Remove a package from the current project
    Remove { name: String },
}

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        // Set default level to INFO (info!, warn!, error!)
        .with_max_level(tracing::Level::INFO)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = Cli::parse();

    match &cli.command {
        Commands::Publish => commands::publish::publish()?,
        Commands::Add { name, dev } => commands::add::add(name, *dev)?,
        Commands::Remove { name } => commands::remove::remove(name)?,
    }

    Ok(())
}
