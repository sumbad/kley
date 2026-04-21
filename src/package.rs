use anyhow::{self, Context, Result};
use std::{fs, path::Path};

use serde::Deserialize;

use crate::lockfile::Lockfile;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageJson {
    pub name: String,
    pub version: String,
    pub files: Option<Vec<String>>,
    pub package_manager: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum PackageManagerType {
    Npm,
    Pnpm,
    Yarn,
}

pub struct Package {
    pub json: PackageJson,
    pub lockfile: Option<Lockfile>,
    pub manager_type: PackageManagerType,
}

impl Package {
    pub fn get(dir: &Path) -> Result<Self> {
        let package_json =
            Package::find_package_json(dir).context("Failed to get package information")?;

        let lockfile = Lockfile::get(dir);

        let manager_type =
            Package::detect_package_manager_type(dir, &package_json, lockfile.as_ref());

        Ok(Package {
            json: package_json,
            lockfile,
            manager_type,
        })
    }

    fn find_package_json(dir: &Path) -> Result<PackageJson> {
        // Read package.json
        let pkg_path = dir.join("package.json");
        if !pkg_path.exists() {
            anyhow::bail!("package.json not found in the current directory");
        }

        let pkg_content = fs::read_to_string(pkg_path)?;
        let pkg: PackageJson = serde_json::from_str(&pkg_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse package.json. Details: {e}"))?;

        Ok(pkg)
    }

    fn detect_package_manager_type(
        dir: &Path,
        package_json: &PackageJson,
        lockfile: Option<&Lockfile>,
    ) -> PackageManagerType {
        if let Some(lf) = lockfile
            && let Some(lf_pm) = &lf.package_manager
            && !lf_pm.is_empty()
        // packageManager exists inside kley.lock and is not empty
        {
            return Package::translate_pm_type(lf_pm);
        }

        // packageManager exists inside package.json and is not empty
        if let Some(json_pm) = &package_json.package_manager
            && !json_pm.is_empty()
        {
            return Package::translate_pm_type(json_pm.as_str());
        }

        if dir.join("pnpm-lock.yaml").exists() {
            return PackageManagerType::Pnpm;
        }

        if dir.join("yarn.lock").exists() {
            return PackageManagerType::Yarn;
        }

        PackageManagerType::Npm
    }

    fn translate_pm_type(pm_string: &str) -> PackageManagerType {
        match pm_string {
            pm if pm.starts_with("yarn") => PackageManagerType::Yarn,
            pm if pm.starts_with("pnpm") => PackageManagerType::Pnpm,
            _ => PackageManagerType::Npm,
        }
    }
}
