#![allow(dead_code)]

use assert_cmd::Command;
use chrono::Utc;
use kley::registry::{PackageMetadata, RegistryData};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Represents a temporary test environment with a project and kley home.
pub struct TestEnv {
    pub temp_dir: TempDir,
    pub kley_registry: PathBuf,
    pub project_dir: PathBuf,
}

impl TestEnv {
    /// Sets up a new isolated test environment.
    /// Creates a temporary directory, a mock kley home, and a mock project.
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap(); // Store the TempDir object
        let kley_home = temp_dir.path().join(".kley"); // Use temp_dir.path()
        let project_dir = temp_dir.path().join("my-project"); // Use temp_dir.path()

        fs::create_dir_all(&kley_home).unwrap();
        fs::create_dir_all(&project_dir).unwrap();

        TestEnv {
            temp_dir,
            kley_registry: kley_home,
            project_dir,
        }
    }

    /// Creates a mock package in the kley registry.
    pub fn create_mock_registry_package(&self, pkg_name: &str, pkg_version: &str) {
        let packages_dir = self.kley_registry.join("packages");
        let pkg_dir = packages_dir.join(pkg_name);
        fs::create_dir_all(&pkg_dir).unwrap();

        let mut pkg_json_file = fs::File::create(pkg_dir.join("package.json")).unwrap();
        writeln!(
            pkg_json_file,
            r#"{{"name": "{}", "version": "{}"}}"#,
            pkg_name, pkg_version
        )
        .unwrap();

        // Update registry.json
        let registry_path = self.kley_registry.join("registry.json");
        let mut registry_data = if registry_path.exists() {
            let content = fs::read_to_string(&registry_path).unwrap();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            RegistryData::default()
        };

        registry_data.packages.insert(
            pkg_name.to_string(),
            PackageMetadata {
                version: pkg_version.to_string(),
                last_updated: Utc::now().to_rfc3339(),
                installations: vec![],
            },
        );

        fs::write(
            &registry_path,
            serde_json::to_string_pretty(&registry_data).unwrap(),
        )
        .unwrap();
    }

    /// Configures the project directory for a specific package manager.
    pub fn setup_project_pm(&self, pm_type: &str) {
        // Create a basic package.json for the project if it doesn't exist
        let pkg_json_path = self.project_dir.join("package.json");
        if !pkg_json_path.exists() {
            let mut pkg_json_file = fs::File::create(&pkg_json_path).unwrap();
            writeln!(
                pkg_json_file,
                r#"{{"name": "my-project", "version": "1.0.0"}}"#
            )
            .unwrap();
        }

        // Simulate package manager detection by creating lock files
        match pm_type {
            "npm" => {
                fs::File::create(self.project_dir.join("package-lock.json")).unwrap();
            }
            "pnpm" => {
                fs::File::create(self.project_dir.join("pnpm-lock.yaml")).unwrap();
            }
            "yarn" => {
                fs::File::create(self.project_dir.join("yarn.lock")).unwrap();
            }
            _ => {} // No specific lock file for default
        }
    }

    /// Runs the kley command within the test project.
    /// This function automatically uses the mocked package managers from `tests/mocks`
    /// by setting the `KLEY_USE_..._COMMAND` environment variables.
    pub fn run_kley_command(&self, args: &[&str]) -> Command {
        let mut cmd = Command::cargo_bin("kley").unwrap();

        // Get absolute paths to the mock scripts
        let mut mocks_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        mocks_dir.push("tests/mocks");

        // On Windows, use .cmd wrappers; on Unix, use .sh wrappers.
        // Both delegate to the .js scripts via Node.js.
        let (npm_mock, pnpm_mock, yarn_mock) = if cfg!(target_os = "windows") {
            (
                mocks_dir.join("npm.cmd"),
                mocks_dir.join("pnpm.cmd"),
                mocks_dir.join("yarn.cmd"),
            )
        } else {
            (
                mocks_dir.join("npm.sh"),
                mocks_dir.join("pnpm.sh"),
                mocks_dir.join("yarn.sh"),
            )
        };

        // Set env vars to point to the mock scripts
        cmd.env("KLEY_USE_NPM_COMMAND", npm_mock);
        cmd.env("KLEY_USE_PNPM_COMMAND", pnpm_mock);
        cmd.env("KLEY_USE_YARN_COMMAND", yarn_mock);
        cmd.env("KLEY_HOME", self.temp_dir.path());

        cmd.current_dir(&self.project_dir).args(args);
        cmd
    }

    /// Creates a kley.lock file in the project directory.
    pub fn create_kley_lock(&self, content: &str) {
        let mut kley_lock_file = fs::File::create(self.project_dir.join("kley.lock")).unwrap();
        kley_lock_file.write_all(content.as_bytes()).unwrap();
    }
}

// --- Existing functions (can be kept or integrated into TestEnv) ---
// These functions are kept for now, but can be refactored to use TestEnv
// or removed if no longer needed after all tests are updated.
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

/// Compares two paths for equality in a platform-independent way.
/// Handles Windows-specific issues:
/// - UNC prefix: `\\?\C:\...` vs `C:\...`
/// - 8.3 short names: `RUNNER~1` vs `runneradmin`
/// - Backslashes vs forward slashes
pub fn paths_match(a: &Path, b: &Path) -> bool {
    // Try canonicalize both — resolves symlinks, UNC prefixes, short names
    if let (Ok(ca), Ok(cb)) = (fs::canonicalize(a), fs::canonicalize(b))
        && ca == cb
    {
        return true;
    }
    // Fallback: string comparison with normalized separators
    let a_str = a.to_string_lossy().replace('\\', "/");
    let b_str = b.to_string_lossy().replace('\\', "/");
    a_str == b_str
}
