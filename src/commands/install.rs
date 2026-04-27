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

    run_update(registry, package_name, &std::env::current_dir()?)?;

    let package = Package::get(&std::env::current_dir()?)?;

    let pkg_kley_path = project_dir
        .join(PROJECT_REGISTRY_DIR_NAME)
        .join(package_name);

    let pkg_kley_path_str = pkg_kley_path.as_os_str().to_str().unwrap();

    let cmd_str = match package.manager_type {
        PackageManagerType::Pnpm => format!("pnpm add {}", pkg_kley_path_str),
        PackageManagerType::Yarn => format!("yarn add {}", pkg_kley_path_str),
        PackageManagerType::Npm => format!("npm install {}", pkg_kley_path_str),
    };

    let status = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", &cmd_str]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", &cmd_str]);
        cmd
    }
    .status()
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

        std::process::exit(1);
    }

    println!(
        "{}",
        format!("✅ Done: {} installed", package_name.cyan()).green(),
    );

    Ok(())
}
