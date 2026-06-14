use assert_cmd::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn setup(home: &std::path::Path, lib: &std::path::Path, app: &std::path::Path) {
    fs::write(
        lib.join("package.json"),
        r#"{"name": "test-lib", "version": "1.0.0"}"#,
    )
    .unwrap();
    fs::write(lib.join("index.js"), "// lib").unwrap();
    fs::write(
        app.join("package.json"),
        r#"{"name": "app", "version": "1.0.0"}"#,
    )
    .unwrap();

    Command::cargo_bin("kley")
        .unwrap()
        .env("KLEY_HOME", home)
        .arg("publish")
        .current_dir(lib)
        .assert()
        .success();
}

#[test]
fn test_remove_linked_package_removes_only_symlink() -> Result<(), Box<dyn std::error::Error>> {
    let home = tempdir()?;
    let lib = tempdir()?;
    let app = tempdir()?;

    setup(home.path(), lib.path(), app.path());

    // Link
    Command::cargo_bin("kley")?
        .env("KLEY_HOME", home.path())
        .args(["link", "test-lib"])
        .current_dir(app.path())
        .assert()
        .success();

    let symlink = app.path().join("node_modules/test-lib");
    assert!(symlink.is_symlink(), "precondition: symlink exists");

    // Remove
    Command::cargo_bin("kley")?
        .env("KLEY_HOME", home.path())
        .args(["remove", "test-lib"])
        .current_dir(app.path())
        .assert()
        .success();

    // Symlink is gone
    assert!(
        !symlink.exists() && !symlink.is_symlink(),
        "symlink should be removed"
    );

    // Source directory is untouched
    assert!(
        lib.path().join("index.js").exists(),
        "source directory must not be modified"
    );

    // .kley dir was never created for link
    assert!(
        !app.path().join(".kley/test-lib").exists(),
        "no .kley copy should exist for linked package"
    );

    // Registry: links[] is now empty
    let registry: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        home.path().join(".kley/registry.json"),
    )?)?;
    let links = &registry["packages"]["test-lib"]["links"];
    let link_count = links.as_array().map_or(0, |a| a.len());
    assert_eq!(link_count, 0, "links[] should be empty after remove");

    Ok(())
}

#[test]
fn test_remove_install_package_full_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let home = tempdir()?;
    let lib = tempdir()?;
    let app = tempdir()?;

    // Library needs no-deps for fast-path install (creates .kley copy + symlink)
    fs::write(
        lib.path().join("package.json"),
        r#"{"name": "test-lib", "version": "1.0.0"}"#,
    )?;
    fs::write(lib.path().join("index.js"), "// lib")?;
    fs::write(
        app.path().join("package.json"),
        r#"{"name": "app", "version": "1.0.0"}"#,
    )?;

    Command::cargo_bin("kley")?
        .env("KLEY_HOME", home.path())
        .arg("publish")
        .current_dir(lib.path())
        .assert()
        .success();

    // Install (no deps → fast-path: .kley copy + symlink)
    Command::cargo_bin("kley")?
        .env("KLEY_HOME", home.path())
        .args(["add", "test-lib"])
        .current_dir(app.path())
        .assert()
        .success();

    assert!(
        app.path().join(".kley/test-lib").exists(),
        "precondition: .kley copy exists after install"
    );

    // Remove
    Command::cargo_bin("kley")?
        .env("KLEY_HOME", home.path())
        .args(["remove", "test-lib"])
        .current_dir(app.path())
        .assert()
        .success();

    // .kley copy removed
    assert!(
        !app.path().join(".kley/test-lib").exists(),
        ".kley/test-lib should be removed"
    );

    // node_modules symlink removed
    let nm = app.path().join("node_modules/test-lib");
    assert!(
        !nm.exists() && !nm.is_symlink(),
        "node_modules symlink should be removed"
    );

    // kley.lock no longer has the entry
    if app.path().join("kley.lock").exists() {
        let lock: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(app.path().join("kley.lock"))?)?;
        assert!(
            lock["packages"].get("test-lib").is_none()
                || lock["packages"]["test-lib"].is_null(),
            "kley.lock should not have test-lib entry after remove"
        );
    }

    // Registry installations[] is empty
    let registry: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        home.path().join(".kley/registry.json"),
    )?)?;
    let installations = &registry["packages"]["test-lib"]["installations"];
    let install_count = installations.as_array().map_or(0, |a| a.len());
    assert_eq!(install_count, 0, "installations[] should be empty after remove");

    Ok(())
}
