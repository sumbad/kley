mod common;

use predicates::prelude::*;
use std::fs;

use common::TestEnv;

///  RUST_LOG=debug cargo test --test install_command_test -- --nocapture
///
/// Checks that package.json contains a file: dependency pointing to .kley/<pkg_name>.
/// This is platform-independent: normalizes both the actual content and the expected
/// path to use forward slashes before comparison.
fn assert_pkg_json_has_file_dep(
    pkg_json_content: &str,
    pkg_name: &str,
    project_dir: &std::path::Path,
) {
    let kley_path = project_dir.join(".kley").join(pkg_name);
    // Normalize expected path to forward slashes (matching what mock scripts write)
    let expected_path = kley_path.to_string_lossy().replace('\\', "/");
    let expected_prefix = format!(r#""{}": "file:"#, pkg_name);
    assert!(
        pkg_json_content.contains(&expected_prefix),
        "package.json should contain a file: dependency for '{}'. Content:\n{}",
        pkg_name,
        pkg_json_content
    );
    assert!(
        pkg_json_content.contains(&expected_path),
        "package.json should contain path '{}'. Content:\n{}",
        expected_path,
        pkg_json_content
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
        .stdout(predicate::str::contains("Done: my-package installed"));

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
        .stdout(predicate::str::contains("Done: my-pnpm-package installed"));

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

    assert_pkg_json_has_file_dep(
        &project_pkg_json_content,
        "my-pnpm-package",
        &env.project_dir,
    );

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
        .stdout(predicate::str::contains("Done: my-yarn-package installed"));

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

    assert_pkg_json_has_file_dep(
        &project_pkg_json_content,
        "my-yarn-package",
        &env.project_dir,
    );

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
            "Done: my-override-package installed",
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

    assert_pkg_json_has_file_dep(
        &project_pkg_json_content,
        "my-override-package",
        &env.project_dir,
    );

    let pm_log_content = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(pm_log_content.contains("pnpm add"));
    assert!(pm_log_content.contains("--ignore-scripts"));
}

#[test_log::test]
fn test_install_strips_dev_dependencies() {
    let env = TestEnv::new();
    env.setup_project_pm("npm");

    // 1. Setup package with devDependencies
    let lib_pkg_name = "my-lib-with-dev-deps";
    let lib_pkg_version = "1.0.0";
    let lib_pkg_json_content = r#"{
        "name": "my-lib-with-dev-deps",
        "version": "1.0.0",
        "dependencies": {
            "prod-dep": "1.0.0"
        },
        "devDependencies": {
            "jest": "27.0.0"
        }
    }"#;
    env.create_mock_package_with_content(lib_pkg_name, lib_pkg_version, lib_pkg_json_content);

    // 2. Setup package without devDependencies (for no-op check)
    let no_dev_pkg_name = "my-lib-no-dev-deps";
    let no_dev_pkg_version = "1.0.0";
    let no_dev_pkg_json_content = r#"{
        "name": "my-lib-no-dev-deps",
        "version": "1.0.0",
        "dependencies": {
            "prod-dep": "1.0.0"
        }
    }"#;
    env.create_mock_package_with_content(
        no_dev_pkg_name,
        no_dev_pkg_version,
        no_dev_pkg_json_content,
    );

    // 3. Act: Install the packages
    env.run_kley_command(&["install", lib_pkg_name])
        .assert()
        .success();
    env.run_kley_command(&["install", no_dev_pkg_name])
        .assert()
        .success();

    // 4. Assert: Check the stripped package
    let installed_path = env.project_dir.join(".kley").join(lib_pkg_name);
    assert!(installed_path.exists());

    let installed_pkg_json_content =
        fs::read_to_string(installed_path.join("package.json")).unwrap();
    assert!(
        !installed_pkg_json_content.contains("devDependencies"),
        "devDependencies should be stripped from the installed package.json"
    );
    assert!(
        installed_pkg_json_content.contains("\"prod-dep\": \"1.0.0\""),
        "production dependencies should be preserved in the installed package.json"
    );

    // 5. Assert: Check that the original package in the registry is untouched
    let original_path = env.kley_registry.join("packages").join(lib_pkg_name);
    let original_pkg_json_content = fs::read_to_string(original_path.join("package.json")).unwrap();
    assert!(
        original_pkg_json_content.contains("devDependencies"),
        "Original package in registry should retain its devDependencies"
    );

    // 6. Assert: Check the no-op package (the one without dev deps)
    let installed_no_dev_path = env.project_dir.join(".kley").join(no_dev_pkg_name);
    assert!(installed_no_dev_path.exists());
    let installed_no_dev_pkg_json_content =
        fs::read_to_string(installed_no_dev_path.join("package.json")).unwrap();
    assert!(
        !installed_no_dev_pkg_json_content.contains("devDependencies"),
        "Package with no devDependencies should not have them after install"
    );
    assert_eq!(
        installed_no_dev_pkg_json_content.trim(),
        no_dev_pkg_json_content.trim(),
        "File content should be identical for package with no dev deps"
    );
}
