mod common;

use std::fs::{self, File};
use std::io::Write;

use kley::package::{Package, PackageManagerType};

use common::TestEnv;

#[test]
fn test_detects_from_kley_lock_first() {
    let env = TestEnv::new();
    // Create a kley.lock with "pnpm"
    env.create_kley_lock(r#"{{"packageManager": "pnpm", "packages": {}}}"#);

    // Create conflicting files to ensure kley.lock has priority
    File::create(env.project_dir.join("yarn.lock")).unwrap();
    let package_json_path = env.project_dir.join("package.json");
    let mut pkg_file = File::create(package_json_path).unwrap();
    writeln!(
        pkg_file,
        r#"{{"name": "test", "version": "0.0.0", "packageManager": "yarn@1.22.19"}}"#
    )
    .unwrap();

    let package = Package::get(&env.project_dir);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Pnpm);
}

#[test]
fn test_detects_from_package_json_second() {
    let env = TestEnv::new();
    // Create a package.json with "yarn"
    let package_json_path = env.project_dir.join("package.json");
    let mut file = File::create(package_json_path).unwrap();
    writeln!(
        file,
        r#"{{"name": "test", "version": "0.0.0", "packageManager": "yarn@1.22.19"}}"#
    )
    .unwrap();

    // Create a conflicting lockfile to ensure package.json has priority
    File::create(env.project_dir.join("package-lock.json")).unwrap();

    let package = Package::get(&env.project_dir);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Yarn);
}

#[test]
fn test_detects_from_lockfile_third() {
    let env = TestEnv::new();
    let package_json_path = env.project_dir.join("package.json");
    let mut file = File::create(package_json_path).unwrap();
    writeln!(file, r#"{{"name": "test", "version": "0.0.0"}}"#).unwrap();

    // Create only a pnpm lockfile
    File::create(env.project_dir.join("pnpm-lock.yaml")).unwrap();
    let package = Package::get(&env.project_dir);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Pnpm);
    fs::remove_file(env.project_dir.join("pnpm-lock.yaml")).unwrap();

    // Create only a yarn lockfile
    File::create(env.project_dir.join("yarn.lock")).unwrap();
    let package = Package::get(&env.project_dir);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Yarn);
    fs::remove_file(env.project_dir.join("yarn.lock")).unwrap();

    // Create only an npm lockfile
    File::create(env.project_dir.join("package-lock.json")).unwrap();
    let package = Package::get(&env.project_dir);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Npm);
}

#[test]
fn test_lockfile_priority() {
    let env = TestEnv::new();
    let package_json_path = env.project_dir.join("package.json");
    let mut file = File::create(package_json_path).unwrap();
    writeln!(file, r#"{{"name": "test", "version": "0.0.0"}}"#).unwrap();

    // Create all three lockfiles
    File::create(env.project_dir.join("pnpm-lock.yaml")).unwrap();
    File::create(env.project_dir.join("yarn.lock")).unwrap();
    File::create(env.project_dir.join("package-lock.json")).unwrap();

    // pnpm should have the highest priority among lockfiles
    let package = Package::get(&env.project_dir);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Pnpm);
}

#[test]
fn test_defaults_to_npm_if_no_indicators() {
    let env = TestEnv::new();
    let package_json_path = env.project_dir.join("package.json");
    let mut file = File::create(package_json_path).unwrap();
    writeln!(file, r#"{{"name": "test", "version": "0.0.0"}}"#).unwrap();

    let package = Package::get(&env.project_dir);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Npm);
}

#[test]
fn test_ignores_empty_package_manager_field() {
    let env = TestEnv::new();
    let package_json_path = env.project_dir.join("package.json");
    let mut file = File::create(package_json_path).unwrap();
    writeln!(file, r#"{{"name": "test", "version": "0.0.0"}}"#).unwrap();

    // Create a kley.lock with an empty packageManager string
    env.create_kley_lock(r#"{{"packageManager": "", "packages": {}}}"#);

    // It should fall back to the next mechanism, in this case the yarn.lock
    File::create(env.project_dir.join("yarn.lock")).unwrap();

    let package = Package::get(&env.project_dir);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Yarn);
}
