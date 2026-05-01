# Ticket 022: Fix `normalized_path` UNC Path Issue on Windows

- **Epic:** VI (DX/UX Improvements)
- **Complexity:** `Low`
- **Depends On:** None

## 1. Problem Statement

On Windows, `normalized_path()` displays full UNC paths instead of the shortened `~/...` format. This affects the output of `kley publish --push` and `kley remove`, making the CLI output ugly and confusing for Windows users.

### Root Cause

`normalized_path()` in `src/utils.rs` uses `fs::canonicalize()` on the input path, which on Windows returns a UNC-extended path (e.g., `\\?\C:\Users\runneradmin\...`). Meanwhile, the `home` parameter (from `get_kley_home_dir()`) returns a regular path (e.g., `C:\Users\runneradmin\...`). The `strip_prefix` call fails because:

1. **UNC prefix mismatch**: `\\?\C:\...` cannot be stripped with prefix `C:\...`
2. **8.3 short name mismatch**: canonicalize may resolve `RUNNER~1` → `runneradmin`, changing the string

### Example

**Expected output (macOS/Linux):**
```
✔️ Updated ~/app-a to the latest version of my-lib
```

**Actual output (Windows):**
```
✔️ Updated \\?\C:\Users\runneradmin\AppData\Local\Temp\.tmpXXX\app-a to the latest version of my-lib
```

## 2. Solution

Skip `fs::canonicalize()` on Windows via conditional compilation. On Unix it is still needed to resolve symlinks (e.g. `/var/folders` → `/private/var/folders` on macOS), but on Windows it only adds the problematic `\\?\` UNC prefix.

```rust
pub fn normalized_path(path: &Path, home: Option<&PathBuf>) -> String {
    #[cfg(not(windows))]
    let path = fs::canonicalize(path).unwrap_or(path.to_path_buf());

    if let Some(home_dir) = home
        && let Ok(stripped_path) = path.strip_prefix(home_dir)
    {
        return format!("~/{}", stripped_path.display());
    }

    path.to_string_lossy().into_owned()
}
```

### Why this approach

- **No new dependencies** — avoids adding `dunce` or similar crates
- **Minimal change** — one `#[cfg]` line instead of canonicalizing both sides + UNC stripping
- **Correct by design** — on Windows, paths passed to `normalized_path` are already absolute (from `current_dir()` or `join()`), so canonicalize is unnecessary
- **Preserves macOS behavior** — symlink resolution via `canonicalize` still works on Unix

## 3. Acceptance Criteria

- On all platforms, `normalized_path()` returns `~/...` when the path is inside the home directory.
- On Windows, UNC prefix `\\?\` is never shown in CLI output.
- Existing tests pass on Windows, macOS, and Linux.
- No regression in path comparison logic used by `kley` internally (registry installations, etc.).
