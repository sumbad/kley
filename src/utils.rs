use anyhow::{Context, Result};
use colored::Colorize;
use std::{fs, path::PathBuf};

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

pub struct WorkDirs {
    pub home_dir: PathBuf,
    pub project_dir: PathBuf,
    pub store_kley_dir: PathBuf,
    pub project_kley_dir: PathBuf,
}

pub fn work_dirs(package_name: &str) -> Result<WorkDirs> {
    let home_dir = dirs::home_dir().context("Failed to find home directory")?;
    let project_dir = std::env::current_dir()?;
    let store_kley_dir = home_dir.join(".kley").join("packages").join(package_name);
    let project_kley_dir = project_dir.join(".kley").join(package_name);

    Ok(WorkDirs {
        home_dir,
        project_dir,
        store_kley_dir,
        project_kley_dir,
    })
}

pub fn copy_from_store(package_name: &str, dirs: &WorkDirs) -> Result<()> {
    if !dirs.store_kley_dir.exists() {
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
    fs_extra::dir::copy(&dirs.store_kley_dir, &dirs.project_kley_dir, &options)?;

    println!(
        "📎 Package {} was coped to current project .kley/{} dir",
        package_name.cyan(),
        package_name.cyan()
    );

    Ok(())
}
