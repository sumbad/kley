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
        .env("KLEY_HOME", home_dir)
        .arg("publish")
        .assert()
        .success();

    // app-a installs my-lib (recorded in installations[])
    Command::cargo_bin("kley")?
        .current_dir(&app_a_dir)
        .env("KLEY_HOME", home_dir)
        .arg("add")
        .arg("my-lib")
        .assert()
        .success();

    // app-b links my-lib (recorded in links[], NOT installations[])
    Command::cargo_bin("kley")?
        .current_dir(&app_b_dir)
        .env("KLEY_HOME", home_dir)
        .arg("link")
        .arg("my-lib")
        .assert()
        .success();

    // Verify initial state
    // app-a: .kley copy exists (installed)
    let app_a_content = fs::read_to_string(app_a_dir.join(".kley/my-lib/index.js"))?;
    assert_eq!(app_a_content, "module.exports = 'v1';");

    // app-b: no .kley copy, only symlink to source
    assert!(
        !app_b_dir.join(".kley/my-lib").exists(),
        "app-b uses link — no .kley copy should exist"
    );
    assert!(
        app_b_dir.join("node_modules/my-lib").is_symlink(),
        "app-b should have a direct symlink in node_modules"
    );

    // 3. Update library and run publish --push
    fs::write(
        lib_dir.join("package.json"),
        r#"{"name": "my-lib", "version": "1.1.0"}"#,
    )?;
    fs::write(lib_dir.join("index.js"), "module.exports = 'v2';")?;

    Command::cargo_bin("kley")?
        .current_dir(&lib_dir)
        .env("KLEY_HOME", home_dir)
        .args(["publish", "--push"])
        .assert()
        .success()
        // Only 1 installation (app-a); app-b is in links[] and is skipped
        .stdout(predicate::str::contains("Pushing my-lib to 1 project"))
        .stdout(predicate::str::contains(format!(
            "Updated {} to the latest version of my-lib",
            normalized_path(&app_a_dir, Some(&home_dir.to_path_buf()))
        )))
        // app-b is linked — skipped, not updated
        .stdout(predicate::str::contains(format!(
            "Skipped {}: my-lib is linked (source is live)",
            normalized_path(&app_b_dir, Some(&home_dir.to_path_buf()))
        )));

    // 4. Assert final state
    let registry_content: Value =
        serde_json::from_str(&fs::read_to_string(home_dir.join(".kley/registry.json"))?)?;
    assert_eq!(registry_content["packages"]["my-lib"]["version"], "1.1.0");

    // app-a (installed): .kley copy updated to v2
    let app_a_new_content = fs::read_to_string(app_a_dir.join(".kley/my-lib/index.js"))?;
    assert_eq!(app_a_new_content, "module.exports = 'v2';");
    let app_a_lock: Value =
        serde_json::from_str(&fs::read_to_string(app_a_dir.join("kley.lock"))?)?;
    assert_eq!(app_a_lock["packages"]["my-lib"]["version"], "1.1.0");

    // app-b (linked): symlink still points to lib source, which now has v2 live
    let app_b_live_content = fs::read_to_string(app_b_dir.join("node_modules/my-lib/index.js"))?;
    assert_eq!(
        app_b_live_content, "module.exports = 'v2';",
        "linked app-b sees v2 immediately via symlink without push"
    );

    Ok(())
}

#[test]
fn test_publish_records_source_path() -> Result<(), Box<dyn std::error::Error>> {
    let home = tempdir()?;
    let lib = tempdir()?;

    fs::write(
        lib.path().join("package.json"),
        r#"{"name": "my-lib", "version": "1.0.0"}"#,
    )?;

    Command::cargo_bin("kley")?
        .current_dir(lib.path())
        .env("KLEY_HOME", home.path())
        .arg("publish")
        .assert()
        .success();

    let registry: Value = serde_json::from_str(&fs::read_to_string(
        home.path().join(".kley/registry.json"),
    )?)?;

    assert!(
        registry["packages"]["my-lib"]["sourcePath"].is_string(),
        "publish should record sourcePath in registry"
    );

    let recorded = registry["packages"]["my-lib"]["sourcePath"]
        .as_str()
        .unwrap();
    let recorded_canon = fs::canonicalize(recorded)?;
    let expected_canon = fs::canonicalize(lib.path())?;
    assert_eq!(
        recorded_canon, expected_canon,
        "sourcePath should equal the directory where publish was run"
    );

    Ok(())
}
