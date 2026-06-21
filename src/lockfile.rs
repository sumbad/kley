use anyhow::Result;

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    #[default]
    Install,
    Link,
}

impl ConnectionType {
    fn is_install(&self) -> bool {
        self == &ConnectionType::Install
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageInfo {
    pub version: String,
    /// Snapshot of dependencies at last PM install, used for fast reinstall check
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, String>,
    /// Snapshot of peer dependencies at last PM install, used for fast reinstall check
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub peer_dependencies: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "ConnectionType::is_install")]
    pub connection: ConnectionType,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Lockfile {
    #[serde(default)]
    pub packages: BTreeMap<String, PackageInfo>,
    pub package_manager: Option<String>,
}

impl Lockfile {
    /// Read existing kley.lock or create a new one
    pub fn new(dir: &Path) -> Lockfile {
        let Some(lockfile) = Lockfile::get(dir) else {
            return Lockfile::default();
        };

        lockfile
    }

    pub fn get(dir: &Path) -> Option<Lockfile> {
        let lock_path = Lockfile::path(dir);

        if !lock_path.exists() {
            tracing::info!("Lockfile does not exist");

            return None;
        }

        let content = match fs::read_to_string(&lock_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to read kley.lock: {e}");

                return None;
            }
        };

        if content.trim().is_empty() {
            return None;
        }

        match serde_json::from_str(&content) {
            Ok(lf) => Some(lf),
            Err(e) => {
                tracing::warn!("Failed to parse kley.lock: {e}");
                None
            }
        }
    }

    pub fn save(&mut self, dir: &Path) -> Result<()> {
        // Write back to kley.lock
        let mut buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        self.serialize(&mut ser)?;

        fs::write(Lockfile::path(dir), buf)?;

        tracing::info!("kley.lock has been updated");

        Ok(())
    }

    fn path(dir: &Path) -> PathBuf {
        dir.join("kley.lock")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_connection_defaults_to_install() {
        let json = r#"{"packages":{"my-pkg":{"version":"1.0.0"}}}"#;
        let lockfile: Lockfile = serde_json::from_str(json).unwrap();
        let info = lockfile.packages.get("my-pkg").unwrap();
        assert_eq!(info.connection, ConnectionType::Install);
    }

    #[test]
    fn test_connection_link_roundtrip() {
        let pkg_info = PackageInfo {
            version: "1.0.0".to_string(),
            dependencies: BTreeMap::new(),
            peer_dependencies: BTreeMap::new(),
            connection: ConnectionType::Link,
        };

        let json = serde_json::to_string(&pkg_info).unwrap();
        assert!(
            json.contains(r#""connection":"link""#),
            "JSON should contain connection:link. Got: {}",
            json
        );

        let deserialized: PackageInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.connection, ConnectionType::Link);
    }

    #[test]
    fn test_connection_install_skip_serializing() {
        let pkg_info = PackageInfo {
            version: "1.0.0".to_string(),
            dependencies: BTreeMap::new(),
            peer_dependencies: BTreeMap::new(),
            connection: ConnectionType::Install,
        };

        let json = serde_json::to_string(&pkg_info).unwrap();
        assert!(
            !json.contains("connection"),
            "ConnectionType::Install should be omitted from JSON. Got: {}",
            json
        );
    }

    #[test]
    fn test_get_connection() {
        let tmp = tempdir().unwrap();
        let lock_content = r#"{"packages":{"my-lib":{"version":"1.0.0","connection":"link"}}}"#;
        std::fs::write(tmp.path().join("kley.lock"), lock_content).unwrap();

        let lockfile = Lockfile::get(tmp.path()).unwrap();
        let info = lockfile.packages.get("my-lib").unwrap();
        assert_eq!(info.connection, ConnectionType::Link);
    }
}
