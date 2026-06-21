mod common;

use kley::{package::PackageJson, utils::normalized_path};
use predicates::prelude::*;
use serde_json::json;
use std::fs;

use common::TestEnv;

///  RUST_LOG=debug cargo test --test install_command_test -- --nocapture
///
///```
///  let output = env.run_kley_command(&["install", pkg_name])
///     .output()
///     .unwrap();
/// tracing::info!("=== STDERR ===\n{}", String::from_utf8_lossy(&output.stderr));
/// tracing::info!("=== STDOUT ===\n{}", String::from_utf8_lossy(&output.stdout));
/// assert!(output.status.success());
///```
///
/// Checks that package.json contains a file: dependency pointing to .kley/<pkg_name>.
/// This is platform-independent: normalizes both the actual content and the expected
/// path to use forward slashes before comparison.
fn assert_pkg_json_has_file_dep(pkg_json_content: &str, pkg_name: &str) {
    let expected_path = format!("file:.kley/{}", pkg_name);
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
    env.create_mock_package_with_content(
        "my-package",
        "1.0.0",
        r#"{"name": "my-package", "version": "1.0.0", "dependencies": {"fake-dep": "1.0.0"}}"#,
    );
    env.setup_project_pm("npm");

    env.run_kley_command(&["install", "my-package"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done: my-package installed"));

    assert!(env.project_dir.join(".kley").join("my-package").exists());

    let mut kley_lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();

    kley_lock_content.retain(|c| !c.is_whitespace());
    // Note: no trailing `}` — a package with deps also snapshots "dependencies" into the entry (f-27)
    assert!(kley_lock_content.contains(r#""my-package":{"version":"1.0.0""#));

    let project_pkg_json_content =
        fs::read_to_string(env.project_dir.join("package.json")).unwrap();

    assert_pkg_json_has_file_dep(&project_pkg_json_content, "my-package");

    let pm_log_content = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(pm_log_content.contains("npm install"));
    assert!(pm_log_content.contains("--ignore-scripts"));
}

#[test]
fn test_install_command_pnpm_project() {
    let env = TestEnv::new();
    env.create_mock_package_with_content(
        "my-pnpm-package",
        "2.0.0",
        r#"{"name": "my-pnpm-package", "version": "2.0.0", "dependencies": {"fake-dep": "1.0.0"}}"#,
    );
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
    assert!(kley_lock_content.contains(r#""my-pnpm-package":{"version":"2.0.0""#));

    let project_pkg_json_content =
        fs::read_to_string(env.project_dir.join("package.json")).unwrap();

    assert_pkg_json_has_file_dep(&project_pkg_json_content, "my-pnpm-package");

    let pm_log_content = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(pm_log_content.contains("pnpm add"));
    assert!(pm_log_content.contains("--ignore-scripts"));
}

#[test]
fn test_install_command_yarn_project() {
    let env = TestEnv::new();
    env.create_mock_package_with_content(
        "my-yarn-package",
        "3.0.0",
        r#"{"name": "my-yarn-package", "version": "3.0.0", "dependencies": {"fake-dep": "1.0.0"}}"#,
    );
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
    assert!(kley_lock_content.contains(r#""my-yarn-package":{"version":"3.0.0""#));

    let project_pkg_json_content =
        fs::read_to_string(env.project_dir.join("package.json")).unwrap();

    assert_pkg_json_has_file_dep(&project_pkg_json_content, "my-yarn-package");

    let pm_log_content = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(pm_log_content.contains("yarn add"));
    assert!(pm_log_content.contains("--ignore-scripts"));
}

#[test]
fn test_install_command_kley_lock_pm_override() {
    let env = TestEnv::new();
    env.create_mock_package_with_content(
        "my-override-package",
        "4.0.0",
        r#"{"name": "my-override-package", "version": "4.0.0", "dependencies": {"fake-dep": "1.0.0"}}"#,
    );
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

    assert!(kley_lock_content.contains(r#""my-override-package":{"version":"4.0.0""#));
    assert!(kley_lock_content.contains(r#""packageManager":"pnpm""#)); // kley.lock should retain its PM setting

    let project_pkg_json_content =
        fs::read_to_string(env.project_dir.join("package.json")).unwrap();

    assert_pkg_json_has_file_dep(&project_pkg_json_content, "my-override-package");

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

#[test_log::test]
fn test_install_no_args_updates_all_packages() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: install 2 packages at v1.0.0, then publish v2.0.0 to registry
    let env = TestEnv::new();
    env.setup_project_pm("npm");

    // Create packages in registry at v1.0.0 with source files
    env.create_mock_package_with_content(
        "pkg-a",
        "1.0.0",
        r#"{"name": "pkg-a", "version": "1.0.0", "dependencies": {"fake": "1.0.0"}}"#,
    );
    env.create_mock_package_with_content(
        "pkg-b",
        "1.0.0",
        r#"{"name": "pkg-b", "version": "1.0.0", "dependencies": {"fake": "1.0.0"}}"#,
    );
    fs::write(
        env.kley_registry
            .join("packages")
            .join("pkg-a")
            .join("index.js"),
        "// pkg-a v1",
    )
    .unwrap();
    fs::write(
        env.kley_registry
            .join("packages")
            .join("pkg-b")
            .join("index.js"),
        "// pkg-b v1",
    )
    .unwrap();

    // Install both packages initially
    env.run_kley_command(&["install", "pkg-a"])
        .assert()
        .success();
    env.run_kley_command(&["install", "pkg-b"])
        .assert()
        .success();

    // Verify initial state: both at v1
    assert_eq!(
        fs::read_to_string(env.project_dir.join(".kley").join("pkg-a").join("index.js")).unwrap(),
        "// pkg-a v1"
    );
    assert_eq!(
        fs::read_to_string(env.project_dir.join(".kley").join("pkg-b").join("index.js")).unwrap(),
        "// pkg-b v1"
    );

    let pkg_json_a = PackageJson::get(&env.project_dir.join(".kley").join("pkg-a"))?;
    assert_eq!(pkg_json_a.version, "1.0.0");
    let pkg_json_b = PackageJson::get(&env.project_dir.join(".kley").join("pkg-b"))?;
    assert_eq!(pkg_json_b.version, "1.0.0");

    // Update registry to v2.0.0 with new content
    env.create_mock_package_with_content(
        "pkg-a",
        "2.0.0",
        r#"{"name": "pkg-a", "version": "2.0.0", "dependencies": {"fake": "1.0.0"}}"#,
    );
    env.create_mock_package_with_content(
        "pkg-b",
        "2.0.0",
        r#"{"name": "pkg-b", "version": "2.0.0", "dependencies": {"fake": "1.0.0"}}"#,
    );
    fs::write(
        env.kley_registry
            .join("packages")
            .join("pkg-a")
            .join("index.js"),
        "// pkg-a v2",
    )
    .unwrap();
    fs::write(
        env.kley_registry
            .join("packages")
            .join("pkg-b")
            .join("index.js"),
        "// pkg-b v2",
    )
    .unwrap();

    env.create_kley_lock(r#"{"packageManager": "npm", "packages": {"pkg-a": {"version": "2.0.0"}, "pkg-b": {"version": "2.0.0"}}}"#);

    // Act: run `kley install` with no args — should update all packages
    env.run_kley_command(&["install"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done"));

    // Assert: both packages updated to v2.0.0
    assert_eq!(
        fs::read_to_string(env.project_dir.join(".kley").join("pkg-a").join("index.js")).unwrap(),
        "// pkg-a v2"
    );
    assert_eq!(
        fs::read_to_string(env.project_dir.join(".kley").join("pkg-b").join("index.js")).unwrap(),
        "// pkg-b v2"
    );

    let pkg_json_a = PackageJson::get(&env.project_dir.join(".kley").join("pkg-a"))?;
    assert_eq!(pkg_json_a.version, "2.0.0");
    let pkg_json_b = PackageJson::get(&env.project_dir.join(".kley").join("pkg-b"))?;
    assert_eq!(pkg_json_b.version, "2.0.0");

    // Assert: kley.lock reflects v2.0.0 for both packages
    let lock: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(env.project_dir.join("kley.lock")).unwrap())
            .unwrap();
    assert_eq!(lock["packages"]["pkg-a"]["version"], "2.0.0");
    assert_eq!(lock["packages"]["pkg-b"]["version"], "2.0.0");

    // Assert: files were updated in node_modules (fast path copies directly)
    let node_modules_a = env
        .project_dir
        .join("node_modules")
        .join("pkg-a")
        .join("index.js");
    let node_modules_b = env
        .project_dir
        .join("node_modules")
        .join("pkg-b")
        .join("index.js");

    assert_eq!(
        fs::read_to_string(&node_modules_a).unwrap(),
        "// pkg-a v2",
        "pkg-a should be updated in node_modules"
    );
    assert_eq!(
        fs::read_to_string(&node_modules_b).unwrap(),
        "// pkg-b v2",
        "pkg-b should be updated in node_modules"
    );

    Ok(())
}

#[test_log::test]
fn test_install_no_args_no_lockfile_warns() {
    // Arrange: project with no kley.lock at all
    let env = TestEnv::new();
    env.setup_project_pm("npm");

    // Act: `kley install` with no args — nothing to update
    env.run_kley_command(&["install"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Warning"));

    // Assert: no PM call, no .kley directory created
    assert!(!env.project_dir.join("pm.log").exists());
    assert!(!env.project_dir.join(".kley").exists());
}

#[test_log::test]
fn test_install_no_args_empty_lockfile_warns() {
    // Arrange: project with empty kley.lock
    let env = TestEnv::new();
    env.setup_project_pm("npm");
    env.create_kley_lock(r#"{"packages": {}}"#);

    // Act: `kley install` with no args — no packages to update
    env.run_kley_command(&["install"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Warning"));

    // Assert: no PM call, no .kley directory created
    assert!(!env.project_dir.join("pm.log").exists());
}

// ─── f-20: --dev flag tests ─────────────────────────────────────────

/// Checks that package.json contains a file: dependency in devDependencies.
fn assert_pkg_json_has_dev_dep(pkg_name: &str, project_dir: &std::path::Path) {
    let package_json = PackageJson::get(project_dir).unwrap();
    let dev_dependencies = package_json.dev_dependencies.clone().unwrap_or(json!({}));
    assert!(
        package_json.dev_dependencies.is_some(),
        "package.json should have devDependencies section. Content:\n{:?}",
        package_json
    );
    assert!(
        dev_dependencies.get(pkg_name).is_some(),
        "devDependencies should contain '{}'. Content:\n{:?}",
        pkg_name,
        &package_json
    );

    let expected_path = format!("file:.kley/{}", pkg_name);

    assert!(
        dev_dependencies
            .get(pkg_name)
            .unwrap_or(&json!(""))
            .as_str()
            .unwrap()
            .contains(&expected_path),
        "devDependencies['{}'] should contain path '{}'. Content:\n{:?}",
        pkg_name,
        expected_path,
        package_json,
    );
}

#[test_log::test]
fn test_install_dev_flag_npm() -> Result<(), Box<dyn std::error::Error>> {
    let pkg_name = "dev-pkg-npm";
    let env = TestEnv::new();
    env.create_mock_package_with_content(
        pkg_name,
        "1.0.0",
        r#"{"name": "dev-pkg-npm", "version": "1.0.0", "dependencies": {"fake-dep": "1.0.0"}}"#,
    );
    env.setup_project_pm("npm");

    env.run_kley_command(&["install", "--dev", pkg_name])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done: dev-pkg-npm installed"));

    // Package copied to .kley
    assert!(env.project_dir.join(".kley").join(pkg_name).exists());

    // kley.lock updated
    let mut lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();
    lock_content.retain(|c| !c.is_whitespace());
    assert!(lock_content.contains(r#""dev-pkg-npm":{"version":"1.0.0""#));

    // PM was called with --save-dev
    let pm_log = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(
        pm_log.contains("--save-dev"),
        "npm should be called with --save-dev. pm.log:\n{}",
        pm_log
    );
    assert!(
        pm_log.contains("--ignore-scripts"),
        "npm should be called with --ignore-scripts. pm.log:\n{}",
        pm_log
    );

    // package.json has the dep in devDependencies
    assert_pkg_json_has_dev_dep(pkg_name, &env.project_dir);

    Ok(())
}

#[test_log::test]
fn test_install_dev_flag_pnpm() {
    let pkg_name = "dev-pkg-pnpm";
    let env = TestEnv::new();
    env.create_mock_package_with_content(
        pkg_name,
        "1.0.0",
        r#"{"name": "dev-pkg-pnpm", "version": "1.0.0", "dependencies": {"fake-dep": "1.0.0"}}"#,
    );
    env.setup_project_pm("pnpm");

    env.run_kley_command(&["install", "--dev", "dev-pkg-pnpm"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done: dev-pkg-pnpm installed"));

    // PM was called with -D
    let pm_log = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(
        pm_log.contains("-D"),
        "pnpm should be called with -D. pm.log:\n{}",
        pm_log
    );

    // package.json has the dep in devDependencies
    assert_pkg_json_has_dev_dep(pkg_name, &env.project_dir);
}

#[test_log::test]
fn test_install_dev_flag_yarn() {
    let env = TestEnv::new();
    env.create_mock_package_with_content(
        "dev-pkg-yarn",
        "1.0.0",
        r#"{"name": "dev-pkg-yarn", "version": "1.0.0", "dependencies": {"fake-dep": "1.0.0"}}"#,
    );
    env.setup_project_pm("yarn");

    env.run_kley_command(&["install", "--dev", "dev-pkg-yarn"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done: dev-pkg-yarn installed"));

    // PM was called with --dev
    let pm_log = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(
        pm_log.contains("--dev"),
        "yarn should be called with --dev. pm.log:\n{}",
        pm_log
    );

    // package.json has the dep in devDependencies
    assert_pkg_json_has_dev_dep("dev-pkg-yarn", &env.project_dir);
}

#[test_log::test]
fn test_install_short_d_flag() {
    let env = TestEnv::new();
    env.create_mock_package_with_content(
        "dev-pkg-short",
        "1.0.0",
        r#"{"name": "dev-pkg-short", "version": "1.0.0", "dependencies": {"fake-dep": "1.0.0"}}"#,
    );
    env.setup_project_pm("npm");

    env.run_kley_command(&["install", "-D", "dev-pkg-short"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done: dev-pkg-short installed"));

    // Same behavior as --dev: npm gets --save-dev
    let pm_log = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(
        pm_log.contains("--save-dev"),
        "npm should be called with --save-dev. pm.log:\n{}",
        pm_log
    );

    // package.json has the dep in devDependencies
    assert_pkg_json_has_dev_dep("dev-pkg-short", &env.project_dir);
}

#[test_log::test]
fn test_install_without_dev_goes_to_dependencies() {
    let env = TestEnv::new();
    env.create_mock_package_with_content(
        "prod-pkg",
        "1.0.0",
        r#"{"name": "prod-pkg", "version": "1.0.0", "dependencies": {"fake-dep": "1.0.0"}}"#,
    );
    env.setup_project_pm("npm");

    env.run_kley_command(&["install", "prod-pkg"])
        .assert()
        .success();

    // package.json should have the dep in dependencies, NOT devDependencies
    let pkg_json = fs::read_to_string(env.project_dir.join("package.json")).unwrap();
    let pkg: serde_json::Value = serde_json::from_str(&pkg_json).unwrap();

    assert!(
        pkg.get("dependencies").is_some(),
        "package.json should have dependencies section. Content:\n{}",
        pkg_json
    );
    assert!(
        pkg["dependencies"].get("prod-pkg").is_some(),
        "dependencies should contain 'prod-pkg'. Content:\n{}",
        pkg_json
    );
    assert!(
        pkg.get("devDependencies").is_none() || pkg["devDependencies"].get("prod-pkg").is_none(),
        "prod-pkg should NOT appear in devDependencies. Content:\n{}",
        pkg_json
    );

    // PM should NOT have received --save-dev
    let pm_log = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(
        !pm_log.contains("--save-dev") && !pm_log.contains("-D"),
        "npm should NOT be called with --save-dev or -D for regular install. pm.log:\n{}",
        pm_log
    );
}

#[test_log::test]
fn test_install_no_args_preserves_dev_and_prod_deps() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: project with one prod dep and one dev dep in package.json + kley.lock
    let env = TestEnv::new();
    // Packages have a real dependency so slow path is triggered (snapshot is empty, deps differ)
    env.create_mock_package_with_content(
        "prod-pkg",
        "1.0.0",
        r#"{"name": "prod-pkg", "version": "1.0.0", "dependencies": {"lodash": "^4.0.0"}}"#,
    );
    env.create_mock_package_with_content(
        "dev-pkg",
        "1.0.0",
        r#"{"name": "dev-pkg", "version": "1.0.0", "dependencies": {"lodash": "^4.0.0"}}"#,
    );
    env.setup_project_pm("npm");

    // Pre-populate package.json with prod-pkg in dependencies, dev-pkg in devDependencies
    let pkg_json_path = env.project_dir.join("package.json");
    let pkg_json_content = r#"{
  "name": "my-project",
  "version": "1.0.0",
  "dependencies": {
    "prod-pkg": "file:.kley/prod-pkg"
  },
  "devDependencies": {
    "dev-pkg": "file:.kley/dev-pkg"
  }
}"#;

    fs::write(&pkg_json_path, pkg_json_content)?;

    // Create kley.lock referencing both packages
    env.create_kley_lock(
        r#"{"packageManager": "npm", "packages": {"prod-pkg": {"version": "1.0.0"}, "dev-pkg": {"version": "1.0.0"}}}"#,
    );

    // Act: `kley install` with no args — should detect dev status from package.json
    env.run_kley_command(&["install"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done"));

    // Assert: both packages installed
    assert!(env.project_dir.join(".kley").join("prod-pkg").exists());
    assert!(env.project_dir.join(".kley").join("dev-pkg").exists());

    // Assert: PM was called twice — once WITHOUT --save-dev (prod-pkg), once WITH --save-dev (dev-pkg)
    let pm_log = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    let lines: Vec<&str> = pm_log.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "PM should be called once per package. pm.log:\n{}",
        pm_log
    );

    let prod_call = lines
        .iter()
        .find(|l| l.contains("prod-pkg"))
        .unwrap_or_else(|| panic!("No PM call for prod-pkg. pm.log:\n{}", pm_log));
    let dev_call = lines
        .iter()
        .find(|l| l.contains("dev-pkg"))
        .unwrap_or_else(|| panic!("No PM call for dev-pkg. pm.log:\n{}", pm_log));

    assert!(
        !prod_call.contains("--save-dev") && !prod_call.contains("-D"),
        "prod-pkg should be installed WITHOUT dev flag. Call: {}",
        prod_call
    );
    assert!(
        dev_call.contains("--save-dev"),
        "dev-pkg should be installed WITH --save-dev. Call: {}",
        dev_call
    );

    Ok(())
}

#[test_log::test]
fn test_install_dev_flag_without_package_name_fails() {
    let env = TestEnv::new();
    env.setup_project_pm("npm");

    env.run_kley_command(&["install", "--dev"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--dev flag requires a package name",
        ));
}

// ─── f-26: Skip PM for dependency-less packages ───────────────────────

#[test_log::test]
fn test_fast_path_no_deps_skips_pm() {
    let env = TestEnv::new();
    let pkg_name = "pkg-no-deps";
    env.create_mock_package_with_content(
        pkg_name,
        "1.0.0",
        r#"{"name": "pkg-no-deps", "version": "1.0.0"}"#,
    );
    env.setup_project_pm("npm");

    env.run_kley_command(&["install", pkg_name])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Package has no dependencies, skipping package manager",
        ))
        .stdout(predicate::str::contains("Done: pkg-no-deps installed"));

    // Assert: PM was NOT called
    assert!(
        !env.project_dir.join("pm.log").exists(),
        "pm.log should not exist for dependency-less install"
    );

    // Assert: package.json is updated correctly
    let project_pkg_json_content =
        fs::read_to_string(env.project_dir.join("package.json")).unwrap();
    assert_pkg_json_has_file_dep(&project_pkg_json_content, pkg_name);

    // Assert: node_modules/<pkg> is a symlink to .kley/<pkg>
    let node_modules_pkg = env.project_dir.join("node_modules").join(pkg_name);
    assert!(
        node_modules_pkg.is_symlink(),
        "node_modules/{} should be a symlink",
        pkg_name
    );

    let link_target = normalized_path(&fs::read_link(&node_modules_pkg).unwrap(), None);
    let kley_cache_pkg = normalized_path(&env.project_dir.join(".kley").join(pkg_name), None);
    assert_eq!(link_target, kley_cache_pkg);
}

#[test_log::test]
fn test_fast_path_no_deps_dev_flag() {
    let env = TestEnv::new();
    let pkg_name = "pkg-no-deps-dev";
    env.create_mock_package_with_content(
        pkg_name,
        "1.0.0",
        r#"{"name": "pkg-no-deps-dev", "version": "1.0.0"}"#,
    );
    env.setup_project_pm("npm");

    env.run_kley_command(&["install", "-D", pkg_name])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Package has no dependencies, skipping package manager",
        ));

    assert!(!env.project_dir.join("pm.log").exists());
    assert_pkg_json_has_dev_dep(pkg_name, &env.project_dir);
}

#[test_log::test]
fn test_fast_path_no_deps_no_save() {
    let env = TestEnv::new();
    let pkg_name = "pkg-no-deps-no-save";
    env.create_mock_package_with_content(
        pkg_name,
        "1.0.0",
        r#"{"name": "pkg-no-deps-no-save", "version": "1.0.0"}"#,
    );
    env.setup_project_pm("npm");
    let original_pkg_json = fs::read_to_string(env.project_dir.join("package.json")).unwrap();

    env.run_kley_command(&["install", "--no-save", pkg_name])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Package has no dependencies, skipping package manager",
        ));

    assert!(!env.project_dir.join("pm.log").exists());

    // Assert: package.json is unchanged
    let current_pkg_json = fs::read_to_string(env.project_dir.join("package.json")).unwrap();
    assert_eq!(
        original_pkg_json, current_pkg_json,
        "package.json should not be modified with --no-save"
    );

    // Assert: symlink is still created
    let node_modules_pkg = env.project_dir.join("node_modules").join(pkg_name);
    assert!(
        node_modules_pkg.is_symlink(),
        "symlink should still be created with --no-save"
    );
}

#[test_log::test]
fn test_slow_path_with_deps_calls_pm() {
    let env = TestEnv::new();
    let pkg_name = "pkg-with-deps";
    env.create_mock_package_with_content(
        pkg_name,
        "1.0.0",
        r#"{"name": "pkg-with-deps", "version": "1.0.0", "dependencies": {"lodash": "1.0.0"}}"#,
    );
    env.setup_project_pm("npm");

    env.run_kley_command(&["install", pkg_name])
        .assert()
        .success();

    // Assert: PM WAS called
    let pm_log = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(pm_log.contains("npm install"));
}

#[test_log::test]
fn test_slow_path_with_peer_deps_calls_pm() {
    let env = TestEnv::new();
    let pkg_name = "pkg-with-peer-deps";
    env.create_mock_package_with_content(
        pkg_name,
        "1.0.0",
        r#"{"name": "pkg-with-peer-deps", "version": "1.0.0", "peerDependencies": {"react": "18.0.0"}}"#,
    );
    env.setup_project_pm("npm");

    env.run_kley_command(&["install", pkg_name])
        .assert()
        .success();

    // Assert: PM WAS called
    let pm_log = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(pm_log.contains("npm install"));
}

#[test_log::test]
fn test_dependencies_snapshotting() {
    let env = TestEnv::new();
    env.create_mock_package_with_content(
        "my-package",
        "1.0.0",
        r#"{"name": "my-package", "version": "1.0.0", "dependencies": {"left-pad": "^1.0.0"}, "peerDependencies": {"react": "^18.0.0"}}"#,
    );
    env.setup_project_pm("npm");

    env.run_kley_command(&["install", "my-package"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done: my-package installed"));

    assert!(env.project_dir.join(".kley").join("my-package").exists());

    let kley_lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();
    let lock: serde_json::Value = serde_json::from_str(&kley_lock_content).unwrap();
    let info = &lock["packages"]["my-package"];

    assert_eq!(info["dependencies"]["left-pad"], "^1.0.0");
    assert_eq!(info["peerDependencies"]["react"], "^18.0.0");
}

/// Case C (happy fast path): node_modules/<pkg> is a directory or doesn't exist,
/// dependencies unchanged. PM should be skipped, files copied directly.
#[test_log::test]
fn test_fast_path_case_c_skips_pm_when_deps_unchanged() {
    let env = TestEnv::new();
    let pkg_name = "fast-pkg";
    let pkg_json =
        r#"{"name": "fast-pkg", "version": "1.0.0", "dependencies": {"lodash": "^4.0.0"}}"#;

    env.create_mock_package_with_content(pkg_name, "1.0.0", pkg_json);
    env.setup_project_pm("npm");

    // 1. First install — slow path, PM is called, snapshot saved
    env.run_kley_command(&["install", pkg_name])
        .assert()
        .success();

    let pm_log_after_first = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(
        pm_log_after_first.contains("npm install"),
        "First install should call PM"
    );

    // Verify snapshot was saved
    let lock: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(env.project_dir.join("kley.lock")).unwrap())
            .unwrap();
    assert_eq!(
        lock["packages"][pkg_name]["dependencies"]["lodash"], "^4.0.0",
        "Snapshot should be saved after first install"
    );

    // 2. Update source file in registry (simulating kley publish with code change, same deps)
    let registry_pkg_dir = env.kley_registry.join("packages").join(pkg_name);
    fs::write(registry_pkg_dir.join("index.js"), "// v2 code").unwrap();

    // 3. Second install — fast path should trigger, PM NOT called
    env.run_kley_command(&["install", pkg_name])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done: fast-pkg installed"));

    let pm_log_after_second = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    let pm_calls_count = pm_log_after_second.lines().count();

    assert_eq!(
        pm_calls_count, 1,
        "PM should NOT be called on second install when deps unchanged. pm.log:\n{}",
        pm_log_after_second
    );

    // 4. Verify the updated file was copied to node_modules
    let node_modules_pkg = env.project_dir.join("node_modules").join(pkg_name);
    let index_content = fs::read_to_string(node_modules_pkg.join("index.js")).unwrap();

    assert_eq!(
        index_content, "// v2 code",
        "Updated source should be copied to node_modules"
    );
}

/// Case A: node_modules/<pkg> is a symlink pointing to .kley/<pkg>.
/// Fast path should do nothing — the symlink is already correct.
#[test_log::test]
#[cfg(unix)]
fn test_fast_path_case_a_symlink_to_kley_cache() {
    use std::os::unix::fs::symlink;

    let env = TestEnv::new();
    let pkg_name = "symlink-pkg";
    let pkg_json =
        r#"{"name": "symlink-pkg", "version": "1.0.0", "dependencies": {"lodash": "^4.0.0"}}"#;

    env.create_mock_package_with_content(pkg_name, "1.0.0", pkg_json);
    env.setup_project_pm("npm");

    // 1. First install — slow path, PM is called, snapshot saved
    env.run_kley_command(&["install", pkg_name])
        .assert()
        .success();

    let pm_log_after_first = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert_eq!(
        pm_log_after_first.lines().count(),
        1,
        "First install should call PM once"
    );

    // 2. Manually create symlink: node_modules/<pkg> -> .kley/<pkg>
    //    (simulating what modern npm does or what kley link creates)
    let node_modules = env.project_dir.join("node_modules");
    let node_modules_pkg = node_modules.join(pkg_name);
    let kley_cache_pkg = env.project_dir.join(".kley").join(pkg_name);

    // Remove existing directory created by mock npm
    if node_modules_pkg.exists() {
        fs::remove_dir_all(&node_modules_pkg).unwrap();
    }
    fs::create_dir_all(&node_modules).unwrap();

    // Create symlink
    symlink(&kley_cache_pkg, &node_modules_pkg).unwrap();
    assert!(node_modules_pkg.is_symlink(), "Should be a symlink");

    // 3. Update source in registry
    let registry_pkg_dir = env.kley_registry.join("packages").join(pkg_name);
    fs::write(registry_pkg_dir.join("index.js"), "// v2 code").unwrap();

    // 4. Second install — fast path Case A: symlink to .kley, do nothing
    env.run_kley_command(&["install", pkg_name])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done: symlink-pkg installed"));

    // PM should NOT be called again
    let pm_log_after_second = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert_eq!(
        pm_log_after_second.lines().count(),
        1,
        "PM should NOT be called when symlink points to .kley cache. pm.log:\n{}",
        pm_log_after_second
    );

    // Symlink should still exist and point to .kley
    assert!(
        node_modules_pkg.is_symlink(),
        "Symlink should still exist after install"
    );

    // The updated file should be accessible via symlink (since .kley was updated)
    let index_content = fs::read_to_string(node_modules_pkg.join("index.js")).unwrap();
    assert_eq!(
        index_content, "// v2 code",
        "Updated source should be accessible via symlink"
    );
}

/// Case B: node_modules/<pkg> is a symlink pointing to an unknown location.
/// Should fall back to slow path (run PM).
#[test_log::test]
#[cfg(unix)]
fn test_fast_path_case_b_symlink_to_unknown_falls_back_to_pm() {
    use std::os::unix::fs::symlink;

    let env = TestEnv::new();
    let pkg_name = "unknown-symlink-pkg";
    let pkg_json = r#"{"name": "unknown-symlink-pkg", "version": "1.0.0", "dependencies": {"lodash": "^4.0.0"}}"#;

    env.create_mock_package_with_content(pkg_name, "1.0.0", pkg_json);
    env.setup_project_pm("npm");

    // 1. First install — slow path
    env.run_kley_command(&["install", pkg_name])
        .assert()
        .success();

    let pm_log_after_first = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert_eq!(
        pm_log_after_first.lines().count(),
        1,
        "First install should call PM once"
    );

    // 2. Manually create symlink to an external location (simulating npm link)
    let node_modules = env.project_dir.join("node_modules");
    let node_modules_pkg = node_modules.join(pkg_name);

    // Create external directory (somewhere else, not .kley)
    let external_dir = env.temp_dir.path().join("external-packages").join(pkg_name);
    fs::create_dir_all(&external_dir).unwrap();
    fs::write(external_dir.join("package.json"), pkg_json).unwrap();
    fs::write(external_dir.join("index.js"), "// external").unwrap();

    // Remove existing directory and create symlink to external
    if node_modules_pkg.exists() {
        fs::remove_dir_all(&node_modules_pkg).unwrap();
    }
    symlink(&external_dir, &node_modules_pkg).unwrap();
    assert!(node_modules_pkg.is_symlink(), "Should be a symlink");

    // Verify symlink points to external, not .kley
    let link_target = fs::read_link(&node_modules_pkg).unwrap();
    assert!(
        !link_target.to_string_lossy().contains(".kley"),
        "Symlink should point to external location, not .kley"
    );

    // 3. Second install — should fall back to slow path because symlink is unknown
    env.run_kley_command(&["install", pkg_name])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Done: unknown-symlink-pkg installed",
        ));

    // PM SHOULD be called again (slow path fallback)
    let pm_log_after_second = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert_eq!(
        pm_log_after_second.lines().count(),
        2,
        "PM SHOULD be called when symlink points to unknown location. pm.log:\n{}",
        pm_log_after_second
    );
}

/// `kley install` (no args) must restore the direct symlink for linked packages.
/// Simulates: user ran `npm install`, which overwrote the link with a real directory,
/// then runs `kley install` to restore the symlink to the source directory.
#[test]
fn test_install_all_restores_linked_symlink() -> Result<(), Box<dyn std::error::Error>> {
    use assert_cmd::Command;
    use tempfile::tempdir;

    let home = tempdir()?;
    let lib = tempdir()?;
    let app = tempdir()?;

    // Publish and link
    fs::write(
        lib.path().join("package.json"),
        r#"{"name": "test-lib", "version": "1.0.0"}"#,
    )?;
    fs::write(lib.path().join("index.js"), "// lib source")?;
    fs::write(
        app.path().join("package.json"),
        r#"{"name": "app", "version": "1.0.0"}"#,
    )?;

    Command::cargo_bin("kley")?
        .env("KLEY_HOME", home.path())
        .arg("publish")
        .current_dir(lib.path())
        .assert()
        .success();

    Command::cargo_bin("kley")?
        .env("KLEY_HOME", home.path())
        .args(["link", "test-lib"])
        .current_dir(app.path())
        .assert()
        .success();

    // Confirm symlink exists
    let symlink = app.path().join("node_modules/test-lib");
    assert!(
        symlink.is_symlink(),
        "precondition: symlink should exist after link"
    );

    // Simulate npm install overwriting the symlink with a real directory
    fs::remove_file(&symlink)?;
    fs::create_dir_all(&symlink)?;
    fs::write(symlink.join("index.js"), "// npm copy")?;
    assert!(
        !symlink.is_symlink(),
        "precondition: symlink was replaced by directory"
    );

    // Run kley install (no args) — should restore the link symlink
    Command::cargo_bin("kley")?
        .env("KLEY_HOME", home.path())
        .arg("install")
        .current_dir(app.path())
        .assert()
        .success();

    // Symlink is restored, pointing to lib source
    assert!(
        symlink.is_symlink(),
        "kley install should restore symlink for linked package"
    );
    let target = fs::canonicalize(fs::read_link(&symlink)?)?;
    let src = fs::canonicalize(lib.path())?;
    assert_eq!(
        target, src,
        "restored symlink should point to lib source directory"
    );

    // Content is live from source again
    let content = fs::read_to_string(symlink.join("index.js"))?;
    assert_eq!(content, "// lib source");

    Ok(())
}
