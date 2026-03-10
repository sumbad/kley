use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Lockfile {
    #[serde(default)]
    pub packages: BTreeMap<String, PackageInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageInfo {
    pub version: String,
}

#[derive(Deserialize, Debug)]
pub struct PackageJson {
    pub version: String,
}
