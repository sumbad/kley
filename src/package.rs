use anyhow::{Context, Ok, Result};
use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::lockfile::Lockfile;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageJson {
    pub name: String,
    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_dependencies: Option<serde_json::Value>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, String>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub peer_dependencies: BTreeMap<String, String>,
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

const PACKAGE_JSON_FILE_NAME: &str = "package.json";

impl Package {
    pub fn get(dir: &Path) -> Result<Self> {
        let package_json = PackageJson::get(dir).context("Failed to get package information")?;

        let lockfile = Lockfile::get(dir);

        let manager_type =
            Package::detect_package_manager_type(dir, &package_json, lockfile.as_ref());

        Ok(Package {
            json: package_json,
            lockfile,
            manager_type,
        })
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
            tracing::debug!("Detected package manager from kley.lock: {}", lf_pm);
            return Package::translate_pm_type(lf_pm);
        }

        // packageManager exists inside package.json and is not empty
        if let Some(json_pm) = &package_json.package_manager
            && !json_pm.is_empty()
        {
            tracing::debug!("Detected package manager from package.json: {}", json_pm);
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

impl PackageJson {
    pub fn get(dir: &Path) -> Result<Self> {
        let content = PackageJson::get_raw(dir)?;
        let pkg: PackageJson = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse package.json. Details: {e}"))?;

        Ok(pkg)
    }

    pub fn get_raw(dir: &Path) -> Result<String> {
        let pkg_path = dir.join(PACKAGE_JSON_FILE_NAME);
        if !pkg_path.exists() {
            anyhow::bail!("package.json not found in the current directory");
        }

        let pkg_content = fs::read_to_string(pkg_path)?;

        Ok(pkg_content)
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        Self::save_raw(self, dir, "  ")?;

        Ok(())
    }

    pub fn save_raw<T: Serialize>(value: T, dir: &Path, indent: &str) -> Result<()> {
        let mut buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        value.serialize(&mut ser)?;

        fs::write(dir.join(PACKAGE_JSON_FILE_NAME), buf)?;

        tracing::info!("package.json has been updated");

        Ok(())
    }
}
