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

## 2. Proposed Solution

Canonicalize **both** the path and the home directory inside `normalized_path()`, so `strip_prefix` compares apples to apples:

```rust
pub fn normalized_path(path: &Path, home: Option<&PathBuf>) -> String {
    let path = fs::canonicalize(path).unwrap_or(path.to_path_buf());

    if let Some(home_dir) = home {
        let home_canonical = fs::canonicalize(home_dir).unwrap_or_else(|_| home_dir.clone());
        if let Ok(stripped_path) = path.strip_prefix(home_canonical) {
            return format!("~/{}", stripped_path.display());
        }
    }

    // On Windows, strip the UNC prefix \\?\ for cleaner output
    let result = path.to_string_lossy().into_owned();
    #[cfg(target_os = "windows")]
    let result = result.strip_prefix(r"\\?\").unwrap_or(&result).to_string();

    result
}
```

### Changes

1. **Canonicalize `home_dir`** inside `normalized_path()` so both sides of `strip_prefix` use the same path format.
2. **Strip UNC prefix** `\\?\` from the final output on Windows — this prefix is an internal Windows detail and should never be shown to users.

## 3. Acceptance Criteria

- On all platforms, `normalized_path()` returns `~/...` when the path is inside the home directory.
- On Windows, UNC prefix `\\?\` is never shown in CLI output.
- Existing tests pass on Windows, macOS, and Linux.
- No regression in path comparison logic used by `kley` internally (registry installations, etc.).
