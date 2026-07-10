use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookPhase {
    Pre,
    Post,
}

/// npm lifecycle hooks kley is willing to run, paired with their phase.
/// Order matches the real npm lifecycle order; iteration order here defines
/// execution order (not JSON key order in `.kley/hooks.json`).
pub const KNOWN_HOOKS: &[(&str, HookPhase)] = &[
    ("prepare", HookPhase::Pre),
    ("prepack", HookPhase::Pre),
    ("prepublishOnly", HookPhase::Pre),
    ("postpack", HookPhase::Post),
    ("publish", HookPhase::Post),
    ("postpublish", HookPhase::Post),
];

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HookEntry {
    pub command: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct HooksConfig {
    #[serde(flatten)]
    pub hooks: HashMap<String, HookEntry>,
}

impl HooksConfig {
    pub fn load(path: &Path) -> Result<HooksConfig> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read hooks config {}", path.display()))?;
        let cfg: HooksConfig = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse hooks config {}", path.display()))?;
        Ok(cfg)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        self.serialize(&mut ser)?;
        let mut file = fs::File::create(path)?;
        file.write_all(&buf)?;
        Ok(())
    }

    /// Hooks belonging to `phase`, in `KNOWN_HOOKS` declaration order.
    pub fn hooks_for_phase(&self, phase: HookPhase) -> Vec<(&str, &HookEntry)> {
        KNOWN_HOOKS
            .iter()
            .filter_map(|(name, p)| {
                if *p == phase {
                    self.hooks.get(*name).map(|entry| (*name, entry))
                } else {
                    None
                }
            })
            .collect()
    }
}
