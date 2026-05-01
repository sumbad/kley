use crate::{commands::remove::remove_package, emoji, package::Package, utils::confirm};
use std::{fs, path::PathBuf};

use crate::registry::Registry;
use anyhow::Result;
use colored::*;

pub fn unpublish(registry: &mut Registry, push: bool) -> Result<()> {
    let package = Package::get(&std::env::current_dir()?)?;

    println!(
        "{} Unpublishing {}...",
        emoji::UNPUBLISH,
        package.json.name.cyan(),
    );

    let pkg_installations = registry.get_installations(&package.json.name).to_vec();

    if !pkg_installations.is_empty() {
        let confirm_msg = if push {
            confirm_hard_msg(&package.json.name, &pkg_installations)
        } else {
            confirm_soft_msg(&package.json.name, pkg_installations.len())
        };

        if !confirm(confirm_msg) {
            println!("Canceled");

            return Ok(());
        }
    }

    // Clean up all projects first while registry metadata is still present
    if push {
        for project_dir in &pkg_installations {
            remove_package(registry, &package.json.name, project_dir)?;
        }
    }

    // Delete the package from the registry
    let pkg_in_registry = registry.get_pkg_dir(&package.json.name);

    // Delete package files if present, but don't abort if already missing
    if pkg_in_registry.exists() {
        fs::remove_dir_all(&pkg_in_registry)?;
    } else {
        println!(
            "Package {} not found in the registry",
            package.json.name.cyan()
        );
    }

    // Always remove metadata to prevent stale registry state
    registry.remove_package_info(&package.json.name)?;

    println!(
        "{}",
        format!(
            "{} Done: {} unpublished",
            emoji::SUCCESS,
            package.json.name.cyan()
        )
        .green()
    );

    Ok(())
}

fn confirm_soft_msg(package_name: &str, count: usize) -> ColoredString {
    let plural = if count == 1 { "project" } else { "projects" };

    let title = format!(
        "Warning: '{}' is used by {} {}",
        package_name.cyan(),
        count,
        plural
    );

    let comment =
        "To remove package and automatically clean up all projects, use `kley unpublish --push`";

    let message = format!(
        "
{}.
This action will remove the package from the store, breaking these projects upon the next install.
{}.
Proceed?",
        title.bold().yellow(),
        comment.italic().dimmed().white()
    );

    message.yellow()
}

fn confirm_hard_msg(package_name: &str, pkg_installations: &[PathBuf]) -> ColoredString {
    let count = pkg_installations.len();
    let plural = if count == 1 { "project" } else { "projects" };

    let title = format!(
        "Attention: This will permanently remove '{}' from the registry AND from the following {} {}:",
        package_name.cyan(),
        count,
        plural,
    );

    let mut comment = String::new();

    pkg_installations
        .iter()
        .for_each(|it| comment.push_str(&format!("\n\t- {}", it.to_string_lossy())));

    let message = format!("{}{}\nProceed?", title.bold().yellow(), comment.white());

    message.yellow()
}
