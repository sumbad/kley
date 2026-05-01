use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_add_package_with_version_syntax() -> Result<(), Box<dyn std::error::Error>> {
    // 1. SETUP: Create a temporary home directory and a library project
    let temp_dir = tempdir()?;
    let home_dir = temp_dir.path();

    let lib_dir = temp_dir.path().join("test-lib");
    fs::create_dir(&lib_dir)?;
    fs::write(
        lib_dir.join("package.json"),
        r#"{"name": "test-lib", "version": "1.0.0"}"#,
    )?;

    // 2. PUBLISH: Publish the library to the local kley registry
    Command::cargo_bin("kley")?
        .current_dir(&lib_dir)
        .env("KLEY_HOME", home_dir)
        .arg("publish")
        .assert()
        .success();

    // 3. SETUP: Create a consumer app project
    let app_dir = temp_dir.path().join("test-app");
    fs::create_dir(&app_dir)?;
    fs::write(app_dir.join("package.json"), r#"{"name": "test-app"}"#)?;

    // 4. EXECUTE & ASSERT: Try to add the package using "name@version" syntax
    Command::cargo_bin("kley")?
        .current_dir(&app_dir)
        .env("KLEY_HOME", home_dir)
        .arg("add")
        .arg("test-lib@1.0.0") // Using the version syntax
        .assert()
        .success() // This assertion will fail
        .stdout(predicate::str::contains("Done: test-lib added"));

    // 5. VERIFY: Check if package.json was updated correctly
    let pkg_json_content = fs::read_to_string(app_dir.join("package.json"))?;
    let pkg_json: serde_json::Value = serde_json::from_str(&pkg_json_content)?;

    assert_eq!(pkg_json["dependencies"]["test-lib"], "file:.kley/test-lib");

    Ok(())
}
