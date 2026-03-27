# Ticket 015: Implement Global Package Registry

- **Epic**: II (Publish Automation & Linking Speed)
- **Complexity**: High

## 1. Description
This task introduces a global **Package Registry**, a foundational feature that tracks all published packages, their metadata, and where they are installed. This registry is crucial for the performance and functionality of commands like `push`, `update`, `list`, `clean`, and `unpublish`.

The registry will be a single JSON file located at `~/.kley/registry.json`.

## 2. Data Structure
The `registry.json` file will contain a single `packages` object. Each key in this object is a package name, and the value is an object containing its metadata and installation paths.

```json
{
  "packages": {
    "my-lib": {
      "version": "1.2.3",
      "lastUpdated": "2026-03-23T10:00:00Z",
      "installations": [
        "/path/to/project-a",
        "/path/to/project-b"
      ]
    },
    "another-lib": {
      "version": "0.5.1",
      "lastUpdated": "2026-03-22T15:30:00Z",
      "installations": []
    }
  }
}
```

## 3. Core Logic
- A dedicated module (e.g., `src/registry.rs`) will abstract all read/write operations to `registry.json`.
- This module must handle file creation, empty/malformed JSON, and provide safe functions for modification.
- Key functions implemented in this phase include:
    - `Registry::new()`: To load or initialize the registry.
    - `Registry::save()`: To persist changes to disk.
    - `update_package_version()`: Called by `publish` to update a package's metadata.
    - `add_package_installation()`: Called by `add` and `link` to register a new installation.
    - `remove_package_installation()`: Called by `remove` to deregister an installation.
    - `remove_all_installations()`: Called by `remove --all`.

## 4. Modifications to Commands

### `publish` command
- **Change:** After successfully copying files to the store, `publish` **must** call `registry::update_package_metadata()` to record the new version and update the `lastUpdated` timestamp.

### `add` & `link` commands
- **Change:** After a package is successfully added or linked, these commands **must** call `registry::add_installation()` to add the current project's path to the package's `installations` list.

### `remove` command
- **Change:** When a package is removed from a project, `remove` **must** call `registry::remove_installation()` to delete the project's path from the package's `installations` list.

### `unpublish` command
- **Change:** This command will be a heavy user of the registry.
    - `unpublish`: Will call `registry::remove_package()` to completely delete the top-level entry for the package.
    - `unpublish --push`: Will use `registry::get_package()` to find all installation paths and then call `registry::remove_package()` at the end.

