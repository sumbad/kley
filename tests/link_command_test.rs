use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn setup_lib(dir: &Path, name: &str, extra_json: &str) -> Result<()> {
    let json = format!(
        r#"{{"name": "{}", "version": "1.0.0"{}}}"#,
        name, extra_json
    );
    fs::write(dir.join("package.json"), json)?;
    fs::write(dir.join("index.js"), "module.exports = 'v1';")?;
    Ok(())
}

fn setup_app(dir: &Path) -> Result<()> {
    fs::write(dir.join("package.json"), r#"{"name": "app"}"#)?;
    Ok(())
}

fn publish(lib_dir: &Path, home_dir: &Path) {
    Command::cargo_bin("kley")
        .unwrap()
        .env("KLEY_HOME", home_dir)
        .arg("publish")
        .current_dir(lib_dir)
        .assert()
        .success();
}

fn link(app_dir: &Path, home_dir: &Path, pkg: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("kley")
        .unwrap()
        .env("KLEY_HOME", home_dir)
        .args(["link", pkg])
        .current_dir(app_dir)
        .assert()
}

// ---------------------------------------------------------------------------

#[test]
fn test_link_creates_direct_symlink_to_source() -> Result<()> {
    let home = tempdir()?;
    let lib = tempdir()?;
    let app = tempdir()?;

    setup_lib(lib.path(), "test-lib", "")?;
    setup_app(app.path())?;

    publish(lib.path(), home.path());
    link(app.path(), home.path(), "test-lib").success();

    let symlink = app.path().join("node_modules/test-lib");
    assert!(
        symlink.is_symlink(),
        "node_modules/test-lib should be a symlink"
    );

    let target = fs::read_link(&symlink)?;
    let target_canon = fs::canonicalize(&target)?;
    let lib_canon = fs::canonicalize(lib.path())?;
    assert_eq!(
        target_canon, lib_canon,
        "symlink should point directly to library source directory"
    );

    Ok(())
}

#[test]
fn test_link_does_not_create_kley_copy() -> Result<()> {
    let home = tempdir()?;
    let lib = tempdir()?;
    let app = tempdir()?;

    setup_lib(lib.path(), "test-lib", "")?;
    setup_app(app.path())?;

    publish(lib.path(), home.path());
    link(app.path(), home.path(), "test-lib").success();

    assert!(
        !app.path().join(".kley/test-lib").exists(),
        ".kley/test-lib should NOT be created — new behavior uses direct symlink"
    );

    Ok(())
}

#[test]
fn test_link_does_not_modify_package_json() -> Result<()> {
    let home = tempdir()?;
    let lib = tempdir()?;
    let app = tempdir()?;

    setup_lib(lib.path(), "test-lib", "")?;
    setup_app(app.path())?;

    publish(lib.path(), home.path());

    let before = fs::read_to_string(app.path().join("package.json"))?;
    link(app.path(), home.path(), "test-lib").success();
    let after = fs::read_to_string(app.path().join("package.json"))?;

    assert_eq!(
        before, after,
        "package.json must not be modified by kley link"
    );

    Ok(())
}

#[test]
fn test_link_sets_connection_link_in_lockfile() -> Result<()> {
    let home = tempdir()?;
    let lib = tempdir()?;
    let app = tempdir()?;

    setup_lib(lib.path(), "test-lib", "")?;
    setup_app(app.path())?;

    publish(lib.path(), home.path());
    link(app.path(), home.path(), "test-lib").success();

    let lock: Value = serde_json::from_str(&fs::read_to_string(app.path().join("kley.lock"))?)?;
    assert_eq!(
        lock["packages"]["test-lib"]["connection"], "link",
        "kley.lock should record connection:link"
    );

    Ok(())
}

#[test]
fn test_link_fails_if_not_published() -> Result<()> {
    let home = tempdir()?;
    let app = tempdir()?;

    setup_app(app.path())?;

    // No publish — source_path not in registry
    link(app.path(), home.path(), "unknown-lib")
        .failure()
        .stderr(predicate::str::contains("not found in registry"));

    Ok(())
}

#[test]
fn test_link_fails_if_source_dir_moved() -> Result<()> {
    let home = tempdir()?;
    let lib = tempdir()?;
    let app = tempdir()?;

    setup_lib(lib.path(), "test-lib", "")?;
    setup_app(app.path())?;

    // Publish records source_path = lib.path()
    publish(lib.path(), home.path());

    // Now simulate source directory being removed/moved
    let gone_lib = lib.path().to_path_buf();
    drop(lib); // TempDir cleanup — directory is deleted

    assert!(!gone_lib.exists(), "precondition: lib dir must be gone");

    link(app.path(), home.path(), "test-lib")
        .failure()
        .stderr(predicate::str::contains("no longer exists"));

    Ok(())
}

#[test]
fn test_link_warns_singleton_deps() -> Result<()> {
    let home = tempdir()?;
    let lib = tempdir()?;
    let app = tempdir()?;

    // react is in both dependencies and peerDependencies → singleton
    setup_lib(
        lib.path(),
        "test-lib",
        r#", "dependencies": {"react": "^18"}, "peerDependencies": {"react": "^18"}"#,
    )?;
    setup_app(app.path())?;

    publish(lib.path(), home.path());
    link(app.path(), home.path(), "test-lib")
        .success()
        .stdout(predicate::str::contains("singleton dependencies"));

    Ok(())
}

#[test]
fn test_link_no_warning_for_stateless_deps() -> Result<()> {
    let home = tempdir()?;
    let lib = tempdir()?;
    let app = tempdir()?;

    // lodash only in dependencies, react only in peerDependencies — no overlap
    setup_lib(
        lib.path(),
        "test-lib",
        r#", "dependencies": {"lodash": "^4"}, "peerDependencies": {"react": "^18"}"#,
    )?;
    setup_app(app.path())?;

    publish(lib.path(), home.path());
    link(app.path(), home.path(), "test-lib")
        .success()
        .stdout(predicate::str::contains("singleton").not());

    Ok(())
}

#[test]
fn test_link_switches_from_install_with_warning() -> Result<()> {
    let home = tempdir()?;
    let lib = tempdir()?;
    let app = tempdir()?;

    setup_lib(lib.path(), "test-lib", "")?;
    setup_app(app.path())?;

    publish(lib.path(), home.path());

    // Install first (no deps → fast-path creates .kley/test-lib + symlink)
    Command::cargo_bin("kley")?
        .env("KLEY_HOME", home.path())
        .args(["add", "test-lib"])
        .current_dir(app.path())
        .assert()
        .success();

    assert!(
        app.path().join(".kley/test-lib").exists(),
        "precondition: .kley/test-lib should exist after install"
    );

    // Now link — should warn, remove .kley copy, create direct symlink
    link(app.path(), home.path(), "test-lib")
        .success()
        .stdout(predicate::str::contains("switching to link mode"));

    // .kley copy is gone
    assert!(
        !app.path().join(".kley/test-lib").exists(),
        ".kley/test-lib should be removed after switching to link"
    );

    // Symlink now points to lib source directly
    let symlink = app.path().join("node_modules/test-lib");
    assert!(symlink.is_symlink());
    let target = fs::canonicalize(fs::read_link(&symlink)?)?;
    let src = fs::canonicalize(lib.path())?;
    assert_eq!(target, src, "symlink should now point to lib source dir");

    Ok(())
}

#[test]
fn test_link_records_in_registry_links() -> Result<()> {
    let home = tempdir()?;
    let lib = tempdir()?;
    let app = tempdir()?;

    setup_lib(lib.path(), "test-lib", "")?;
    setup_app(app.path())?;

    publish(lib.path(), home.path());
    link(app.path(), home.path(), "test-lib").success();

    let registry: Value = serde_json::from_str(&fs::read_to_string(
        home.path().join(".kley/registry.json"),
    )?)?;

    let links = &registry["packages"]["test-lib"]["links"];
    assert!(links.is_array(), "links field should be an array");
    assert_eq!(
        links.as_array().unwrap().len(),
        1,
        "should have one link entry"
    );

    let installations = &registry["packages"]["test-lib"]["installations"];
    let install_count = installations.as_array().map_or(0, |a| a.len());
    assert_eq!(
        install_count, 0,
        "link should not appear in installations[]"
    );

    Ok(())
}
