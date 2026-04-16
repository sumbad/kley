# Ticket 018: Implement Robust Package Manager Detection

- **Epic:** III (Streamlined Local Package Workflow)
- **Complexity:** `Medium`

## 1. Problem Statement

Many of `kley`'s commands need to interact with the host project's package manager (`npm`, `pnpm`, `yarn`). A simple lockfile check is not sufficient, as a project might contain a `yarn.lock` file but the user may still be using `npm` to manage dependencies. A more reliable detection mechanism is required.

## 2. Proposed Solution

A utility function will be created to provide a reliable guess of the currently used package manager. The detection will use a multi-layered approach.

### Detection Logic

The function will check the following sources in order, returning the first positive result:

1.  **Environment Variable:** Check the `npm_config_user_agent` environment variable. This is the most reliable source, as it's set by the package managers themselves during execution.
    - `npm/...` -> npm
    - `yarn/...` -> yarn
    - `pnpm/...` -> pnpm

2.  **Lock Files:** If the environment variable is not set, fall back to the presence of lock files in the project root.
    - `pnpm-lock.yaml` -> pnpm
    - `yarn.lock` -> yarn
    - `package-lock.json` -> npm

The function should return a simple enum, e.g., `PackageManager::{Npm, Pnpm, Yarn}`.

## 3. Acceptance Criteria

- A reusable function `detect_package_manager()` is implemented.
- The function correctly identifies the package manager based on environment variables first, then lock files.
- The function is used by the `kley install` command (and any other future commands that need it).
