use anyhow::{Context, Result};
use colored::*;
use serde::Serialize;
use serde_json;
use std::fs;
use std::path::Path;

/// Add logic
pub fn add(package_name: &str, is_dev: bool) -> Result<()> {
    let home_dir = dirs::home_dir().context("Failed to find home directory")?;
    let source_path = home_dir.join(".kley").join("packages").join(package_name);

    if !source_path.exists() {
        anyhow::bail!(
            "Package '{}' not found in store. Run 'kley publish' in the package folder first.",
            package_name
        );
    }

    // Determine the local storage folder (.kley/package-name)
    let dest_path = Path::new(".kley").join(package_name);

    if dest_path.exists() {
        fs::remove_dir_all(&dest_path)?;
    }
    fs::create_dir_all(&dest_path)?;

    // Copy from store to local project
    let mut options = fs_extra::dir::CopyOptions::new();
    options.content_only = true;
    fs_extra::dir::copy(&source_path, &dest_path, &options)?;

    println!(
        "📎 Package {} added to .kley/{}",
        package_name.cyan(),
        package_name.cyan()
    );

    // --- Automate package.json modification ---
    update_package_json(Path::new("package.json"), package_name, is_dev)?;

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

/// Detects the indentation of a JSON string.
fn detect_indent(json_str: &str) -> String {
    for line in json_str.lines() {
        if line.starts_with("  ") {
            let indent_len = line.find(|c: char| !c.is_whitespace()).unwrap_or(0);
            if indent_len > 0 {
                return line[..indent_len].to_string();
            }
        }
    }
    "  ".to_string() // Default to 2 spaces
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
