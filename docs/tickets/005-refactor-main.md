# TICKET: 005 - Refactor main.rs

**Epic:** V (Code Quality & Testing)
**Complexity:** Medium

## Description
The `main.rs` file currently contains all application logic for commands, helpers, and tests. This makes the file large and difficult to maintain.

## Plan
1. Create a new directory `src/commands`.
2. Move the `publish` command logic into `src/commands/publish.rs`.
3. Move the `add` command logic and its helpers into `src/commands/add.rs`.
4. Move associated tests to their respective command modules.
5. Update `main.rs` to be a simple entry point that calls functions from the new modules.
