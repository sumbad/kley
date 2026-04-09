# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
- **Project Refactoring**: Project restructured into a library (`src/lib.rs`) for better modularity and testability.
- **CLI Styling**: Implemented styling for a more visually appealing command-line interface.
- **Path Normalization Utility**: Added `utils::normalized_path` to display user-friendly, tilde-prefixed paths.
- **Demo Assets**: Included GIF demonstrations for `README.md`.

### Changed
- **Improved CLI Output**: Enhanced output messages across `add`, `link`, `publish`, `remove`, `unpublish`, and `update` commands with better clarity, colors, emojis, and contextual information.
- **`README.md` Enhancements**: Updated "Getting started" section with detailed scenarios, GIF demos, and Mermaid diagrams. Reordered installation methods.
- **Logging Consistency**: Replaced many direct `println!` calls with `tracing::info!` for internal messages, improving logging separation.
- **Test Updates**: Adjusted integration tests to assert against the new, improved CLI output.

---

## [0.4.0] - 2026-04-06

### Added
- **`unpublish` Command**: Implemented the `kley unpublish` command with a `--push` flag to remove packages from the local store and optionally from all consumer projects.
- **`update` Command**: Implemented the `kley update [package...]` command to sync installed packages with the latest versions from the local store. It can update all packages or specific ones.
- **`--push` flag for `publish`**: The `publish` command now accepts a `--push` flag to automatically update all consumer projects with the new version.

---

## [0.3.0] - 2026-03-27

### Added
- **Global Package Registry**: Implemented a new global registry (`~/.kley/registry.json`) to track metadata and installation locations for all published packages. This is a foundational feature for upcoming automation commands.

### Changed
- The `publish`, `add`, and `remove` commands have been updated to interact with the new Global Package Registry.

---

## [0.2.0] - 2026-03-22

### Added
- **`link` Command**: Implemented the `kley link <package-name>` command to create a direct symbolic link from a project's `kley` store to the project's `node_modules` directory. This provides a lightweight alternative to `add` without modifying `package.json`.

---

## [0.1.3] - 2026-03-20

### Added
- Publish to npm like `kley-cli`.
- Publish to crates.io like `kley`.
- Create unified release process for crates.io and npm.

---

## [0.1.2] - 2026-03-16

### Fixed
- **`add` Command**: The `kley add` command no longer reorders properties in `package.json`, preventing unnecessary diffs.

---

## [0.1.1] - 2026-03-15

### Fixed
- **Publish Command**: Improved the file filtering logic in the `publish` command to more accurately simulate npm's behavior, correctly respecting `.npmignore` and `.gitignore` rules.


---

## [0.1.0] - 2026-03-12

### Added
- **Core Commands**: Implemented the initial set of core commands for local package management.
  - `kley publish`: Publishes a package to the local kley store.
  - `kley add <pkg>`: Adds a published package to a project, updating `package.json` and creating `kley.lock`.
  - `kley remove <pkg>`: Removes a kley-managed package from a project.
- **Lockfile**: Introduced `kley.lock` to track locally added dependencies.
- **Automatic `package.json` Modification**: `add` and `remove` commands now automatically update `package.json`.

### Changed
- **Project Structure**: Refactored the CLI into a modular command-based architecture for better maintainability.
- **Internal**: Added a comprehensive suite of unit tests for core commands.
