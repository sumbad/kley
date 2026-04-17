# Ticket 018: Implement Robust Package Manager Detection

- **Epic:** III (Streamlined Local Package Workflow)
- **Complexity:** `Medium`

## 1. Problem Statement

Many of `kley`'s commands need to interact with the host project's package manager (`npm`, `pnpm`, `yarn`). A simple lockfile check is not sufficient, as a project might contain a `yarn.lock` file but the user may still be using `npm` to manage dependencies. A more reliable detection mechanism is required.

## 2. Proposed Solution

A utility function will be created to provide a reliable guess of the project's package manager. The detection will use a clear hierarchy of configuration sources.

### Detection Logic Hierarchy

The function will check the following sources in order, returning the first valid result found:

1.  **`kley.lock`:** Check for a `packageManager` field within the project's `kley.lock` file. This is the highest priority and allows users to specify a package manager for `kley`'s use explicitly.
    ```json
    // kley.lock
    {
      "packageManager": "pnpm",
      "packages": { ... }
    }
    ```

2.  **`package.json`:** If not found in `kley.lock`, check for the standard `packageManager` field in the project's `package.json`.
    ```json
    // package.json
    {
      "name": "my-project",
      "packageManager": "yarn@1.22.19"
    }
    ```

3.  **Lock Files:** If no explicit configuration is found, fall back to detecting the presence of lock files in the project root.
    - `pnpm-lock.yaml` -> `pnpm`
    - `yarn.lock` -> `yarn`
    - `package-lock.json` -> `npm`

4.  **Default:** If no package manager can be determined, default to `npm` and display a warning to the user.

The function should return a simple enum, e.g., `PackageManager::{Npm, Pnpm, Yarn}`.

## 3. Acceptance Criteria

- A reusable function `detect_package_manager()` is implemented.
- The function correctly identifies the package manager based on the specified hierarchy: `kley.lock` -> `package.json` -> lock files -> default.
- The function is used by the `kley install` command.
