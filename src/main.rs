use anyhow::Result;
use clap::builder::styling::{AnsiColor, Styles};
use clap::{Parser, Subcommand};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use kley::commands;
use kley::package::PackageJson;
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
enum HooksAction {
    /// Show the current .kley/hooks.json
    List,
    /// Reconfigure .kley/hooks.json via the interactive wizard
    Edit,
}

#[derive(Subcommand)]
enum Commands {
    /// Publish the current package to the registry
    Publish {
        #[arg(long)]
        push: bool,
        /// Do not prompt for hooks configuration; pure file copy
        #[arg(short = 'y', long = "non-interactive")]
        non_interactive: bool,
        /// Ignore .kley/hooks.json for this run (pure file copy)
        #[arg(long)]
        no_hooks: bool,
        /// Disable resolution of `workspace:` protocol dependencies on push
        #[arg(long = "no-workspace-resolve")]
        no_workspace_resolve: bool,
    },
    /// Manage publish hooks (.kley/hooks.json)
    #[command(subcommand)]
    Hooks(HooksAction),
    /// Add a package from the registry to the current project
    Add {
        name: String,
        /// Install as devDependency
        #[arg(long, short = 'D')]
        dev: bool,
        /// Do not modify package.json (workspace-friendly). Defaults to on when
        /// the project declares a `workspaces` field.
        #[arg(long, overrides_with = "_no_pure")]
        pure: bool,
        /// Force the default (non-pure) behavior even inside a workspace project
        #[arg(long = "no-pure")]
        _no_pure: bool,
        /// Disable resolution of `workspace:` protocol dependencies
        #[arg(long = "no-workspace-resolve")]
        no_workspace_resolve: bool,
    },
    /// Install a package from the registry to the current project
    #[command(visible_alias = "i")]
    Install {
        /// Package name to install. If omitted, installs all packages from kley.lock
        name: Option<String>,
        /// Install as devDependency
        #[arg(long, short = 'D')]
        dev: bool,
        #[arg(long)]
        no_save: bool,
        /// Disable resolution of `workspace:` protocol dependencies
        #[arg(long = "no-workspace-resolve")]
        no_workspace_resolve: bool,
    },
    /// Link a package from the registry to the current project
    Link { name: String },
    /// Remove a package from the current project
    Remove {
        /// Package name to remove. If omitted, removes all packages from kley.lock
        name: Option<String>,
        #[arg(long)]
        all: bool,
    },
    /// Update packages from the registry
    Update {
        /// Specific packages to update. If not provided, all packages will be updated.
        packages: Vec<String>,
        /// Disable resolution of `workspace:` protocol dependencies
        #[arg(long = "no-workspace-resolve")]
        no_workspace_resolve: bool,
    },
    /// Unpublish the current package from the registry
    Unpublish {
        #[arg(long)]
        push: bool,
    },
    /// Watch for file changes and automatically publish --push
    Watch {
        /// Path to watch files. If omitted, watches all files from current directory
        path: Option<String>,
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
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    let subscriber = FmtSubscriber::builder().with_env_filter(filter).finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let project_dir = std::env::current_dir()?;
    let mut registry = Registry::new()?;

    match &cli.command {
        Commands::Publish {
            push,
            non_interactive,
            no_hooks,
            no_workspace_resolve,
        } => commands::publish::publish(
            &mut registry,
            *push,
            *non_interactive,
            *no_hooks,
            *no_workspace_resolve,
        )?,
        Commands::Hooks(action) => match action {
            HooksAction::List => commands::hooks::list(&project_dir)?,
            HooksAction::Edit => commands::hooks::edit(&project_dir)?,
        },
        Commands::Unpublish { push } => commands::unpublish::unpublish(&mut registry, *push)?,
        Commands::Add {
            name,
            dev,
            pure,
            _no_pure,
            no_workspace_resolve,
        } => {
            let effective_pure = if *_no_pure {
                false
            } else if *pure {
                true
            } else {
                // neither → workspace detection
                PackageJson::get(&project_dir)
                    .map(|p| p.has_workspaces())
                    .unwrap_or(false)
            };

            let resolve_workspace = !*no_workspace_resolve;

            commands::add::add(&mut registry, name, *dev, effective_pure, resolve_workspace)?;
        }
        Commands::Install {
            name,
            dev,
            no_save,
            no_workspace_resolve,
        } => commands::install::install(
            &mut registry,
            name.as_deref(),
            &project_dir,
            *dev,
            *no_save,
            !*no_workspace_resolve,
        )?,
        Commands::Link { name } => commands::link::link(&mut registry, name)?,
        Commands::Remove { name, all } => {
            commands::remove::remove(&mut registry, name, *all, &project_dir)?
        }
        Commands::Update {
            packages,
            no_workspace_resolve,
        } => commands::update::update(
            &mut registry,
            packages,
            &project_dir,
            !*no_workspace_resolve,
        )?,
        Commands::Watch { path } => commands::watch::watch(&mut registry, path)?,
    }

    Ok(())
}
