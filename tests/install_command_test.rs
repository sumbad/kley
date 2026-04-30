mod common;

use kley::utils::normalized_path;
use predicates::prelude::*;
use std::fs;

use common::TestEnv;

///  RUST_LOG=debug cargo test --test install_command_test -- --nocapture

#[test_log::test]
fn test_install_command_npm_project() {
    let env = TestEnv::new();
    env.create_mock_registry_package("my-package", "1.0.0");
    env.setup_project_pm("npm");

    env.run_kley_command(&["install", "my-package"])
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Done: my-package installed"));

    assert!(env.project_dir.join(".kley").join("my-package").exists());

    let mut kley_lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();
    // tracing::debug!("kley_lock_content: {}", kley_lock_content);

    kley_lock_content.retain(|c| !c.is_whitespace());
    assert!(kley_lock_content.contains(r#""my-package":{"version":"1.0.0"}"#));

    let project_pkg_json_content =
        fs::read_to_string(env.project_dir.join("package.json")).unwrap();
    // tracing::debug!("project_pkg_json_content: {}", project_pkg_json_content);

    let expected_path = normalized_path(&env.project_dir.join(".kley").join("my-package"), None);
    let expected_dep_string = format!(r#""my-package": "file:{}"#, expected_path);

    assert!(project_pkg_json_content.contains(&expected_dep_string));

    let pm_log_content = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(pm_log_content.contains("npm install"));
}

#[test]
fn test_install_command_pnpm_project() {
    let env = TestEnv::new();
    env.create_mock_registry_package("my-pnpm-package", "2.0.0");
    env.setup_project_pm("pnpm");

    env.run_kley_command(&["install", "my-pnpm-package"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "✅ Done: my-pnpm-package installed",
        ));

    assert!(
        env.project_dir
            .join(".kley")
            .join("my-pnpm-package")
            .exists()
    );

    let mut kley_lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();
    kley_lock_content.retain(|c| !c.is_whitespace());
    assert!(kley_lock_content.contains(r#""my-pnpm-package":{"version":"2.0.0"}"#));

    let project_pkg_json_content =
        fs::read_to_string(env.project_dir.join("package.json")).unwrap();

    let expected_path =
        normalized_path(&env.project_dir.join(".kley").join("my-pnpm-package"), None);
    let expected_dep_string = format!(r#""my-pnpm-package": "file:{}"#, expected_path);
    // tracing::debug!("project_pkg_json_content: {}", project_pkg_json_content);
    // tracing::debug!("expected_dep_string: {}", expected_dep_string);

    assert!(project_pkg_json_content.contains(&expected_dep_string));

    let pm_log_content = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(pm_log_content.contains("pnpm add"));
}

#[test]
fn test_install_command_yarn_project() {
    let env = TestEnv::new();
    env.create_mock_registry_package("my-yarn-package", "3.0.0");
    env.setup_project_pm("yarn");

    env.run_kley_command(&["install", "my-yarn-package"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "✅ Done: my-yarn-package installed",
        ));

    assert!(
        env.project_dir
            .join(".kley")
            .join("my-yarn-package")
            .exists()
    );

    let mut kley_lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();
    kley_lock_content.retain(|c| !c.is_whitespace());
    assert!(kley_lock_content.contains(r#""my-yarn-package":{"version":"3.0.0"}"#));

    let project_pkg_json_content =
        fs::read_to_string(env.project_dir.join("package.json")).unwrap();

    let expected_path =
        normalized_path(&env.project_dir.join(".kley").join("my-yarn-package"), None);
    let expected_dep_string = format!(r#""my-yarn-package": "file:{}"#, expected_path);

    assert!(project_pkg_json_content.contains(&expected_dep_string));

    let pm_log_content = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(pm_log_content.contains("yarn add"));
}

#[test]
fn test_install_command_kley_lock_pm_override() {
    let env = TestEnv::new();
    env.create_mock_registry_package("my-override-package", "4.0.0");
    env.setup_project_pm("npm"); // Project has npm lockfile

    // Create kley.lock to override to pnpm
    env.create_kley_lock(r#"{"packageManager": "pnpm", "packages": {}}"#);

    env.run_kley_command(&["install", "my-override-package"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "✅ Done: my-override-package installed",
        ));

    assert!(
        env.project_dir
            .join(".kley")
            .join("my-override-package")
            .exists()
    );

    let mut kley_lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();
    kley_lock_content.retain(|c| !c.is_whitespace());
    // tracing::debug!("{}", kley_lock_content);

    assert!(kley_lock_content.contains(r#""my-override-package":{"version":"4.0.0"}"#));
    assert!(kley_lock_content.contains(r#""packageManager":"pnpm""#)); // kley.lock should retain its PM setting

    let project_pkg_json_content =
        fs::read_to_string(env.project_dir.join("package.json")).unwrap();

    let expected_path = normalized_path(
        &env.project_dir.join(".kley").join("my-override-package"),
        None,
    );
    let expected_dep_string = format!(r#""my-override-package": "file:{}"#, expected_path);

    assert!(project_pkg_json_content.contains(&expected_dep_string));

    let pm_log_content = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(pm_log_content.contains("pnpm add"));
}
