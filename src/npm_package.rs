use anyhow::{Context, Result};
use std::{fs, path::Path};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PackageJson {
    pub name: String,
    pub version: String,
    pub files: Option<Vec<String>>,
}

pub fn find_npm_package(dir: &Path) -> Result<PackageJson> {
    // Read package.json
    let pkg_path = dir.join("package.json");
    if !pkg_path.exists() {
        anyhow::bail!("package.json not found in the current directory");
    }

    let pkg_content = fs::read_to_string(pkg_path)?;
    let pkg: PackageJson =
        serde_json::from_str(&pkg_content).context("Failed to parse package.json")?;

    Ok(pkg)
}
