# Feature parity comparison: kley vs yalc

> Document for understanding the current parity between **kley** (this
> repository) and **yalc** (the reference tool, https://github.com/wclr/yalc),
> to decide where to take kley, what real users need, and how to make the tool
> truly useful.
>
> All claims about kley are based on the source code (`src/`, `Cargo.toml`
> v0.14.0) and `README.md`. All claims about yalc are based on the official yalc
> README (https://raw.githubusercontent.com/wclr/yalc/master/README.md) as of
> writing. Where facts are absent, marked "no data".

---

## 1. What is it anyway

| | kley | yalc |
|---|---|---|
| Purpose | Local store + install/link without publishing to npm | Local store + install/link without publishing to npm |
| Implementation | Rust, single self-contained binary | Node.js (npm package, requires Node) |
| Node dependency | **No** | Yes |
| Installation | installer script, `npm i -g kley-cli`, `cargo install kley` | `npm i yalc -g` / `yarn global add yalc` |
| Store | `~/.kley/packages/<name>` (override `KLEY_HOME`) | `~/.yalc` (override `--store-folder`) |
| Lock file | `kley.lock` | `yalc.lock` |
| PM support | npm, pnpm, yarn (auto-detect) | npm, yarn (and partially pnpm) |

Both tools solve the same problem — replacing `npm/yarn link` with a reliable
local store using copying/symlinks. kley is positioned as "yalc, but without
the Node.js dependency" and is noticeably faster (per benchmarks in
`README.md`: cold start ~9 ms vs ~390 ms, iteration ~7 ms vs ~106 ms).

Also in the ecosystem there is **@jimsheen/yalc** — a Rust port of yalc (also
fast). kley competes with it too; `README.md` includes benchmarks of all three.

---

## 2. Command matrix

| Action | kley | yalc | Comment |
|---|---|---|---|
| Publish to store | `kley publish` | `yalc publish` | both copy "publishable" files |
| Publish + propagate | `kley publish --push` | `yalc publish --push` / `yalc push` | yalc has a separate `push`, kley uses a flag |
| Unpublish | `kley unpublish [--push]` | `yalc installations clean <pkg>` | different approach (see §4) |
| Add to project (file) | `kley add <name> [--dev]` | `yalc add <pkg> [--dev] [--link] [@ver]` | yalc is richer (version, link:) |
| Install into node_modules | `kley install [name] [-D] [--no-save]` | — | yalc has no direct `install` analog |
| Symlink to sources | `kley link <name>` | `yalc link <pkg>` | both leave package.json untouched |
| Update | `kley update [pkg...]` | `yalc update [pkg...]` | parity |
| Remove | `kley remove [name] [--all]` | `yalc remove [pkg] [--all]` | parity |
| Watch for changes | `kley watch [path]` | — (needs external watcher + `--changed`) | **kley wins** |
| Hook management | `kley hooks list\|edit` | — (via `pre/post`yalc scripts) | different models (see §4) |
| Show/clean installations | — | `yalc installations show\|clean` | **not in kley** |
| Temporarily remove/restore | — | `yalc retreat [--all]` / `yalc restore` | **not in kley** |
| Pre-commit check | — | `yalc check` | **not in kley** |
| Default config | — | `.yalcrc` | **not in kley** (`.kleyrc` missing) |

---

## 3. Feature matrix (who has what)

Legend: ✅ yes / ⚠️ partial / ❌ no / — no data

| Feature | kley | yalc | Where |
|---|---|---|---|
| Local package store | ✅ | ✅ | both |
| `kley.lock` / `yalc.lock` | ✅ | ✅ | both |
| `add` → `file:` in package.json | ✅ | ✅ | both |
| `link` → symlink without editing package.json | ✅ | ✅ | both |
| Package manager auto-detect | ✅ | ⚠️ (npm/yarn, weak pnpm) | kley |
| `--push` / auto-propagate to installations | ✅ | ✅ | both |
| Track installations for push | ✅ (internal) | ✅ | both |
| File filters (`.npmignore`/`.gitignore`) | ✅ | ✅ | both |
| `.kleyignore` / `.yalcignore` support | ✅ | ✅ | both |
| Respect `files` field in package.json | ✅ | ✅ | both |
| Mandatory files (README/LICENSE) | ✅ | ✅ | both |
| Publish hooks/scripts | ✅ (opt-in, safe) | ✅ (on by default) | both, different models |
| `--dev` / devDependency | ✅ | ✅ | both |
| `--no-save` (node_modules only, no package.json) | ✅ | ❌ | kley |
| Build deps via PM on install | ✅ (fast-reinstall / symlink) | ✅ | both |
| Auto-restore symlinks after `npm install` | ✅ (`kley install` no args) | ❌ (manual) | **kley** |
| Singleton/peer-dep warning on link | ✅ | ❌ | **kley** |
| Lifecycle scripts disabled by default | ✅ (`--ignore-scripts`) | ❌ (on by default, `--no-scripts` disables) | **kley** (safer) |
| Strip devDependencies on install | ✅ | ❌ | **kley** |
| Built-in `watch` | ✅ | ❌ | **kley** |
| No Node.js dependency | ✅ | ❌ | **kley** |
| Speed (cold/iter) | ✅ (faster ~10-50x) | ⚠️ slower | **kley** |
| Version pin `add pkg@version` | ❌ | ✅ | yalc |
| `add --link` (inject `link:` into package.json) | ✅ (f-41) | ✅ | both |
| Resolve `workspace:` protocol in deps | ❌ | ✅ (by default) | yalc |
| `add --workspace` / `-W` | ❌ | ✅ | yalc |
| `--pure` (no package.json, no node_modules) | ✅ (f-38, auto in workspaces) | ✅ | both |
| Version hash signature `--sig` (`1.2.3+ffff`) | ❌ | ✅ | yalc |
| `push --changed` (publish only on changes) | ❌ (`watch` always pushes) | ✅ | yalc |
| `push --replace` / `--update` (run PM update) | ❌ | ✅ | yalc |
| `retreat` / `restore` | ❌ | ✅ | yalc |
| `installations show` / `installations clean` | ❌ | ✅ | yalc |
| `check` (pre-commit check for yalc'd deps) | ❌ | ✅ | yalc |
| `publish --content` (show what goes into package) | ❌ | ✅ | yalc |
| `publish <sub-project>` (publish from subdir/monorepo) | ❌ (cwd only) | ✅ | yalc |
| `--store-folder` (flag) / `KLEY_HOME` (env) | ⚠️ (env only) | ✅ (flag) | yalc (flag), kley (env) |
| `--quiet` / `--no-colors` | ❌ (`-v` verbose only) | ✅ | yalc |
| `.kleyrc` / `.yalcrc` (default options) | ❌ | ✅ | yalc |
| `pre/post`yalc scripts in package.json | ❌ (different hook model) | ✅ | yalc |
| Override PM paths via env | ✅ (`KLEY_USE_*_COMMAND`) | — | kley |

---

## 4. What kley is missing (gaps)

These are things yalc can do that kley cannot. Sorted by likely importance for
real users.

### 4.1 High priority (blocks some scenarios)

1. **Version pin on `add`** — `yalc add pkg@1.2.3`. In kley `add` accepts only
   a name; the version is taken from the current store. Users who need to pin a
   specific version in `kley.lock` currently cannot.
2. **Resolve `workspace:` protocol** in a package's dependencies. yalc does this
   by default (`-no-workspace-resolve` disables it). In a monorepo/kley this
   would break.
3. **Workspaces support (pnpm/yarn)** — **partially implemented (f-38)**:
   `--pure`/`--no-pure` is done: `kley add` inside a project with a
   `workspaces` field defaults to pure (no `package.json` mutation), and
   `--no-pure` restores the `file:` injection. Remaining gaps:
   - No `--workspace` / `-W` short alias (yalc injects `workspace:*` instead
     of `file:`).
   - No automatic `workspace:` protocol resolution in a package's dependencies
     (yalc resolves `workspace:*` by default, `-no-workspace-resolve`
     disables).
4. **`retreat` / `restore`** — temporarily remove a local dependency before
   publishing to the real registry and bring it back. In yalc this is a basic
   "release preparation" scenario. kley has nothing.
5. **`installations show` / `installations clean`** — kley internally stores the
   list of installations for push, but does not let the user view or clean it
   (e.g., when a project was deleted from disk — yalc warns on push).

### 4.2 Medium priority (convenience / clear scenarios)

6. **`add --link` (manual flag) + `install` defaults to `link:`** — `kley
   add --link` manually injects `link:.kley/<pkg>` into package.json (low-level
   control; `add` without the flag stays `file:`). `kley install` by default
   behaves like `kley add --link` + `npm install`: it writes `link:`, the PM
   symlinks `node_modules`→`.kley`, and `publish --push`/`watch` propagate
   changes automatically regardless of npm defaults. The singleton/peer-dep risk
   is neutralized because `.kley/` lives inside the project (resolution of
   `require` reaches the project's `node_modules`). See tickets `f-41`, `f-42`.
7. **`push --changed`** — publish/push only if the content changed. Currently
   `watch` triggers `publish --push` on any FS event; for large packages this
   is wasted work.
8. **Version hash signature `--sig`** (`1.2.3+ffffffff`). Useful for unambiguous
   content identification in the lock file.
9. **`publish <sub-project>`** — publish from a subdirectory (monorepo).
   Currently `kley publish` only works in cwd.
10. **`.kleyrc` / default options** — yalc lets you set options once (e.g.
    `workspace-resolve=false`, `sig=false`). kley has no config file; only env
    variables.
11. **`check`** — a pre-commit hook that fails if `file:`/local dependencies
    remain in package.json. Helps avoid committing local links.
12. **`--store-folder` flag** — currently only the `KLEY_HOME` env. A flag is
    more convenient for one-off calls/CI.
13. **`--quiet` / `--no-colors`** — output control. kley only adds `-v` but
    cannot go silent. Needed in CI/scripts.

### 4.3 Low priority (nice-to-have)

14. **`publish --content`** — show the list of files that will go into the
    package.
15. **`push --replace` / `--update`** — force replacement / run PM update.
16. **`pre/post`yalc scripts in package.json** — kley has its own hook model
    (`.kley/hooks.json`), so this is more an alternative than a gap. But
    compatibility with `preyalc`/`postyalc`-style scripts is absent.

---

## 5. What kley has that yalc doesn't (advantages)

- **No Node.js dependency** — a single Rust binary; works at any Node version
  and even if npm is broken.
- **Speed** — an order of magnitude faster (see benchmarks in README).
- **PM auto-detect + install optimizations**: fast-reinstall (skip PM if deps
  unchanged) and direct symlink for dependency-free packages.
- **Auto-restore symlinks** after `npm install` via `kley install` with no
  arguments (including restoring link mode).
- **Singleton/peer-dependency warning** on `link` (protects against duplicate
  React, etc.).
- **Safe by default**: lifecycle scripts are disabled (`--ignore-scripts`),
  publishing is a pure copy, hooks are opt-in via an interactive wizard and an
  explicit `.kley/hooks.json`.
- **Strip devDependencies** on install (lighter node_modules).
- **Built-in `watch`** (yalc needs an external watcher).
- **Convenient `install`** — one step (add + install via PM) + restore all
  dependencies from lock with no arguments.
- **Env config** `KLEY_HOME`, `KLEY_USE_NPM_COMMAND`, `KLEY_USE_PNPM_COMMAND`,
  `KLEY_USE_YARN_COMMAND`.

---

## 6. Where to take kley (priorities)

Based on §4 and §5, ranked for "real usefulness":

1. **Workspaces support** — `--pure`/`--no-pure` is done (f-38). Remaining:
   `--workspace`/`-W` short alias and `workspace:` protocol resolution in deps.
2. **Version pin `add pkg@version`** — small change, big benefit.
3. **`installations show` / `clean`** — give access to what kley already stores
   internally; removes warnings on deleted projects.
4. **`retreat` / `restore`** — the "npm release preparation" scenario.
5. **`push --changed`** — speeds up `watch` on large packages.
6. **`.kleyrc`** with default options + `check` (pre-commit).
7. **`--quiet`/`--no-colors`**, `--store-folder` flag — CI polish.
8. Version hash `--sig`, `publish <sub-project>`, `add --link`,
   `publish --content` — on user request.

### What NOT to copy from yalc

- The model "scripts in package.json on by default" — kley is safer as is.
- `pre/post`yalc scripts — kley already has explicit hooks; keep its own model,
  but add compatibility if desired.

---

## 7. Sources

- kley: source code `src/` (commands in `src/commands/`, publish and file
  filters in `src/commands/publish.rs`), `Cargo.toml` (v0.14.0), `README.md`.
- yalc: official README https://raw.githubusercontent.com/wclr/yalc/master/README.md
- Competitor context: `@jimsheen/yalc` (Rust port) and benchmarks in `README.md`
  section "Benchmarks".
