use std::{path::Path, process::Command};

use anyhow::Result;
use colored::*;
use serde_json::json;

use crate::{
    commands::update::run_update,
    emoji,
    lockfile::Lockfile,
    package::{Package, PackageJson, PackageManagerType},
    registry::Registry,
    utils::{self, PROJECT_REGISTRY_DIR_NAME},
};

/// On Windows, npm/pnpm/yarn are `.cmd` scripts that `Command::new` cannot find directly.
/// We must run them through `cmd /C` so the shell resolves the `.cmd` extension.
#[cfg(windows)]
fn create_command(program: &str, args: &[&str]) -> Command {
    let mut cmd = Command::new("cmd");
    cmd.arg("/C").arg(program);
    for arg in args {
        cmd.arg(arg);
    }
    cmd
}

#[cfg(not(windows))]
fn create_command(program: &str, args: &[&str]) -> Command {
    let mut cmd = Command::new(program);
    for arg in args {
        cmd.arg(arg);
    }
    cmd
}

pub fn install(
    registry: &mut Registry,
    package_name_version: Option<&str>,
    project_dir: &Path,
    dev: bool,
) -> Result<()> {
    match package_name_version {
        Some(pkg_name_version) => {
            install_package(registry, pkg_name_version, project_dir, dev)?;

            println!(
                "{}",
                format!(
                    "{} Done: {} installed",
                    emoji::SUCCESS,
                    pkg_name_version.cyan()
                )
                .green(),
            );
        }
        None => {
            if dev {
                anyhow::bail!(
                    "--dev flag requires a package name. Usage: kley install --dev <package>"
                );
            }

            install_all(registry, project_dir)?
        }
    }

    Ok(())
}

/// Core install logic — copies package, delegates to PM, updates registry.
/// Terminal output is limited to the PM command being run.
fn install_package(
    registry: &mut Registry,
    package_name_version: &str,
    project_dir: &Path,
    dev: bool,
) -> Result<()> {
    let (package_name, package_version) = utils::package_name_version_parse(package_name_version);

    utils::validate_version_in_registry(registry, package_name, package_version);

    run_update(registry, package_name, project_dir)?;

    let package = Package::get(project_dir)?;

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
            vec![
                Some("add"),
                Some(pkg_kley_path_str),
                Some("--ignore-scripts"),
                dev.then_some("-D"),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<&str>>(),
        ),
        PackageManagerType::Yarn => (
            &yarn_command,
            vec![
                Some("add"),
                Some(pkg_kley_path_str),
                Some("--ignore-scripts"),
                dev.then_some("--dev"),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<&str>>(),
        ),
        PackageManagerType::Npm => (
            &npm_command,
            vec![
                Some("install"),
                Some(pkg_kley_path_str),
                Some("--ignore-scripts"),
                dev.then_some("--save-dev"),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<&str>>(),
        ),
    };

    let cmd_display = format!("{} {}", cmd_name, cmd_args.join(" "));
    println!(
        "{} Running...\n{}",
        emoji::WAITING,
        cmd_display.bright_black()
    );

    let status = create_command(cmd_name, &cmd_args)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run {:?}: {}", cmd_name, e))?;

    if !status.success() {
        eprintln!(
            "{}",
            format!(
                "{} Error: {:?} command failed with status: {:?}",
                emoji::ERROR,
                package.manager_type,
                status.code(),
            )
            .red(),
        );

        anyhow::bail!(
            "Package manager {:?} failed with status: {:?}",
            package.manager_type,
            status.code(),
        );
    }

    registry.add_package_installation(package_name, project_dir)?;

    Ok(())
}

fn install_all(registry: &mut Registry, project_dir: &Path) -> Result<()> {
    let lockfile = if let Some(lockfile) = Lockfile::get(project_dir) {
        lockfile
    } else {
        println!(
            "{}",
            format!(
                "{} Warning: No packages to install. kley.lock not found.",
                emoji::WARNING
            )
            .yellow()
        );
        return Ok(());
    };

    if lockfile.packages.is_empty() {
        println!(
            "{}",
            format!("{} Warning: No packages found to install.", emoji::WARNING).yellow()
        );
        return Ok(());
    }

    let package_json = PackageJson::get(project_dir)?;
    let dev_dependencies = package_json.dev_dependencies.unwrap_or(json!({}));

    println!("{}", "Install...".green().dimmed());
    for (package_name, package_info) in lockfile.packages {
        install_package(
            registry,
            &format!("{}@{}", package_name, package_info.version),
            project_dir,
            dev_dependencies.get(&package_name).is_some(),
        )?;

        println!(
            "{}",
            format!("   {} {}", emoji::UPDATED, package_name)
                .green()
                .dimmed()
        );
    }

    println!(
        "{}",
        format!("{} Done: all packages installed", emoji::SUCCESS).green()
    );

    Ok(())
}
