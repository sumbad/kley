mod common;

use kley::package::PackageJson;
use predicates::prelude::*;
use serde_json::json;
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

#[test_log::test]
fn test_install_no_args_updates_all_packages() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: install 2 packages at v1.0.0, then publish v2.0.0 to registry
    let env = TestEnv::new();
    env.setup_project_pm("npm");

    // Create packages in registry at v1.0.0 with source files
    env.create_mock_registry_package("pkg-a", "1.0.0");
    env.create_mock_registry_package("pkg-b", "1.0.0");
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
    env.create_mock_registry_package("pkg-a", "2.0.0");
    env.create_mock_registry_package("pkg-b", "2.0.0");
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

    // Capture pm.log before no-args install (should not change after)
    let pm_log_before = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();

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

    // Assert: PM call during no-args install
    let pm_log_after = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();

    assert_ne!(
        pm_log_before, pm_log_after,
        "PM should be called during `kley install` with no args"
    );

    let new_calls = pm_log_after.lines().count() - pm_log_before.lines().count();
    assert_eq!(new_calls, 2, "PM should be called once per package");

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

    let kley_path = project_dir.join(".kley").join(pkg_name);
    let expected_path = kley_path.to_string_lossy().replace('\\', "/");

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
    env.create_mock_registry_package(pkg_name, "1.0.0");
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
    assert!(lock_content.contains(r#""dev-pkg-npm":{"version":"1.0.0"}"#));

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
    env.create_mock_registry_package(pkg_name, "1.0.0");
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
//
#[test_log::test]
fn test_install_dev_flag_yarn() {
    let env = TestEnv::new();
    env.create_mock_registry_package("dev-pkg-yarn", "1.0.0");
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
    env.create_mock_registry_package("dev-pkg-short", "1.0.0");
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
    env.create_mock_registry_package("prod-pkg", "1.0.0");
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
    env.create_mock_registry_package("prod-pkg", "1.0.0");
    env.create_mock_registry_package("dev-pkg", "1.0.0");
    env.setup_project_pm("npm");

    // Pre-populate package.json with prod-pkg in dependencies, dev-pkg in devDependencies
    let pkg_json_path = env.project_dir.join("package.json");
    let pkg_json_content = r#"{{
  "name": "my-project",
  "version": "1.0.0",
  "dependencies": {{
    "prod-pkg": "file:.kley/prod-pkg"
  }},
  "devDependencies": {{
    "dev-pkg": "file:.kley/dev-pkg"
  }}
}}"#;

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
