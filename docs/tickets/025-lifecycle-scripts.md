# Ticket 025: Implement Lifecycle Scripts

- **Epic:** VI (DX/UX Improvements)
- **Complexity:** Medium
- **Depends On:** #001 (push command), #011 (update command)

## 1. Problem Statement

When `kley publish --push` updates a package in a consumer project, the consumer may need to perform actions afterward — rebuild, regenerate code, restart a dev server, etc. Currently there is no way to hook into the update process automatically.

## 2. Proposed Solution

Allow consumer projects to define lifecycle scripts in their `package.json` that kley will execute at specific points during the package lifecycle.

### Script Names

Following the pattern established by npm and yalc:

| Script | When it runs | Where (consumer project) |
|--------|-------------|------------------------|
| `prekley` | Before package files are updated | Consumer `package.json` |
| `postkley` | After package files are updated | Consumer `package.json` |
| `prekley.<pkg>` | Before specific package is updated | Consumer `package.json` |
| `postkley.<pkg>` | After specific package is updated | Consumer `package.json` |

### Execution Triggers

Scripts run in the **consumer project** when:

- `kley add <pkg>` — runs `postkley` / `postkley.<pkg>` after adding
- `kley update <pkg>` — runs `prekley` / `postkley` around the update
- `kley publish --push` — runs `prekley` / `postkley` in each consumer project during push
- `kley install <pkg>` — runs `postkley` / `postkley.<pkg>` after install

### Execution Order

For a push that updates `my-lib` in a consumer project:

1. `prekley`
2. `prekley.my-lib`
3. *(files are updated)*
4. `postkley.my-lib`
5. `postkley`

### Package Author Scripts (on publish)

On the **publishing** side, kley already respects npm lifecycle scripts (`prepublish`, `prepare`, `prepublishOnly`, `prepack`, `postpack`, `publish`, `postpublish`). No changes needed here.

### `--scripts` / `--no-scripts` Flag

By default, lifecycle scripts are **disabled** during `push` (same as yalc) to avoid unexpected side effects. The user must opt in:

```bash
kley publish --push --scripts    # enable lifecycle scripts during push
kley add my-lib --scripts        # enable lifecycle scripts during add
```

This can be changed as a default in `.kleyrc` (see Ticket 026).

## 3. Usage Examples

```json
// consumer project package.json
{
  "scripts": {
    "postkley": "npm run build",
    "postkley.my-ui-lib": "npm run generate:components"
  }
}
```

```bash
# Push with scripts enabled — consumer's postkley.my-ui-lib will run
kley publish --push --scripts
```

## 4. Implementation Plan

1. Add a `--scripts` flag to `publish`, `add`, `update`, and `install` commands.
2. Create a `run_lifecycle_scripts()` utility function that:
   a. Reads `package.json` from the target project.
   b. Looks for `prekley` / `postkley` / `prekley.<pkg>` / `postkley.<pkg>` in `scripts`.
   c. Executes found scripts using the detected package manager (or `npm run`).
   d. Handles errors gracefully — log warning but don't abort the main operation.
3. Integrate into `run_update()` (used by both `update` and `publish --push`).
4. Integrate into `add` and `install` commands.

## 5. Acceptance Criteria

- `prekley` and `postkley` scripts are executed in the consumer project during update/push when `--scripts` is passed.
- `prekley.<pkg>` and `postkley.<pkg>` package-specific scripts are executed for the matching package.
- Execution order: general `prekley` → package-specific `prekley.<pkg>` → update → package-specific `postkley.<pkg>` → general `postkley`.
- By default (without `--scripts`), no lifecycle scripts run during push — backward compatible.
- If a lifecycle script fails, a warning is logged but the main operation continues.
- Scripts are executed using the project's detected package manager.
