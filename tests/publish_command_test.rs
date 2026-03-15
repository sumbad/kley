use anyhow::Result;
use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// Helper to create a dummy project structure inside a temporary directory.
fn setup_test_project(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir.join(".git"))?;
    fs::write(
        dir.join("package.json"),
        r#"{"name": "test-pkg-integration", "version": "1.0.0", "files": ["dist", "index.js"]}"#,
    )?;
    fs::write(dir.join("index.js"), "console.log('hello');")?;
    fs::create_dir_all(dir.join("dist"))?;
    fs::write(dir.join("dist/bundle.js"), "/* bundle */")?;
    fs::write(dir.join("secret.log"), "sensitive data")?;
    fs::create_dir_all(dir.join("node_modules/some_dep"))?;
    fs::write(dir.join("node_modules/some_dep/index.js"), "...")?;
    fs::write(dir.join(".npmignore"), "secret.log")?;
    Ok(())
}

#[test]
fn test_publish_command_e2e() -> Result<()> {
    let proj_dir = tempdir()?;
    let proj_path = proj_dir.path();
    setup_test_project(proj_path)?;

    let home_dir = dirs::home_dir().expect("Failed to find home directory");
    let store_path = home_dir.join(".kley/packages/test-pkg-integration");

    // Cleanup before the test
    if store_path.exists() {
        fs::remove_dir_all(&store_path)?;
    }

    let mut cmd = Command::cargo_bin("kley")?;
    cmd.arg("publish").current_dir(proj_path);

    cmd.assert().success().stdout(predicates::str::contains(
        "Package successfully published to store!",
    ));

    // Assert that whitelisted files ARE included
    assert!(
        store_path.join("package.json").exists(),
        "package.json should exist"
    );
    assert!(
        store_path.join("index.js").exists(),
        "index.js should exist"
    );
    assert!(
        store_path.join("dist/bundle.js").exists(),
        "dist/bundle.js should exist"
    );

    // Assert that ignored files ARE NOT included
    assert!(
        !store_path.join("secret.log").exists(),
        "secret.log should NOT exist"
    );
    assert!(!store_path.join(".git").exists(), ".git should NOT exist");
    assert!(
        !store_path.join("node_modules").exists(),
        "node_modules should NOT exist"
    );

    // Cleanup after the test
    fs::remove_dir_all(&store_path)?;
    proj_dir.close()?;

    Ok(())
}
