use anyhow::{Context, Ok, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::utils::{current_formatted_time, get_kley_home_dir};

pub static REGISTRY_DIR_NAME: &str = ".kley";
pub static REGISTRY_FILE_NAME: &str = "registry.json";

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct RegistryData {
    #[serde(default)]
    pub packages: BTreeMap<String, PackageMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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
    /// The in-memory registry state, mutated by the current process.
    data: RegistryData,
    /// Snapshot of the on-disk state as it was when this `Registry` was loaded.
    /// Used to compute a correct 3-way merge on save so that our own removals
    /// are honoured while concurrent additions by other processes survive.
    loaded: RegistryData,
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
                loaded: RegistryData::default(),
                dir_path: registry_dir,
                file_path: registry_file,
            });
        }

        let registry_data_content = fs::read_to_string(&registry_file)?;
        let registry_data: RegistryData = serde_json::from_str(&registry_data_content)
            .context(format!("Failed to parse {}", REGISTRY_FILE_NAME))?;

        Ok(Registry {
            data: registry_data.clone(),
            loaded: registry_data,
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

    /// Atomically persist the registry.
    ///
    /// Because multiple `kley` processes (e.g. a long-running `kley watch`
    /// next to a `kley install`) each keep their own in-memory copy and write
    /// the same file, a naive read-modify-write loses the other process's
    /// concurrent changes. To avoid that:
    ///   1. take an exclusive advisory lock on the registry file itself
    ///      (`std::fs::File::lock`, stable since Rust 1.89; `flock` on Unix,
    ///      `LockFileEx` on Windows) — released automatically when the file is
    ///      dropped,
    ///   2. re-read the latest on-disk state,
    ///   3. merge our in-memory changes into it (union of `installations`/`links`,
    ///      newer `last_updated` wins for `version`/`source_path`),
    ///   4. truncate and write the merged result, then release the lock on drop.
    ///
    /// Merge three views of a path list using a 3-way strategy.
    ///
    /// `our` is the intended list in this process, `loaded` is the list as
    /// it was when this `Registry` was loaded, and `on_disk` is the
    /// latest on-disk list (which may include concurrent changes).
    ///
    /// Our own additions are kept, our own removals are honoured, and
    /// entries added concurrently by other processes (present on disk but
    /// unknown to us) are preserved.
    fn merge_path_lists(our: &[PathBuf], loaded: &[PathBuf], on_disk: &[PathBuf]) -> Vec<PathBuf> {
        let added: Vec<&PathBuf> = our.iter().filter(|p| !loaded.contains(p)).collect();
        let removed: Vec<&PathBuf> = loaded.iter().filter(|p| !our.contains(p)).collect();

        let mut result: Vec<PathBuf> = on_disk
            .iter()
            .filter(|p| !removed.contains(p))
            .cloned()
            .collect();

        for p in added {
            if !result.contains(p) {
                result.push(p.clone());
            }
        }

        result
    }

    /// 3-way merge of a single package's metadata into its on-disk view.
    fn merge_package_metadata(
        our: &PackageMetadata,
        loaded: Option<&PackageMetadata>,
        on_disk: &PackageMetadata,
    ) -> PackageMetadata {
        let loaded = loaded.cloned().unwrap_or_default();

        let mut merged = on_disk.clone();
        merged.installations = Self::merge_path_lists(
            &our.installations,
            &loaded.installations,
            &on_disk.installations,
        );
        merged.links = Self::merge_path_lists(&our.links, &loaded.links, &on_disk.links);

        if our.last_updated >= on_disk.last_updated {
            merged.version = our.version.clone();
            merged.last_updated = our.last_updated.clone();
            merged.source_path = our.source_path.clone();
        }

        merged
    }

    /// 3-way merge of the whole registry: `our` is this process's intended
    /// state, `loaded` is the snapshot taken at load time, `on_disk` is the
    /// latest state read from disk (may reflect concurrent processes).
    fn merge_registry_data(
        our: &RegistryData,
        loaded: &RegistryData,
        on_disk: &RegistryData,
    ) -> RegistryData {
        let mut merged = on_disk.clone();

        for (name, our_meta) in &our.packages {
            let loaded_meta = loaded.packages.get(name);
            let disk_meta = merged
                .packages
                .entry(name.clone())
                .or_insert_with(|| our_meta.clone());
            *disk_meta = Self::merge_package_metadata(our_meta, loaded_meta, disk_meta);
        }

        // Packages removed since load time should be dropped, unless a concurrent
        // process re-created them (in which case the loop above kept them).
        for name in loaded
            .packages
            .keys()
            .filter(|n| !our.packages.contains_key(*n))
        {
            merged.packages.remove(name);
        }

        merged
    }

    fn save(&mut self) -> Result<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.file_path)
            .context("Failed to open registry file")?;
        file.lock().context("Failed to acquire registry lock")?;

        // Re-read the latest on-disk state so concurrent processes' changes
        // are not lost.
        let mut content = String::new();
        file.seek(SeekFrom::Start(0))?;
        file.read_to_string(&mut content)?;
        let on_disk: RegistryData = serde_json::from_str(&content).unwrap_or_default();

        let merged = Self::merge_registry_data(&self.data, &self.loaded, &on_disk);

        let mut buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        merged.serialize(&mut ser)?;

        file.seek(SeekFrom::Start(0))?;
        file.set_len(buf.len() as u64)?;
        file.write_all(&buf)?;
        file.sync_all().ok();

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
    fn test_concurrent_save_merges_installations_and_version() {
        let tmp = tempdir().unwrap();
        let project_path = Path::new("/tmp/concurrent-project");

        // Process A: publishes the package and records an installation.
        {
            let mut registry_a = make_registry(tmp.path());
            registry_a
                .update_package_version("my-lib", "1.0.0")
                .unwrap();
            registry_a
                .add_package_installation("my-lib", project_path)
                .unwrap();
        }

        // Process B (fresh in-memory registry, same home): bumps the version.
        {
            let mut registry_b = make_registry(tmp.path());
            registry_b
                .update_package_version("my-lib", "2.0.0")
                .unwrap();
        }

        // Reload and verify both the installation and the new version survived.
        let registry = Registry::with_home_dir(tmp.path()).unwrap();
        assert_eq!(registry.get_pkg_version("my-lib"), Some("2.0.0"));
        assert!(
            registry.has_installation("my-lib", project_path),
            "installation must survive a concurrent version bump"
        );
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

    fn meta(version: &str, installations: &[&str], links: &[&str]) -> PackageMetadata {
        PackageMetadata {
            version: version.to_string(),
            last_updated: format!("2024-01-0{}T00:00:00Z", version),
            installations: installations.iter().map(PathBuf::from).collect(),
            source_path: None,
            links: links.iter().map(PathBuf::from).collect(),
        }
    }

    #[test]
    fn test_merge_path_lists_concurrent_add_preserved() {
        let our = vec![PathBuf::from("/a"), PathBuf::from("/b")];
        let loaded = vec![PathBuf::from("/a")];
        // Another process added /c concurrently.
        let on_disk = vec![PathBuf::from("/a"), PathBuf::from("/c")];

        let merged = Registry::merge_path_lists(&our, &loaded, &on_disk);

        assert!(merged.contains(&PathBuf::from("/a")));
        assert!(merged.contains(&PathBuf::from("/b")), "our addition kept");
        assert!(
            merged.contains(&PathBuf::from("/c")),
            "concurrent addition kept"
        );
    }

    #[test]
    fn test_merge_path_lists_own_removal_preserved() {
        let our = vec![PathBuf::from("/a")];
        let loaded = vec![PathBuf::from("/a"), PathBuf::from("/b")];
        // /b was removed by us; another process did not touch it.
        let on_disk = vec![PathBuf::from("/a"), PathBuf::from("/b")];

        let merged = Registry::merge_path_lists(&our, &loaded, &on_disk);

        assert!(merged.contains(&PathBuf::from("/a")));
        assert!(
            !merged.contains(&PathBuf::from("/b")),
            "our removal must survive"
        );
    }

    #[test]
    fn test_merge_registry_data_newer_version_wins() {
        let our = RegistryData {
            packages: {
                let mut m = BTreeMap::new();
                m.insert("my-lib".to_string(), meta("2.0.0", &["/p"], &[]));
                m
            },
        };
        let loaded = RegistryData::default();
        let on_disk = RegistryData {
            packages: {
                let mut m = BTreeMap::new();
                m.insert("my-lib".to_string(), meta("1.5.0", &["/p", "/q"], &[]));
                m
            },
        };

        let merged = Registry::merge_registry_data(&our, &loaded, &on_disk);
        let m = merged.packages.get("my-lib").unwrap();

        assert_eq!(m.version, "2.0.0", "newer version wins");
        assert!(m.installations.contains(&PathBuf::from("/p")));
        assert!(
            m.installations.contains(&PathBuf::from("/q")),
            "concurrent addition /q preserved"
        );
    }

    #[test]
    fn test_merge_registry_data_whole_package_removal() {
        let our = RegistryData::default();
        let loaded = RegistryData {
            packages: {
                let mut m = BTreeMap::new();
                m.insert("old-lib".to_string(), meta("1.0.0", &["/p"], &[]));
                m
            },
        };
        let on_disk = RegistryData {
            packages: {
                let mut m = BTreeMap::new();
                m.insert("old-lib".to_string(), meta("1.0.0", &["/p"], &[]));
                m
            },
        };

        let merged = Registry::merge_registry_data(&our, &loaded, &on_disk);
        assert!(
            !merged.packages.contains_key("old-lib"),
            "whole-package removal must be applied"
        );
    }
}
