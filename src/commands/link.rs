use anyhow::Result;
use colored::*;
use std::fs;

use crate::{commands::update::run_update, emoji, registry::Registry, utils::work_dirs};

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
    std::os::windows::fs::symlink_dir(&dirs.project_kley_dir, &node_modules_pkg_dir)?;

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
