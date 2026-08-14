mod common;
use common::TestEnv;

use anyhow::Result;
use std::fs;

/// A package with a `workspace:` dependency should, on `kley add`, cause the
/// referenced package to be copied into the project's `.kley/` and linked via a
/// `file:.kley/<pkg>` entry, while the library's own manifest gets the
/// `workspace:` protocol stripped to a plain semver range.
#[test_log::test]
fn test_workspace_dep_resolved_to_local_file_link() -> Result<()> {
    let env = TestEnv::new();
    env.setup_project_pm("npm");

    env.create_mock_package_with_content(
        "my-lib",
        "1.2.3",
        r#"{"name":"my-lib","version":"1.2.3"}"#,
    );
    env.create_mock_package_with_content(
        "app-lib",
        "2.0.0",
        r#"{"name":"app-lib","version":"2.0.0","dependencies":{"my-lib":"workspace:^1.2.0"}}"#,
    );

    let output = env.run_kley_command(&["add", "app-lib"]).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The workspace dependency is copied into the project's .kley/.
    assert!(
        env.project_dir.join(".kley").join("my-lib").exists(),
        "my-lib should be copied into .kley"
    );

    // Root package.json gets both `file:.kley` links.
    let root_pkg: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(env.project_dir.join("package.json"))?)?;
    assert_eq!(root_pkg["dependencies"]["app-lib"], "file:.kley/app-lib");
    assert_eq!(root_pkg["dependencies"]["my-lib"], "file:.kley/my-lib");

    // The library's own manifest has `workspace:` stripped to a plain range.
    let app_pkg: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        env.project_dir
            .join(".kley")
            .join("app-lib")
            .join("package.json"),
    )?)?;
    assert_eq!(app_pkg["dependencies"]["my-lib"], "^1.2.0");

    Ok(())
}

/// `--no-workspace-resolve` must leave `workspace:` specifiers untouched and not
/// copy the referenced package.
#[test_log::test]
fn test_no_workspace_resolve_keeps_raw() -> Result<()> {
    let env = TestEnv::new();
    env.setup_project_pm("npm");

    env.create_mock_package_with_content(
        "my-lib",
        "1.2.3",
        r#"{"name":"my-lib","version":"1.2.3"}"#,
    );
    env.create_mock_package_with_content(
        "app-lib",
        "2.0.0",
        r#"{"name":"app-lib","version":"2.0.0","dependencies":{"my-lib":"workspace:^1.2.0"}}"#,
    );

    let output = env
        .run_kley_command(&["add", "app-lib", "--no-workspace-resolve"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The referenced package is NOT copied.
    assert!(!env.project_dir.join(".kley").join("my-lib").exists());

    // The library's internal specifier stays `workspace:`.
    let app_pkg: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        env.project_dir
            .join(".kley")
            .join("app-lib")
            .join("package.json"),
    )?)?;
    assert_eq!(app_pkg["dependencies"]["my-lib"], "workspace:^1.2.0");

    // Root package.json has app-lib but not my-lib.
    let root_pkg: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(env.project_dir.join("package.json"))?)?;
    assert_eq!(root_pkg["dependencies"]["app-lib"], "file:.kley/app-lib");
    assert!(root_pkg["dependencies"].get("my-lib").is_none());

    Ok(())
}

/// A `workspace:` dependency whose package is absent from the kley registry
/// cannot be linked locally: it is stripped to a plain range and a warning is
/// printed, but nothing is copied.
#[test_log::test]
fn test_workspace_dep_unresolvable_warns() -> Result<()> {
    let env = TestEnv::new();
    env.setup_project_pm("npm");

    // NOTE: my-lib is intentionally NOT published to the registry.
    env.create_mock_package_with_content(
        "app-lib",
        "2.0.0",
        r#"{"name":"app-lib","version":"2.0.0","dependencies":{"my-lib":"workspace:^1.2.0"}}"#,
    );

    let output = env.run_kley_command(&["add", "app-lib"]).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("could not be resolved"),
        "expected a warning on stderr, got: {stderr}"
    );

    // The missing package is NOT copied.
    assert!(!env.project_dir.join(".kley").join("my-lib").exists());

    // The library's internal specifier is stripped to a plain range.
    let app_pkg: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        env.project_dir
            .join(".kley")
            .join("app-lib")
            .join("package.json"),
    )?)?;
    assert_eq!(app_pkg["dependencies"]["my-lib"], "^1.2.0");

    Ok(())
}

/// Transitive resolution: A → B → C. Each level is copied into `.kley/` and its
/// own `workspace:` specifier is stripped.
#[test_log::test]
fn test_workspace_dep_resolved_transitively() -> Result<()> {
    let env = TestEnv::new();
    env.setup_project_pm("npm");

    env.create_mock_package_with_content(
        "core-lib",
        "1.0.0",
        r#"{"name":"core-lib","version":"1.0.0"}"#,
    );
    env.create_mock_package_with_content(
        "my-lib",
        "1.2.3",
        r#"{"name":"my-lib","version":"1.2.3","dependencies":{"core-lib":"workspace:^1.0.0"}}"#,
    );
    env.create_mock_package_with_content(
        "app-lib",
        "2.0.0",
        r#"{"name":"app-lib","version":"2.0.0","dependencies":{"my-lib":"workspace:^1.2.0"}}"#,
    );

    let output = env.run_kley_command(&["add", "app-lib"]).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(env.project_dir.join(".kley").join("my-lib").exists());
    assert!(env.project_dir.join(".kley").join("core-lib").exists());

    let app_pkg: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        env.project_dir
            .join(".kley")
            .join("app-lib")
            .join("package.json"),
    )?)?;
    assert_eq!(app_pkg["dependencies"]["my-lib"], "^1.2.0");

    let my_pkg: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        env.project_dir
            .join(".kley")
            .join("my-lib")
            .join("package.json"),
    )?)?;
    assert_eq!(my_pkg["dependencies"]["core-lib"], "^1.0.0");

    Ok(())
}

/// Regression: `kley update` (all packages) must not leave a raw `workspace:`
/// in a package that is both a top-level lock entry and a `workspace:` dep of
/// another package. Stripping must be unconditional/idempotent.
#[test_log::test]
fn test_update_does_not_reintroduce_raw_workspace() -> Result<()> {
    let env = TestEnv::new();
    env.setup_project_pm("npm");

    env.create_mock_package_with_content(
        "core-lib",
        "1.0.0",
        r#"{"name":"core-lib","version":"1.0.0"}"#,
    );
    env.create_mock_package_with_content(
        "my-lib",
        "1.2.3",
        r#"{"name":"my-lib","version":"1.2.3","dependencies":{"core-lib":"workspace:^1.0.0"}}"#,
    );
    env.create_mock_package_with_content(
        "app-lib",
        "2.0.0",
        r#"{"name":"app-lib","version":"2.0.0","dependencies":{"my-lib":"workspace:^1.2.0"}}"#,
    );

    let add = env.run_kley_command(&["add", "app-lib"]).output().unwrap();
    assert!(
        add.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let update = env.run_kley_command(&["update"]).output().unwrap();
    assert!(
        update.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&update.stderr)
    );

    // my-lib is re-processed as a top-level lock entry after being resolved as a
    // workspace dep; its own workspace: dep on core-lib must still be stripped.
    let my_lib: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        env.project_dir
            .join(".kley")
            .join("my-lib")
            .join("package.json"),
    )?)?;
    assert_eq!(my_lib["dependencies"]["core-lib"], "^1.0.0");

    Ok(())
}

/// A `workspace:` cycle must terminate (guard on the recursion) and both ends
/// must still be stripped to plain ranges.
#[test_log::test]
fn test_workspace_dep_cycle_terminates() -> Result<()> {
    let env = TestEnv::new();
    env.setup_project_pm("npm");

    env.create_mock_package_with_content(
        "a-lib",
        "1.0.0",
        r#"{"name":"a-lib","version":"1.0.0","dependencies":{"b-lib":"workspace:^1.0.0"}}"#,
    );
    env.create_mock_package_with_content(
        "b-lib",
        "1.0.0",
        r#"{"name":"b-lib","version":"1.0.0","dependencies":{"a-lib":"workspace:^1.0.0"}}"#,
    );

    let output = env.run_kley_command(&["add", "a-lib"]).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let a_pkg: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        env.project_dir
            .join(".kley")
            .join("a-lib")
            .join("package.json"),
    )?)?;
    assert_eq!(a_pkg["dependencies"]["b-lib"], "^1.0.0");

    let b_pkg: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        env.project_dir
            .join(".kley")
            .join("b-lib")
            .join("package.json"),
    )?)?;
    assert_eq!(b_pkg["dependencies"]["a-lib"], "^1.0.0");

    Ok(())
}

/// A `workspace:` dependency whose stored version does NOT satisfy the range is
/// stripped to a plain range with a warning, but nothing is copied or linked.
#[test_log::test]
fn test_workspace_dep_version_mismatch_warns() -> Result<()> {
    let env = TestEnv::new();
    env.setup_project_pm("npm");

    // my-lib is in the registry at 1.2.3, but app-lib requires workspace:^9.0.0.
    env.create_mock_package_with_content(
        "my-lib",
        "1.2.3",
        r#"{"name":"my-lib","version":"1.2.3"}"#,
    );
    env.create_mock_package_with_content(
        "app-lib",
        "2.0.0",
        r#"{"name":"app-lib","version":"2.0.0","dependencies":{"my-lib":"workspace:^9.0.0"}}"#,
    );

    let output = env.run_kley_command(&["add", "app-lib"]).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("could not be resolved"),
        "expected a warning on stderr, got: {stderr}"
    );

    assert!(!env.project_dir.join(".kley").join("my-lib").exists());
    let root_pkg: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(env.project_dir.join("package.json"))?)?;
    assert!(root_pkg["dependencies"].get("my-lib").is_none());

    let app_pkg: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        env.project_dir
            .join(".kley")
            .join("app-lib")
            .join("package.json"),
    )?)?;
    assert_eq!(app_pkg["dependencies"]["my-lib"], "^9.0.0");

    Ok(())
}

/// In `--pure` mode the `workspace:` dep is still copied into `.kley/` and its
/// internal specifier stripped, but `package.json` is left untouched.
#[test_log::test]
fn test_workspace_dep_pure_strips_without_injection() -> Result<()> {
    let env = TestEnv::new();
    env.setup_project_pm("npm");

    env.create_mock_package_with_content(
        "my-lib",
        "1.2.3",
        r#"{"name":"my-lib","version":"1.2.3"}"#,
    );
    env.create_mock_package_with_content(
        "app-lib",
        "2.0.0",
        r#"{"name":"app-lib","version":"2.0.0","dependencies":{"my-lib":"workspace:^1.2.0"}}"#,
    );

    let output = env
        .run_kley_command(&["add", "app-lib", "--pure"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(env.project_dir.join(".kley").join("my-lib").exists());

    let app_pkg: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        env.project_dir
            .join(".kley")
            .join("app-lib")
            .join("package.json"),
    )?)?;
    assert_eq!(app_pkg["dependencies"]["my-lib"], "^1.2.0");

    let root_pkg: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(env.project_dir.join("package.json"))?)?;
    assert!(root_pkg.get("dependencies").is_none());

    Ok(())
}
