# Ticket 020: Implement `install` (no args) as Update All

- **Epic:** V (DX/UX Improvements)
- **Complexity:** `Medium`
- **Depends On:** #017

## 1. Problem Statement

Currently, to update all `kley`-managed packages in a project, a user must run `kley update`. It would be convenient and intuitive for `kley install` (with no arguments) to perform this same "update all" action, similar to how `npm install` refreshes the `node_modules` directory.

## 2. Proposed Solution

The `kley install` command will be modified to handle the case where no package name is provided.

### Logic for `kley install` (no arguments)

1.  **Read Lockfile:** The command will read the project's local `kley.lock` file to identify all packages currently managed by `kley`.
2.  **Iterate and Update:** For each package listed in the lockfile, `kley` will perform the same logic as the `kley update <package-name>` command. This involves:
    a. Checking the global `~/.kley/registry.json` for the latest published version of the package.
    b. If the local version is outdated, copy the new version from the global store into the project's `./.kley/<package-name>` directory.
    c. If the package was installed via a direct copy to `node_modules`, update that copy as well.
3.  **Provide Summary:** After attempting to update all packages, provide a summary to the user about which packages were updated.

## 3. Acceptance Criteria

- Running `kley install` with no arguments updates all packages listed in `kley.lock` to their latest published versions.
- The behavior is functionally equivalent to running `kley update` for every installed package.
- The user is informed about the actions taken.
