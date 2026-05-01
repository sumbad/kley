use std::path::Path;

use anyhow::Result;
use colored::Colorize;

use crate::{
    emoji,
    lockfile::{Lockfile, PackageInfo},
    registry::Registry,
    utils::copy_from_registry,
};

/// Main entry point for the `update` command.
pub fn update(registry: &mut Registry, packages: &[String], project_dir: &Path) -> Result<()> {
    let packages_to_update = if packages.is_empty() {
        // If no packages are specified, update all packages in kley.lock
        let Some(lockfile) = Lockfile::get(project_dir) else {
            println!(
                "{}",
                format!(
                    "{} Warning: No packages to update. kley.lock not found.",
                    emoji::WARNING
                )
                .yellow()
            );
            return Ok(());
        };

        lockfile.packages.keys().cloned().collect()
    } else {
        packages.to_vec()
    };

    if packages_to_update.is_empty() {
        println!(
            "{}",
            format!("{} Warning: No packages found to update.", emoji::WARNING).yellow()
        );
        return Ok(());
    }

    println!("{}", "Updating...".green().dimmed());
    for package_name in packages_to_update {
        run_update(registry, &package_name, project_dir)?;

        println!(
            "{}",
            format!("   {} {}", emoji::UPDATED, &package_name.clone())
                .green()
                .dimmed()
        );
    }

    println!(
        "{}",
        format!("{} Done: packages were updated", emoji::SUCCESS).green()
    );

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

    let mut lockfile = Lockfile::new(project_dir);

    // Insert or update package info
    let package_info = PackageInfo {
        version: version.to_string(),
    };

    lockfile
        .packages
        .insert(package_name.to_string(), package_info);

    lockfile.save(project_dir)?;

    Ok(())
}

#[cfg(test)]
mod kley_lock_tests {
    use super::*;
    use std::{fs, io::Write};
    use tempfile::tempdir;

    #[test]
    fn test_create_new_kley_lock() -> Result<()> {
        let dir = tempdir()?;
        let project_dir = dir.path();

        let tmp_home_dir = tempdir()?;
        let mut registry = Registry::with_home_dir(tmp_home_dir.path())?;

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
        let mut registry = Registry::with_home_dir(tmp_home_dir.path())?;

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
