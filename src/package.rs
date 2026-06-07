use colored::*;

use anyhow::{Context, Ok, Result};
use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{emoji, lockfile::Lockfile, utils::detect_indent};

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
        {
            tracing::debug!("Detected package manager from kley.lock: {}", lf_pm);
            return Package::translate_pm_type(lf_pm);
        }

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
        let mut buff = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
        let mut ser = serde_json::Serializer::with_formatter(&mut buff, formatter);
        value.serialize(&mut ser)?;

        fs::write(dir.join(PACKAGE_JSON_FILE_NAME), buff)?;

        tracing::info!("package.json has been updated");

        Ok(())
    }

    /// Modifies package.json to add or update a dependency.
    pub fn update_package_json(project_dir: &Path, package_name: &str, is_dev: bool) -> Result<()> {
        let content = match PackageJson::get_raw(project_dir) {
            Result::Err(error) => {
                println!(
                    "{}",
                    format!(
                        "{} Warning: {}, skipping modification.",
                        emoji::WARNING,
                        error
                    )
                    .yellow()
                );
                return Ok(());
            }
            Result::Ok(value) => value,
        };

        let indent = detect_indent(&content);

        let mut value: serde_json::Value =
            serde_json::from_str(&content).context("Failed to parse package.json")?;

        let dep_path = format!("file:.kley/{}", package_name);
        let dep_keys = ["dependencies", "devDependencies", "peerDependencies"];
        let mut updated = false;

        if let Some(obj) = value.as_object_mut() {
            for key in &dep_keys {
                if let Some(dep) = obj
                    .get_mut(*key)
                    .and_then(|d| d.as_object_mut())
                    .and_then(|d| d.get_mut(package_name))
                {
                    *dep = serde_json::Value::String(dep_path.clone());
                    updated = true;
                    break;
                }
            }

            if !updated {
                let target_key = if is_dev {
                    "devDependencies"
                } else {
                    "dependencies"
                };
                if !obj.contains_key(target_key) {
                    obj.insert(target_key.to_string(), serde_json::json!({}));
                }
                obj[target_key].as_object_mut().unwrap().insert(
                    package_name.to_string(),
                    serde_json::Value::String(dep_path),
                );
            }
        }

        PackageJson::save_raw(value, project_dir, &indent)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_add_new_dependency() -> Result<()> {
        let initial_content = r#"{
  "name": "test-project",
  "version": "1.0.0",
  "dependencies": {}
}"#;
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", initial_content)?;
        let dir = file.path().parent().unwrap();

        PackageJson::update_package_json(dir, "my-local-lib", false)?;

        let updated_content = fs::read_to_string(file.path())?;
        let updated_json: serde_json::Value = serde_json::from_str(&updated_content)?;

        assert_eq!(
            updated_json["dependencies"]["my-local-lib"],
            "file:.kley/my-local-lib"
        );

        Ok(())
    }

    #[test]
    fn test_add_new_dev_dependency() -> Result<()> {
        let initial_content = r#"{
  "name": "test-project",
  "version": "1.0.0",
  "devDependencies": {}
}"#;
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", initial_content)?;
        let dir = file.path().parent().unwrap();

        PackageJson::update_package_json(dir, "my-local-lib", true)?;

        let updated_content = fs::read_to_string(file.path())?;
        let updated_json: serde_json::Value = serde_json::from_str(&updated_content)?;

        assert_eq!(
            updated_json["devDependencies"]["my-local-lib"],
            "file:.kley/my-local-lib"
        );

        Ok(())
    }

    #[test]
    fn test_update_existing_dependency() -> Result<()> {
        let initial_content = r#"{
  "name": "test-project",
  "version": "1.0.0",
  "dependencies": {
    "my-local-lib": "1.0.0"
  }
}"#;
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", initial_content)?;
        let dir = file.path().parent().unwrap();

        PackageJson::update_package_json(dir, "my-local-lib", false)?;

        let updated_content = fs::read_to_string(file.path())?;
        let updated_json: serde_json::Value = serde_json::from_str(&updated_content)?;

        assert_eq!(
            updated_json["dependencies"]["my-local-lib"],
            "file:.kley/my-local-lib"
        );

        Ok(())
    }

    #[test]
    fn test_create_dependencies_section() -> Result<()> {
        let initial_content = r#"{
  "name": "test-project",
  "version": "1.0.0"
}"#;
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", initial_content)?;
        let dir = file.path().parent().unwrap();

        PackageJson::update_package_json(dir, "my-local-lib", false)?;

        let updated_content = fs::read_to_string(file.path())?;
        let updated_json: serde_json::Value = serde_json::from_str(&updated_content)?;

        assert_eq!(
            updated_json["dependencies"]["my-local-lib"],
            "file:.kley/my-local-lib"
        );

        Ok(())
    }

    #[test]
    fn test_create_dev_dependencies_section() -> Result<()> {
        let initial_content = r#"{
  "name": "test-project",
  "version": "1.0.0"
}"#;
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", initial_content)?;
        let dir = file.path().parent().unwrap();

        PackageJson::update_package_json(dir, "my-local-lib", true)?;

        let updated_content = fs::read_to_string(file.path())?;
        let updated_json: serde_json::Value = serde_json::from_str(&updated_content)?;

        assert_eq!(
            updated_json["devDependencies"]["my-local-lib"],
            "file:.kley/my-local-lib"
        );

        Ok(())
    }
}
