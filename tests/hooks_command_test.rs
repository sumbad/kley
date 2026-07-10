mod common;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn setup_project(dir: &Path, pkg_json: &str) {
    fs::create_dir_all(dir.join(".git")).unwrap();
    fs::write(dir.join("package.json"), pkg_json).unwrap();
    fs::write(dir.join("index.js"), "console.log('hello');").unwrap();
    fs::create_dir_all(dir.join(".kley")).unwrap();
}

#[test]
fn test_hooks_list_shows_config() {
    let proj = tempdir().unwrap();
    let proj_path = proj.path();
    setup_project(
        proj_path,
        r#"{"name": "hooks-list-pkg", "version": "1.0.0"}"#,
    );

    fs::write(
        proj_path.join(".kley/hooks.json"),
        r#"{"prepare": {"command": "npm run build"}}"#,
    )
    .unwrap();

    let mut cmd = common::kley_cmd();
    cmd.arg("hooks")
        .arg("list")
        .current_dir(proj_path);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("prepare"))
        .stdout(predicates::str::contains("npm run build"));
}

#[test]
fn test_hooks_edit_preserves_manual_entries_non_interactive() {
    let proj = tempdir().unwrap();
    let proj_path = proj.path();
    setup_project(
        proj_path,
        r#"{"name": "hooks-edit-pkg", "version": "1.0.0"}"#,
    );

    // A manual hook that is NOT present in package.json scripts.
    fs::write(
        proj_path.join(".kley/hooks.json"),
        r#"{"prepare": {"command": "manual-cmd"}}"#,
    )
    .unwrap();

    // `kley hooks edit` in a non-interactive context (no TTY) must preserve
    // the existing config rather than wipe manual entries (Defect #2).
    let mut cmd = common::kley_cmd();
    cmd.arg("hooks")
        .arg("edit")
        .env("KLEY_HOME", proj.path())
        .current_dir(proj_path);
    cmd.assert().success();

    let content = fs::read_to_string(proj_path.join(".kley/hooks.json")).unwrap();
    assert!(
        content.contains("manual-cmd"),
        "kley hooks edit must preserve manual entries absent from package.json"
    );
}
