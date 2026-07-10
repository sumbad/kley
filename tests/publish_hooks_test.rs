mod common;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// Create a minimal publishable project. `extra` is appended to package.json.
fn setup_project(dir: &Path, pkg_json: &str) {
    fs::create_dir_all(dir.join(".git")).unwrap();
    fs::write(dir.join("package.json"), pkg_json).unwrap();
    fs::write(dir.join("index.js"), "console.log('hello');").unwrap();
    fs::create_dir_all(dir.join("dist")).unwrap();
    fs::write(dir.join("dist/bundle.js"), "/* bundle */").unwrap();
    // Ensure the .kley/ dir exists so tests can write .kley/hooks.json.
    fs::create_dir_all(dir.join(".kley")).unwrap();
}

fn basic_pkg(name: &str) -> String {
    format!(
        r#"{{"name": "{}", "version": "1.0.0", "files": ["dist", "index.js"]}}"#,
        name
    )
}

#[test]
fn test_publish_runs_hooks_when_hooks_json_present() {
    let home = tempdir().unwrap();
    let proj = tempdir().unwrap();
    let proj_path = proj.path();
    setup_project(proj_path, &basic_pkg("hooks-run-pkg"));

    // A pre-phase hook that writes a marker file into the project dir.
    fs::write(
        proj_path.join(".kley/hooks.json"),
        r#"{"prepack": {"command": "echo ran > .kley-hook-ran"}}"#,
    )
    .unwrap();

    let mut cmd = common::kley_cmd();
    cmd.arg("publish")
        .env("KLEY_HOME", home.path())
        .current_dir(proj_path);
    cmd.assert().success();

    assert!(
        proj_path.join(".kley-hook-ran").exists(),
        "prepack hook should have executed and created the marker file"
    );
}

#[test]
fn test_publish_no_hooks_flag_skips_hooks() {
    let home = tempdir().unwrap();
    let proj = tempdir().unwrap();
    let proj_path = proj.path();
    setup_project(proj_path, &basic_pkg("hooks-no-flag-pkg"));

    fs::write(
        proj_path.join(".kley/hooks.json"),
        r#"{"prepack": {"command": "echo ran > .kley-hook-ran"}}"#,
    )
    .unwrap();

    let mut cmd = common::kley_cmd();
    cmd.arg("publish")
        .arg("--no-hooks")
        .env("KLEY_HOME", home.path())
        .current_dir(proj_path);
    cmd.assert().success();

    assert!(
        !proj_path.join(".kley-hook-ran").exists(),
        "--no-hooks must skip execution of .kley/hooks.json"
    );
}

#[test]
fn test_publish_non_interactive_pure_copy_without_file() {
    let home = tempdir().unwrap();
    let proj = tempdir().unwrap();
    let proj_path = proj.path();
    setup_project(proj_path, &basic_pkg("hooks-nonint-pkg"));

    // No .kley/hooks.json; --non-interactive must do a pure copy and NOT
    // create a hooks file.
    let mut cmd = common::kley_cmd();
    cmd.arg("publish")
        .arg("--non-interactive")
        .env("KLEY_HOME", home.path())
        .current_dir(proj_path);
    cmd.assert().success();

    let store = home
        .path()
        .join(".kley/packages/hooks-nonint-pkg/package.json");
    assert!(store.exists(), "package should be published to the store");
    assert!(
        !proj_path.join(".kley/hooks.json").exists(),
        "--non-interactive with no file must not create a hooks file"
    );
}

#[test]
fn test_publish_short_y_non_interactive_pure_copy() {
    let home = tempdir().unwrap();
    let proj = tempdir().unwrap();
    let proj_path = proj.path();
    setup_project(proj_path, &basic_pkg("hooks-short-y-pkg"));

    // `-y` is the short alias for `--non-interactive`: pure copy, no wizard,
    // no hooks file created.
    let mut cmd = common::kley_cmd();
    cmd.arg("publish")
        .arg("-y")
        .env("KLEY_HOME", home.path())
        .current_dir(proj_path);
    cmd.assert().success();

    let store = home
        .path()
        .join(".kley/packages/hooks-short-y-pkg/package.json");
    assert!(store.exists(), "package should be published to the store");
    assert!(
        !proj_path.join(".kley/hooks.json").exists(),
        "-y with no file must not create a hooks file"
    );
}

#[test]
fn test_publish_pre_hook_failure_aborts_before_copy() {
    let home = tempdir().unwrap();
    let proj = tempdir().unwrap();
    let proj_path = proj.path();
    setup_project(proj_path, &basic_pkg("hooks-fail-pkg"));

    // A failing pre-phase hook must abort publish before files are copied.
    fs::write(
        proj_path.join(".kley/hooks.json"),
        r#"{"prepack": {"command": "exit 1"}}"#,
    )
    .unwrap();

    let mut cmd = common::kley_cmd();
    cmd.arg("publish")
        .env("KLEY_HOME", home.path())
        .current_dir(proj_path);
    cmd.assert().failure();

    let store = home.path().join(".kley/packages/hooks-fail-pkg");
    assert!(
        !store.exists(),
        "pre-hook failure must prevent file copy into the store"
    );
}

#[test]
fn test_publish_excludes_project_kley_dir() {
    let home = tempdir().unwrap();
    let proj = tempdir().unwrap();
    let proj_path = proj.path();
    setup_project(proj_path, &basic_pkg("hooks-exclude-pkg"));

    // The library itself consumes local packages -> has its own .kley/ dir.
    // It must NOT be bundled into the published store (Defect #1).
    fs::create_dir_all(proj_path.join(".kley/some-dep")).unwrap();
    fs::write(proj_path.join(".kley/some-dep/index.js"), "dep").unwrap();
    fs::write(
        proj_path.join(".kley/hooks.json"),
        r#"{"prepare": {"command": "echo ran > .kley-hook-ran"}}"#,
    )
    .unwrap();

    let mut cmd = common::kley_cmd();
    cmd.arg("publish")
        .env("KLEY_HOME", home.path())
        .current_dir(proj_path);
    cmd.assert().success();

    let store = home.path().join(".kley/packages/hooks-exclude-pkg");
    assert!(
        !store.join(".kley").exists(),
        "a library's own .kley/ dir must be excluded from the published package"
    );
}

#[test]
fn test_publish_post_hook_failure_still_publishes() {
    let home = tempdir().unwrap();
    let proj = tempdir().unwrap();
    let proj_path = proj.path();
    setup_project(proj_path, &basic_pkg("hooks-postfail-pkg"));

    // A failing POST hook must not undo the already-copied package.
    fs::write(
        proj_path.join(".kley/hooks.json"),
        r#"{"postpublish": {"command": "exit 1"}}"#,
    )
    .unwrap();

    let mut cmd = common::kley_cmd();
    cmd.arg("publish")
        .env("KLEY_HOME", home.path())
        .current_dir(proj_path);
    cmd.assert().failure();

    let store = home.path().join(".kley/packages/hooks-postfail-pkg");
    assert!(
        store.join("package.json").exists(),
        "post-hook failure must NOT undo the already-published package"
    );
}
