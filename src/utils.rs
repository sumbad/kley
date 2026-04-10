use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::*;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::registry::Registry;

pub static PROJECT_REGISTRY_DIR_NAME: &str = ".kley";

pub struct WorkDirs {
    pub project_dir: PathBuf,
    pub project_kley_dir: PathBuf,
}

pub fn work_dirs(package_name: &str) -> Result<WorkDirs> {
    let project_dir = std::env::current_dir()?;
    let project_kley_dir = project_dir
        .join(PROJECT_REGISTRY_DIR_NAME)
        .join(package_name);

    Ok(WorkDirs {
        project_dir,
        project_kley_dir,
    })
}

pub fn copy_from_registry(
    registry: &Registry,
    package_name: &str,
    project_dir: &Path,
) -> Result<()> {
    tracing::debug!(
        "copy_from_registry:\n package_name: {package_name}\n project_dir: {project_dir:?}"
    );

    let registry_dir = registry.dir_path.join("packages").join(package_name);
    tracing::debug!("registry_dir: {registry_dir:?}");

    if !registry_dir.exists() {
        anyhow::bail!(
            "Package '{}' not found inside registry. Run 'kley publish' in the package folder first.",
            package_name
        );
    }

    let project_kley_dir = project_dir
        .join(PROJECT_REGISTRY_DIR_NAME)
        .join(package_name);

    if project_kley_dir.exists() {
        fs::remove_dir_all(&project_kley_dir)?;
    }
    fs::create_dir_all(&project_kley_dir)?;

    // Copy from store to local project
    let mut options = fs_extra::dir::CopyOptions::new();
    options.content_only = true;
    fs_extra::dir::copy(registry_dir, &project_kley_dir, &options)?;

    tracing::info!(
        "Package {} was copied from registry to {} dir",
        package_name,
        project_dir.display()
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

/// Get the current time in UTC
pub fn current_formatted_time() -> String {
    let now_utc: DateTime<Utc> = Utc::now();

    now_utc.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Confirm to proceed or abort
pub fn confirm(prompt: ColoredString) -> bool {
    loop {
        print!("{} (y/n): ", prompt);
        io::stdout().flush().unwrap(); // Ensure the prompt is printed before reading

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please enter 'y' for yes or 'n' for no."),
        }
    }
}

pub fn normalized_path(path: &Path, home: Option<&PathBuf>) -> String {
    let path = fs::canonicalize(path).unwrap_or(path.to_path_buf());

    if let Some(home_dir) = home
        && let Ok(stripped_path) = path.strip_prefix(&home_dir)
    {
        return format!("~/{}", stripped_path.display());
    }

    path.to_string_lossy().into_owned()
}
