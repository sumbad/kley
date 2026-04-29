mod common;

use predicates::prelude::*;
use std::fs;

use common::TestEnv;

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
    tracing::info!("kley_lock_content: {}", kley_lock_content);

    kley_lock_content.retain(|c| !c.is_whitespace());
    assert!(kley_lock_content.contains(r#""my-package":{"version":"1.0.0"}"#));

    let mut project_pkg_json_content =
        fs::read_to_string(env.project_dir.join("package.json")).unwrap();
    tracing::info!("project_pkg_json_content: {}", project_pkg_json_content);

    project_pkg_json_content.retain(|c| !c.is_whitespace());
    assert!(project_pkg_json_content.contains(r#""my-package":"file:.kley/my-package""#));
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

    // assert!(
    //     env.project_dir
    //         .join(".kley")
    //         .join("my-pnpm-package")
    //         .exists()
    // );
    // let kley_lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();
    // assert!(kley_lock_content.contains(r#""my-pnpm-package":{"version":"2.0.0"}"#));
    //
    // let project_pkg_json_content =
    //     fs::read_to_string(env.project_dir.join("package.json")).unwrap();
    // assert!(
    //     project_pkg_json_content.contains(r#""my-pnpm-package": "file:./.kley/my-pnpm-package""#)
    // );
}

// #[test]
// fn test_install_command_yarn_project() {
//     let env = TestEnv::new();
//     env.create_mock_registry_package("my-yarn-package", "3.0.0");
//     env.setup_project_pm("yarn");
//
//     env.run_kley_command(&["install", "my-yarn-package"])
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("✅ Done: my-yarn-package installed"));
//
//     assert!(env.project_dir.join(".kley").join("my-yarn-package").exists());
//     let kley_lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();
//     assert!(kley_lock_content.contains(r#""my-yarn-package":{"version":"3.0.0"}"#));
//
//     let project_pkg_json_content = fs::read_to_string(env.project_dir.join("package.json")).unwrap();
//     assert!(project_pkg_json_content.contains(r#""my-yarn-package": "file:./.kley/my-yarn-package""#));
// }
//
// #[test]
// fn test_install_command_kley_lock_pm_override() {
//     let env = TestEnv::new();
//     env.create_mock_registry_package("my-override-package", "4.0.0");
//     env.setup_project_pm("npm"); // Project has npm lockfile
//
//     // Create kley.lock to override to pnpm
//     env.create_kley_lock(r#"{{"packageManager": "pnpm", "packages": {}}}"#);
//
//     env.run_kley_command(&["install", "my-override-package"])
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("✅ Done: my-override-package installed"));
//
//     assert!(env.project_dir.join(".kley").join("my-override-package").exists());
//     let kley_lock_content = fs::read_to_string(env.project_dir.join("kley.lock")).unwrap();
//     assert!(kley_lock_content.contains(r#""my-override-package":{"version":"4.0.0"}"#));
//     assert!(kley_lock_content.contains(r#""packageManager":"pnpm""#)); // kley.lock should retain its PM setting
//
//     let project_pkg_json_content = fs::read_to_string(env.project_dir.join("package.json")).unwrap();
//     assert!(project_pkg_json_content.contains(r#""my-override-package": "file:./.kley/my-override-package""#));
// }
