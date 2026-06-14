use std::path::Path;

use anyhow::Result;
use colored::Colorize;

use crate::{
    emoji,
    lockfile::{ConnectionType, Lockfile, PackageInfo},
    package::PackageJson,
    registry::Registry,
    utils::{PROJECT_REGISTRY_DIR_NAME, copy_from_registry, strip_dev_dependencies},
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
        let connection = Lockfile::get(project_dir)
            .and_then(|lf| lf.packages.get(&package_name).map(|p| p.connection.clone()))
            .unwrap_or_default();

        if connection == ConnectionType::Link {
            println!(
                "{} Skipping {}: linked (source is live)",
                emoji::WARNING,
                package_name.cyan()
            );
            continue;
        }

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

    let project_kley_dir = project_dir
        .join(PROJECT_REGISTRY_DIR_NAME)
        .join(package_name);

    copy_from_registry(registry, package_name, &project_kley_dir)?;

    strip_dev_dependencies(&project_kley_dir)?;

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
            format!(
                "{} Warning: package {} version not found inside registry",
                emoji::WARNING,
                package_name
            )
            .yellow()
        );
        return Ok(());
    };

    let mut lockfile = Lockfile::new(project_dir);

    let package_json = PackageJson::get(&registry.get_pkg_dir(package_name))?;

    // Insert or update package info
    let package_info = PackageInfo {
        version: version.to_string(),
        dependencies: package_json.dependencies,
        peer_dependencies: package_json.peer_dependencies,
        connection: Default::default(),
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

    #[test_log::test]
    fn test_create_new_kley_lock() -> Result<()> {
        let dir = tempdir()?;
        let project_dir = dir.path();

        let tmp_home_dir = tempdir()?;
        let mut registry = Registry::with_home_dir(tmp_home_dir.path())?;

        let package_name = "test-lib";
        // The snapshot is read from the package.json in the registry store
        let source_path = registry.get_pkg_dir(package_name);
        fs::create_dir_all(&source_path)?;
        let pkg_json_path = source_path.join("package.json");
        let mut file = fs::File::create(pkg_json_path)?;
        write!(
            file,
            r#"{{"name": "test-lib", "version": "1.2.3", "dependencies": {{"left-pad": "^1.0.0"}}, "peerDependencies": {{"react": "^18.0.0"}}}}"#
        )?;

        registry.update_package_version(package_name, "1.2.3")?;

        update_kley_lock(&registry, package_name, project_dir)?;

        let lock_content = fs::read_to_string(project_dir.join("kley.lock"))?;
        let lockfile: Lockfile = serde_json::from_str(&lock_content)?;

        let info = lockfile.packages.get("test-lib").unwrap();
        assert_eq!(info.version, "1.2.3");
        assert_eq!(info.dependencies.get("left-pad").unwrap(), "^1.0.0");
        assert_eq!(info.peer_dependencies.get("react").unwrap(), "^18.0.0");

        Ok(())
    }

    #[test]
    fn test_update_existing_kley_lock() -> Result<()> {
        let dir = tempdir()?;
        let project_dir = dir.path();

        let tmp_home_dir = tempdir()?;
        let mut registry = Registry::with_home_dir(tmp_home_dir.path())?;

        let package_name = "test-lib";
        // The snapshot is read from the package.json in the registry store
        let source_path = registry.get_pkg_dir(package_name);
        fs::create_dir_all(&source_path)?;
        let pkg_json_path = source_path.join("package.json");
        let mut file = fs::File::create(pkg_json_path)?;
        write!(file, r#"{{"name": "test-lib", "version": "2.0.0"}}"#)?;

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

    #[test]
    fn test_update_skips_linked_packages() -> Result<()> {
        let tmp_home = tempdir()?;
        let project_dir = tempdir()?;
        let mut registry = Registry::with_home_dir(tmp_home.path())?;

        // Create package in registry
        let pkg_dir = registry.get_pkg_dir("test-lib");
        fs::create_dir_all(&pkg_dir)?;
        let mut file = fs::File::create(pkg_dir.join("package.json"))?;
        write!(file, r#"{{"name": "test-lib", "version": "1.0.0"}}"#)?;
        registry.update_package_version("test-lib", "1.0.0")?;

        // kley.lock marks test-lib as linked
        fs::write(
            project_dir.path().join("kley.lock"),
            r#"{"packages":{"test-lib":{"version":"1.0.0","connection":"link"}}}"#,
        )?;

        // Marker file: if run_update is called it would overwrite this
        let project_kley_pkg = project_dir.path().join(".kley").join("test-lib");
        fs::create_dir_all(&project_kley_pkg)?;
        fs::write(project_kley_pkg.join("marker.txt"), "untouched")?;

        update(&mut registry, &["test-lib".to_string()], project_dir.path())?;

        // Marker should be untouched because update was skipped
        assert_eq!(
            fs::read_to_string(project_kley_pkg.join("marker.txt"))?,
            "untouched",
            "linked package should not be touched by update"
        );

        Ok(())
    }

    #[test]
    fn test_update_runs_for_install_packages() -> Result<()> {
        let tmp_home = tempdir()?;
        let project_dir = tempdir()?;
        let mut registry = Registry::with_home_dir(tmp_home.path())?;

        // Create package in registry at v2.0.0
        let pkg_dir = registry.get_pkg_dir("test-lib");
        fs::create_dir_all(&pkg_dir)?;
        let mut file = fs::File::create(pkg_dir.join("package.json"))?;
        write!(file, r#"{{"name": "test-lib", "version": "2.0.0"}}"#)?;
        registry.update_package_version("test-lib", "2.0.0")?;

        // kley.lock with no connection field (defaults to Install)
        fs::write(
            project_dir.path().join("kley.lock"),
            r#"{"packages":{"test-lib":{"version":"2.0.0"}}}"#,
        )?;

        // Old copy at v1
        let project_kley_pkg = project_dir.path().join(".kley").join("test-lib");
        fs::create_dir_all(&project_kley_pkg)?;
        let mut file = fs::File::create(project_kley_pkg.join("package.json"))?;
        write!(file, r#"{{"name": "test-lib", "version": "1.0.0"}}"#)?;

        update(&mut registry, &["test-lib".to_string()], project_dir.path())?;

        let updated: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(project_kley_pkg.join("package.json"))?)?;
        assert_eq!(updated["version"], "2.0.0", "installed package should be updated");

        Ok(())
    }
}
