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
    tracing::debug!("link: starting for package '{}'", package_name);

    let dirs = work_dirs(package_name)?;

    tracing::debug!("link: calling run_update...");
    run_update(registry, package_name, &std::env::current_dir()?)
        .map_err(|e| {
            tracing::debug!("link: run_update failed: {e:?}");
            e
        })?;
    tracing::debug!("link: run_update completed");

    let node_modules_dir = dirs.project_dir.join("node_modules");
    let node_modules_pkg_dir = node_modules_dir.join(package_name);

    tracing::debug!(
        "link: project_dir={:?}, node_modules_pkg_dir={:?}, project_kley_dir={:?}",
        dirs.project_dir,
        node_modules_pkg_dir,
        dirs.project_kley_dir,
    );

    // Ensure node_modules directory exists
    fs::create_dir_all(&node_modules_dir)?;
    tracing::debug!("link: ensured node_modules dir exists");

    if node_modules_pkg_dir.exists() {
        let is_symlink = node_modules_pkg_dir.is_symlink();
        let is_file = node_modules_pkg_dir.is_file();
        let is_dir = node_modules_pkg_dir.is_dir();
        tracing::debug!(
            "link: node_modules_pkg_dir exists, is_symlink={}, is_file={}, is_dir={}",
            is_symlink,
            is_file,
            is_dir,
        );

        if is_symlink {
            // On Unix, symlinks are files and remove_file works.
            // On Windows, directory symlinks/junctions are directories,
            // so remove_file fails with Access Denied. Try both.
            tracing::debug!("link: removing existing symlink...");
            fs::remove_file(&node_modules_pkg_dir)
                .or_else(|e| {
                    tracing::debug!("link: remove_file failed ({}), trying remove_dir...", e);
                    fs::remove_dir(&node_modules_pkg_dir)
                })?;
            tracing::debug!("link: symlink removed");
        } else if is_file {
            fs::remove_file(&node_modules_pkg_dir)?;
            tracing::debug!("link: file removed");
        } else {
            fs::remove_dir_all(&node_modules_pkg_dir)?;
            tracing::debug!("link: directory removed");
        }
    } else {
        tracing::debug!("link: node_modules_pkg_dir does not exist, nothing to remove");
    }

    #[cfg(unix)]
    {
        tracing::debug!("link: creating unix symlink...");
        std::os::unix::fs::symlink(&dirs.project_kley_dir, &node_modules_pkg_dir)?;
        tracing::debug!("link: unix symlink created");
    }

    #[cfg(windows)]
    {
        tracing::debug!("link: attempting symlink_dir on Windows...");
        match std::os::windows::fs::symlink_dir(&dirs.project_kley_dir, &node_modules_pkg_dir) {
            Ok(()) => tracing::debug!("link: symlink_dir created"),
            Err(e) => {
                tracing::debug!("link: symlink_dir failed ({}), falling back to junction...", e);
                create_junction(&dirs.project_kley_dir, &node_modules_pkg_dir)?;
                tracing::debug!("link: junction created");
            }
        }
    }

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
