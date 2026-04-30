mod common;

use predicates::prelude::*;
use std::fs;

use common::TestEnv;

///  RUST_LOG=debug cargo test --test install_command_test -- --nocapture

/// Checks that package.json contains a file: dependency pointing to .kley/<pkg_name>.
/// This is platform-independent: normalizes both the actual content and the expected
/// path to use forward slashes before comparison.
fn assert_pkg_json_has_file_dep(pkg_json_content: &str, pkg_name: &str, project_dir: &std::path::Path) {
    let kley_path = project_dir.join(".kley").join(pkg_name);
    // Normalize expected path to forward slashes (matching what mock scripts write)
    let expected_path = kley_path.to_string_lossy().replace('\\', "/");
    let expected_prefix = format!(r#""{}": "file:"#, pkg_name);
    assert!(
        pkg_json_content.contains(&expected_prefix),
        "package.json should contain a file: dependency for '{}'. Content:\n{}",
        pkg_name, pkg_json_content
    );
    assert!(
        pkg_json_content.contains(&expected_path),
        "package.json should contain path '{}'. Content:\n{}",
        expected_path, pkg_json_content
    );
}

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

    kley_lock_content.retain(|c| !c.is_whitespace());
    assert!(kley_lock_content.contains(r#""my-package":{"version":"1.0.0"}"#));

    let project_pkg_json_content =
        fs::read_to_string(env.project_dir.join("package.json")).unwrap();

    assert_pkg_json_has_file_dep(&project_pkg_json_content, "my-package", &env.project_dir);

    let pm_log_content = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(pm_log_content.contains("npm install"));
    assert!(pm_log_content.contains("--ignore-scripts"));
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

    assert_pkg_json_has_file_dep(&project_pkg_json_content, "my-pnpm-package", &env.project_dir);

    let pm_log_content = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(pm_log_content.contains("pnpm add"));
    assert!(pm_log_content.contains("--ignore-scripts"));
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

    assert_pkg_json_has_file_dep(&project_pkg_json_content, "my-yarn-package", &env.project_dir);

    let pm_log_content = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(pm_log_content.contains("yarn add"));
    assert!(pm_log_content.contains("--ignore-scripts"));
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

    assert!(kley_lock_content.contains(r#""my-override-package":{"version":"4.0.0"}"#));
    assert!(kley_lock_content.contains(r#""packageManager":"pnpm""#)); // kley.lock should retain its PM setting

    let project_pkg_json_content =
        fs::read_to_string(env.project_dir.join("package.json")).unwrap();

    assert_pkg_json_has_file_dep(&project_pkg_json_content, "my-override-package", &env.project_dir);

    let pm_log_content = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(pm_log_content.contains("pnpm add"));
    assert!(pm_log_content.contains("--ignore-scripts"));
}
