# Ticket 012: Implement `list` command

- **Epic**: V (DX/UX Improvements)
- **Complexity**: Low

## 1. Description
The `list` command provides visibility into the local `kley` store. It allows users to see which packages have been published, their versions, and, most importantly, where they are currently being used. This serves as a simpler, more ergonomic replacement for `yalc`'s `installations` command group.

## 2. Acceptance Criteria
1.  A new command `kley list [package-name]` is implemented.
2.  **Default Behavior (`kley list`):**
    *   If no package name is given, the command scans the `~/.kley/packages` directory.
    *   It reads the global installation registry (e.g., `~/.kley/registry.json`).
    *   It displays a list of all published packages, their versions, and a sub-list of project paths where each package is installed.
3.  **Filtered Behavior (`kley list <package-name>`):**
    *   If a package name is provided, it displays the version and installation locations for only that specific package.
    *   If the package is not found in the store, a "Package not found" message is displayed.
4.  The output must be formatted for clear readability.

## 3. Prerequisites
- This command's full functionality depends on a global installation tracking mechanism (e.g., `~/.kley/registry.json`). The `add` and `link` commands must be responsible for populating this registry upon successful execution.

## 4. Example Output

### `kley list`
```
Found 2 packages in the store:

● my-lib (v1.2.0)
  └─ used in: /Users/dev/projects/my-app

● another-lib (v0.5.1)
  ├─ used in: /Users/dev/projects/my-app
  └─ used in: /Users/dev/projects/another-app
```

### `kley list my-lib`
```
● my-lib (v1.2.0)
  └─ used in: /Users/dev/projects/my-app
```

## 5. Implementation Plan
1.  Add `List` to the `Commands` enum in `src/main.rs` with an optional `name` argument.
2.  Create a new command module: `src/commands/list.rs`.
3.  Implement helper functions to:
    *   Read package directories from `~/.kley/packages`.
    *   Parse the `package.json` in each to get the version.
    *   Read and parse the `~/.kley/registry.json` file.
4.  Implement the main `list` function to handle both the default and filtered cases.
5.  Use a library like `colored` to format the output for better readability.
6.  Wire up the command in `main.rs`.
7.  Add tests to verify output formatting and correctness.
