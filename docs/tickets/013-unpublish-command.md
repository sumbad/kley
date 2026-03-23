# Ticket 013: Implement `unpublish` command

- **Epic**: II (Publish Automation & Linking Speed)
- **Complexity**: Medium

## 1. Description
The `unpublish` command removes a package from the local `kley` store. It has two modes: a default "soft" unpublish that preserves consumer projects, and a "--push" mode that performs a "hard" unpublish, cleaning the package from all projects where it is used.

## 2. Acceptance Criteria

### `kley unpublish` (Default "Soft" Unpublish)
1.  The command must be run from a directory containing a `package.json` file.
2.  It checks the global registry (`~/.kley/installations.json`) to see if the package is in use.
3.  If the package is in use, it displays a non-blocking warning and requires confirmation:
    > **Warning:** 'my-lib' is used by 2 projects. This action will remove the package from the store, breaking these projects upon the next install. To clean up all projects, use `kley unpublish --push`. Proceed? (y/N)
4.  Upon confirmation, it deletes the package from the store (`~/.kley/packages/<package-name>`).
5.  It also removes all installation entries for the package from the global registry (`~/.kley/installations.json`) to maintain data consistency.

### `kley unpublish --push` ("Hard" Unpublish)
1.  A `--push` flag is added to the `unpublish` command.
2.  The command finds all consumer projects from the global registry.
3.  It displays a list of affected projects and asks for a final confirmation:
    > **Attention:** This will permanently remove 'my-lib' from the store AND from the following 2 projects:
    >   - `/path/to/project-a`
    >   - `/path/to/project-b`
    >
    > Proceed? (y/N)
4.  Upon confirmation, it iterates through each consumer project and performs the full `kley remove` logic (updating `package.json`, `kley.lock`, and deleting the `.kley/<pkg>` directory).
5.  After cleaning all consumer projects, it deletes the package from the store.
6.  Finally, it cleans all related entries from the global registry.

## 3. Implementation Plan
1.  Add `Unpublish` to the `Commands` enum in `src/main.rs`, including an optional `--push` flag.
2.  Create a new command module: `src/commands/unpublish.rs`.
3.  Implement the core logic, branching based on the presence of the `--push` flag.
4.  **Shared Logic:** Reading `package.json`, reading the global registry, and user confirmation prompts.
5.  **Default Logic:** Implement file system deletion for the store and registry filtering.
6.  **Push Logic:** Implement the loop that executes `remove` logic in each consumer project. This will likely require refactoring the `remove` command's core logic into a reusable function that can be called from `unpublish`.
7.  Wire up the command in `main.rs`.
8.  Add tests for both modes.

