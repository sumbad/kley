use anyhow::Result;
use colored::*;
use std::fs;

use crate::{
    registry::Registry,
    utils::{copy_from_store, work_dirs},
};

pub fn link(registry: &mut Registry, package_name: &str) -> Result<()> {
    let dirs = work_dirs(package_name, registry)?;

    copy_from_store(package_name, &dirs)?;

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
        "{}\\n{}",
        format!(
            "✅ Package {} successfully linked to this project!",
            package_name
        )
        .green(),
        format!("Warning: `npm install` will overwrite this link. Run `kley link {}` again to restore it.", package_name)
            .yellow()
    );

    Ok(())
}
