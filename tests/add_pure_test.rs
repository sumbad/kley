use anyhow::Result;
use predicates::prelude::predicate;
use std::fs;

mod common;
use common::TestEnv;

#[test_log::test]
fn test_add_pure_skips_package_json() -> Result<()> {
    let env = TestEnv::new();

    env.create_mock_package_with_content(
        "test-lib",
        "1.0.0",
        r#"{ "name": "test-lib", "version": "1.0.0" }"#,
    );

    fs::write(
        env.kley_registry
            .join("packages")
            .join("test-lib")
            .join("index.js"),
        "module.exports = 'v1';",
    )
    .unwrap();

    env.setup_project_pm("npm");

    env.run_kley_command(&["add", "test-lib", "--pure"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done: test-lib added"));

    // Package copied to .kley/<pkg>
    assert!(env.project_dir.join(".kley").join("test-lib").exists());

    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(env.project_dir.join("package.json"))?)?;

    assert!(
        content.get("dependencies").is_none(),
        "dependencies should not be added in pure mode"
    );

    assert!(
        &env.project_dir
            .join(".kley")
            .join("test-lib")
            .join("index.js")
            .exists()
    );

    assert!(&env.project_dir.join("kley.lock").exists());

    Ok(())
}

#[test_log::test]
fn test_add_no_pure_injects_file_dependency() -> Result<()> {
    let env = TestEnv::new();

    env.create_mock_package_with_content(
        "test-lib",
        "1.0.0",
        r#"{ "name": "test-lib", "version": "1.0.0" }"#,
    );

    fs::write(
        env.kley_registry
            .join("packages")
            .join("test-lib")
            .join("index.js"),
        "module.exports = 'v1';",
    )
    .unwrap();

    env.setup_project_pm("npm");

    env.run_kley_command(&["add", "test-lib", "--no-pure"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done: test-lib added"));

    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(env.project_dir.join("package.json"))?)?;
    assert_eq!(content["dependencies"]["test-lib"], "file:.kley/test-lib");

    Ok(())
}

#[test_log::test]
fn test_add_in_workspace_defaults_to_pure() -> Result<()> {
    let env = TestEnv::new();

    fs::write(
        env.project_dir.join("package.json"),
        r#"{ "name": "test-proj", "version": "1.0.0", "workspaces": ["packages/*"] }"#,
    )
    .unwrap();

    env.setup_project_pm("npm");

    env.create_mock_package_with_content(
        "test-lib",
        "1.0.0",
        r#"{ "name": "test-lib", "version": "1.0.0" }"#,
    );

    fs::write(
        env.kley_registry
            .join("packages")
            .join("test-lib")
            .join("index.js"),
        "module.exports = 'v1';",
    )
    .unwrap();

    let output = env.run_kley_command(&["add", "test-lib"]).output().unwrap();
    // tracing::info!(
    //     "=== STDERR ===\n{}",
    //     String::from_utf8_lossy(&output.stderr)
    // );
    // tracing::info!(
    //     "=== STDOUT ===\n{}",
    //     String::from_utf8_lossy(&output.stdout)
    // );
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Done: test-lib added"));

    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(env.project_dir.join("package.json"))?)?;

    assert!(
        content.get("dependencies").is_none(),
        "workspace project should default to pure"
    );

    Ok(())
}

#[test_log::test]
fn test_add_non_workspace_default_injects() -> Result<()> {
    let env = TestEnv::new();

    fs::write(
        env.project_dir.join("package.json"),
        r#"{ "name": "test-proj", "version": "1.0.0" }"#,
    )
    .unwrap();

    env.setup_project_pm("npm");

    env.create_mock_package_with_content(
        "test-lib",
        "1.0.0",
        r#"{ "name": "test-lib", "version": "1.0.0" }"#,
    );

    fs::write(
        env.kley_registry
            .join("packages")
            .join("test-lib")
            .join("index.js"),
        "module.exports = 'v1';",
    )
    .unwrap();

    let output = env.run_kley_command(&["add", "test-lib"]).output().unwrap();
    // tracing::info!(
    //     "=== STDERR ===\n{}",
    //     String::from_utf8_lossy(&output.stderr)
    // );
    // tracing::info!(
    //     "=== STDOUT ===\n{}",
    //     String::from_utf8_lossy(&output.stdout)
    // );
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Done: test-lib added"));

    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(env.project_dir.join("package.json"))?)?;

    assert_eq!(content["dependencies"]["test-lib"], "file:.kley/test-lib");

    Ok(())
}
