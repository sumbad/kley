use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
struct Registry {
    packages: BTreeMap<String, PackageMetadata>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackageMetadata {
    version: String,
    last_updated: String,
    installations: Vec<PathBuf>,
}

#[test]
fn test_registry_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    // 1. SETUP
    let temp_dir = tempdir()?;
    let home_dir = temp_dir.path();
    let registry_path = home_dir.join(".kley/registry.json");

    // Create dummy library project
    let lib_dir = temp_dir.path().join("my-lib");
    fs::create_dir(&lib_dir)?;
    fs::write(
        lib_dir.join("package.json"),
        r#"{"name": "my-lib", "version": "1.0.0"}"#,
    )?;
    fs::write(lib_dir.join("index.js"), "console.log('my-lib');")?;

    // Create dummy consumer app project
    let app_dir = temp_dir.path().join("my-app");
    fs::create_dir(&app_dir)?;
    fs::write(
        app_dir.join("package.json"),
        r#"{"name": "my-app", "version": "1.0.0"}"#,
    )?;

    let mut cmd = Command::cargo_bin("kley")?;

    // 2. TEST `kley publish`
    cmd.current_dir(&lib_dir)
        .env("HOME", home_dir) // Override home dir for the test
        .arg("publish")
        .assert()
        .success()
        .stdout(predicate::str::contains("Package successfully published"));

    // Assert registry content after publish
    let registry_content = fs::read_to_string(&registry_path)?;
    let registry: Registry = serde_json::from_str(&registry_content)?;

    assert!(registry.packages.contains_key("my-lib"));
    let lib_meta = registry.packages.get("my-lib").unwrap();
    assert_eq!(lib_meta.version, "1.0.0");
    assert!(lib_meta.installations.is_empty());
    assert!(!lib_meta.last_updated.is_empty());

    cmd = Command::cargo_bin("kley")?;

    // 3. TEST `kley add`
    cmd.current_dir(&app_dir)
        .env("HOME", home_dir)
        .arg("add")
        .arg("my-lib")
        .assert()
        .success()
        .stdout(predicate::str::contains("package.json has been updated"));

    // Assert registry content after add
    let registry_content_after_add = fs::read_to_string(&registry_path)?;
    let registry_after_add: Registry = serde_json::from_str(&registry_content_after_add)?;
    let lib_meta_after_add = registry_after_add.packages.get("my-lib").unwrap();

    assert_eq!(lib_meta_after_add.installations.len(), 1);
    assert_eq!(lib_meta_after_add.installations[0], app_dir.canonicalize()?);

    cmd = Command::cargo_bin("kley")?;

    // 4. TEST `kley remove`
    cmd.current_dir(&app_dir)
        .env("HOME", home_dir)
        .arg("remove")
        .arg("my-lib")
        .assert()
        .success()
        .stdout(predicate::str::contains("kley.lock has been updated"));

    // Assert registry content after remove
    let registry_content_after_remove = fs::read_to_string(&registry_path)?;
    let registry_after_remove: Registry = serde_json::from_str(&registry_content_after_remove)?;
    let lib_meta_after_remove = registry_after_remove.packages.get("my-lib").unwrap();

    assert!(lib_meta_after_remove.installations.is_empty());

    Ok(())
}
