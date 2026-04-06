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
        .env("HOME", home_dir);
    cmd.assert().success().stdout(predicate::str::contains(
        "Package successfully published to store!",
    ));

    // 3. Add the source project to the target project
    let mut cmd = Command::cargo_bin("kley")?;
    cmd.arg("add")
        .arg("source_project")
        .current_dir(&target_project_path)
        .env("HOME", home_dir);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("package.json has been updated!"));

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
        .env("HOME", home_dir);
    cmd.assert().success().stdout(predicate::str::contains(
        "Package successfully published to store!",
    ));

    // 6. Run `kley update` in the target project
    let mut cmd = Command::cargo_bin("kley")?;
    cmd.arg("update")
        .arg("source_project")
        .current_dir(&target_project_path)
        .env("HOME", home_dir);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Updated source_project"));

    // 7. Assert that the package was updated
    assert_eq!(fs::read_to_string(&kley_path)?, "console.log('v1.1.0');");

    let kley_pkg_json_path = target_project_path.join(".kley/source_project/package.json");
    let pkg_json_content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(kley_pkg_json_path)?)?;
    assert_eq!(pkg_json_content["version"], "1.1.0");

    Ok(())
}
