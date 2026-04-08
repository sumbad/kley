use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

use kley::utils::normalized_path;

#[test]
fn test_publish_push_flow() -> Result<(), Box<dyn std::error::Error>> {
    // 1. SETUP
    let temp_dir = tempdir()?;
    let home_dir = temp_dir.path();

    // Create library project
    let lib_dir = temp_dir.path().join("my-lib");
    fs::create_dir(&lib_dir)?;
    fs::write(
        lib_dir.join("package.json"),
        r#"{"name": "my-lib", "version": "1.0.0"}"#,
    )?;
    fs::write(lib_dir.join("index.js"), "module.exports = 'v1';")?;

    // Create consumer apps
    let app_a_dir = temp_dir.path().join("app-a");
    fs::create_dir(&app_a_dir)?;
    fs::write(app_a_dir.join("package.json"), r#"{"name": "app-a"}"#)?;

    let app_b_dir = temp_dir.path().join("app-b");
    fs::create_dir(&app_b_dir)?;
    fs::write(app_b_dir.join("package.json"), r#"{"name": "app-b"}"#)?;

    // 2. Initial publish and setup
    Command::cargo_bin("kley")?
        .current_dir(&lib_dir)
        .env("HOME", home_dir)
        .arg("publish")
        .assert()
        .success();

    // Add to app-a
    Command::cargo_bin("kley")?
        .current_dir(&app_a_dir)
        .env("HOME", home_dir)
        .arg("add")
        .arg("my-lib")
        .assert()
        .success();

    // Link to app-b
    Command::cargo_bin("kley")?
        .current_dir(&app_b_dir)
        .env("HOME", home_dir)
        .arg("link")
        .arg("my-lib")
        .assert()
        .success();

    // Verify initial state
    let app_a_content = fs::read_to_string(app_a_dir.join(".kley/my-lib/index.js"))?;
    assert_eq!(app_a_content, "module.exports = 'v1';");
    let app_b_content = fs::read_to_string(app_b_dir.join(".kley/my-lib/index.js"))?;
    assert_eq!(app_b_content, "module.exports = 'v1';");

    // 3. Update library and run publish --push
    fs::write(
        lib_dir.join("package.json"),
        r#"{"name": "my-lib", "version": "1.1.0"}"#,
    )?;
    fs::write(lib_dir.join("index.js"), "module.exports = 'v2';")?;

    Command::cargo_bin("kley")?
        .current_dir(&lib_dir)
        .env("HOME", home_dir)
        .args(["publish", "--push"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pushing my-lib to 2 projects"))
        .stdout(predicate::str::contains(format!("Updated {} to the latest version of my-lib", normalized_path(&app_a_dir))))
        .stdout(predicate::str::contains(format!("Updated {} to the latest version of my-lib", normalized_path(&app_b_dir))));


    // 4. Assert final state
    // Check registry
    let registry_content: Value =
        serde_json::from_str(&fs::read_to_string(home_dir.join(".kley/registry.json"))?)?;
    assert_eq!(registry_content["packages"]["my-lib"]["version"], "1.1.0");

    // Check app-a (added)
    let app_a_new_content = fs::read_to_string(app_a_dir.join(".kley/my-lib/index.js"))?;
    assert_eq!(app_a_new_content, "module.exports = 'v2';");
    let app_a_lock: Value =
        serde_json::from_str(&fs::read_to_string(app_a_dir.join("kley.lock"))?)?;
    assert_eq!(app_a_lock["packages"]["my-lib"]["version"], "1.1.0");

    // Check app-b (linked)
    let app_b_new_content = fs::read_to_string(app_b_dir.join(".kley/my-lib/index.js"))?;
    assert_eq!(app_b_new_content, "module.exports = 'v2';");

    Ok(())
}
