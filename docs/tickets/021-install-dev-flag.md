# Ticket 021: Add `--dev` Flag to `install` Command

- **Epic:** III (Streamlined Local Package Workflow)
- **Complexity:** `Low`
- **Depends On:** #017

## 1. Problem Statement

The `kley install` command always installs packages as regular dependencies (in `dependencies`). However, some packages — such as test utilities, linters, or build tools — should be installed as `devDependencies` instead.

Currently, the README explicitly states: *"If you need to add a package as a dev dependency (`--dev`), use `kley add --dev` instead and run the package manager manually."* This is a two-step workaround that defeats the purpose of `kley install` being a one-step command.

## 2. Proposed Solution

Add a `--dev` (alias `-D`) flag to `kley install`, consistent with the existing `kley add --dev` flag and the conventions of `npm install --save-dev`, `pnpm add -D`, and `yarn add --dev`.

### Usage

```bash
# Install as regular dependency (default, unchanged)
kley install my-package

# Install as dev dependency
kley install --dev my-package
kley install -D my-package
```

### Implementation Details

1. **CLI**: Add `--dev` flag to the `Install` subcommand in `src/main.rs`, same pattern as `Add`:
   ```rust
   Install {
       name: String,
       #[arg(long, short = 'D')]
       dev: bool,
   }
   ```

2. **Command logic**: Pass `dev` flag through to `install()` in `src/commands/install.rs`.

3. **Package manager delegation**: When `--dev` is set, add the appropriate flag to the PM command:
   | PM | Regular | With `--dev` |
   |----|---------|-------------|
   | **npm** | `npm install <path> --ignore-scripts` | `npm install <path> --save-dev --ignore-scripts` |
   | **pnpm** | `pnpm add <path> --ignore-scripts` | `pnpm add <path> -D --ignore-scripts` |
   | **yarn** | `yarn add <path> --ignore-scripts` | `yarn add <path> --dev --ignore-scripts` |

   This is sufficient — the PM itself will place the dependency in `devDependencies` in `package.json`. No need to manually modify `package.json` as `kley add` does, since the PM handles it.

4. **kley.lock**: No changes needed. The `kley.lock` file does not distinguish between `dependencies` and `devDependencies` — it tracks all kley-managed packages uniformly.

## 3. Why `--dev` and not `-D` only

- `--dev` is consistent with `kley add --dev` (already implemented).
- `-D` is provided as a short alias, consistent with `pnpm add -D` and `yarn add --dev` / `-D`.
- npm uses `--save-dev`, but `--dev` is shorter and already established in the kley API.

## 4. Acceptance Criteria

- `kley install --dev <package>` installs the package into `devDependencies` via the native PM.
- `kley install -D <package>` works identically to `--dev`.
- `kley install <package>` (without flag) behaves as before — installs into `dependencies`.
- The `kley.lock` is updated correctly regardless of `--dev` flag.
- Tests cover both regular and dev install for at least one PM (npm).
- Mock scripts in `tests/mocks/` correctly handle `-D` / `--save-dev` / `--dev` flags.
