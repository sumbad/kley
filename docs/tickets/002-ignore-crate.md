# TICKET: 002 - Use 'ignore' crate for filtering

**Epic:** I (Core Publishing & Linking)
**Complexity:** High

## Description
The current file copying logic in the `publish` command is a simplification for the MVP. It manually excludes common directories like `node_modules` and `.git`. This approach is not robust and doesn't respect standard project-specific ignore files like `.gitignore` and `.npmignore`.

This task is to replace the manual file walk with the `ignore` crate, which is the standard solution for this problem in the Rust ecosystem.

## Plan
1.  Add the `ignore` crate as a dependency to `Cargo.toml`.
2.  Modify the `publish` function in `src/commands/publish.rs`.
3.  Replace the `fs::read_dir` logic with a `ignore::WalkBuilder`.
4.  Configure the `WalkBuilder` to read rules from `.gitignore` and also look for `.npmignore` if it exists.
5.  Iterate through the results of the walk.
6.  For each valid file, copy it to the store, making sure to preserve its relative directory structure.
7.  Add a new test case to verify that files listed in a `.gitignore` are correctly excluded from the publishing process.
