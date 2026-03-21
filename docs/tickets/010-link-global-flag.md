# T010: Add --global flag to the 'link' command

## Goal

To add an optional `--global` flag to the `kley link` command. When this flag is present, kley will bypass the local copy step and create a symlink directly from the package's location in the global kley store to the project's `node_modules` directory.

This provides a "fast-refresh" mode for developers who want immediate updates after publishing a new version of a dependency.

## Expected Result

- A user runs `kley link my-lib --global`.
- A symlink is successfully created at `./node_modules/my-lib` pointing **directly** to the package in the global kley store.
- A success message clearly indicates that a **global** link was created.
- The project's `package.json` and `kley.lock` files are **not** modified.

## Schema of Work

```mermaid
graph TD
    A[Start] --> B{User runs 'kley link <pkg> [--global]'};
    B --> C{Parse command};
    C --> D{--global flag present?};
    D -- Yes --> E_Global[Source = Global Store Path];
    D -- No --> E_Local[Copy to local .kley folder];
    E_Local --> F_Local[Source = Local Copy Path];
    E_Global --> G;
    F_Local --> G;
    G{Define target symlink path: './node_modules/<pkg>'};
    G --> H{Target path already exists?};
    H -- Yes --> I[Remove existing file/directory];
    I --> J;
    H -- No --> J;
    J[Create symlink from Source to target path];
    J --> K[Display success message];
    K --> L[End];
```

## Implementation Details

1.  **CLI Parsing (`clap`):**
    - Add an optional `--global` boolean flag to the `link` subcommand.
2.  **Conditional Logic:**
    - In the `link` command's handler, check if the `--global` flag is active.
    - If `true`, set the source path for the symlink to the package's directory in the global store.
    - If `false` (the default), perform the copy to the local `.kley` folder and set the source path to that local copy.
