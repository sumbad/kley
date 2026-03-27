use colored::*;
use std::fs;

use anyhow::Result;

use crate::{
    registry::Registry,
    utils::{copy_from_store, work_dirs},
};

pub fn link(registry: &mut Registry, package_name: &str) -> Result<()> {
    let dirs = work_dirs(package_name, &registry)?;

    copy_from_store(package_name, &dirs)?;

    let node_modules_pkg_dir = dirs.project_dir.join("node_modules").join(package_name);

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

    println!(
        "{}",
        format!(
            "✅ Package {} successfully linked to this project!",
            package_name
        )
        .green()
    );

    Ok(())
}
