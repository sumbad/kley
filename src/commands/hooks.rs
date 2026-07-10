use std::io::IsTerminal;
use std::path::Path;

use anyhow::Result;
use colored::*;

use crate::hooks::registry::{HooksConfig, HookPhase, KNOWN_HOOKS};

/// Print the current `.kley/hooks.json` in a readable form.
pub fn list(repo_root: &Path) -> Result<()> {
    let hooks_path = repo_root.join(".kley").join("hooks.json");

    if !hooks_path.exists() {
        println!(
            "{} No .kley/hooks.json found. Run `kley publish` to configure publish hooks.",
            "ℹ".yellow()
        );
        return Ok(());
    }

    let config = HooksConfig::load(&hooks_path)?;

    if config.hooks.is_empty() {
        println!("{} .kley/hooks.json exists but no hooks are configured.", "ℹ".yellow());
        return Ok(());
    }

    println!("Configured publish hooks ({}):", hooks_path.display());
    for (name, entry) in &config.hooks {
        let phase = KNOWN_HOOKS
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, p)| match p {
                HookPhase::Pre => "PRE ",
                HookPhase::Post => "POST",
            })
            .unwrap_or("?   ");
        println!(
            "  [{}] {} -> {}",
            phase,
            name.cyan(),
            entry.command
        );
    }

    Ok(())
}

/// Reconfigure `.kley/hooks.json`.
///
/// In a non-interactive context (no TTY, e.g. CI or tests) the existing file
/// is preserved as-is, so manually-added hooks absent from `package.json`
/// (Defect #2) are never wiped.
pub fn edit(repo_root: &Path) -> Result<()> {
    let hooks_path = repo_root.join(".kley").join("hooks.json");

    if !std::io::stdin().is_terminal() {
        if hooks_path.exists() {
            println!(
                "{} Non-interactive mode: keeping existing .kley/hooks.json unchanged.",
                "ℹ".yellow()
            );
        } else {
            println!(
                "{} Non-interactive mode: no .kley/hooks.json to edit.",
                "ℹ".yellow()
            );
        }
        return Ok(());
    }

    // Interactive: remember pre-existing entries, re-run wizard, then merge
    // back any manual entries not present in package.json scripts.
    let existing = if hooks_path.exists() {
        HooksConfig::load(&hooks_path).ok()
    } else {
        None
    };

    let mut config = crate::hooks::wizard::run_hooks_wizard(repo_root, &hooks_path)?;

    if let Some(existing) = existing {
        for (name, entry) in existing.hooks {
            config.hooks.entry(name).or_insert(entry);
        }
        config.save(&hooks_path)?;
    }

    println!("{} Updated .kley/hooks.json.", "✔".green());
    Ok(())
}
