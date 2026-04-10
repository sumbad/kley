use std::{fs, path::Path};

use anyhow::{Context, Result};
use colored::Colorize;
use serde::Serialize;

use crate::{
    lockfile::{Lockfile, PackageInfo},
    registry::Registry,
    utils::copy_from_registry,
};

/// Main entry point for the `update` command.
pub fn update(registry: &mut Registry, packages: &Vec<String>, project_dir: &Path) -> Result<()> {
    let packages_to_update = if packages.is_empty() {
        // If no packages are specified, update all packages in kley.lock
        let lock_path = project_dir.join("kley.lock");
        if !lock_path.exists() {
            println!(
                "{}",
                "⚠️ Warning: No packages to update. kley.lock not found.".yellow()
            );
            return Ok(());
        }
        let lockfile: Lockfile = serde_json::from_str(&fs::read_to_string(lock_path)?)
            .context("🚨 Error: Failed to parse kley.lock")?;

        lockfile.packages.keys().cloned().collect()
    } else {
        packages.clone()
    };

    if packages_to_update.is_empty() {
        println!("{}", "⚠️ Warning: No packages found to update.".yellow());
        return Ok(());
    }

    println!("{}", "Updating...".green().dimmed());
    for package_name in packages_to_update {
        run_update(registry, &package_name, project_dir)?;

        println!(
            "{}",
            format!("   ✔️ {}", &package_name.clone())
                .green()
                .dimmed()
        );
    }

    println!("{}", "✅ Done: packages were updated".green());

    Ok(())
}

pub fn run_update(registry: &mut Registry, package_name: &str, project_dir: &Path) -> Result<()> {
    tracing::debug!("run_update:\n package_name: {package_name}\n project_dir: {project_dir:?}");

    copy_from_registry(registry, package_name, project_dir)?;

    update_kley_lock(registry, package_name, project_dir)?;

    tracing::debug!("Updated directory {project_dir:?}");

    Ok(())
}

/// Creates or updates kley.lock file.
fn update_kley_lock(registry: &Registry, package_name: &str, project_dir: &Path) -> Result<()> {
    let lock_path = project_dir.join("kley.lock");

    let version = if let Some(pkg_version) = registry.get_pkg_version(package_name) {
        pkg_version
    } else {
        println!(
            "{}",
            format!("⚠️ Warning: package {package_name} version not found inside registry")
                .yellow()
        );
        return Ok(());
    };

    // 2. Read existing kley.lock or create a new one
    let mut lockfile: Lockfile = if lock_path.exists() {
        let content = fs::read_to_string(&lock_path)?;
        if content.trim().is_empty() {
            Lockfile::default()
        } else {
            serde_json::from_str(&content).context("Failed to parse kley.lock")?
        }
    } else {
        Lockfile::default()
    };

    // 3. Insert or update package info
    let package_info = PackageInfo {
        version: version.to_string(),
    };
    lockfile
        .packages
        .insert(package_name.to_string(), package_info);

    // 4. Write back to kley.lock
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    lockfile.serialize(&mut ser)?;

    fs::write(lock_path, buf)?;

    tracing::info!("kley.lock has been updated");

    Ok(())
}

#[cfg(test)]
mod kley_lock_tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_create_new_kley_lock() -> Result<()> {
        let dir = tempdir()?;
        let project_dir = dir.path();

        let tmp_home_dir = tempdir()?;
        let home_dir = tmp_home_dir.path();
        let mut registry = Registry::new(home_dir.to_path_buf())?;

        let package_name = "test-lib";
        // Create a dummy package.json in a dummy source path
        let source_path = project_dir.join(package_name);
        fs::create_dir_all(&source_path)?;
        let pkg_json_path = source_path.join("package.json");
        let mut file = fs::File::create(pkg_json_path)?;
        write!(file, r#"{{"version": "1.2.3"}}"#)?;

        registry.update_package_version(package_name, "1.2.3")?;

        update_kley_lock(&registry, package_name, project_dir)?;

        let lock_content = fs::read_to_string(project_dir.join("kley.lock"))?;
        let lockfile: Lockfile = serde_json::from_str(&lock_content)?;

        assert_eq!(lockfile.packages.get("test-lib").unwrap().version, "1.2.3");

        Ok(())
    }

    #[test]
    fn test_update_existing_kley_lock() -> Result<()> {
        let dir = tempdir()?;
        let project_dir = dir.path();

        let tmp_home_dir = tempdir()?;
        let home_dir = tmp_home_dir.path();
        let mut registry = Registry::new(home_dir.to_path_buf())?;

        let package_name = "test-lib";
        // Create a dummy source package
        let source_path = project_dir.join(package_name);
        fs::create_dir_all(&source_path)?;
        let pkg_json_path = source_path.join("package.json");
        let mut file = fs::File::create(pkg_json_path)?;
        write!(file, r#"{{"version": "2.0.0"}}"#)?;

        // Create an existing kley.lock in the project dir
        let lock_path = project_dir.join("kley.lock");
        fs::write(
            &lock_path,
            r#"{"packages":{"another-lib":{"version":"0.5.0"}}}"#,
        )?;

        registry.update_package_version(package_name, "2.0.0")?;

        update_kley_lock(&registry, package_name, project_dir)?;

        let lock_content = fs::read_to_string(lock_path)?;
        let lockfile: Lockfile = serde_json::from_str(&lock_content)?;

        assert_eq!(lockfile.packages.len(), 2);
        assert_eq!(lockfile.packages.get("test-lib").unwrap().version, "2.0.0");
        assert_eq!(
            lockfile.packages.get("another-lib").unwrap().version,
            "0.5.0"
        );

        Ok(())
    }
}
