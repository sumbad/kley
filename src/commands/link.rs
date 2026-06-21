use anyhow::Result;
use colored::*;
use std::{fs, path::Path};

use crate::{
    emoji,
    lockfile::{ConnectionType, Lockfile, PackageInfo},
    package::PackageJson,
    registry::Registry,
};

/// Creates a junction point on Windows as a fallback when symlink creation fails
/// due to insufficient privileges (e.g. Developer Mode is not enabled).
/// Junction points work identically to symlinks for local directories and
/// don't require elevated permissions.
#[cfg(windows)]
fn create_junction(source: &std::path::Path, junction: &std::path::Path) -> anyhow::Result<()> {
    use std::process::Command;

    let source_str = source
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Source path contains non-UTF8 characters: {:?}", source))?;
    let junction_str = junction.to_str().ok_or_else(|| {
        anyhow::anyhow!("Junction path contains non-UTF8 characters: {:?}", junction)
    })?;

    let status = Command::new("cmd")
        .args(["/C", "mklink", "/J", junction_str, source_str])
        .status()
        .expect("Failed to run cmd");

    if !status.success() {
        anyhow::bail!(
            "Failed to create junction from '{}' to '{}'. \
             Try enabling Developer Mode in Windows Settings, \
             or run the terminal as Administrator.",
            source.display(),
            junction.display()
        );
    }

    Ok(())
}

/// Returns names of dependencies that also appear in peer_dependencies.
/// These are "singleton" packages that expect exactly one runtime instance.
fn get_singleton_dep_names(pkg: &PackageJson) -> Vec<String> {
    pkg.dependencies
        .keys()
        .filter(|name| pkg.peer_dependencies.contains_key(*name))
        .cloned()
        .collect()
}

/// Prints a warning if the package has singleton dependencies.
fn warn_singleton_deps(pkg: &PackageJson, package_name: &str) {
    let singletons = get_singleton_dep_names(pkg);
    if !singletons.is_empty() {
        let list = singletons.join(", ");
        println!(
            "{}  {} has singleton dependencies: {}\n   Linking may cause duplicate instances. Consider `kley install {}` instead.",
            emoji::WARNING,
            package_name.cyan(),
            list.magenta(),
            package_name
        );
    }
}

/// Writes or updates `kley.lock` with a link entry for the package.
fn write_link_kley_lock(registry: &Registry, package_name: &str, project_dir: &Path) -> Result<()> {
    let mut lockfile = Lockfile::new(project_dir);

    let pkg_info = PackageInfo {
        version: registry
            .get_pkg_version(package_name)
            .unwrap_or("latest")
            .to_string(),
        connection: ConnectionType::Link,
        dependencies: Default::default(),
        peer_dependencies: Default::default(),
    };

    lockfile.packages.insert(package_name.to_string(), pkg_info);
    lockfile.save(project_dir)?;

    Ok(())
}

/// Cleans up an existing install for the package.
/// Removes the `.kley/<pkg>/` directory and the entry from `kley.lock`.
/// Does NOT modify `package.json`.
fn remove_install_files(package_name: &str, project_dir: &Path) -> Result<()> {
    // Remove .kley/<pkg>/ directory if it exists
    let kley_pkg_dir = project_dir.join(".kley").join(package_name);
    if kley_pkg_dir.exists() {
        fs::remove_dir_all(&kley_pkg_dir)?;
        tracing::debug!("link: removed .kley/{} directory", package_name);
    }

    // Remove entry from kley.lock
    if let Some(mut lockfile) = Lockfile::get(project_dir) {
        lockfile.packages.remove(package_name);
        lockfile.save(project_dir)?;
        tracing::debug!("link: removed {} from kley.lock", package_name);
    }

    Ok(())
}

pub fn link(registry: &mut Registry, package_name: &str) -> Result<()> {
    tracing::debug!("link: starting for package '{}'", package_name);

    let project_dir = std::env::current_dir()?;

    // Get source_path from registry
    let source_path = registry
        .get_source_path(package_name)
        .map(|p| p.to_path_buf())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "{} not found in registry. Run `kley publish` in the package directory first.",
                package_name
            )
        })?;

    tracing::debug!("link: source_path={:?}", source_path);

    // Verify source directory still exists
    if !source_path.exists() {
        anyhow::bail!(
            "Source directory {:?} no longer exists.\n   Run `kley publish` in the new location of {} to update the source path.",
            source_path,
            package_name
        );
    }

    // Load PackageJson from source to check for singleton deps
    let pkg_json = PackageJson::get(&source_path)?;
    warn_singleton_deps(&pkg_json, package_name);

    // Switch from install to link if needed
    if registry.has_installation(package_name, &project_dir) {
        println!(
            "{}  {} was installed, switching to link mode.\n   Files in .kley/{}/ will be removed.",
            emoji::WARNING,
            package_name.cyan(),
            package_name
        );
        remove_install_files(package_name, &project_dir)?;
        registry.remove_package_installation(package_name, &project_dir)?;
    }

    // Create symlink: node_modules/<pkg> -> source_path
    let node_modules_dir = project_dir.join("node_modules");
    let node_modules_pkg_dir = node_modules_dir.join(package_name);

    fs::create_dir_all(&node_modules_dir)?;
    tracing::debug!(
        "link: creating symlink {:?} -> {:?}",
        node_modules_pkg_dir,
        source_path
    );
    create_symlink(&source_path, &node_modules_pkg_dir)?;

    // Register the link
    registry.add_package_link(package_name, &project_dir)?;

    // Write kley.lock
    write_link_kley_lock(registry, package_name, &project_dir)?;

    println!(
        "{}\n{}",
        format!(
            "Note: `npm install` will overwrite links. Run `kley link {}` again to restore.",
            package_name
        )
        .italic()
        .bright_black(),
        format!("{} Done: {} linked", emoji::SUCCESS, package_name.cyan()).green(),
    );

    Ok(())
}

pub fn create_symlink(source_path: &Path, target_path: &Path) -> Result<()> {
    // Use is_symlink() in addition to exists() to catch dangling symlinks,
    // which exists() returns false for (it follows the symlink to the missing target).
    if target_path.exists() || target_path.is_symlink() {
        let is_symlink = target_path.is_symlink();
        let is_file = target_path.is_file();
        let is_dir = target_path.is_dir();
        tracing::debug!(
            "link: target_path exists, is_symlink={}, is_file={}, is_dir={}",
            is_symlink,
            is_file,
            is_dir,
        );

        if is_symlink {
            // On Unix, symlinks are files and remove_file works.
            // On Windows, directory symlinks/junctions are directories,
            // so remove_file fails with Access Denied. Try both.
            tracing::debug!("link: removing existing symlink...");
            fs::remove_file(target_path).or_else(|e| {
                tracing::debug!("link: remove_file failed ({}), trying remove_dir...", e);
                fs::remove_dir(target_path)
            })?;
            tracing::debug!("link: symlink removed");
        } else if is_file {
            fs::remove_file(target_path)?;
            tracing::debug!("link: file removed");
        } else {
            fs::remove_dir_all(target_path)?;
            tracing::debug!("link: directory removed");
        }
    } else {
        tracing::debug!("link: target_path does not exist, nothing to remove");
    }

    #[cfg(unix)]
    {
        tracing::debug!("link: creating unix symlink...");
        std::os::unix::fs::symlink(source_path, target_path)?;
        tracing::debug!("link: unix symlink created");
    }

    #[cfg(windows)]
    {
        tracing::debug!("link: attempting symlink_dir on Windows...");
        match std::os::windows::fs::symlink_dir(source_path, target_path) {
            Ok(()) => tracing::debug!("link: symlink_dir created"),
            Err(e) => {
                tracing::debug!(
                    "link: symlink_dir failed ({}), falling back to junction...",
                    e
                );
                create_junction(source_path, target_path)?;
                tracing::debug!("link: junction created");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::PackageJson;

    fn make_pkg_json(deps: &[(&str, &str)], peer_deps: &[(&str, &str)]) -> PackageJson {
        PackageJson {
            name: "test-pkg".to_string(),
            version: "1.0.0".to_string(),
            files: None,
            package_manager: None,
            dev_dependencies: None,
            dependencies: deps
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            peer_dependencies: peer_deps
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn test_get_singleton_dep_names_returns_intersection() {
        let pkg = make_pkg_json(
            &[("react", "^18.0.0"), ("lodash", "^4.0.0")],
            &[("react", "^18.0.0")],
        );
        let singletons = get_singleton_dep_names(&pkg);
        assert_eq!(singletons, vec!["react"]);
    }

    #[test]
    fn test_get_singleton_dep_names_empty_if_no_overlap() {
        let pkg = make_pkg_json(&[("lodash", "^4.0.0")], &[("react", "^18.0.0")]);
        let singletons = get_singleton_dep_names(&pkg);
        assert!(singletons.is_empty());
    }

    #[test]
    fn test_get_singleton_dep_names_empty_if_no_peer_deps() {
        let pkg = make_pkg_json(&[("react", "^18.0.0"), ("lodash", "^4.0.0")], &[]);
        let singletons = get_singleton_dep_names(&pkg);
        assert!(singletons.is_empty());
    }
}
