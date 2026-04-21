use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

use kley::package::{Package, PackageManagerType};

#[test]
fn test_detects_from_kley_lock_first() {
    let dir = tempdir().unwrap();
    let path = dir.path();

    // Create a kley.lock with "pnpm"
    let kley_lock_path = path.join("kley.lock");
    let mut file = File::create(kley_lock_path).unwrap();
    writeln!(file, r#"{{"packageManager": "pnpm", "packages": {{}}}}"#).unwrap();

    // Create conflicting files to ensure kley.lock has priority
    File::create(path.join("yarn.lock")).unwrap();
    let package_json_path = path.join("package.json");
    let mut pkg_file = File::create(package_json_path).unwrap();
    writeln!(
        pkg_file,
        r#"{{"name": "test", "version": "0.0.0", "packageManager": "yarn@1.22.19"}}"#
    )
    .unwrap();

    let package = Package::get(path);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Pnpm);
}

#[test]
fn test_detects_from_package_json_second() {
    let dir = tempdir().unwrap();
    let path = dir.path();

    // Create a package.json with "yarn"
    let package_json_path = path.join("package.json");
    let mut file = File::create(package_json_path).unwrap();
    writeln!(
        file,
        r#"{{"name": "test", "version": "0.0.0", "packageManager": "yarn@1.22.19"}}"#
    )
    .unwrap();

    // Create a conflicting lockfile to ensure package.json has priority
    File::create(path.join("package-lock.json")).unwrap();

    let package = Package::get(path);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Yarn);
}

#[test]
fn test_detects_from_lockfile_third() {
    let dir = tempdir().unwrap();
    let path = dir.path();

    let package_json_path = path.join("package.json");
    let mut file = File::create(package_json_path).unwrap();
    writeln!(file, r#"{{"name": "test", "version": "0.0.0"}}"#).unwrap();

    // Create only a pnpm lockfile
    File::create(path.join("pnpm-lock.yaml")).unwrap();
    let package = Package::get(path);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Pnpm);
    fs::remove_file(path.join("pnpm-lock.yaml")).unwrap();

    // Create only a yarn lockfile
    File::create(path.join("yarn.lock")).unwrap();
    let package = Package::get(path);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Yarn);
    fs::remove_file(path.join("yarn.lock")).unwrap();

    // Create only an npm lockfile
    File::create(path.join("package-lock.json")).unwrap();
    let package = Package::get(path);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Npm);
}

#[test]
fn test_lockfile_priority() {
    let dir = tempdir().unwrap();
    let path = dir.path();

    let package_json_path = path.join("package.json");
    let mut file = File::create(package_json_path).unwrap();
    writeln!(file, r#"{{"name": "test", "version": "0.0.0"}}"#).unwrap();

    // Create all three lockfiles
    File::create(path.join("pnpm-lock.yaml")).unwrap();
    File::create(path.join("yarn.lock")).unwrap();
    File::create(path.join("package-lock.json")).unwrap();

    // pnpm should have the highest priority among lockfiles
    let package = Package::get(path);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Pnpm);
}

#[test]
fn test_defaults_to_npm_if_no_indicators() {
    let dir = tempdir().unwrap();
    let path = dir.path();

    let package_json_path = path.join("package.json");
    let mut file = File::create(package_json_path).unwrap();
    writeln!(file, r#"{{"name": "test", "version": "0.0.0"}}"#).unwrap();

    let package = Package::get(path);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Npm);
}

#[test]
fn test_ignores_empty_package_manager_field() {
    let dir = tempdir().unwrap();
    let path = dir.path();

    let package_json_path = path.join("package.json");
    let mut file = File::create(package_json_path).unwrap();
    writeln!(file, r#"{{"name": "test", "version": "0.0.0"}}"#).unwrap();

    // Create a kley.lock with an empty packageManager string
    let kley_lock_path = path.join("kley.lock");
    let mut file = File::create(kley_lock_path).unwrap();
    writeln!(file, r#"{{"packageManager": "", "packages": {{}}}}"#).unwrap();

    // It should fall back to the next mechanism, in this case the yarn.lock
    File::create(path.join("yarn.lock")).unwrap();

    let package = Package::get(path);
    assert_eq!(package.unwrap().manager_type, PackageManagerType::Yarn);
}
