use anyhow::Result;
use colored::*;

use std::collections::HashSet;

use crate::commands::update::add_package_into_project;
use crate::emoji;
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

    add_package_into_project(
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
