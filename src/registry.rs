use anyhow::{Context, Ok, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::utils::{current_formatted_time, get_kley_home_dir};

pub static REGISTRY_DIR_NAME: &str = ".kley";
pub static REGISTRY_FILE_NAME: &str = "registry.json";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RegistryData {
    #[serde(default)]
    pub packages: BTreeMap<String, PackageMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackageMetadata {
    pub version: String,
    pub last_updated: String,
    pub installations: Vec<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<PathBuf>,
}

pub struct Registry {
    data: RegistryData,
    pub dir_path: PathBuf,
    pub file_path: PathBuf,
}

impl Registry {
    pub fn new() -> Result<Registry> {
        let home_dir = get_kley_home_dir()?;
        Registry::from_home_dir(&home_dir)
    }

    /// Create a Registry with an explicit home directory.
    /// Used for testing to avoid mutating global env vars.
    pub fn with_home_dir(home_dir: &Path) -> Result<Registry> {
        Registry::from_home_dir(home_dir)
    }

    fn from_home_dir(home_dir: &Path) -> Result<Registry> {
        let registry_dir = home_dir.join(REGISTRY_DIR_NAME);
        let registry_file = registry_dir.join(REGISTRY_FILE_NAME);

        if !registry_dir.exists() || !registry_file.exists() {
            return Ok(Registry {
                data: RegistryData::default(),
                dir_path: registry_dir,
                file_path: registry_file,
            });
        }

        let registry_data_content = fs::read_to_string(&registry_file)?;
        let registry_data: RegistryData = serde_json::from_str(&registry_data_content)
            .context(format!("Failed to parse {}", REGISTRY_FILE_NAME))?;

        Ok(Registry {
            data: registry_data,
            dir_path: registry_dir,
            file_path: registry_file,
        })
    }

    pub fn get_pkg_dir(&self, package_name: &str) -> PathBuf {
        self.dir_path.join("packages").join(package_name)
    }

    pub fn is_empty(&self) -> bool {
        !self.dir_path.exists() || !self.file_path.exists()
    }

    pub fn update_package_version(&mut self, package_name: &str, version: &str) -> Result<()> {
        self.data
            .packages
            .entry(package_name.to_string())
            .and_modify(|it| {
                it.version = version.to_string();
                it.last_updated = current_formatted_time();
            })
            .or_insert_with(|| PackageMetadata {
                version: version.to_string(),
                last_updated: current_formatted_time(),
                installations: vec![],
                source_path: None,
                links: vec![],
            });

        self.save()?;

        Ok(())
    }

    pub fn add_package_installation(
        &mut self,
        package_name: &str,
        project_path: &Path,
    ) -> Result<()> {
        if let Some(meta_data) = self.data.packages.get_mut(package_name) {
            let path_buf = project_path.to_path_buf();

            if !meta_data.installations.contains(&path_buf) {
                meta_data.last_updated = current_formatted_time();
                meta_data.installations.push(path_buf);

                self.save()?;
            }
        } else {
            tracing::warn!("Package {} not found in the registry", package_name);
        }

        Ok(())
    }

    pub fn remove_package_installation(
        &mut self,
        package_name: &str,
        project_path: &Path,
    ) -> Result<()> {
        if let Some(meta_data) = self.data.packages.get_mut(package_name) {
            meta_data.last_updated = current_formatted_time();
            meta_data.installations.retain(|it| it != project_path);

            self.save()?;
        } else {
            tracing::warn!("Package {} not found in the registry", package_name);
        }
        Ok(())
    }

    pub fn remove_all_installations(&mut self, project_path: &Path) -> Result<()> {
        let mut was_updated = false;
        for pkg in self.data.packages.iter_mut() {
            if pkg.1.installations.contains(&project_path.to_path_buf()) {
                pkg.1.installations.retain(|it| it != project_path);
                pkg.1.last_updated = current_formatted_time();

                was_updated = true;
            }
        }

        if was_updated {
            self.save()?;
        } else {
            tracing::debug!(
                "No installations in the {} project was found inside the registry",
                project_path.to_string_lossy()
            )
        }
        Ok(())
    }

    pub fn remove_package_info(&mut self, package_name: &str) -> Result<()> {
        if self.data.packages.remove(package_name).is_some() {
            self.save()?;
            tracing::info!("Package {} was removed from the registry", package_name);
        } else {
            tracing::warn!("Package {} not found in the registry", package_name);
        }
        Ok(())
    }

    pub fn get_installations(&self, package_name: &str) -> &[PathBuf] {
        self.data
            .packages
            .get(package_name)
            .map_or(&[], |it| &it.installations)
    }

    pub fn get_pkg_version(&self, package_name: &str) -> Option<&str> {
        self.data
            .packages
            .get(package_name)
            .map(|it| it.version.as_str())
    }

    pub fn has_version_in_registry(
        &self,
        package_name: &str,
        package_version: Option<&str>,
    ) -> bool {
        let registry_pkg_version = self.get_pkg_version(package_name);

        package_version.is_none()
            || registry_pkg_version == package_version
            || (registry_pkg_version.is_some() && package_version == Some("latest"))
    }

    pub fn set_source_path(&mut self, package_name: &str, source_path: &Path) -> Result<()> {
        if let Some(meta_data) = self.data.packages.get_mut(package_name) {
            if meta_data.source_path.as_deref() != Some(source_path) {
                meta_data.source_path = Some(source_path.to_path_buf());
                meta_data.last_updated = current_formatted_time();
                self.save()?;
            }
        } else {
            tracing::warn!("Package {} not found in the registry", package_name);
        }
        Ok(())
    }

    pub fn get_source_path(&self, package_name: &str) -> Option<&Path> {
        self.data
            .packages
            .get(package_name)
            .and_then(|m| m.source_path.as_deref())
    }

    pub fn add_package_link(&mut self, package_name: &str, project_path: &Path) -> Result<()> {
        if let Some(meta_data) = self.data.packages.get_mut(package_name) {
            let path_buf = project_path.to_path_buf();

            if !meta_data.links.contains(&path_buf) {
                meta_data.last_updated = current_formatted_time();
                meta_data.links.push(path_buf);

                self.save()?;
            }
        } else {
            tracing::warn!("Package {} not found in the registry", package_name);
        }

        Ok(())
    }

    pub fn remove_package_link(&mut self, package_name: &str, project_path: &Path) -> Result<()> {
        if let Some(meta_data) = self.data.packages.get_mut(package_name) {
            meta_data.last_updated = current_formatted_time();
            meta_data.links.retain(|it| it != project_path);

            self.save()?;
        } else {
            tracing::warn!("Package {} not found in the registry", package_name);
        }
        Ok(())
    }

    pub fn get_links(&self, package_name: &str) -> &[PathBuf] {
        self.data
            .packages
            .get(package_name)
            .map_or(&[], |it| &it.links)
    }

    pub fn has_installation(&self, package_name: &str, project_path: &Path) -> bool {
        self.data
            .packages
            .get(package_name)
            .is_some_and(|m| m.installations.contains(&project_path.to_path_buf()))
    }

    pub fn has_link(&self, package_name: &str, project_path: &Path) -> bool {
        self.data
            .packages
            .get(package_name)
            .is_some_and(|m| m.links.contains(&project_path.to_path_buf()))
    }

    fn save(&mut self) -> Result<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        self.data.serialize(&mut ser)?;

        fs::write(&self.file_path, buf)?;

        tracing::info!("Updated registry has been saved!");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_registry(home_dir: &Path) -> Registry {
        Registry::with_home_dir(home_dir).unwrap()
    }

    #[test]
    fn test_add_package_link() {
        let tmp = tempdir().unwrap();
        let mut registry = make_registry(tmp.path());
        registry.update_package_version("my-lib", "1.0.0").unwrap();

        let project_path = Path::new("/tmp/project");
        registry.add_package_link("my-lib", project_path).unwrap();

        assert!(registry.has_link("my-lib", project_path));
        assert!(!registry.has_installation("my-lib", project_path));
        assert_eq!(registry.get_links("my-lib").len(), 1);
        assert_eq!(registry.get_installations("my-lib").len(), 0);
    }

    #[test]
    fn test_remove_package_link() {
        let tmp = tempdir().unwrap();
        let mut registry = make_registry(tmp.path());
        registry.update_package_version("my-lib", "1.0.0").unwrap();

        let project_path = Path::new("/tmp/project");
        let other_path = Path::new("/tmp/other");

        registry.add_package_link("my-lib", project_path).unwrap();
        registry
            .add_package_installation("my-lib", other_path)
            .unwrap();

        registry
            .remove_package_link("my-lib", project_path)
            .unwrap();

        assert!(!registry.has_link("my-lib", project_path));
        assert!(
            registry.has_installation("my-lib", other_path),
            "installations should be untouched"
        );
    }

    #[test]
    fn test_get_source_path_none() {
        let tmp = tempdir().unwrap();
        let mut registry = make_registry(tmp.path());
        registry.update_package_version("my-lib", "1.0.0").unwrap();

        assert!(registry.get_source_path("my-lib").is_none());
    }

    #[test]
    fn test_set_and_get_source_path() {
        let tmp = tempdir().unwrap();
        let mut registry = make_registry(tmp.path());
        registry.update_package_version("my-lib", "1.0.0").unwrap();

        let source = Path::new("/tmp/my-lib-source");
        registry.set_source_path("my-lib", source).unwrap();

        assert_eq!(registry.get_source_path("my-lib"), Some(source));
    }

    #[test]
    fn test_source_path_persisted_in_json() {
        let tmp = tempdir().unwrap();
        let mut registry = make_registry(tmp.path());
        registry.update_package_version("my-lib", "1.0.0").unwrap();

        let source = Path::new("/tmp/my-lib-source");
        registry.set_source_path("my-lib", source).unwrap();

        let registry_json = fs::read_to_string(&registry.file_path).unwrap();
        assert!(
            registry_json.contains("sourcePath"),
            "registry.json should use camelCase 'sourcePath'. Content:\n{}",
            registry_json
        );
    }

    #[test]
    fn test_has_installation() {
        let tmp = tempdir().unwrap();
        let mut registry = make_registry(tmp.path());
        registry.update_package_version("my-lib", "1.0.0").unwrap();

        let project_path = Path::new("/tmp/project");
        assert!(!registry.has_installation("my-lib", project_path));
        registry
            .add_package_installation("my-lib", project_path)
            .unwrap();
        assert!(registry.has_installation("my-lib", project_path));
    }

    #[test]
    fn test_has_link() {
        let tmp = tempdir().unwrap();
        let mut registry = make_registry(tmp.path());
        registry.update_package_version("my-lib", "1.0.0").unwrap();

        let project_path = Path::new("/tmp/project");
        assert!(!registry.has_link("my-lib", project_path));
        registry.add_package_link("my-lib", project_path).unwrap();
        assert!(registry.has_link("my-lib", project_path));
    }

    #[test]
    fn test_legacy_registry_no_source_path() {
        let tmp = tempdir().unwrap();

        let registry_dir = tmp.path().join(".kley");
        fs::create_dir_all(&registry_dir).unwrap();
        fs::write(
            registry_dir.join("registry.json"),
            r#"{"packages":{"old-lib":{"version":"1.0.0","lastUpdated":"2024-01-01T00:00:00Z","installations":[]}}}"#,
        )
        .unwrap();

        let registry = Registry::with_home_dir(tmp.path()).unwrap();
        assert!(registry.get_source_path("old-lib").is_none());
        assert_eq!(registry.get_links("old-lib"), &[] as &[PathBuf]);
    }
}
