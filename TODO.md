# kley Project TODO List

This document outlines the current tasks and future enhancements for the Kley project.

## Current Development Tasks

- [ ] **Implement `ignore` crate for file filtering during `publish`**: Enhance the `publish` command to use the `ignore` crate for respecting `.gitignore` files and providing configurable file exclusion patterns, rather than hardcoding exclusions.
- [ ] **Improve file copying logic in `publish`**: Ensure robust and efficient copying, potentially handling incremental updates or checksums for large files if performance becomes an issue.
- [ ] **Consider automatic `package.json` modification for `add` command**: Explore options to automatically add the `file:` dependency to the host project's `package.json` upon running `kley add`, reducing manual steps for the user.

## Future Enhancements / New Features

- [ ] **Add `kley uninstall <package-name>` command**: Implement functionality to remove a locally added package from a host project. This would involve removing the copied files and potentially updating `package.json` (if automatic modification is implemented).
- [ ] **Add `kley list` command**: Display a list of all packages currently stored in the Kley's local repository (`~/.kley/packages/`).
- [ ] **Add `kley clear` command**: Remove all packages from the Kley's local repository.
- [ ] **Add `kley prune` command**: Remove packages from the local repository that are no longer referenced by any project (requires tracking references).
- [ ] **Integrate `walkdir` more effectively**: Utilize `walkdir` for more controlled and efficient directory traversal, especially when combined with the `ignore` crate.
- [ ] **Implement proper versioning in the store**: Currently, `publish` overwrites existing packages. Consider a mechanism for storing multiple versions of the same package, similar to `yalc`.
- [ ] **Improve error messages**: Make error messages more user-friendly and actionable.

## Code Quality & Maintainability

- [ ] **Add Unit Tests**: Implement comprehensive unit tests for core functionalities (e.g., parsing `package.json`, path manipulation).
- [ ] **Add Integration Tests**: Implement integration tests to verify the end-to-end flow of `publish` and `add` commands.
- [ ] **Refine `fs_extra` usage**: Review and optimize calls to `fs_extra` for performance and reliability.
- [ ] **Cross-platform compatibility**: Ensure full compatibility and testing across Windows, macOS, and Linux.
