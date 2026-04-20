use anyhow::{Context, Result};
use std::{fs, path::Path};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageJson {
    pub name: String,
    pub version: String,
    pub files: Option<Vec<String>>,
    pub package_manager: Option<String>,
}

pub enum PackageManagerType {
    Npm,
    Pnpm,
    Yarn,
}

pub struct Package {
    pub manager_type: PackageManagerType,
    pub json: PackageJson,
}

impl Package {
    pub fn get(dir: &Path) -> Result<Self> {
        let package_json = Package::find_package_json(dir)?;

        Ok(Package {
            json: package_json,
            manager_type: PackageManagerType::Npm,
        })
    }

    fn find_package_json(dir: &Path) -> Result<PackageJson> {
        // Read package.json
        let pkg_path = dir.join("package.json");
        if !pkg_path.exists() {
            anyhow::bail!("package.json not found in the current directory");
        }

        let pkg_content = fs::read_to_string(pkg_path)?;
        let pkg: PackageJson =
            serde_json::from_str(&pkg_content).context("Failed to parse package.json")?;

        Ok(pkg)
    }

    // fn detect_package_manager(dir: &Path, package_json: PackageJson) -> PackageManagerType {}
}
