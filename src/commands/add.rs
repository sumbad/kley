use anyhow::Result;
use colored::*;

use crate::commands::update::run_update;
use crate::emoji;
use crate::package::PackageJson;
use crate::registry::Registry;
use crate::utils::{self, work_dirs};

/// Add logic
pub fn add(registry: &mut Registry, package_name_version: &str, is_dev: bool) -> Result<()> {
    let (package_name, package_version) = utils::package_name_version_parse(package_name_version);

    utils::validate_version_in_registry(registry, package_name, package_version);

    let dirs = work_dirs(package_name)?;

    run_update(registry, package_name, &std::env::current_dir()?)?;

    // --- Automate package.json modification ---
    PackageJson::update_dependency(&dirs.project_dir, package_name, is_dev)?;

    registry.add_package_installation(package_name, &dirs.project_dir)?;

    println!(
        "{}\n{}",
        "Note: run `npm install` to update project's node_modules."
            .italic()
            .bright_black(),
        format!("{} Done: {} added", emoji::SUCCESS, package_name.cyan()).green(),
    );

    Ok(())
}
