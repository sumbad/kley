use anyhow::Result;
use colored::*;
use std::fs;

use crate::{commands::update::run_update, emoji, registry::Registry, utils::work_dirs};

/// Creates a junction point on Windows as a fallback when symlink creation fails
/// due to insufficient privileges (e.g. Developer Mode is not enabled).
/// Junction points work identically to symlinks for local directories and
/// don't require elevated permissions.
#[cfg(windows)]
fn create_junction(source: &std::path::Path, junction: &std::path::Path) -> anyhow::Result<()> {
    use std::process::Command;

    let source_str = source.to_str().ok_or_else(|| {
        anyhow::anyhow!("Source path contains non-UTF8 characters: {:?}", source)
    })?;
    let junction_str = junction.to_str().ok_or_else(|| {
        anyhow::anyhow!("Junction path contains non-UTF8 characters: {:?}", junction)
    })?;

    let status = Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            junction_str,
            source_str,
        ])
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

pub fn link(registry: &mut Registry, package_name: &str) -> Result<()> {
    let dirs = work_dirs(package_name)?;

    run_update(registry, package_name, &std::env::current_dir()?)?;

    let node_modules_dir = dirs.project_dir.join("node_modules");
    let node_modules_pkg_dir = node_modules_dir.join(package_name);

    // Ensure node_modules directory exists
    fs::create_dir_all(&node_modules_dir)?;

    if node_modules_pkg_dir.exists() {
        if node_modules_pkg_dir.is_symlink() || node_modules_pkg_dir.is_file() {
            fs::remove_file(&node_modules_pkg_dir)?;
        } else {
            fs::remove_dir_all(&node_modules_pkg_dir)?;
        }
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(&dirs.project_kley_dir, &node_modules_pkg_dir)?;

    #[cfg(windows)]
    // Junction points don't require elevated privileges unlike symlink_dir on Windows.
    // They work identically for local directories, which is exactly our use case.
    std::os::windows::fs::symlink_dir(&dirs.project_kley_dir, &node_modules_pkg_dir)
        .or_else(|_| create_junction(&dirs.project_kley_dir, &node_modules_pkg_dir))?;

    // Add to registry
    registry.add_package_installation(package_name, &dirs.project_dir)?;

    println!(
        "{}\n{}",
        format!(
            "Note: `npm install` will overwrite links. Don't forget to run `kley link {}` again after it to restore the link.",
            package_name
        )
        .italic()
        .bright_black(),
        format!("{} Done: {} linked", emoji::SUCCESS, package_name.cyan()).green(),
    );

    Ok(())
}
