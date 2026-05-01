use anyhow::Result;
use clap::builder::styling::{AnsiColor, Styles};
use clap::{Parser, Subcommand};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use kley::commands;
use kley::registry::Registry;

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
#[command(version, about = "Local package manager for Node.js projects", long_about = None)]
struct Cli {
    /// Enable debug output (-vv for trace)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,

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
    /// Install a package from the registry to the current project
    #[command(visible_alias = "i")]
    Install { name: String },
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
    let cli = Cli::parse();

    // Priority: RUST_LOG env > -v flag > default (error, only println! output visible)
    let default_level = match cli.verbose {
        0 => "error",
        1 => "info",
        _ => "debug,trace",
    };
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level));

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let project_dir = std::env::current_dir()?;
    let mut registry = Registry::new()?;

    match &cli.command {
        Commands::Publish { push } => commands::publish::publish(&mut registry, *push)?,
        Commands::Unpublish { push } => commands::unpublish::unpublish(&mut registry, *push)?,
        Commands::Add { name, dev } => commands::add::add(&mut registry, name, *dev)?,
        Commands::Install { name } => {
            commands::install::install(&mut registry, name, &project_dir)?
        }
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
