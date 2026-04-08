use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use clap::builder::styling::{AnsiColor, Styles};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::registry::Registry;

mod commands;
pub mod lockfile;
pub mod npm_package;
pub mod registry;
pub mod utils;

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().bold())
        .usage(AnsiColor::Yellow.on_default())
        .literal(AnsiColor::Cyan.on_default().bold())
        .placeholder(AnsiColor::Cyan.on_default())
        .error(AnsiColor::Red.on_default().bold())
        .valid(AnsiColor::Green.on_default())
        .invalid(AnsiColor::Red.on_default())
}

#[derive(Parser)]
#[command(name = "kley")]
#[command(styles = styles())]
#[command(version, about = "Fast local package manager for npm (JS/TS)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Publish the current package to the registry
    Publish {
        #[arg(long)]
        push: bool,
    },
    /// Add a package from the registry to the current project
    Add {
        name: String,
        #[arg(long)]
        dev: bool,
    },
    /// Link a package from the registry to the current project
    Link { name: String },
    /// Remove a package from the current project
    Remove {
        name: Option<String>,
        #[arg(long)]
        all: bool,
    },
    /// Update packages from the registry
    Update {
        /// Specific packages to update. If not provided, all packages will be updated.
        packages: Vec<String>,
    },
    /// Unpublish the current package from the registry
    Unpublish {
        #[arg(long)]
        push: bool,
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
        Commands::Publish { push } => commands::publish::publish(&mut registry, *push)?,
        Commands::Unpublish { push } => commands::unpublish::unpublish(&mut registry, *push)?,
        Commands::Add { name, dev } => commands::add::add(&mut registry, name, *dev)?,
        Commands::Link { name } => commands::link::link(&mut registry, name)?,
        Commands::Remove { name, all } => {
            commands::remove::remove(&mut registry, name, *all, &project_dir)?
        }
        Commands::Update { packages } => {
            commands::update::update(&mut registry, packages, &project_dir)?
        }
    }

    Ok(())
}
