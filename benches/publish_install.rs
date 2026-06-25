use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::tempdir;

// Built by cargo before running benches (bench profile inherits release optimizations).
const KLEY: &str = env!("CARGO_BIN_EXE_kley");

/// Resolve a required env var. Panics with setup instructions if missing.
fn require_env(var: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| {
        panic!(
            "\nRequired env var `{var}` is not set.\n\
             Install bench deps once and export the paths:\n\n  \
             npm install --prefix target/bench_deps yalc @jimsheen/yalc\n  \
             export BENCH_YALC_JS=$PWD/target/bench_deps/node_modules/yalc/src/yalc.js\n  \
             export BENCH_JIMSHEEN_YALC_JS=$PWD/target/bench_deps/node_modules/@jimsheen/yalc/dist/yalc.js\n"
        )
    })
}

#[derive(Debug, Clone, Copy)]
enum Tool {
    Kley,
    Yalc,
    JimsheenYalc,
}

impl Tool {
    fn name(&self) -> &'static str {
        match self {
            Tool::Kley => "kley",
            Tool::Yalc => "yalc",
            Tool::JimsheenYalc => "jimsheen_yalc",
        }
    }

    fn publish(&self, lib_dir: &std::path::Path, home_dir: &std::path::Path) {
        match self {
            Tool::Kley => {
                Command::new(KLEY)
                    .arg("publish")
                    .env("KLEY_HOME", home_dir)
                    .current_dir(lib_dir)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .expect("kley publish failed");
            }
            Tool::Yalc => {
                let js = require_env("BENCH_YALC_JS");
                Command::new("node")
                    .args([&js, "publish"])
                    .env("HOME", home_dir)
                    .current_dir(lib_dir)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .expect("yalc publish failed");
            }
            Tool::JimsheenYalc => {
                let js = require_env("BENCH_JIMSHEEN_YALC_JS");
                Command::new("node")
                    .args([&js, "publish"])
                    .env("HOME", home_dir)
                    .current_dir(lib_dir)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .expect("@jimsheen/yalc publish failed");
            }
        }
    }

    /// Install into the app project.
    ///
    /// For kley: `kley install` handles everything in one command.
    /// For yalc tools: `yalc add` copies files but does NOT populate node_modules —
    /// an explicit `npm install` is required for a fair end-state comparison.
    fn install(&self, app_dir: &std::path::Path, home_dir: &std::path::Path, lib_name: &str) {
        match self {
            Tool::Kley => {
                Command::new(KLEY)
                    .args(["install", lib_name])
                    .env("KLEY_HOME", home_dir)
                    .current_dir(app_dir)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .expect("kley install failed");
            }
            Tool::Yalc => {
                let js = require_env("BENCH_YALC_JS");
                Command::new("node")
                    .args([&js, "add", lib_name])
                    .env("HOME", home_dir)
                    .current_dir(app_dir)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .expect("yalc add failed");
                npm_install(app_dir);
            }
            Tool::JimsheenYalc => {
                let js = require_env("BENCH_JIMSHEEN_YALC_JS");
                Command::new("node")
                    .args([&js, "add", lib_name])
                    .env("HOME", home_dir)
                    .current_dir(app_dir)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .expect("@jimsheen/yalc add failed");
                npm_install(app_dir);
            }
        }
    }

    fn push(&self, lib_dir: &std::path::Path, home_dir: &std::path::Path) {
        match self {
            Tool::Kley => {
                Command::new(KLEY)
                    .args(["publish", "--push"])
                    .env("KLEY_HOME", home_dir)
                    .current_dir(lib_dir)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .expect("kley publish --push failed");
            }
            Tool::Yalc => {
                let js = require_env("BENCH_YALC_JS");
                Command::new("node")
                    .args([&js, "publish", "--push"])
                    .env("HOME", home_dir)
                    .current_dir(lib_dir)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .expect("yalc publish --push failed");
            }
            Tool::JimsheenYalc => {
                let js = require_env("BENCH_JIMSHEEN_YALC_JS");
                Command::new("node")
                    .args([&js, "publish", "--push"])
                    .env("HOME", home_dir)
                    .current_dir(lib_dir)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .expect("@jimsheen/yalc publish --push failed");
            }
        }
    }
}

/// Run `npm install --ignore-scripts` in the given directory.
/// Used after `yalc add` to bring node_modules to the same end state kley install produces.
fn npm_install(dir: &std::path::Path) {
    Command::new("npm")
        .args(["install", "--ignore-scripts"])
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("npm install failed — is npm in PATH?");
}

/// Create a realistic library + app pair in a temp directory.
///
/// Library:
///   - has `files: ["dist"]` so publish copies only compiled output (not src/)
///   - has `devDependencies` (typescript) to verify they are stripped on install
///   - mirrors the structure of a real compiled TS library
///
/// App:
///   - has `package-lock.json` so kley detects npm as the package manager
fn setup_fixtures() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let root = tempdir().unwrap();
    let lib_path = root.path().join("my-lib");
    let app_path = root.path().join("my-app");

    std::fs::create_dir_all(lib_path.join("src")).unwrap();
    std::fs::create_dir_all(lib_path.join("dist")).unwrap();

    std::fs::write(
        lib_path.join("package.json"),
        r#"{
  "name": "my-lib",
  "version": "1.0.0",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "files": ["dist"],
  "devDependencies": { "typescript": "*" }
}"#,
    )
    .unwrap();

    std::fs::write(
        lib_path.join("src/index.ts"),
        "export const greet = (name: string): string => `Hello, ${name}!`;\n",
    )
    .unwrap();

    std::fs::write(
        lib_path.join("dist/index.js"),
        "'use strict';\nObject.defineProperty(exports, '__esModule', { value: true });\nexports.greet = (name) => `Hello, ${name}!`;\n",
    )
    .unwrap();

    std::fs::write(
        lib_path.join("dist/index.d.ts"),
        "export declare const greet: (name: string) => string;\n",
    )
    .unwrap();

    std::fs::create_dir(&app_path).unwrap();
    std::fs::write(
        app_path.join("package.json"),
        r#"{"name": "my-app", "version": "1.0.0"}"#,
    )
    .unwrap();
    // package-lock.json tells kley to use npm explicitly
    std::fs::write(
        app_path.join("package-lock.json"),
        r#"{"name":"my-app","version":"1.0.0","lockfileVersion":3,"requires":true,"packages":{"":{"name":"my-app","version":"1.0.0"}}}"#,
    )
    .unwrap();

    (root, lib_path, app_path)
}

/// Cold-start: full publish → install cycle from an empty registry.
/// kley: one command; yalc tools: yalc-add + npm-install.
fn bench_cold_start(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_start");
    group.sample_size(10);
    // npm install dominates per iteration; 30 s gives enough samples
    group.measurement_time(Duration::from_secs(30));

    for tool in &[Tool::Kley, Tool::Yalc, Tool::JimsheenYalc] {
        group.bench_with_input(
            BenchmarkId::new("publish_install", tool.name()),
            tool,
            |b, t| {
                b.iter_with_setup(
                    || {
                        let home = tempdir().unwrap();
                        let (root, lib, app) = setup_fixtures();
                        (home, root, lib, app)
                    },
                    |(home, _root, lib, app)| {
                        t.publish(&lib, home.path());
                        t.install(&app, home.path(), "my-lib");
                    },
                );
            },
        );
    }
    group.finish();
}

/// Iteration speed: publish --push after initial install.
/// After the first install, yalc symlinks .yalc/ into node_modules/ —
/// so --push updates node_modules instantly without a second npm install.
/// kley uses the same symlink fast path for dependency-free packages.
fn bench_iteration_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("iteration_push");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));

    for tool in &[Tool::Kley, Tool::Yalc, Tool::JimsheenYalc] {
        group.bench_with_input(
            BenchmarkId::new("publish_push", tool.name()),
            tool,
            |b, t| {
                b.iter_with_setup(
                    || {
                        let home = tempdir().unwrap();
                        let (root, lib, app) = setup_fixtures();
                        t.publish(&lib, home.path());
                        t.install(&app, home.path(), "my-lib");
                        (home, root, lib, app)
                    },
                    |(home, _root, lib, _app)| {
                        std::fs::write(
                            lib.join("dist/index.js"),
                            "'use strict';\nexports.greet = (name) => `Hi, ${name}!`;\n",
                        )
                        .unwrap();
                        t.push(&lib, home.path());
                    },
                );
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_cold_start, bench_iteration_push);
criterion_main!(benches);
