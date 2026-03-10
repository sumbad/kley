# Ticket 006: Create and manage kley.lock

- **Epic**: I (Core Publishing & Linking)
- **Complexity**: High

## 1. Description
To enhance the robustness and traceability of locally managed packages, this task introduces a `kley.lock` file. This file will serve as a manifest, tracking all packages added to a host project via the `kley add` command. It will lock their versions and provide a clear record of kley-managed dependencies.

This is a foundational feature that will enable future capabilities such as `kley update`, `kley status`, and `kley remove`.

## 2. Acceptance Criteria
1.  When `kley add <package-name>` is executed successfully, a `kley.lock` file must be created in the root of the host project if it does not already exist.
2.  The `kley.lock` file must be a valid, pretty-printed JSON file for readability.
3.  The file structure should be as follows:
    ```json
    {
      "packages": {
        "<package-name>": {
          "version": "<package-version>"
        }
      }
    }
    ```
4.  The `<package-version>` must be read from the `package.json` of the published package located in the kley store (`~/.kley/packages/<package-name>`).
5.  If `kley.lock` already exists, the `kley add` command must update it by adding the new package information or updating the version of an existing package. The command should not overwrite the entire file.
6.  The command's output should confirm that `kley.lock` has been created or updated.

## 3. Technical Details
- The core logic will be implemented within the `add` command handler in `src/commands/add.rs`.
- The implementation will need to:
    - Read the existing `kley.lock` file if it exists.
    - Deserialize the JSON content into a Rust struct.
    - Read the `package.json` of the package being added to retrieve its version.
    - Update the struct with the new package information.
    - Serialize the struct back into a pretty-printed JSON string.
    - Write the updated content back to `kley.lock`.
- Error handling must be robust, covering file I/O errors and JSON parsing errors.
