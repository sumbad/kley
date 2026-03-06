use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
    Add { name: String },
}

#[derive(Deserialize, Serialize, Debug)]
struct PackageJson {
    name: String,
    version: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Publish => publish()?,
        Commands::Add { name } => add(name)?,
    }

    Ok(())
}

/// Publish logic
fn publish() -> Result<()> {
    // 1. Read package.json
    let pkg_path = Path::new("package.json");
    if !pkg_path.exists() {
        anyhow::bail!("package.json not found in the current directory");
    }

    let pkg_content = fs::read_to_string(pkg_path)?;
    let pkg: PackageJson =
        serde_json::from_str(&pkg_content).context("Failed to parse package.json")?;

    println!(
        "🚀 Publishing {}@{}...",
        pkg.name.cyan(),
        pkg.version.magenta()
    );

    // 2. Determine the path in the store (~/.kley/packages/name)
    let home_dir = dirs::home_dir().context("Failed to find home directory")?;
    let store_path = home_dir.join(".kley").join("packages").join(&pkg.name);

    if store_path.exists() {
        fs::remove_dir_all(&store_path)?;
    }
    fs::create_dir_all(&store_path)?;

    // 3. Copy files (MVP: copy everything except node_modules and .git)
    // TODO: In the future, the 'ignore' library will be used here
    let options = fs_extra::dir::CopyOptions::new();

    // Get the list of files to copy (simplified for MVP)
    let entries = fs::read_dir(".")?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap();

        if name != "node_modules" && name != ".git" && name != ".kley" && name != "target" {
            if path.is_dir() {
                fs_extra::dir::copy(&path, &store_path, &options)?;
            } else {
                fs::copy(&path, store_path.join(name))?;
            }
        }
    }

    println!("{}", "✅ Package successfully published to store!".green());
    Ok(())
}

/// Add logic
fn add(package_name: &str) -> Result<()> {
    let home_dir = dirs::home_dir().context("Failed to find home directory")?;
    let source_path = home_dir.join(".kley").join("packages").join(package_name);

    if !source_path.exists() {
        anyhow::bail!(
            "Package '{}' not found in store. Run 'kley publish' in the package folder first.",
            package_name
        );
    }

    // Determine the local storage folder (.kley/package-name)
    let dest_path = Path::new(".kley").join(package_name);

    if dest_path.exists() {
        fs::remove_dir_all(&dest_path)?;
    }
    fs::create_dir_all(&dest_path)?;

    // Copy from store to local project
    let mut options = fs_extra::dir::CopyOptions::new();
    options.content_only = true;
    fs_extra::dir::copy(&source_path, &dest_path, &options)?;

    println!(
        "📎 Package {} added to .kley/{}",
        package_name.cyan(),
        package_name.cyan()
    );
    println!(
        "{}",
        "⚠️  Don't forget to add the dependency to package.json:".yellow()
    );
    println!("\"{}\": \"file:.kley/{}\"", package_name, package_name);

    Ok(())
}
