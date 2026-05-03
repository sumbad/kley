# Ticket 026: Implement `.kleyrc` Configuration File

- **Epic:** VI (DX/UX Improvements)
- **Complexity:** Low
- **Depends On:** None

## 1. Problem Statement

Some kley flags are used consistently in a particular project or by a particular user. For example, a developer who always runs `kley publish --push --changed --scripts` must type all three flags every time. There is no way to set default flags per-project or per-user.

## 2. Proposed Solution

Support a `.kleyrc` configuration file that specifies default options for kley commands. The file is looked up in two locations with the following priority:

1. **Project-level**: `./.kleyrc` (committed to git or gitignored, team-shared or personal)
2. **User-level**: `~/.kleyrc` (personal defaults across all projects)

Project-level settings override user-level settings.

### File Format

Simple key-value format, one setting per line:

```ini
# .kleyrc
push=true
changed=true
scripts=false
store-folder=/custom/path/.kley-store
workspace-resolve=true
```

### Supported Settings

| Setting | Applies to | Type | Description |
|---------|-----------|------|-------------|
| `push` | `publish` | bool | Default `--push` flag |
| `changed` | `publish` | bool | Default `--changed` flag |
| `scripts` | `publish`, `add`, `update`, `install` | bool | Default `--scripts` flag |
| `store-folder` | global | string | Override default store location (`~/.kley`) |
| `workspace-resolve` | `add` | bool | Resolve `workspace:` protocol in dependencies |
| `sig` | `publish` | bool | Append content hash to version |

### Precedence

CLI flags **always override** `.kleyrc` settings:

```
CLI flag > ./.kleyrc > ~/.kleyrc > built-in default
```

## 3. Implementation Plan

1. Add a `kleyrc` module (e.g. `src/kleyrc.rs`).
2. Implement a `load_kleyrc()` function that:
   a. Checks `~/.kleyrc` for user-level defaults.
   b. Checks `./.kleyrc` for project-level defaults (starting from cwd).
   c. Merges them with project-level taking priority.
3. Modify `main.rs` to load `.kleyrc` after CLI parsing and apply defaults to unset flags.
4. Use the `ini` or simple custom parser (the format is trivial enough for a hand-rolled parser — avoid adding a dependency).

## 4. Usage Examples

```ini
# ~/.kleyrc — personal defaults
scripts=true

# ./.kleyrc — project defaults
push=true
changed=true
```

```bash
# With the above .kleyrc, this is equivalent to:
kley publish                # → kley publish --push --changed

# CLI flag overrides .kleyrc:
kley publish --no-push      # → publish without push, despite .kleyrc
```

## 5. Acceptance Criteria

- kley reads `.kleyrc` from the project directory and user home directory.
- Project-level `.kleyrc` overrides user-level `.kleyrc`.
- CLI flags always override `.kleyrc` settings.
- Supported settings are applied as defaults for their respective commands.
- If `.kleyrc` doesn't exist, kley behaves exactly as before — backward compatible.
- Invalid settings in `.kleyrc` produce a warning but don't crash kley.
