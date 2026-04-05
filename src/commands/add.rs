use anyhow::{Context, Result};
use colored::*;
use serde::Serialize;
use serde_json;
use std::fs;
use std::path::Path;

use crate::commands::update::run_update;
use crate::registry::Registry;
use crate::utils::{detect_indent, work_dirs};

/// Add logic
pub fn add(registry: &mut Registry, package_name: &str, is_dev: bool) -> Result<()> {
    let dirs = work_dirs(package_name, registry)?;

    run_update(registry, package_name, &std::env::current_dir()?)?;

    // --- Automate package.json modification ---
    update_package_json(&dirs.project_dir.join("package.json"), package_name, is_dev)?;

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

