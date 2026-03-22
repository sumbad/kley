use colored::*;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::os::windows::symlink_dir;

use anyhow::Result;

use crate::utils::{copy_from_store, work_dirs};

pub fn link(package_name: &str) -> Result<()> {
    let dirs = work_dirs(package_name)?;

    copy_from_store(package_name, &dirs)?;

    let node_modules_pkg_dir = dirs.project_dir.join("node_modules").join(package_name);

    if node_modules_pkg_dir.exists() {
        fs::remove_dir_all(&node_modules_pkg_dir)?;
    }

    #[cfg(unix)]
    symlink(dirs.project_kley_dir, node_modules_pkg_dir)?;
    #[cfg(windows)]
    symlink_dir(dirs.project_kley_dir, node_modules_pkg_dir)?;

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
