use anyhow::Result;
use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// Creates a dummy library project.
fn setup_test_lib(dir: &Path) -> Result<()> {
    fs::write(
        dir.join("package.json"),
        r#"{"name": "test-lib", "version": "1.0.0"}"#,
    )?;
    fs::write(dir.join("index.js"), "module.exports = 'hello';")?;
    Ok(())
}

/// Creates a dummy consumer app.
fn setup_test_app(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir.join("node_modules"))?;
    fs::write(
        dir.join("package.json"),
        r#"{"name": "test-app", "dependencies": {}}"#,
    )?;
    Ok(())
}

#[test]
fn test_link_command_e2e() -> Result<()> {
    // 1. Setup temporary directories for our test projects
    let lib_dir = tempdir()?;
    let app_dir = tempdir()?;
    let temp_home = tempdir()?;

    setup_test_lib(lib_dir.path())?;
    setup_test_app(app_dir.path())?;

    let original_app_pkg_json = fs::read_to_string(app_dir.path().join("package.json"))?;

    // 2. Publish the library to our temporary kley store
    let mut publish_cmd = Command::cargo_bin("kley")?;
    publish_cmd
        .env("HOME", temp_home.path()) // Isolate kley store
        .arg("publish")
        .current_dir(lib_dir.path());

    publish_cmd
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Package successfully published to store!",
        ));

    // 3. Run the link command in the app directory
    let mut link_cmd = Command::cargo_bin("kley")?;
    link_cmd
        .env("HOME", temp_home.path()) // Use the same isolated store
        .arg("link")
        .arg("test-lib")
        .current_dir(app_dir.path());

    // This will fail until the command is implemented
    link_cmd
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "✅ Package test-lib successfully linked to this project!",
        ));

    // 4. Assertions: Verify the results after the command runs
    let app_path = app_dir.path();
    let symlink_path = app_path.join("node_modules/test-lib");
    let local_copy_path = app_path.join(".kley/test-lib");

    // a. Check that the symlink exists and is actually a symlink
    assert!(
        symlink_path.exists(),
        "Symlink should be created in node_modules"
    );
    assert!(
        symlink_path.is_symlink(),
        "The path in node_modules should be a symlink"
    );

    // b. Check that the symlink points to the local .kley copy
    let link_target = fs::read_link(&symlink_path)?;
    assert_eq!(
        link_target,
        fs::canonicalize(&local_copy_path)?,
        "Symlink should point to the .kley directory"
    );

    // c. Check that the local .kley copy exists and has content
    assert!(
        local_copy_path.join("index.js").exists(),
        "The local .kley copy of the package should exist"
    );

    // d. Verify that package.json was NOT modified
    let final_app_pkg_json = fs::read_to_string(app_dir.path().join("package.json"))?;
    assert_eq!(
        original_app_pkg_json, final_app_pkg_json,
        "package.json should not be modified by the link command"
    );

    Ok(())
}
