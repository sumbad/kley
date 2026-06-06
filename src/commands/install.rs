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
    utils::{self, PROJECT_REGISTRY_DIR_NAME, normalized_path},
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
    no_save: bool,
) -> Result<()> {
    match package_name_version {
        Some(pkg_name_version) => {
            install_package(registry, pkg_name_version, project_dir, dev, no_save)?;

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

            install_all(registry, project_dir, no_save)?
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
    no_save: bool,
) -> Result<()> {
    let (package_name, package_version) = utils::package_name_version_parse(package_name_version);

    utils::validate_version_in_registry(registry, package_name, package_version);

    let package = Package::get(project_dir)?;

    let pkg_kley_path = project_dir
        .join(PROJECT_REGISTRY_DIR_NAME)
        .join(package_name);

    let deps_path = project_dir.join("node_modules").join(package_name);

    let deps_snapshot = package
        .lockfile
        .as_ref()
        .and_then(|it| it.packages.get(package_name));

    run_update(registry, package_name, project_dir)?;
    registry.add_package_installation(package_name, project_dir)?;

    let installed_pkg_json = PackageJson::get(&pkg_kley_path)?;

    let is_same_deps = if let Some(info) = deps_snapshot {
        info.dependencies == installed_pkg_json.dependencies
            && info.peer_dependencies == installed_pkg_json.peer_dependencies
    } else {
        false
    };

    if is_same_deps {
        if deps_path.is_symlink() {
            let link_target = std::fs::read_link(&deps_path)?;

            if normalized_path(&link_target, None) == normalized_path(&pkg_kley_path, None) {
                // Case A: symlink already points to .kley/<pkg> — nothing to do
                tracing::info!("Fast path Case A: correct symlink, nothing to do");
                return Ok(());
            }

            // Case B: symlink points to an unknown location — fall back to PM
            tracing::info!("Destination is an unknown symlink, falling back to PM");
            pm_install_command(&pkg_kley_path, &package, dev, no_save)?;
            return Ok(());
        }

        if deps_path.exists() {
            // Case C: regular directory — copy directly, skip PM
            tracing::info!("Destination directory exists, copying directly to node_modules");
            let mut options = fs_extra::dir::CopyOptions::new();
            options.overwrite = true;
            options.content_only = true;
            fs_extra::dir::copy(&pkg_kley_path, &deps_path, &options)?;
            return Ok(());
        }

        // deps_path does not exist — fall through to PM (slow path),
        // so that the PM registers the file: dependency in package.json
        tracing::info!("Destination directory node_modules/<pkg> absent, falling back to PM");
    }

    // Slow path: no snapshot, deps changed, or node_modules/<pkg> doesn't exist
    pm_install_command(&pkg_kley_path, &package, dev, no_save)?;

    Ok(())
}

fn pm_install_command(
    pkg_kley_path: &Path,
    package: &Package,
    dev: bool,
    no_save: bool,
) -> Result<()> {
    let pkg_kley_path_str = pkg_kley_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Path contains non-UTF8 characters: {:?}", pkg_kley_path))?;

    let npm_command = std::env::var("KLEY_USE_NPM_COMMAND").unwrap_or("npm".to_string());
    let pnpm_command = std::env::var("KLEY_USE_PNPM_COMMAND").unwrap_or("pnpm".to_string());
    let yarn_command = std::env::var("KLEY_USE_YARN_COMMAND").unwrap_or("yarn".to_string());

    let (cmd_name, cmd_args): (&str, Vec<&str>) = match package.manager_type {
        PackageManagerType::Npm => (
            &npm_command,
            vec![
                Some("install"),
                Some(pkg_kley_path_str),
                Some("--ignore-scripts"),
                dev.then_some("--save-dev"),
                no_save.then_some("--no-save"),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<&str>>(),
        ),
        PackageManagerType::Pnpm => (
            &pnpm_command,
            vec![
                Some("add"),
                Some(pkg_kley_path_str),
                Some("--ignore-scripts"),
                dev.then_some("-D"),
                no_save.then_some("--save=false"),
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
                // Yarn v1 has no --no-save equivalent.
                // The package.json modification will happen regardless.
                // This is documented as a known limitation.
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

    Ok(())
}

fn install_all(registry: &mut Registry, project_dir: &Path, no_save: bool) -> Result<()> {
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
            no_save,
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
