use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
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
    Add {
        name: String,
        #[arg(long)]
        dev: bool,
    },
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
        Commands::Add { name, dev } => add(name, *dev)?,
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
fn add(package_name: &str, is_dev: bool) -> Result<()> {
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

    // --- Automate package.json modification ---
    let pkg_json_path = Path::new("package.json");
    if !pkg_json_path.exists() {
        println!(
            "{}",
            "⚠️ package.json not found, skipping modification.".yellow()
        );
        return Ok(());
    }

    let content = fs::read_to_string(pkg_json_path)?;
    let indent = detect_indent(&content);

    let mut value: serde_json::Value =
        serde_json::from_str(&content).context("Failed to parse package.json")?;

    let dep_path = format!("file:.kley/{}", package_name);
    let dep_keys = ["dependencies", "devDependencies", "peerDependencies"];
    let mut updated = false;

    if let Some(obj) = value.as_object_mut() {
        for key in &dep_keys {
            if let Some(dep) = obj
                .get_mut(*key)
                .and_then(|d| d.as_object_mut())
                .and_then(|d| d.get_mut(package_name))
            {
                *dep = serde_json::Value::String(dep_path.clone());
                updated = true;
                break;
            }
        }

        if !updated {
            let target_key = if is_dev {
                "devDependencies"
            } else {
                "dependencies"
            };
            if !obj.contains_key(target_key) {
                obj.insert(target_key.to_string(), serde_json::json!({}));
            }
            obj[target_key].as_object_mut().unwrap().insert(
                package_name.to_string(),
                serde_json::Value::String(dep_path),
            );
        }
    }

    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    value.serialize(&mut ser)?;

    fs::write(pkg_json_path, buf)?;

    println!("{}", "✅ package.json has been updated!".green());

    Ok(())
}

/// Detects the indentation of a JSON string.
fn detect_indent(json_str: &str) -> String {
    for line in json_str.lines() {
        if line.starts_with("  ") {
            let indent_len = line.find(|c: char| !c.is_whitespace()).unwrap_or(0);
            if indent_len > 0 {
                return line[..indent_len].to_string();
            }
        }
    }
    "  ".to_string() // Default to 2 spaces
}
