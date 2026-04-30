use assert_cmd::prelude::*;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::tempdir;

mod common;

#[test]
fn test_unpublish_soft() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup
    let temp_dir = tempdir()?;
    let home_dir = temp_dir.path();

    // Create source and target projects
    let source_project_path = home_dir.join("source_project");
    common::setup_kley_and_project(&source_project_path, "source_project", "1.0.0")?;

    let target_project_path = home_dir.join("target_project");
    common::setup_kley_and_project(&target_project_path, "target_project", "1.0.0")?;

    // Publish and add the package
    Command::cargo_bin("kley")?
        .current_dir(&source_project_path)
        .env("KLEY_HOME", home_dir)
        .arg("publish")
        .assert()
        .success();

    Command::cargo_bin("kley")?
        .current_dir(&target_project_path)
        .env("KLEY_HOME", home_dir)
        .arg("add")
        .arg("source_project")
        .assert()
        .success();

    // 2. Execute `unpublish` with "y" piped to stdin
    let mut cmd = Command::cargo_bin("kley")?;
    let mut child = cmd
        .current_dir(&source_project_path)
        .env("KLEY_HOME", home_dir)
        .arg("unpublish")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"y\n")?;
    }

    let output = child.wait_with_output()?;
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)?
        .contains("Warning: 'source_project' is used by 1 project."));

    // 3. Assertions
    // Global store should be clean
    let store_path = home_dir.join(".kley/packages/source_project");
    assert!(!store_path.exists(), "Package should be removed from store");

    // Global registry should not contain the package
    let registry_path = home_dir.join(".kley/registry.json");
    let registry_content: Value = serde_json::from_str(&fs::read_to_string(registry_path)?)?;
    assert!(registry_content["packages"]["source_project"].is_null());

    // Target project should NOT be cleaned
    let target_kley_path = target_project_path.join(".kley/source_project");
    assert!(
        target_kley_path.exists(),
        "Target project's .kley dir should not be cleaned"
    );

    Ok(())
}

#[test]
fn test_unpublish_hard_push() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup
    let temp_dir = tempdir()?;
    let home_dir = temp_dir.path();

    // Create two source projects
    let source_a_path = home_dir.join("source_a");
    common::setup_kley_and_project(&source_a_path, "source_a", "1.0.0")?;
    let source_b_path = home_dir.join("source_b");
    common::setup_kley_and_project(&source_b_path, "source_b", "1.0.0")?;

    // Create two target projects
    let target_1_path = home_dir.join("target_1");
    common::setup_kley_and_project(&target_1_path, "target_1", "1.0.0")?;
    let target_2_path = home_dir.join("target_2");
    common::setup_kley_and_project(&target_2_path, "target_2", "1.0.0")?;

    // Publish both source projects
    Command::cargo_bin("kley")?
        .current_dir(&source_a_path)
        .env("KLEY_HOME", home_dir)
        .arg("publish")
        .assert()
        .success();
    Command::cargo_bin("kley")?
        .current_dir(&source_b_path)
        .env("KLEY_HOME", home_dir)
        .arg("publish")
        .assert()
        .success();

    // Add A and B to target_1
    Command::cargo_bin("kley")?
        .current_dir(&target_1_path)
        .env("KLEY_HOME", home_dir)
        .args(["add", "source_a"])
        .assert()
        .success();
    Command::cargo_bin("kley")?
        .current_dir(&target_1_path)
        .env("KLEY_HOME", home_dir)
        .args(["add", "source_b"])
        .assert()
        .success();

    // Add A to target_2
    Command::cargo_bin("kley")?
        .current_dir(&target_2_path)
        .env("KLEY_HOME", home_dir)
        .args(["add", "source_a"])
        .assert()
        .success();

    // 2. Execute `unpublish --push` for source_a
    let mut cmd = Command::cargo_bin("kley")?;
    let mut child = cmd
        .current_dir(&source_a_path)
        .env("KLEY_HOME", home_dir)
        .args(["unpublish", "--push"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"y\n")?;
    }

    let output = child.wait_with_output()?;
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)?.contains(
        "This will permanently remove 'source_a' from the registry AND from the following 2 projects"
    ));

    // 3. Assertions
    // Global store
    assert!(
        !home_dir.join(".kley/packages/source_a").exists(),
        "source_a should be removed from store"
    );
    assert!(
        home_dir.join(".kley/packages/source_b").exists(),
        "source_b should remain in store"
    );

    // Global registry
    let registry_content: Value =
        serde_json::from_str(&fs::read_to_string(home_dir.join(".kley/registry.json"))?)?;
    assert!(
        registry_content["packages"]["source_a"].is_null(),
        "source_a should be removed from registry"
    );
    assert!(
        !registry_content["packages"]["source_b"].is_null(),
        "source_b should remain in registry"
    );

    // Target 1
    assert!(
        !target_1_path.join(".kley/source_a").exists(),
        "source_a should be removed from target_1's .kley"
    );
    assert!(
        target_1_path.join(".kley/source_b").exists(),
        "source_b should remain in target_1's .kley"
    );
    let target_1_pkg: Value =
        serde_json::from_str(&fs::read_to_string(target_1_path.join("package.json"))?)?;
    assert!(
        target_1_pkg["dependencies"]["source_a"].is_null(),
        "source_a should be removed from target_1's package.json"
    );
    assert!(
        !target_1_pkg["dependencies"]["source_b"].is_null(),
        "source_b should remain in target_1's package.json"
    );

    // Target 2
    assert!(
        !target_2_path.join(".kley/source_a").exists(),
        "source_a should be removed from target_2's .kley"
    );
    let target_2_pkg: Value =
        serde_json::from_str(&fs::read_to_string(target_2_path.join("package.json"))?)?;
    assert!(
        target_2_pkg["dependencies"]["source_a"].is_null(),
        "source_a should be removed from target_2's package.json"
    );

    Ok(())
}
