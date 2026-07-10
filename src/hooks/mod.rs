pub mod registry;
pub mod runner;
pub mod wizard;

use std::path::Path;

use anyhow::Result;

use crate::hooks::registry::HooksConfig;

/// Resolve the hooks configuration for a `kley publish` invocation.
///
/// Flow (see `docs/kley-publish-hooks-spec.md`):
/// 1. `--no-hooks`      -> empty config (pure copy), always.
/// 2. `.kley/hooks.json` exists -> load it (wizard already ran).
/// 3. no file + interactive   -> run the wizard (writes the file).
/// 4. no file + non-interactive -> empty config (pure copy).
pub fn load_hooks_config(
    repo_root: &Path,
    non_interactive: bool,
    no_hooks: bool,
) -> Result<HooksConfig> {
    if no_hooks {
        return Ok(HooksConfig::default());
    }

    let hooks_path = repo_root.join(".kley").join("hooks.json");

    if hooks_path.exists() {
        return HooksConfig::load(&hooks_path);
    }

    if !non_interactive {
        return wizard::run_hooks_wizard(repo_root, &hooks_path);
    }

    Ok(HooksConfig::default())
}
