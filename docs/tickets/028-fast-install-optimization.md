# Ticket 028: Implement "Fast Install" Optimization (Skip PM When Possible)

- **Epic:** III (Streamlined Local Package Workflow)
- **Complexity:** High
- **Depends On:** #017 (install command), #018 (PM detection), #019 (strip devDependencies)

## 1. Problem Statement

The `kley install` command always delegates to the native package manager, which can be slow if all required dependencies are already present in the project. For large projects, calling `npm install` can take several seconds even when nothing needs to change.

## 2. Proposed Solution

Add a "fast path" to `kley install` that skips the native package manager call when all of the package's dependencies are already satisfied. This optimization applies **only** when `npm` or `yarn v1` is detected — pnpm and yarn v2+ have different resolution strategies that make this approach unreliable.

### Logic

1. **Dependency Check:** Before delegating to the native PM, parse the `dependencies` (not `devDependencies` — those are stripped by #019) of the package being installed from its `.kley/<pkg>/package.json`.
2. **Verify Installation:** Check if all of those dependencies (respecting semver ranges) are already present in the host project's `node_modules`. This can be done by:
   - Parsing the output of `npm list --json` or `yarn list --json`, OR
   - Checking `node_modules/<dep>/package.json` for each dependency and comparing version ranges.
3. **Execute Fast Path:** If all dependencies are met, perform a direct installation:
   a. Manually add the `file:./.kley/<pkg>` entry to `package.json`.
   b. Copy the package files directly into `node_modules/<pkg>` (or create symlink if already linked).
   c. Skip the call to `npm install`.
4. **Fallback:** If any dependency is missing or cannot be verified, fall back to the standard behavior of delegating to the native package manager.

### Safety Considerations

- The fast path must be **opt-in initially** via a `--fast` flag to gather feedback and validate correctness.
- After proving reliable, it can become the default with a `--no-fast` escape hatch.
- If the fast path produces an inconsistent state, the user can always run `npm install` manually to fix it.

## 3. Why This Is Deferred

This optimization has a high complexity-to-value ratio:

- **Parsing `npm list --json`** is fragile — format varies between npm versions.
- **Semver range matching** requires implementing or importing a semver library.
- **Edge cases**: peer dependencies, optional dependencies, bundled dependencies, workspaces.
- **Risk**: If kley skips `npm install` and gets it wrong, the project is in a broken state with no obvious cause.
- **Savings**: With npm's package cache, installing a local package typically takes 2-3 seconds. The fast path saves this at the cost of significant implementation and maintenance complexity.

The devDependencies leak (#019) delivers more concrete value with far less complexity and should be implemented first.

## 4. Acceptance Criteria

- When using `npm` or `yarn v1`, `kley install --fast` is significantly faster if all dependencies are already met.
- The fast path correctly verifies dependencies using semantic versioning.
- The optimization does not run when `pnpm` or `yarn v2+` is detected.
- If the fast path cannot verify all dependencies, it falls back to the standard PM delegation.
- A `--no-fast` flag is available to explicitly disable the fast path.
- `kley install` without `--fast` behaves exactly as before — backward compatible.
