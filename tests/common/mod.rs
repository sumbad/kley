use std::fs;
use std::path::Path;

pub fn setup_package_json(
    project_path: &Path,
    name: &str,
    version: &str,
) -> Result<(), std::io::Error> {
    let package_json_path = project_path.join("package.json");
    let package_json_content = format!(r#"{{"name": "{}", "version": "{}"}}"#, name, version);
    fs::write(package_json_path, package_json_content)
}

pub fn setup_kley_and_project(
    project_path: &Path,
    name: &str,
    version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(project_path)?;
    setup_package_json(project_path, name, version)?;
    // Simulate `kley init`
    fs::create_dir_all(project_path.join(".kley"))?;
    fs::write(project_path.join("kley.lock"), r#"{"packages":{}}"#)?;
    Ok(())
}
