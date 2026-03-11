# Ticket 001: Implement `push` command

- **Epic**: I (Core Publishing & Linking)
- **Complexity**: Very High

## 1. Description
The `push` command is a critical workflow feature that first publishes the current package and then propagates the new version to all projects where it has been added. This avoids the tedious manual process of running `publish` and then `kley add` in every consuming project. This command is the cornerstone of a fast and efficient iterative development loop.

## 2. Core Prerequisite: Installation Tracking
To know where to push updates, `kley` must track which projects are using which packages.

- The `add` command must be modified to record the installation.
- A new global file, `~/.kley/installations.json`, will be created to store this data.
- The structure will be a map of package names to a list of project paths:
  ```json
  {
    "my-local-lib": [
      "/path/to/project-a",
      "/path/to/project-b"
    ]
  }
  ```

## 3. Acceptance Criteria
1.  A new command `kley push` is implemented, which takes no arguments.
2.  The command must be run from within the directory of the package to be pushed.
3.  It first performs a `publish` operation on the current directory.
4.  After a successful publish, it reads `~/.kley/installations.json` to find all project paths associated with the just-published package.
5.  For each registered project path:
    a. It verifies that the path still exists and contains a `.kley` folder. If not, it should report a warning and continue.
    b. It re-copies the latest version of the package from the store to the project's `.kley/<package-name>` directory.
    c. It updates the `version` field for the package in the project's `kley.lock` file.
6.  The command provides clear console output for both the publish and push steps, listing each project that was updated.

## 4. Implementation Plan
1.  **Modify `add` command**: Update the `add` command to record the project path in `~/.kley/installations.json`.
2.  **Create `push` command module**: Create a new file `src/commands/push.rs`.
3.  **Implement `installations.json` logic**: Create helper functions to read and write to the installations file.
4.  **Implement `push` logic**:
    - First, call the existing `publish()` logic from `publish.rs`.
    - On success, read the local `package.json` to get the package name.
    - Read the `installations.json` file.
    - Iterate through the list of project paths for the package.
    - For each path, perform the copy and `kley.lock` update operations.
5.  **Add `push` to `main.rs`**: Wire up the new command in `main.rs` and `src/commands.rs`.
6.  **Add Tests**: Create unit tests for the push logic.

## 5. Workflow Diagram
```mermaid
flowchart TD
    A[Start: kley push (in 'my-lib' dir)] --> B[Call publish logic for current dir];
    B --> C{Publish successful?};
    C -- No --> D[End with error];
    C -- Yes --> E{Read package name 'my-lib' from local package.json};
    E --> F{Read ~/.kley/installations.json};
    F --> G{Find projects for 'my-lib'};
    G --> H[For each project path];
    H --> I{Path exists?};
    I -- No --> J[Log warning & continue];
    I -- Yes --> K[Copy latest 'my-lib' from store to project's .kley/];
    K --> L{Update project's kley.lock};
    L --> M[Log success for project];
    M --> H;
    H --> N[End];
```
