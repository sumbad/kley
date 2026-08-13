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
        env.project_dir.join(".kley").join("app-lib").join("package.json"),
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
        env.project_dir.join(".kley").join("app-lib").join("package.json"),
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
        env.project_dir.join(".kley").join("app-lib").join("package.json"),
    )?)?;
    assert_eq!(app_pkg["dependencies"]["my-lib"], "^1.2.0");

    Ok(())
}
