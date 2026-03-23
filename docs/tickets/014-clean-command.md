# Ticket 014: Implement `clean` command

- **Epic**: V (DX/UX Improvements)
- **Complexity**: Low

## 1. Description
The `clean` command is a general maintenance utility for the `kley` environment. Its primary purpose is to "garbage collect" stale data, ensuring the global installation registry remains accurate and up-to-date.

## 2. Acceptance Criteria
1.  A new command `kley clean` is implemented.
2.  The command can be run from any directory.
3.  It reads the global installation registry (e.g., `~/.kley/registry.json`).
4.  For each package, it iterates through the list of project paths where it is supposedly installed.
5.  For each project path, it checks if the directory still exists on the filesystem.
6.  If a project path no longer exists, the command removes that path from the installation list for that package.
7.  The command prints a summary of its actions (e.g., "Removed 2 stale installation entries.").
8.  If no stale entries are found, it prints a confirmation that the registry is already clean.

## 3. Future Enhancements
- A `--store` flag could be added to also find and offer to remove packages from the store that are no longer installed in any projects.

## 4. Implementation Plan
1.  Add `Clean` to the `Commands` enum in `src/main.rs`.
2.  Create a new command module: `src/commands/clean.rs`.
3.  Implement the core logic:
    a. Read and parse the global installation registry.
    b. Create a mutable copy of the data.
    c. Iterate through the packages and their installation paths, checking for existence with `Path::exists()`.
    d. Remove non-existent paths from the data structure.
    e. Compare the original and modified data to see if changes were made.
    f. If changed, serialize and write the new data back to the registry file and print a summary.
    g. If unchanged, print a "nothing to clean" message.
4.  Wire up the command in `main.rs`.
5.  Add tests to verify that stale entries are correctly identified and removed.
