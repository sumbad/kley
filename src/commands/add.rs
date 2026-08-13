use anyhow::Result;
use colored::*;

use std::collections::HashSet;
use std::path::Path;

use crate::commands::update::run_update;
use crate::emoji;
use crate::package::PackageJson;
use crate::registry::Registry;
use crate::utils;

/// Add logic
pub fn add(
    registry: &mut Registry,
    package_name_version: &str,
    is_dev: bool,
    pure: bool,
    resolve_workspace: bool,
) -> Result<()> {
    let (package_name, package_version) = utils::package_name_version_parse(package_name_version);

    utils::validate_version_in_registry(registry, package_name, package_version);

    let project_dir = std::env::current_dir()?;
    let mut visited = HashSet::new();

    install_package_into_project(
        registry,
        package_name,
        is_dev,
        pure,
        resolve_workspace,
        &project_dir,
        &mut visited,
    )?;

    if pure {
        println!(
            "{}",
            "Note: package.json and node_modules left untouched (--pure)."
                .italic()
                .bright_black()
        );
    }

    println!(
        "{}\n{}",
        "Note: run `npm install` to update project's node_modules."
            .italic()
            .bright_black(),
        format!("{} Done: {} added", emoji::SUCCESS, package_name.cyan()).green(),
    );

    Ok(())
}

/// Install a package into a project: copy it into `.kley/`, optionally inject a
/// `file:.kley/<pkg>` entry into the project's `package.json`, and record the
/// installation in the registry. This is the shared implementation behind both
/// `kley add` and the resolution of `workspace:` dependencies during `run_update`.
pub fn install_package_into_project(
    registry: &mut Registry,
    package_name: &str,
    is_dev: bool,
    pure: bool,
    resolve_workspace: bool,
    project_dir: &Path,
    visited: &mut HashSet<String>,
) -> Result<()> {
    run_update(
        registry,
        package_name,
        project_dir,
        pure,
        resolve_workspace,
        visited,
    )?;

    if !pure {
        PackageJson::update_dependency(project_dir, package_name, is_dev)?;
    }

    registry.add_package_installation(package_name, project_dir)?;

    Ok(())
}
