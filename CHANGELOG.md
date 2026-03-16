# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
