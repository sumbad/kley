use anyhow::{Context, Result};
use colored::*;
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use tracing;

#[derive(Deserialize, Debug)]
struct PackageJson {
    name: String,
    version: String,
}

/// Publish logic
pub fn publish() -> Result<()> {
    // 1. Read package.json
    let pkg_path = Path::new("package.json");
    if !pkg_path.exists() {
        anyhow::bail!("package.json not found in the current directory");
    }

    let pkg_content = fs::read_to_string(pkg_path)?;
    let pkg: PackageJson =
        serde_json::from_str(&pkg_content).context("Failed to parse package.json")?;

    tracing::info!(
        "🚀 Publishing {}@{}...",
        pkg.name.cyan(),
        pkg.version.magenta()
    );

    // 2. Determine the path in the store (~/.kley/packages/name)
    let home_dir = dirs::home_dir().context("Failed to find home directory")?;
    let store_path = home_dir.join(".kley").join("packages").join(&pkg.name);

    if store_path.exists() {
        fs::remove_dir_all(&store_path)?;
    }
    fs::create_dir_all(&store_path)?;

    // Apply npm built-in rules via OverrideBuilder
    let mut override_builder = OverrideBuilder::new(Path::new("."));
    override_builder.add("!node_modules/")?;
    override_builder.add("!*.swp")?;
    override_builder.add("!._*")?;
    override_builder.add("!.DS_Store")?;
    override_builder.add("!npm-debug.log*")?;

    let walk_with_ignores = WalkBuilder::new(".")
        .hidden(false)
        .git_ignore(!Path::new(".npmignore").exists()) // Correctly use .gitignore as a fallback
        .add_custom_ignore_filename(".npmignore")
        .add_custom_ignore_filename(".kleyignore")
        .overrides(override_builder.build()?)
        .build();

    for entry in walk_with_ignores {
        let entry = entry?;
        let path = entry.path();

        if path == Path::new(".") {
            continue;
        }

        tracing::debug!(path = %path.to_string_lossy(), "Packing entry");

        let relative_path = path.strip_prefix(".")?;
        let target_path = store_path.join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&target_path)?;
        } else {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &target_path)?;
        }
    }

    tracing::info!("{}", "✅ Package successfully published to store!".green());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    /// Helper to create a dummy project structure inside a temporary directory.
    fn setup_test_project(dir: &Path) -> Result<()> {
        fs::create_dir_all(dir.join(".git"))?; // Trick the `ignore` crate into thinking this is a git repo
        fs::write(
            dir.join("package.json"),
            r#"{"name": "test-pkg", "version": "1.0.0"}"#,
        )?;
        fs::write(dir.join("index.js"), "console.log('hello');")?;
        fs::create_dir_all(dir.join("dist"))?;
        fs::write(dir.join("dist/bundle.js"), "/* bundle */")?;
        fs::write(dir.join("secret.log"), "sensitive data")?;
        // This file should also be ignored by default git rules
        fs::create_dir_all(dir.join("node_modules/some_dep"))?;
        fs::write(dir.join("node_modules/some_dep/index.js"), "...")?;
        Ok(())
    }

    #[test]
    fn test_publish_filtering_logic() -> Result<()> {
        let original_dir = std::env::current_dir()?;
        let home_dir = dirs::home_dir().context("Failed to find home directory")?;
        let store_path = home_dir.join(".kley/packages/test-pkg");

        // --- SCENARIO 1: .npmignore exists ---
        {
            let proj_dir = tempdir()?;
            let proj_path = proj_dir.path();
            setup_test_project(proj_path)?;
            fs::write(
                proj_path.join(".gitignore"),
                "dist\nsecret.log\nnode_modules",
            )?;
            fs::write(proj_path.join(".npmignore"), "secret.log")?;

            std::env::set_current_dir(proj_path)?;
            publish()?;

            // Assert: build artifact IS included, secret IS NOT, node_modules IS NOT
            assert!(
                store_path.join("dist/bundle.js").exists(),
                "Scenario 1: dist/bundle.js should exist"
            );
            assert!(
                !store_path.join("secret.log").exists(),
                "Scenario 1: secret.log should NOT exist"
            );
            assert!(
                !store_path.join("node_modules").exists(),
                "Scenario 1: node_modules should NOT exist"
            );

            // Cleanup for next scenario
            fs::remove_dir_all(&store_path)?;
        }

        // --- SCENARIO 2: Only .gitignore exists ---
        {
            let proj_dir = tempdir()?;
            let proj_path = proj_dir.path();
            setup_test_project(proj_path)?;
            fs::write(
                proj_path.join(".gitignore"),
                "dist\nsecret.log\nnode_modules",
            )?;

            std::env::set_current_dir(proj_path)?;
            publish()?;

            // Assert: build artifact IS NOT included, secret IS NOT, node_modules IS NOT
            assert!(
                !store_path.join("dist/bundle.js").exists(),
                "Scenario 2: dist/bundle.js should NOT exist"
            );
            assert!(
                !store_path.join("secret.log").exists(),
                "Scenario 2: secret.log should NOT exist"
            );
            assert!(
                !store_path.join("node_modules").exists(),
                "Scenario 2: node_modules should NOT exist"
            );
        }

        // --- Final Cleanup ---
        std::env::set_current_dir(original_dir)?;
        if store_path.exists() {
            fs::remove_dir_all(&store_path)?;
        }

        Ok(())
    }
}
