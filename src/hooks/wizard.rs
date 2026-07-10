use anyhow::Result;
use std::path::Path;

use dialoguer::{theme::ColorfulTheme, MultiSelect};

use crate::hooks::registry::{HookEntry, HooksConfig, KNOWN_HOOKS};
use crate::package::Package;

/// Show an interactive multiselect of the package's known npm lifecycle
/// scripts and persist the chosen subset to `.kley/hooks.json`.
///
/// The file is always written (even when empty `{}`), which is the marker
/// that the wizard has already run. When the package has no known hooks,
/// an empty file is written silently.
pub fn run_hooks_wizard(repo_root: &Path, hooks_path: &Path) -> Result<HooksConfig> {
    let pkg = Package::get(repo_root)?;
    let scripts = pkg.json.scripts.clone().unwrap_or_default();

    let candidates: Vec<(&str, &str)> = KNOWN_HOOKS
        .iter()
        .filter_map(|(name, _)| scripts.get(*name).map(|cmd| (*name, cmd.as_str())))
        .collect();

    let mut config = HooksConfig::default();

    if !candidates.is_empty() {
        let items: Vec<String> = candidates
            .iter()
            .map(|(name, cmd)| format!("{name} -> {cmd}"))
            .collect();

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(
                "Kley found the following npm lifecycle scripts. \
                 Which should run automatically during `kley publish`?",
            )
            .items(&items)
            .defaults(&vec![false; items.len()])
            .interact()?;

        for i in selections {
            let (name, cmd) = candidates[i];
            config
                .hooks
                .insert(name.to_string(), HookEntry { command: cmd.to_string() });
        }
    }

    config.save(hooks_path)?;
    Ok(config)
}
