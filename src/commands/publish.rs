use anyhow::{Context, Result};
use colored::*;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct PackageJson {
    name: String,
    version: String,
}

/// Publish logic
pub fn publish() -> Result<()> {
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
