use anyhow::Result;
use colored::Colorize;
use std::{fs, path::PathBuf};

use crate::registry::Registry;

pub static PROJECT_REGISTRY_DIR_NAME: &str = ".kley";

pub struct WorkDirs {
    pub project_dir: PathBuf,
    pub registry_dir: PathBuf,
    pub project_kley_dir: PathBuf,
}

pub fn work_dirs(package_name: &str, registry: &Registry) -> Result<WorkDirs> {
    let project_dir = std::env::current_dir()?;
    let registry_dir = registry.dir_path.join("packages").join(package_name);
    let project_kley_dir = project_dir
        .join(PROJECT_REGISTRY_DIR_NAME)
        .join(package_name);

    Ok(WorkDirs {
        project_dir,
        registry_dir,
        project_kley_dir,
    })
}

pub fn copy_from_store(package_name: &str, dirs: &WorkDirs) -> Result<()> {
    if !dirs.registry_dir.exists() {
        anyhow::bail!(
            "Package '{}' not found in store. Run 'kley publish' in the package folder first.",
            package_name
        );
    }

    if dirs.project_kley_dir.exists() {
        fs::remove_dir_all(&dirs.project_kley_dir)?;
    }
    fs::create_dir_all(&dirs.project_kley_dir)?;

    // Copy from store to local project
    let mut options = fs_extra::dir::CopyOptions::new();
    options.content_only = true;
    fs_extra::dir::copy(&dirs.registry_dir, &dirs.project_kley_dir, &options)?;

    println!(
        "📎 Package {} was coped to current project .kley/{} dir",
        package_name.cyan(),
        package_name.cyan()
    );

    Ok(())
}

/// Detects the indentation of a JSON string.
pub fn detect_indent(json_str: &str) -> String {
    for line in json_str.lines() {
        if line.starts_with("  ") {
            let indent_len = line.find(|c: char| !c.is_whitespace()).unwrap_or(0);
            if indent_len > 0 {
                return line[..indent_len].to_string();
            }
        }
    }
    "  ".to_string() // Default to 2 spaces
}
