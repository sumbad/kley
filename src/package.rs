use anyhow::{Context, Ok, Result};
use std::collections::{BTreeMap, HashMap};
use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{lockfile::Lockfile, utils::detect_indent};

#[derive(Serialize, Deserialize, Debug, Default)]
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

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scripts: Option<HashMap<String, String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<serde_json::Value>,
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

    /// Modifies package.json in a dir to add or update a dependency
    pub fn update_dependency(
        project_dir: &Path,
        dependency_name: &str,
        is_dev: bool,
    ) -> Result<()> {
        let content = PackageJson::get_raw(project_dir)?;

        let indent = detect_indent(&content);

        let mut value: serde_json::Value =
            serde_json::from_str(&content).context("Failed to parse package.json")?;

        let dep_path = format!("file:.kley/{}", dependency_name);
        let dep_keys = ["dependencies", "devDependencies", "peerDependencies"];
        let mut updated = false;

        if let Some(obj) = value.as_object_mut() {
            for key in &dep_keys {
                if let Some(dep) = obj
                    .get_mut(*key)
                    .and_then(|d| d.as_object_mut())
                    .and_then(|d| d.get_mut(dependency_name))
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
                    dependency_name.to_string(),
                    serde_json::Value::String(dep_path),
                );
            }
        }

        PackageJson::save_raw(value, project_dir, &indent)?;

        Ok(())
    }

    /// Returns true if the project's package.json declares a `workspaces` field,
    /// in either form: an array of globs (`"workspaces": [...]`) or the
    /// object form (`"workspaces": { "packages": [...] }`)
    pub fn has_workspaces(&self) -> bool {
        match &self.workspaces {
            Some(serde_json::Value::Array(a)) => !a.is_empty(),
            Some(serde_json::Value::Object(o)) => o
                .get("packages")
                .and_then(|p| p.as_array())
                .map(|a| !a.is_empty())
                .unwrap_or(false),
            _ => false,
        }
    }
}

/// A dependency that used the `workspace:` protocol, extracted from a
/// package's manifest and rewritten to a plain semver range.
pub struct WorkspaceDep {
    pub name: String,
    pub range: String,
    /// Whether this dependency should be injected into the consumer's root
    /// `package.json` (`true` for `dependencies`, `false` for
    /// `peerDependencies`).
    pub inject: bool,
}

/// Find `workspace:` protocol dependencies in a package's `package.json` (under
/// `dependencies` and `peerDependencies`), strip the `workspace:` prefix so the
/// specifier becomes a plain semver range, and return the extracted
/// dependencies so the caller can locally resolve them.
///
/// Returns an empty vec when there are no `workspace:` deps (or no file).
pub fn extract_and_strip_workspace_protocol(dir: &Path) -> Result<Vec<WorkspaceDep>> {
    let pkg_path = dir.join("package.json");
    if !pkg_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&pkg_path)?;
    let indent = detect_indent(&content);
    let mut value: serde_json::Value = serde_json::from_str(&content)?;

    let mut deps = Vec::new();

    let sections: [(&str, bool); 2] = [("dependencies", true), ("peerDependencies", false)];
    if let Some(obj) = value.as_object_mut() {
        for (section, inject) in sections {
            if let Some(map) = obj.get_mut(section).and_then(|s| s.as_object_mut()) {
                let keys: Vec<String> = map.keys().cloned().collect();
                for key in keys {
                    if let Some(serde_json::Value::String(spec)) = map.get(&key) {
                        if let Some(range) = spec.strip_prefix("workspace:") {
                            deps.push(WorkspaceDep {
                                name: key.clone(),
                                range: range.to_string(),
                                inject,
                            });
                            map.insert(key, serde_json::Value::String(range.to_string()));
                        }
                    }
                }
            }
        }
    }

    if !deps.is_empty() {
        PackageJson::save_raw(&value, dir, &indent)?;
    }

    Ok(deps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_add_new_dependency() -> Result<()> {
        let initial_content = r#"{
  "name": "test-project",
  "version": "1.0.0",
  "dependencies": {}
}"#;
        let tmp = tempdir()?;
        fs::write(tmp.path().join("package.json"), initial_content)?;
        let dir = tmp.path();

        PackageJson::update_dependency(dir, "my-local-lib", false)?;

        let updated_content = fs::read_to_string(dir.join("package.json"))?;
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
        let tmp = tempdir()?;
        fs::write(tmp.path().join("package.json"), initial_content)?;
        let dir = tmp.path();

        PackageJson::update_dependency(dir, "my-local-lib", true)?;

        let updated_content = fs::read_to_string(dir.join("package.json"))?;
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
        let tmp = tempdir()?;
        fs::write(tmp.path().join("package.json"), initial_content)?;
        let dir = tmp.path();

        PackageJson::update_dependency(dir, "my-local-lib", false)?;

        let updated_content = fs::read_to_string(dir.join("package.json"))?;
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
        let tmp = tempdir()?;
        fs::write(tmp.path().join("package.json"), initial_content)?;
        let dir = tmp.path();

        PackageJson::update_dependency(dir, "my-local-lib", false)?;

        let updated_content = fs::read_to_string(dir.join("package.json"))?;
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
        let tmp = tempdir()?;
        fs::write(tmp.path().join("package.json"), initial_content)?;
        let dir = tmp.path();

        PackageJson::update_dependency(dir, "my-local-lib", true)?;

        let updated_content = fs::read_to_string(dir.join("package.json"))?;
        let updated_json: serde_json::Value = serde_json::from_str(&updated_content)?;

        assert_eq!(
            updated_json["devDependencies"]["my-local-lib"],
            "file:.kley/my-local-lib"
        );

        Ok(())
    }

    #[test]
    fn test_has_workspaces_array_form() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(
            tmp.path().join("package.json"),
            r#"{ "name": "ws", "version": "1.0.0", "workspaces": ["packages/*"] }"#,
        )?;

        let pkg = PackageJson::get(tmp.path())?;
        assert!(pkg.has_workspaces());
        Ok(())
    }

    #[test]
    fn test_has_workspaces_object_form() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(
            tmp.path().join("package.json"),
            r#"{ "name": "ws", "version": "1.0.0", "workspaces": {"packages": ["app/*"]} }"#,
        )?;

        let pkg = PackageJson::get(tmp.path())?;
        assert!(pkg.has_workspaces());
        Ok(())
    }

    #[test]
    fn test_has_workspaces_absent() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(
            tmp.path().join("package.json"),
            r#"{ "name": "ws", "version": "1.0.0" }"#,
        )?;
        let pkg = PackageJson::get(tmp.path())?;
        assert!(!pkg.has_workspaces());
        Ok(())
    }

    #[test]
    fn test_has_workspaces_empty_array() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(
            tmp.path().join("package.json"),
            r#"{ "name": "ws", "version": "1.0.0", "workspaces": [] }"#,
        )?;
        let pkg = PackageJson::get(tmp.path())?;
        assert!(!pkg.has_workspaces());
        Ok(())
    }

    #[test]
    fn test_extract_and_strip_workspace_protocol() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(
            tmp.path().join("package.json"),
            r#"{
  "name": "app",
  "version": "1.0.0",
  "dependencies": {
    "my-lib": "workspace:^1.2.0",
    "plain": "^1.0.0"
  },
  "peerDependencies": {
    "peer-lib": "workspace:*"
  }
}"#,
        )?;

        let deps = extract_and_strip_workspace_protocol(tmp.path())?;
        assert_eq!(deps.len(), 2);
        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"my-lib"));
        assert!(names.contains(&"peer-lib"));

        let my_lib = deps.iter().find(|d| d.name == "my-lib").unwrap();
        assert_eq!(my_lib.range, "^1.2.0");
        assert!(my_lib.inject);

        let peer = deps.iter().find(|d| d.name == "peer-lib").unwrap();
        assert_eq!(peer.range, "*");
        assert!(!peer.inject);

        // The manifest on disk must have the protocol stripped to plain ranges.
        let updated: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(tmp.path().join("package.json"))?)?;
        assert_eq!(updated["dependencies"]["my-lib"], "^1.2.0");
        assert_eq!(updated["dependencies"]["plain"], "^1.0.0");
        assert_eq!(updated["peerDependencies"]["peer-lib"], "*");

        Ok(())
    }
}
