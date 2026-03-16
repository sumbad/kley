use anyhow::Result;
use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_add_command_preserves_order() -> Result<()> {
    let proj_dir = tempdir()?;
    let proj_path = proj_dir.path();
    let package_json_path = proj_path.join("package.json");

    // The initial content has a specific, non-alphabetical order
    let initial_content = r#"{
  "name": "test-pkg",
  "version": "1.0.0",
  "description": "A test package",
  "dependencies": {}
}"#;

    fs::write(&package_json_path, initial_content)?;

    let mut cmd = Command::cargo_bin("kley")?;
    cmd.arg("add").arg("test-lib").current_dir(proj_path);

    cmd.assert().success();

    let final_content = fs::read_to_string(&package_json_path)?;

    // The expected content after adding the dependency.
    // Note that the order of the top-level keys is preserved.
    let expected_content = r#"{
  "name": "test-pkg",
  "version": "1.0.0",
  "description": "A test package",
  "dependencies": {
    "test-lib": "file:.kley/test-lib"
  }
}"#;

    // This assertion will fail if the order of keys is changed by the `add` command.
    assert_eq!(
        final_content, expected_content,
        "The order of properties in package.json should be preserved."
    );

    proj_dir.close()?;

    Ok(())
}
