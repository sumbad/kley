use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::registry::Registry;

mod commands;
pub mod lockfile;
pub mod registry;
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
    /// Link a package from the local store to the current project
    Link { name: String },
    /// Remove a package from the current project
    Remove {
        name: Option<String>,
        #[arg(long)]
        all: bool,
    },
}

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        // Set default level to INFO (info!, warn!, error!)
        .with_max_level(tracing::Level::INFO)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let home_dir = dirs::home_dir().context("Failed to find home directory")?;

    let cli = Cli::parse();
    let project_dir = std::env::current_dir()?;
    let mut registry = Registry::new(home_dir)?;

    match &cli.command {
        Commands::Publish => commands::publish::publish(&mut registry)?,
        Commands::Add { name, dev } => commands::add::add(&mut registry, name, *dev)?,
        Commands::Link { name } => commands::link::link(&mut registry, name)?,
        Commands::Remove { name, all } => {
            commands::remove::remove(&mut registry, name, *all, &project_dir)?
        }
    }

    Ok(())
}
