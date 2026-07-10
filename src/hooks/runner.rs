use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

use crate::hooks::registry::{HookPhase, HooksConfig};

/// Execute every hook of `phase` via the system shell, in `KNOWN_HOOKS` order.
///
/// Any hook exiting with a non-zero status aborts immediately: a failing
/// `Pre` hook means files are never copied; a failing `Post` hook means the
/// files are already copied but `publish` still reports failure.
pub fn run_phase(config: &HooksConfig, phase: HookPhase, cwd: &Path) -> Result<()> {
    for (name, entry) in config.hooks_for_phase(phase) {
        println!("  ↳ running {name} hook: {}", entry.command);

        let status = if cfg!(windows) {
            Command::new("cmd")
                .args(["/C", entry.command.as_str()])
                .current_dir(cwd)
                .status()
        } else {
            Command::new("sh")
                .args(["-c", entry.command.as_str()])
                .current_dir(cwd)
                .status()
        }
        .with_context(|| format!("Failed to spawn hook '{name}'"))?;

        if !status.success() {
            bail!("Hook '{name}' failed with exit code {:?}", status.code());
        }
    }

    Ok(())
}
