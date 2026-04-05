use anyhow::Result;
use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_add_command_preserves_order() -> Result<()> {
    let temp_dir = tempdir()?;
    let home_dir = temp_dir.path();

    // Create library project
    let lib_dir = temp_dir.path().join("test-lib");
    fs::create_dir(&lib_dir)?;
    fs::write(
        lib_dir.join("package.json"),
        r#"{"name": "test-lib", "version": "1.0.0"}"#,
    )?;
    fs::write(lib_dir.join("index.js"), "module.exports = 'v1';")?;

    let mut cmd = Command::cargo_bin("kley")?;
    let assert = cmd
        .env("HOME", home_dir)
        .env("RUST_LOG", "debug")
        .current_dir(&lib_dir)
        .arg("publish")
        .assert();
    let output = assert.get_output();
    println!("STDOUT:\n{:#}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR:\n{:#}", String::from_utf8_lossy(&output.stderr));
    assert.success();

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

    let assert = cmd
        .env("HOME", home_dir)
        .env("RUST_LOG", "debug")
        .current_dir(proj_path)
        .arg("add")
        .arg("test-lib")
        .assert();
    let output = assert.get_output();
    println!("STDOUT:\n{:#}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR:\n{:#}", String::from_utf8_lossy(&output.stderr));
    assert.success();

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
