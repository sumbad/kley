use anyhow::Result;

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageInfo {
    pub version: String,
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
        let lock_path = dir.join("kley.lock");

        lock_path
    }
}
