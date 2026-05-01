use std::{path::Path, process::Command};

use anyhow::Result;
use colored::*;

use crate::{
    commands::update::run_update,
    package::{Package, PackageManagerType},
    registry::Registry,
    utils::{self, PROJECT_REGISTRY_DIR_NAME},
};

pub fn install(
    registry: &mut Registry,
    package_name_version: &str,
    project_dir: &Path,
) -> Result<()> {
    let (package_name, package_version) = utils::package_name_version_parse(package_name_version);

    utils::validate_version_in_registry(registry, package_name, package_version);

    run_update(registry, package_name, project_dir)?;

    let package = Package::get(&std::env::current_dir()?)?;

    let pkg_kley_path = project_dir
        .join(PROJECT_REGISTRY_DIR_NAME)
        .join(package_name);

    let pkg_kley_path_str = pkg_kley_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Path contains non-UTF8 characters: {:?}", pkg_kley_path))?;

    let npm_command = std::env::var("KLEY_USE_NPM_COMMAND").unwrap_or("npm".to_string());
    let pnpm_command = std::env::var("KLEY_USE_PNPM_COMMAND").unwrap_or("pnpm".to_string());
    let yarn_command = std::env::var("KLEY_USE_YARN_COMMAND").unwrap_or("yarn".to_string());

    let (cmd_name, cmd_args): (&str, Vec<&str>) = match package.manager_type {
        PackageManagerType::Pnpm => (
            &pnpm_command,
            vec!["add", pkg_kley_path_str, "--ignore-scripts"],
        ),
        PackageManagerType::Yarn => (
            &yarn_command,
            vec!["add", pkg_kley_path_str, "--ignore-scripts"],
        ),
        PackageManagerType::Npm => (
            &npm_command,
            vec!["install", pkg_kley_path_str, "--ignore-scripts"],
        ),
    };

    let cmd_display = format!("{} {}", cmd_name, cmd_args.join(" "));
    println!("⏳ Running...\n{}", cmd_display.dimmed());

    let status = match package.manager_type {
        PackageManagerType::Pnpm => Command::new(&pnpm_command).args(&cmd_args).status(),
        PackageManagerType::Yarn => Command::new(&yarn_command).args(&cmd_args).status(),
        PackageManagerType::Npm => Command::new(&npm_command).args(&cmd_args).status(),
    }
    .expect("Failed to run command");

    if !status.success() {
        eprintln!(
            "{}",
            format!(
                "❌ Error: {:?} command failed with status: {:?}",
                package.manager_type,
                status.code(),
            )
            .red(),
        );

        // TODO: change to return Error for supporting RAII destructors
        std::process::exit(1);
    }

    registry.add_package_installation(package_name, project_dir)?;

    println!(
        "{}",
        format!("✅ Done: {} installed", package_name.cyan()).green(),
    );

    Ok(())
}
