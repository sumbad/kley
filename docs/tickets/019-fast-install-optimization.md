# Ticket 019: Future - Implement "Fast Install" Optimization

- **Epic:** III (Streamlined Local Package Workflow)
- **Complexity:** `High`
- **Depends On:** #017, #018

## 1. Problem Statement

The `kley install` command (as defined in #017) always delegates to the native package manager, which can be slow if all required dependencies are already present in the project.

This ticket describes a future optimization to create a "fast path" for `npm` and `yarn v1` projects, where `kley` can skip the native package manager call if it's not needed.

**Note:** This is a future enhancement and should only be implemented after the base `install` command is stable and proven.

## 2. Proposed Solution

The `kley install` command can be augmented with the following logic **only when `npm` or `yarn v1` is detected**:

1.  **Dependency Check:** Before delegating to the native PM, `kley` will parse the dependencies of the package being installed.
2.  **Verify Installation:** It will then check if all of those dependencies (respecting semver ranges) are already present in the host project's `node_modules`. This can be done by parsing the output of `npm list --json` or `yarn list --json`.
3.  **Execute Fast Path:** If all dependencies are met, `kley` will perform a direct installation:
    a. Manually add the `file:./.kley/<pkg>` entry to `package.json`.
    b. Copy the package files directly into `node_modules/<pkg>`.
    c. Skip the call to `npm install`.
4.  **Fallback:** If any dependency is missing, `kley` will fall back to the standard behavior of delegating to the native package manager.

## 3. Acceptance Criteria (for future implementation)

- When using `npm` or `yarn v1`, `kley install` is significantly faster if all dependencies are already met.
- The "fast path" correctly verifies dependencies using semantic versioning.
- The optimization does not run when `pnpm` or `yarn v2+` are detected.
