use crate::lockfile::{Lockfile};
use anyhow::{Context, Result};
use colored::*;
use serde::Serialize;
use serde_json;
use std::fs;
use std::path::Path;

use crate::utils::detect_indent;

pub fn remove(package_name: &str) -> Result<()> {
    let project_dir = std::env::current_dir()?;
    let project_kley_path = project_dir.join(".kley").join(package_name);

    if project_kley_path.exists() {
        fs::remove_dir_all(&project_kley_path)?;
    }

    // --- Automate package.json modification ---
    update_package_json(&project_dir.join("package.json"), package_name)?;

    // --- Update kley.lock ---
    update_kley_lock(package_name, &project_dir)?;

    Ok(())
}

/// Modifies package.json to add or update a dependency.
fn update_package_json(pkg_json_path: &Path, package_name: &str) -> Result<()> {
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

    if let Some(obj) = value.as_object_mut() {
        for key in &dep_keys {
            if let Some(dep) = obj
                .get(*key)
                .and_then(|d| d.as_object())
                .and_then(|d| d.get(package_name))
                && dep.is_string()
                && *dep == dep_path
            {
                let removed_value = obj.remove(*key);

                println!("{:?} was removed from package.json", removed_value);

                break;
            }
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

/// Updates kley.lock file.
fn update_kley_lock(package_name: &str, project_dir: &Path) -> Result<()> {
    let lock_path = project_dir.join("kley.lock");

    if !lock_path.exists() {
        tracing::debug!("kley.lock file was not found");

        return Ok(());
    }

    // Read existing kley.lock
    let mut lockfile: Lockfile = if lock_path.exists() {
        let content = fs::read_to_string(&lock_path)?;
        serde_json::from_str(&content).context("Failed to parse kley.lock")?
    } else {
        Lockfile::default()
    };

    lockfile.packages.remove(package_name);

    // Write back to kley.lock
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    lockfile.serialize(&mut ser)?;

    fs::write(lock_path, buf)?;

    println!("{}", "🔒 kley.lock has been updated!".green());

    Ok(())
}
