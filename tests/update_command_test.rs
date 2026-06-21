use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

mod common;

#[test]
fn test_update_single_package_success() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup
    let temp_dir = tempdir()?;
    let home_dir = temp_dir.path();

    // Create a source project to be published
    let source_project_path = home_dir.join("source_project");
    fs::create_dir_all(&source_project_path)?;
    common::setup_package_json(&source_project_path, "source_project", "1.0.0")?;
    let source_file_path = source_project_path.join("index.js");
    fs::write(&source_file_path, "console.log('v1.0.0');")?;

    // Create a target project that will consume the source project
    let target_project_path = home_dir.join("target_project");
    common::setup_kley_and_project(&target_project_path, "target_project", "1.0.0")?;

    // 2. Publish the initial version of the source project
    let mut cmd = Command::cargo_bin("kley")?;
    cmd.arg("publish")
        .current_dir(&source_project_path)
        .env("KLEY_HOME", home_dir);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Done: source_project published"));

    // 3. Add the source project to the target project
    let mut cmd = Command::cargo_bin("kley")?;
    cmd.arg("add")
        .arg("source_project")
        .current_dir(&target_project_path)
        .env("KLEY_HOME", home_dir);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Done: source_project added"));

    // Verify initial version is installed
    let kley_path = target_project_path.join(".kley/source_project/index.js");
    assert_eq!(fs::read_to_string(&kley_path)?, "console.log('v1.0.0');");

    // 4. Update the source project
    fs::write(&source_file_path, "console.log('v1.1.0');")?;
    common::setup_package_json(&source_project_path, "source_project", "1.1.0")?;

    // 5. Re-publish the updated source project
    let mut cmd = Command::cargo_bin("kley")?;
    cmd.arg("publish")
        .current_dir(&source_project_path)
        .env("KLEY_HOME", home_dir);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Done: source_project published"));

    // 6. Run `kley update` in the target project
    let mut cmd = Command::cargo_bin("kley")?;
    cmd.arg("update")
        .arg("source_project")
        .current_dir(&target_project_path)
        .env("KLEY_HOME", home_dir);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("source_project"))
        .stdout(predicate::str::contains("Done: packages were updated"));

    // 7. Assert that the package was updated
    assert_eq!(fs::read_to_string(&kley_path)?, "console.log('v1.1.0');");

    let kley_pkg_json_path = target_project_path.join(".kley/source_project/package.json");
    let pkg_json_content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(kley_pkg_json_path)?)?;
    assert_eq!(pkg_json_content["version"], "1.1.0");

    Ok(())
}

/// `kley update` must skip linked packages — the source is live, no copy exists.
#[test]
fn test_update_skips_linked_package_e2e() -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = tempdir()?;
    let lib_dir = tempdir()?;
    let app_dir = tempdir()?;

    // Setup and publish lib at v1.0.0
    fs::write(
        lib_dir.path().join("package.json"),
        r#"{"name": "test-lib", "version": "1.0.0"}"#,
    )?;
    fs::write(lib_dir.path().join("index.js"), "// v1")?;
    fs::write(
        app_dir.path().join("package.json"),
        r#"{"name": "app", "version": "1.0.0"}"#,
    )?;

    Command::cargo_bin("kley")?
        .env("KLEY_HOME", home_dir.path())
        .arg("publish")
        .current_dir(lib_dir.path())
        .assert()
        .success();

    // Link in app
    Command::cargo_bin("kley")?
        .env("KLEY_HOME", home_dir.path())
        .args(["link", "test-lib"])
        .current_dir(app_dir.path())
        .assert()
        .success();

    // Publish v2.0.0 to registry (updates the registry copy)
    fs::write(
        lib_dir.path().join("package.json"),
        r#"{"name": "test-lib", "version": "2.0.0"}"#,
    )?;
    fs::write(lib_dir.path().join("index.js"), "// v2")?;

    Command::cargo_bin("kley")?
        .env("KLEY_HOME", home_dir.path())
        .arg("publish")
        .current_dir(lib_dir.path())
        .assert()
        .success();

    // Run `kley update` — should skip test-lib because it is linked
    Command::cargo_bin("kley")?
        .env("KLEY_HOME", home_dir.path())
        .args(["update", "test-lib"])
        .current_dir(app_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Skipping test-lib: linked (source is live)",
        ));

    // kley.lock still shows the link entry (connection:link, version may be old)
    let lock: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(app_dir.path().join("kley.lock"))?)?;
    assert_eq!(
        lock["packages"]["test-lib"]["connection"], "link",
        "kley.lock should retain connection:link after skipped update"
    );

    // The symlink still points to lib source — no .kley copy was created
    assert!(
        !app_dir.path().join(".kley/test-lib").exists(),
        "update should not create .kley copy for linked package"
    );

    Ok(())
}
