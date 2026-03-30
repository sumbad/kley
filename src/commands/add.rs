use anyhow::{Context, Result};
use colored::*;
use serde::Serialize;
use serde_json;
use std::fs;
use std::path::Path;

use crate::lockfile::{Lockfile, PackageInfo, PackageJson};
use crate::registry::Registry;
use crate::utils::{copy_from_store, detect_indent, work_dirs};

/// Add logic
pub fn add(registry: &mut Registry, package_name: &str, is_dev: bool) -> Result<()> {
    let dirs = work_dirs(package_name, registry)?;

    copy_from_store(package_name, &dirs)?;

    // --- Automate package.json modification ---
    update_package_json(&dirs.project_dir.join("package.json"), package_name, is_dev)?;

    // --- Create or update kley.lock ---
    update_kley_lock(package_name, &dirs.registry_dir, &dirs.project_dir)?;

    registry.add_package_installation(package_name, &dirs.project_dir)?;

    Ok(())
}

/// Modifies package.json to add or update a dependency.
fn update_package_json(pkg_json_path: &Path, package_name: &str, is_dev: bool) -> Result<()> {
    if !pkg_json_path.exists() {
        println!(
            "{}",
            "⚠️ package.json not found, skipping modification.".yellow()
        );
        return Ok(());
    }

    let content = fs::read_to_string(pkg_json_path)?;
    let indent = detect_indent(&content);

    let mut value: serde_json::Value =
        serde_json::from_str(&content).context("Failed to parse package.json")?;

    let dep_path = format!("file:.kley/{}", package_name);
    let dep_keys = ["dependencies", "devDependencies", "peerDependencies"];
    let mut updated = false;

    if let Some(obj) = value.as_object_mut() {
        for key in &dep_keys {
            if let Some(dep) = obj
                .get_mut(*key)
                .and_then(|d| d.as_object_mut())
                .and_then(|d| d.get_mut(package_name))
            {
                *dep = serde_json::Value::String(dep_path.clone());
                updated = true;
                break;
            }
        }

        if !updated {
            let target_key = if is_dev {
                "devDependencies"
            } else {
                "dependencies"
            };
            if !obj.contains_key(target_key) {
                obj.insert(target_key.to_string(), serde_json::json!({}));
            }
            obj[target_key].as_object_mut().unwrap().insert(
                package_name.to_string(),
                serde_json::Value::String(dep_path),
            );
        }
    }

    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    value.serialize(&mut ser)?;

    fs::write(pkg_json_path, buf)?;

    println!("{}", "✅ package.json has been updated!".green());

    Ok(())
}

/// Creates or updates kley.lock file.
fn update_kley_lock(package_name: &str, source_path: &Path, project_dir: &Path) -> Result<()> {
    let lock_path = project_dir.join("kley.lock");

    // 1. Read source package.json to get version
    let source_pkg_json_path = source_path.join("package.json");
    let content = fs::read_to_string(&source_pkg_json_path).context(format!(
        "Failed to read package.json from {:?}",
        source_path
    ))?;
    let pkg_json: PackageJson =
        serde_json::from_str(&content).context("Failed to parse source package.json")?;

    // 2. Read existing kley.lock or create a new one
    let mut lockfile: Lockfile = if lock_path.exists() {
        let content = fs::read_to_string(&lock_path)?;
        if content.trim().is_empty() {
            Lockfile::default()
        } else {
            serde_json::from_str(&content).context("Failed to parse kley.lock")?
        }
    } else {
        Lockfile::default()
    };

    // 3. Insert or update package info
    let package_info = PackageInfo {
        version: pkg_json.version,
    };
    lockfile
        .packages
        .insert(package_name.to_string(), package_info);

    // 4. Write back to kley.lock
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    lockfile.serialize(&mut ser)?;

    fs::write(lock_path, buf)?;

    println!("{}", "🔒 kley.lock has been updated!".green());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_add_new_dependency() -> Result<()> {
        let initial_content = r#"{
  "name": "test-project",
  "version": "1.0.0",
  "dependencies": {}
}"#;
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", initial_content)?;

        update_package_json(file.path(), "my-local-lib", false)?;

        let updated_content = fs::read_to_string(file.path())?;
        let updated_json: serde_json::Value = serde_json::from_str(&updated_content)?;

        assert_eq!(
            updated_json["dependencies"]["my-local-lib"],
            "file:.kley/my-local-lib"
        );

        Ok(())
    }

    #[test]
    fn test_add_new_dev_dependency() -> Result<()> {
        let initial_content = r#"{
  "name": "test-project",
  "version": "1.0.0",
  "devDependencies": {}
}"#;
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", initial_content)?;

        update_package_json(file.path(), "my-local-lib", true)?;

        let updated_content = fs::read_to_string(file.path())?;
        let updated_json: serde_json::Value = serde_json::from_str(&updated_content)?;

        assert_eq!(
            updated_json["devDependencies"]["my-local-lib"],
            "file:.kley/my-local-lib"
        );

        Ok(())
    }

    #[test]
    fn test_update_existing_dependency() -> Result<()> {
        let initial_content = r#"{
  "name": "test-project",
  "version": "1.0.0",
  "dependencies": {
    "my-local-lib": "1.0.0"
  }
}"#;
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", initial_content)?;

        update_package_json(file.path(), "my-local-lib", false)?;

        let updated_content = fs::read_to_string(file.path())?;
        let updated_json: serde_json::Value = serde_json::from_str(&updated_content)?;

        assert_eq!(
            updated_json["dependencies"]["my-local-lib"],
            "file:.kley/my-local-lib"
        );

        Ok(())
    }

    #[test]
    fn test_create_dependencies_section() -> Result<()> {
        let initial_content = r#"{
  "name": "test-project",
  "version": "1.0.0"
}"#;
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", initial_content)?;

        update_package_json(file.path(), "my-local-lib", false)?;

        let updated_content = fs::read_to_string(file.path())?;
        let updated_json: serde_json::Value = serde_json::from_str(&updated_content)?;

        assert_eq!(
            updated_json["dependencies"]["my-local-lib"],
            "file:.kley/my-local-lib"
        );

        Ok(())
    }

    #[test]
    fn test_create_dev_dependencies_section() -> Result<()> {
        let initial_content = r#"{
  "name": "test-project",
  "version": "1.0.0"
}"#;
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", initial_content)?;

        update_package_json(file.path(), "my-local-lib", true)?;

        let updated_content = fs::read_to_string(file.path())?;
        let updated_json: serde_json::Value = serde_json::from_str(&updated_content)?;

        assert_eq!(
            updated_json["devDependencies"]["my-local-lib"],
            "file:.kley/my-local-lib"
        );

        Ok(())
    }
}

#[cfg(test)]
mod kley_lock_tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_create_new_kley_lock() -> Result<()> {
        let dir = tempdir()?;
        let project_dir = dir.path();

        // Create a dummy package.json in a dummy source path
        let source_path = project_dir.join("source-lib");
        fs::create_dir_all(&source_path)?;
        let pkg_json_path = source_path.join("package.json");
        let mut file = fs::File::create(pkg_json_path)?;
        write!(file, r#"{{"version": "1.2.3"}}"#)?;

        update_kley_lock("my-test-lib", &source_path, project_dir)?;

        let lock_content = fs::read_to_string(project_dir.join("kley.lock"))?;
        let lockfile: Lockfile = serde_json::from_str(&lock_content)?;

        assert_eq!(
            lockfile.packages.get("my-test-lib").unwrap().version,
            "1.2.3"
        );

        Ok(())
    }

    #[test]
    fn test_update_existing_kley_lock() -> Result<()> {
        let dir = tempdir()?;
        let project_dir = dir.path();

        // Create a dummy source package
        let source_path = project_dir.join("my-lib");
        fs::create_dir_all(&source_path)?;
        let pkg_json_path = source_path.join("package.json");
        let mut file = fs::File::create(pkg_json_path)?;
        write!(file, r#"{{"version": "2.0.0"}}"#)?;

        // Create an existing kley.lock in the project dir
        let lock_path = project_dir.join("kley.lock");
        fs::write(
            &lock_path,
            r#"{"packages":{"another-lib":{"version":"0.5.0"}}}"#,
        )?;

        update_kley_lock("my-lib", &source_path, project_dir)?;

        let lock_content = fs::read_to_string(lock_path)?;
        let lockfile: Lockfile = serde_json::from_str(&lock_content)?;

        assert_eq!(lockfile.packages.len(), 2);
        assert_eq!(lockfile.packages.get("my-lib").unwrap().version, "2.0.0");
        assert_eq!(
            lockfile.packages.get("another-lib").unwrap().version,
            "0.5.0"
        );

        Ok(())
    }
}
