use anyhow::{Context, Result};
use colored::*;
use serde::Serialize;
use serde_json;
use std::fs;
use std::path::Path;

use crate::lockfile::Lockfile;
use crate::registry::Registry;
use crate::utils::{detect_indent, normalized_path};

pub fn remove(
    registry: &mut Registry,
    package_name: &Option<String>,
    is_all: bool,
    project_dir: &Path,
) -> Result<()> {
    if package_name.is_none() && !is_all {
        println!("{}",
            "⚠️ Warning: Pass a package name to remove it or use --all flag to delete all local dependencies".yellow()
        );
        return Ok(());
    }

    if is_all {
        // Remove kley.lock file
        let lock_path = project_dir.join("kley.lock");
        if lock_path.exists() {
            fs::remove_file(lock_path)?;
        }

        // Remove all local packages from package.json
        remove_all_from_package_json(&project_dir.join("package.json"))?;

        registry.remove_all_installations(project_dir)?;

        let local_store = project_dir.join(".kley");
        if local_store.exists() {
            fs::remove_dir_all(&local_store)?;
            tracing::info!(
                "removed directory: {}",
                normalized_path(&local_store, dirs::home_dir().as_ref())
            );
        }

        println!("{}", "✅ Done: all packages removed".green());
    } else if let Some(pkg_name) = package_name {
        remove_package(registry, pkg_name, project_dir)?;

        println!(
            "{}",
            format!("✅ Done: {} removed", pkg_name.cyan()).green()
        );
    }

    Ok(())
}

pub fn remove_package(
    registry: &mut Registry,
    package_name: &str,
    project_dir: &Path,
) -> Result<()> {
    let local_store_package_dir = project_dir.join(".kley").join(package_name);

    update_package_json(&project_dir.join("package.json"), package_name)?;
    update_kley_lock(package_name, project_dir)?;

    registry.remove_package_installation(package_name, project_dir)?;

    if local_store_package_dir.exists() {
        fs::remove_dir_all(&local_store_package_dir)?;
        tracing::info!(
            "removed directory: {}",
            normalized_path(&local_store_package_dir, dirs::home_dir().as_ref())
        );
    }

    Ok(())
}

/// Modifies package.json to remove a dependency.
fn update_package_json(pkg_json_path: &Path, package_name: &str) -> Result<()> {
    if !pkg_json_path.exists() {
        println!(
            "{}",
            "⚠️ Warning: package.json not found, skipping modification.".yellow()
        );
        return Ok(());
    }

    let content = fs::read_to_string(pkg_json_path)?;
    let indent = detect_indent(&content);

    let mut value: serde_json::Value =
        serde_json::from_str(&content).context("Failed to parse package.json")?;

    let dep_value = format!("file:.kley/{}", package_name);
    let dep_keys = ["dependencies", "devDependencies", "peerDependencies"];

    if let Some(obj) = value.as_object_mut() {
        for key in &dep_keys {
            if let Some(deps_obj) = obj.get_mut(*key).and_then(|d| d.as_object_mut())
                && let Some(dep) = deps_obj.get(package_name)
                && *dep == dep_value
                && deps_obj.remove(package_name).is_some()
            {
                tracing::info!("Removed '{}' from '{}' in package.json", package_name, key);
                break;
            }
        }
    }

    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    value.serialize(&mut ser)?;

    fs::write(pkg_json_path, buf)?;

    tracing::info!("package.json has been updated!");

    Ok(())
}

/// Updates kley.lock file
fn update_kley_lock(package_name: &str, project_dir: &Path) -> Result<()> {
    let lock_path = project_dir.join("kley.lock");

    if !lock_path.exists() {
        tracing::debug!("kley.lock file was not found");

        return Ok(());
    }

    // Read existing kley.lock
    let mut lockfile: Lockfile = if lock_path.exists() {
        let content = fs::read_to_string(&lock_path)?;
        serde_json::from_str(&content).context("Failed to parse kley.lock")?
    } else {
        Lockfile::default()
    };

    lockfile.packages.remove(package_name);

    // Write back to kley.lock
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    lockfile.serialize(&mut ser)?;

    fs::write(lock_path, buf)?;

    tracing::info!("{}", "kley.lock has been updated!".green());

    Ok(())
}

fn remove_all_from_package_json(pkg_json_path: &Path) -> Result<()> {
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

    let dep_keys = ["dependencies", "devDependencies", "peerDependencies"];

    if let Some(obj) = value.as_object_mut() {
        for key in &dep_keys {
            if let Some(deps_obj) = obj.get_mut(*key).and_then(|d| d.as_object_mut()) {
                let to_remove: Vec<String> = deps_obj
                    .iter()
                    .filter(|(_, v)| v.as_str().unwrap_or("").starts_with("file:.kley/"))
                    .map(|(k, _)| k.clone())
                    .collect();

                tracing::debug!("deps_obj: {:#?} \n to_remove: {:?}", deps_obj, to_remove);

                for dep in to_remove {
                    if let Some(val) = deps_obj.remove(&dep) {
                        tracing::info!("Removed '{}' from '{}' in package.json", val, key);
                    }
                }
            }
        }
    }

    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    value.serialize(&mut ser)?;

    fs::write(pkg_json_path, buf)?;

    tracing::info!("package.json has been updated");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    /// Helper to create a dummy project structure inside a temporary directory.
    fn setup_test_project(dir: &Path) -> Result<()> {
        fs::create_dir_all(dir.join(".git"))?; // Trick the `ignore` crate into thinking this is a git repo
        fs::write(
            dir.join("package.json"),
            r#"{
  "author": "",
  "dependencies": {
    "test-lib": "file:.kley/test-lib"
  },
  "description": "",
  "keywords": [],
  "license": "ISC",
  "main": "index.js",
  "name": "test-app",
  "scripts": {
    "test": "echo \"Error: no test specified\" && exit 1"
  },
  "type": "commonjs",
  "version": "1.0.0"
}"#,
        )?;
        fs::create_dir_all(dir.join(".kley/test-lib"))?;
        fs::write(
            dir.join(".kley/test-lib/package.json"),
            r#"{
  "author": "",
  "dependencies": {
    "test-lib": "file:.kley/test-lib"
  },
  "description": "",
  "keywords": [],
  "license": "ISC",
  "main": "index.js",
  "name": "test-lib",
  "scripts": {
    "test": "echo \"Error: no test specified\" && exit 1"
  },
  "type": "commonjs",
  "version": "1.0.0"
}%"#,
        )?;
        fs::write(dir.join("secret.log"), "sensitive data")?;
        // This file should also be ignored by default git rules
        fs::create_dir_all(dir.join("node_modules/some_dep"))?;
        fs::write(
            dir.join("kley.lock"),
            r#"{
  "packages": {
    "test-lib": {
      "version": "1.0.0"
    }
  }
}"#,
        )?;
        Ok(())
    }

    #[test]
    fn test_remove_single_package() -> Result<()> {
        let tmp_home_dir = tempdir()?;
        let home_dir = tmp_home_dir.path();
        let mut registry = Registry::new(home_dir.to_path_buf())?;

        let proj_dir = tempdir()?;
        let proj_path = proj_dir.path();
        setup_test_project(proj_path)?;

        remove(
            &mut registry,
            &Some(String::from("test-lib")),
            false,
            proj_path,
        )?;

        assert!(
            !proj_path.join(".kley/test-lib").exists(),
            "The removed package should be deleted from .kley local dir"
        );

        let updated_content = fs::read_to_string(proj_path.join("package.json"))?;
        let updated_json: serde_json::Value = serde_json::from_str(&updated_content)?;

        assert_eq!(
            updated_json["dependencies"],
            json!({}),
            "The removed package should be deleted from packages.json"
        );

        let updated_lock_content = fs::read_to_string(proj_path.join("kley.lock"))?;
        let updated_lock_json: serde_json::Value = serde_json::from_str(&updated_lock_content)?;

        assert_eq!(
            updated_lock_json["packages"],
            json!({}),
            "The removed package should be deleted from kley.lock"
        );

        Ok(())
    }

    #[test]
    fn test_remove_all_packages() -> Result<()> {
        let tmp_home_dir = tempdir()?;
        let home_dir = tmp_home_dir.path();
        let mut registry = Registry::new(home_dir.to_path_buf())?;

        let proj_dir = tempdir()?;
        let proj_path = proj_dir.path();
        setup_test_project(proj_path)?;

        remove(&mut registry, &None, true, proj_path)?;

        assert!(
            !proj_path.join(".kley").exists(),
            ".kley local dir should be deleted"
        );

        assert!(
            !proj_path.join("kley.lock").exists(),
            "kley.lock local dir should be deleted"
        );

        let updated_content = fs::read_to_string(proj_path.join("package.json"))?;
        let updated_json: serde_json::Value = serde_json::from_str(&updated_content)?;

        assert_eq!(
            updated_json["dependencies"],
            json!({}),
            "The removed packages should be deleted from packages.json"
        );

        Ok(())
    }

    #[test]
    fn test_remove_is_idempotent() -> Result<()> {
        let tmp_home_dir = tempdir()?;
        let home_dir = tmp_home_dir.path();
        let mut registry = Registry::new(home_dir.to_path_buf())?;

        let proj_dir = tempdir()?;
        let proj_path = proj_dir.path();
        setup_test_project(proj_path)?;

        remove(
            &mut registry,
            &Some(String::from("test-lib")),
            false,
            proj_path,
        )?;
        remove(
            &mut registry,
            &Some(String::from("test-lib")),
            false,
            proj_path,
        )?;

        Ok(())
    }

    #[test]
    fn test_remove_missing_file() -> Result<()> {
        let tmp_home_dir = tempdir()?;
        let home_dir = tmp_home_dir.path();
        let mut registry = Registry::new(home_dir.to_path_buf())?;

        let proj_dir = tempdir()?;
        let proj_path = proj_dir.path();
        setup_test_project(proj_path)?;

        remove(
            &mut registry,
            &Some(String::from("test-uknown-lib")),
            false,
            proj_path,
        )?;

        Ok(())
    }
}
