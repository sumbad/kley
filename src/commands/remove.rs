use crate::lockfile::Lockfile;
use anyhow::{Context, Result};
use colored::*;
use serde::Serialize;
use serde_json;
use std::fs;
use std::path::Path;

use crate::utils::detect_indent;

pub fn remove(package_name: &Option<String>, is_all: bool) -> Result<()> {
    if package_name.is_none() && !is_all {
        println!(
            "⚠️ Set a package name to command for removing it or use --all flag to delete all local dependencies"
        );
        return Ok(());
    }

    let project_dir = std::env::current_dir()?;
    let mut project_kley_path = project_dir.join(".kley");

    if is_all {
        // Remove kley.lock file
        let lock_path = project_dir.join("kley.lock");
        if lock_path.exists() {
            fs::remove_file(lock_path)?;
        }

        // Remove all local packages from package.json
        remove_all_from_package_json(&project_dir.join("package.json"))?;
    } else if let Some(pkg_name) = package_name {
        project_kley_path = project_kley_path.join(pkg_name);

        // --- Automate package.json modification ---
        update_package_json(&project_dir.join("package.json"), pkg_name)?;

        // --- Update kley.lock ---
        update_kley_lock(pkg_name, &project_dir)?;
    }

    if project_kley_path.exists() {
        fs::remove_dir_all(&project_kley_path)?;
        println!(
            "{}",
            format!("✅ removed directory: {:?}", project_kley_path).green()
        );
    }

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

    let dep_value = format!("file:.kley/{}", package_name);
    let dep_keys = ["dependencies", "devDependencies", "peerDependencies"];

    if let Some(obj) = value.as_object_mut() {
        for key in &dep_keys {
            if let Some(deps_obj) = obj.get_mut(*key).and_then(|d| d.as_object_mut())
                && let Some(dep) = deps_obj.get(package_name)
                && *dep == dep_value
                && deps_obj.remove(package_name).is_some()
            {
                println!("Removed '{}' from '{}' in package.json", package_name, key);
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

fn remove_all_from_package_json(pkg_json_path: &Path) -> Result<()> {
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

    let dep_keys = ["dependencies", "devDependencies", "peerDependencies"];

    if let Some(obj) = value.as_object_mut() {
        for key in &dep_keys {
            if let Some(deps_obj) = obj.get_mut(*key).and_then(|d| d.as_object_mut()) {
                let to_remove: Vec<String> = deps_obj
                    .iter()
                    .filter(|(_, v)| v.to_string().starts_with("file:.kley/"))
                    .map(|(k, _)| k.clone())
                    .collect();

                for dep in to_remove {
                    if let Some(val) = deps_obj.remove(&dep) {
                        println!("Removed '{}' from '{}' in package.json", val, key);
                    }
                }
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
