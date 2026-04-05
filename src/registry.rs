use anyhow::{Context, Ok, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::utils::current_formatted_time;

pub static REGISTRY_DIR_NAME: &str = ".kley";
pub static REGISTRY_FILE_NAME: &str = "registry.json";

#[derive(Serialize, Deserialize, Debug, Default)]
struct RegistryData {
    #[serde(default)]
    packages: BTreeMap<String, PackageMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PackageMetadata {
    version: String,
    last_updated: String,
    installations: Vec<PathBuf>,
}

pub struct Registry {
    data: RegistryData,
    pub dir_path: PathBuf,
    pub file_path: PathBuf,
}

impl Registry {
    pub fn new(home_dir: PathBuf) -> Result<Registry> {
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
        self.data.packages.get(package_name).map(|it| it.version.as_str())
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

        println!("{}", "Updated registry has been saved!".green());

        Ok(())
    }
}
