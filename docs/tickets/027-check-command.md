# Ticket 027: Implement `kley check` Command

- **Epic:** VI (DX/UX Improvements)
- **Complexity:** Low
- **Depends On:** None

## 1. Problem Statement

When using `kley add` in a project, `package.json` is modified to include `file:./.kley/<pkg>` references. If a developer accidentally commits these references to git and the project is published to npm (or deployed), the build will break — `file:` references are invalid outside the local machine.

Currently there is no automated way to catch this mistake before it reaches CI or production.

## 2. Proposed Solution

A `kley check` command that verifies the project's `package.json` does not contain any kley-managed `file:./.kley/` or `link:./.kley/` references. This command is designed to be used in:

- **pre-commit hooks** (e.g. husky, lint-staged)
- **CI pipelines** (as a validation step before build/deploy)

### Logic

1. Read the project's `package.json`.
2. Scan `dependencies`, `devDependencies`, and `peerDependencies` for any value matching `file:./.kley/*` or `link:./.kley/*`.
3. If any such references are found:
   - Print an error listing each kley-managed dependency and which section it's in.
   - Exit with code `1` (failure).
4. If no kley references are found:
   - Print a brief confirmation.
   - Exit with code `0` (success).

### Additional Check: `kley.lock` vs `package.json` Consistency

Optionally (with `--strict` flag), also verify that every package in `kley.lock` has a corresponding entry in `package.json` and vice versa. This catches orphaned state after manual `package.json` edits.

## 3. Usage Examples

```bash
# Basic check — are there kley references in package.json?
kley check
# Output: Error: Found kley-managed dependencies in package.json:
#   - "my-lib": "file:./.kley/my-lib" (dependencies)
#   - "test-utils": "file:./.kley/test-utils" (devDependencies)
# Run 'kley retreat --all' to temporarily switch to remote versions.

# In a pre-commit hook (husky + lint-staged):
# .husky/pre-commit:
kley check || exit 1

# In CI:
kley check  # fails the pipeline if kley refs leaked into package.json

# Strict mode — also check kley.lock consistency
kley check --strict
```

## 4. Implementation Plan

1. Add `Check` to the `Commands` enum in `src/main.rs`, with an optional `--strict` flag.
2. Create a new command module: `src/commands/check.rs`.
3. Implement the core logic:
   a. Read `package.json` from the current directory.
   b. Iterate over dependency sections, checking for `file:./.kley/` or `link:./.kley/` prefixes.
   c. If found, collect and report them.
   d. If `--strict`, also compare `kley.lock` entries against `package.json` entries.
4. Wire up the command in `main.rs`.
5. Add tests covering: clean project, project with kley refs, strict mode.

## 5. Acceptance Criteria

- `kley check` exits with code `0` when no `file:./.kley/` or `link:./.kley/` references are found in `package.json`.
- `kley check` exits with code `1` and lists all kley-managed references when found.
- The error message suggests running `kley retreat --all` as a fix.
- `kley check --strict` also verifies `kley.lock` / `package.json` consistency.
- The command is fast (< 100ms) and suitable for pre-commit hooks.
- Works correctly when no `kley.lock` exists (no error, just check package.json).
