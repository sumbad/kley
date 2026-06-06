mod common;

use predicates::prelude::*;
use std::fs;

use common::TestEnv;

// ─── f-33: --no-save flag tests ─────────────────────────────────────────────

/// Asserts that package.json does NOT contain any dependency entry for `pkg_name`.
fn assert_pkg_json_has_no_dep(pkg_name: &str, project_dir: &std::path::Path) {
    let pkg_json_content = fs::read_to_string(project_dir.join("package.json")).unwrap();
    let pkg: serde_json::Value = serde_json::from_str(&pkg_json_content).unwrap();

    let in_deps = pkg
        .get("dependencies")
        .and_then(|d| d.get(pkg_name))
        .is_some();
    let in_dev_deps = pkg
        .get("devDependencies")
        .and_then(|d| d.get(pkg_name))
        .is_some();

    assert!(
        !in_deps && !in_dev_deps,
        "package.json should NOT contain '{}' in any dependency section. Content:\n{}",
        pkg_name,
        pkg_json_content
    );
}

/// --no-save with npm: PM receives --no-save, package.json untouched, kley.lock updated.
#[test_log::test]
fn test_install_no_save_npm() {
    let env = TestEnv::new();
    env.create_mock_registry_package("no-save-pkg", "1.0.0");
    env.setup_project_pm("npm");

    env.run_kley_command(&["install", "no-save-pkg", "--no-save"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done: no-save-pkg installed"));

    // Package copied to .kley/<pkg>
    assert!(env.project_dir.join(".kley").join("no-save-pkg").exists());

    // kley.lock updated normally
    let mut lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();
    lock_content.retain(|c| !c.is_whitespace());
    assert!(
        lock_content.contains(r#""no-save-pkg":{"version":"1.0.0"}"#),
        "kley.lock should be updated. Content:\n{}",
        lock_content
    );

    // PM called with --no-save
    let pm_log = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(
        pm_log.contains("--no-save"),
        "npm should receive --no-save. pm.log:\n{}",
        pm_log
    );

    // package.json must NOT have the dependency added
    assert_pkg_json_has_no_dep("no-save-pkg", &env.project_dir);
}

/// --no-save with pnpm: PM receives --save=false, package.json untouched, kley.lock updated.
#[test_log::test]
fn test_install_no_save_pnpm() {
    let env = TestEnv::new();
    env.create_mock_registry_package("no-save-pnpm-pkg", "1.0.0");
    env.setup_project_pm("pnpm");

    env.run_kley_command(&["install", "no-save-pnpm-pkg", "--no-save"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done: no-save-pnpm-pkg installed"));

    // Package copied to .kley/<pkg>
    assert!(
        env.project_dir
            .join(".kley")
            .join("no-save-pnpm-pkg")
            .exists()
    );

    // kley.lock updated
    let mut lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();
    lock_content.retain(|c| !c.is_whitespace());
    assert!(
        lock_content.contains(r#""no-save-pnpm-pkg":{"version":"1.0.0"}"#),
        "kley.lock should be updated. Content:\n{}",
        lock_content
    );

    // PM called with --save=false
    let pm_log = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(
        pm_log.contains("--save=false"),
        "pnpm should receive --save=false. pm.log:\n{}",
        pm_log
    );

    // package.json must NOT have the dependency added
    assert_pkg_json_has_no_dep("no-save-pnpm-pkg", &env.project_dir);
}

/// --no-save with Yarn v1: documented limitation — package.json IS still modified.
/// Yarn v1 has no --no-save equivalent. PM should NOT receive any --no-save flag.
#[test_log::test]
fn test_install_no_save_yarn_modifies_package_json() {
    let env = TestEnv::new();
    env.create_mock_registry_package("no-save-yarn-pkg", "1.0.0");
    env.setup_project_pm("yarn");

    env.run_kley_command(&["install", "no-save-yarn-pkg", "--no-save"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done: no-save-yarn-pkg installed"));

    // kley.lock updated
    let mut lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();
    lock_content.retain(|c| !c.is_whitespace());
    assert!(
        lock_content.contains(r#""no-save-yarn-pkg":{"version":"1.0.0"}"#),
        "kley.lock should be updated. Content:\n{}",
        lock_content
    );

    // Yarn should NOT receive --no-save (it doesn't support it)
    let pm_log = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(
        !pm_log.contains("--no-save") && !pm_log.contains("--save=false"),
        "yarn should NOT receive --no-save or --save=false. pm.log:\n{}",
        pm_log
    );
}

/// --no-save combined with -D: npm receives both --no-save and --save-dev.
/// package.json remains untouched (--no-save takes precedence in npm).
#[test_log::test]
fn test_install_no_save_with_dev_flag_npm() {
    let env = TestEnv::new();
    env.create_mock_registry_package("no-save-dev-pkg", "1.0.0");
    env.setup_project_pm("npm");

    env.run_kley_command(&["install", "no-save-dev-pkg", "--no-save", "-D"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done: no-save-dev-pkg installed"));

    // PM receives --no-save
    let pm_log = fs::read_to_string(env.project_dir.join("pm.log")).unwrap();
    assert!(
        pm_log.contains("--no-save"),
        "npm should receive --no-save. pm.log:\n{}",
        pm_log
    );

    // package.json must NOT have the dependency added
    assert_pkg_json_has_no_dep("no-save-dev-pkg", &env.project_dir);
}

/// kley remove after --no-save install: removes .kley/<pkg> and lockfile entry,
/// does NOT attempt to modify package.json (entry was never added).
#[test_log::test]
fn test_remove_after_no_save_install() {
    let env = TestEnv::new();
    env.create_mock_registry_package("removable-pkg", "1.0.0");
    env.setup_project_pm("npm");

    // Install with --no-save
    env.run_kley_command(&["install", "removable-pkg", "--no-save"])
        .assert()
        .success();

    // Remove
    env.run_kley_command(&["remove", "removable-pkg"])
        .assert()
        .success();

    // .kley/<pkg> removed
    assert!(
        !env.project_dir.join(".kley").join("removable-pkg").exists(),
        ".kley/removable-pkg should be removed"
    );

    // kley.lock entry removed
    let lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();
    assert!(
        !lock_content.contains("removable-pkg"),
        "kley.lock should not contain removable-pkg after remove. Content:\n{}",
        lock_content
    );

    // package.json should not contain removable-pkg in any dependency section
    // (it was never added by --no-save, and remove should not have added it either)
    assert_pkg_json_has_no_dep("removable-pkg", &env.project_dir);
}

/// kley install (no args) re-installs packages originally installed with --no-save.
#[test_log::test]
fn test_install_no_args_reinstalls_no_save_packages() {
    let env = TestEnv::new();
    env.create_mock_registry_package("reinstall-pkg", "1.0.0");
    env.setup_project_pm("npm");

    // Install with --no-save
    env.run_kley_command(&["install", "reinstall-pkg", "--no-save"])
        .assert()
        .success();

    // Verify kley.lock was created with the package
    let lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();
    assert!(
        lock_content.contains("reinstall-pkg"),
        "kley.lock should contain reinstall-pkg"
    );

    // Remove node_modules to simulate fresh project clone
    let node_modules_pkg = env.project_dir.join("node_modules").join("reinstall-pkg");
    if node_modules_pkg.exists() {
        fs::remove_dir_all(&node_modules_pkg).unwrap();
    }

    // kley install (no args) should reinstall from lockfile
    env.run_kley_command(&["install"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done"));

    // Package should be back in .kley
    assert!(
        env.project_dir.join(".kley").join("reinstall-pkg").exists(),
        ".kley/reinstall-pkg should be reinstalled"
    );
}
