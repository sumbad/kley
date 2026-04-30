mod common;

use assert_cmd::prelude::*;
use common::paths_match;
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

    // Create dummy consumer app projects
    let app_dir = temp_dir.path().join("my-app");
    fs::create_dir(&app_dir)?;
    fs::write(
        app_dir.join("package.json"),
        r#"{"name": "my-app", "version": "1.0.0"}"#,
    )?;
    let app_dir_2 = temp_dir.path().join("my-app-2");
    fs::create_dir(&app_dir_2)?;
    fs::write(
        app_dir_2.join("package.json"),
        r#"{"name": "my-app-2", "version": "1.0.0"}"#,
    )?;

    let mut cmd = Command::cargo_bin("kley")?;

    // 2. TEST `kley publish`
    cmd.current_dir(&lib_dir)
        .env("KLEY_HOME", home_dir)
        .arg("publish")
        .assert()
        .success();

    // Assert registry content after publish
    let registry: Registry = serde_json::from_str(&fs::read_to_string(&registry_path)?)?;
    assert!(registry.packages.contains_key("my-lib"));
    assert!(registry
        .packages
        .get("my-lib")
        .unwrap()
        .installations
        .is_empty());

    // 3. TEST `kley add`
    Command::cargo_bin("kley")?
        .current_dir(&app_dir)
        .env("KLEY_HOME", home_dir)
        .arg("add")
        .arg("my-lib")
        .assert()
        .success();

    // Assert registry content after add
    let registry_after_add: Registry = serde_json::from_str(&fs::read_to_string(&registry_path)?)?;
    let lib_meta_after_add = registry_after_add.packages.get("my-lib").unwrap();
    assert_eq!(lib_meta_after_add.installations.len(), 1);
    assert!(
        paths_match(&lib_meta_after_add.installations[0], &app_dir),
        "installation path {:?} should match {:?}",
        lib_meta_after_add.installations[0],
        app_dir
    );

    // 4. TEST `kley link`
    Command::cargo_bin("kley")?
        .current_dir(&app_dir_2)
        .env("KLEY_HOME", home_dir)
        .arg("link")
        .arg("my-lib")
        .assert()
        .success();

    // Assert registry content after link
    let registry_after_link: Registry = serde_json::from_str(&fs::read_to_string(&registry_path)?)?;
    let lib_meta_after_link = registry_after_link.packages.get("my-lib").unwrap();
    assert_eq!(lib_meta_after_link.installations.len(), 2);
    assert!(
        lib_meta_after_link
            .installations
            .iter()
            .any(|p| paths_match(p, &app_dir)),
        "installations should contain {:?}",
        app_dir
    );
    assert!(
        lib_meta_after_link
            .installations
            .iter()
            .any(|p| paths_match(p, &app_dir_2)),
        "installations should contain {:?}",
        app_dir_2
    );

    // 5. TEST `kley remove` from first app
    Command::cargo_bin("kley")?
        .current_dir(&app_dir)
        .env("KLEY_HOME", home_dir)
        .arg("remove")
        .arg("my-lib")
        .assert()
        .success();

    // Assert registry content after first remove
    let registry_after_remove1: Registry =
        serde_json::from_str(&fs::read_to_string(&registry_path)?)?;
    let lib_meta_after_remove1 = registry_after_remove1.packages.get("my-lib").unwrap();
    assert_eq!(lib_meta_after_remove1.installations.len(), 1);
    assert!(
        paths_match(&lib_meta_after_remove1.installations[0], &app_dir_2),
        "remaining installation {:?} should match {:?}",
        lib_meta_after_remove1.installations[0],
        app_dir_2
    );

    // 6. TEST `kley remove` from second app
    Command::cargo_bin("kley")?
        .current_dir(&app_dir_2)
        .env("KLEY_HOME", home_dir)
        .arg("remove")
        .arg("my-lib")
        .assert()
        .success();

    // Assert registry content after second remove
    let registry_after_remove2: Registry =
        serde_json::from_str(&fs::read_to_string(&registry_path)?)?;
    let lib_meta_after_remove2 = registry_after_remove2.packages.get("my-lib").unwrap();
    assert!(lib_meta_after_remove2.installations.is_empty());

    Ok(())
}
