# Ticket 030: `publish --push` deletes package dependencies in consumer projects

- **Epic:** III (Streamlined Local Package Workflow)
- **Complexity:** Low
- **Depends On:** None

## 1. Problem Statement

When `kley publish --push` updates a package in consumer projects, it calls `copy_from_registry()` which does `fs::remove_dir_all()` on `.kley/<pkg>/` before copying the new version from the store. This deletes everything — including `node_modules/` inside `.kley/<pkg>/` that was created by the package manager during `kley install`.

Starting with npm 5, `npm install <local-path>` creates a **symlink** from `node_modules/<pkg>/` to the local folder (`.kley/<pkg>/`), not a physical copy. This means:

```
.kley/my-lib/                    ← copy from store
node_modules/my-lib/             ← symlink → .kley/my-lib/
.kley/my-lib/node_modules/       ← my-lib dependencies (npm install here)
```

When `publish --push` runs `remove_dir_all(.kley/my-lib/)`, all three locations are affected — the symlink target and its `node_modules/` are destroyed. After the fresh copy from the store, the package's own dependencies are gone and the library breaks until the user manually runs `npm install` again.

### Affected Scenarios

| Command | After `publish --push` | Broken? |
|---------|----------------------|---------|
| `kley install my-lib` | `.kley/my-lib/node_modules/` deleted | ✅ Yes — deps gone |
| `kley link my-lib` | `.kley/my-lib/node_modules/` deleted | ✅ Yes — deps gone (same symlink) |
| `kley add my-lib` | `.kley/my-lib/node_modules/` deleted | ✅ Yes — deps gone (same symlink) |

All three commands result in npm creating a symlink to `.kley/my-lib/`, so all are affected.

## 2. Proposed Solution

Replace `remove_dir_all()` + full copy with a **synchronization** approach that preserves `node_modules/` inside `.kley/<pkg>/`.

### Logic

1. **Sync files** from store to `.kley/<pkg>/`:
   - Overwrite existing files with new versions.
   - Add new files that didn't exist before.
   - Remove files that exist in the old version but not in the new one.
   - **Never touch** `node_modules/` directory inside `.kley/<pkg>/`.
2. **Check dependencies**: Compare `dependencies` and `peerDependencies` in the new `package.json` against the previous version.
3. **Warn if changed**: If dependencies changed, print a warning:
   ```
   ⚠️ Dependencies of 'my-lib' have changed. Run 'kley install my-lib' to update them.
   ```

### Implementation

Modify `copy_from_registry()` in `src/utils.rs`:

- Instead of `remove_dir_all()` + `fs_extra::dir::copy()`, use a sync walk:
  1. Walk the new version's file tree from the store.
  2. For each file: copy to `.kley/<pkg>/`, overwriting if exists.
  3. For each directory: create if not exists, skip `node_modules/`.
  4. For cleanup: walk the old version's file tree, remove files/dirs that no longer exist in the new version (again, skip `node_modules/`).
- Or alternatively: use `fs_extra::dir::copy()` with `overwrite` option into the existing directory (without removing it first), then clean up stale files by comparing the old and new file lists.

## 3. Acceptance Criteria

- After `kley publish --push`, `.kley/<pkg>/node_modules/` is preserved in consumer projects.
- Package source files in `.kley/<pkg>/` are correctly updated to the new version.
- Files removed in the new version are cleaned up from `.kley/<pkg>/` (except `node_modules/`).
- If the package's `dependencies` or `peerDependencies` changed, a warning is shown advising to run `kley install <pkg>`.
- If dependencies are unchanged, the package works correctly after `publish --push` without any manual step.
