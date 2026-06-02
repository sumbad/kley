# F-27: Fast Reinstall Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Optimize the `kley install` command to skip the package manager call when a package's dependencies have not changed, making re-installs significantly faster.

**Architecture:** The plan introduces a "Fast Path" into the `install` command. This path is chosen when a package's dependencies match a snapshot in `kley.lock`. It intelligently handles existing `node_modules` structures, including symlinks created by `kley link` or modern package managers, falling back to the "Slow Path" (running the PM) when necessary.

**Tech Stack:** Rust, `serde`, `assert_cmd`

---

### Task 1: Update Lockfile Structure

**Files:**
- Modify: `src/lockfile.rs`
- Test: `src/lockfile.rs` (existing tests might need updates)

- [ ] **Step 1: Add dependency snapshot fields to `PackageInfo`**

Modify the `PackageInfo` struct in `src/lockfile.rs` to include fields for storing dependency snapshots.

```rust
// In src/lockfile.rs

use std::collections::BTreeMap;
// ... other imports

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PackageInfo {
    pub version: String,
    /// Snapshot of dependencies at last PM install, used for fast reinstall check
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, String>,
    /// Snapshot of peer dependencies at last PM install, used for fast reinstall check
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub peer_dependencies: BTreeMap<String, String>,
}
```
*Note: Added `skip_serializing_if` for cleaner lockfiles.*

- [ ] **Step 2: Run existing lockfile tests**

Ensure that adding the new fields doesn't break any existing serialization or deserialization tests for the lockfile.

Run: `cargo test -p kley -- --test lockfile`
Expected: PASS

- [ ] **Step 3: Commit changes**

```bash
git add src/lockfile.rs
git commit -m "feat(lockfile): add dependency snapshots to PackageInfo"
```

---

### Task 2: Implement the "Slow Path" Snapshotting

**Files:**
- Modify: `src/commands/install.rs`
- Modify: `src/package.rs` (likely need a helper to read deps)

- [ ] **Step 1: Create a helper to read dependencies from `package.json`**

In `src/package.rs`, add a method to the `Package` struct to extract `dependencies` and `peerDependencies`.

```rust
// In src/package.rs

use std::collections::BTreeMap;
// ... other imports

impl Package {
    // ... existing methods

    pub fn get_dependencies(&self) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
        let dependencies = self.json.dependencies.clone().unwrap_or_default();
        let peer_dependencies = self.json.peer_dependencies.clone().unwrap_or_default();
        (dependencies, peer_dependencies)
    }
}
```

- [ ] **Step 2: Update `install_package` to save snapshot on slow path**

In `src/commands/install.rs`, find the section in `install_package` where the package manager is called (the "slow path"). After it succeeds, read the dependencies from the cached `package.json` and save them to the lockfile.

```rust
// In src/commands/install.rs, inside install_package function

// ... after the pm.run().status()? check ...

if status.success() {
    // This is the slow path, so we update the snapshot.
    let (deps, peer_deps) = package.get_dependencies();
    lockfile.set_dependencies(package_name, deps)?;
    lockfile.set_peer_dependencies(package_name, peer_deps)?;
    lockfile.save()?;

    // ... rest of the function
}
```
*(You will need to add the `set_dependencies` and `set_peer_dependencies` methods to `KleyLock` in `src/lockfile.rs`)*

- [ ] **Step 3: Add `set_dependencies` methods to `KleyLock`**

```rust
// In src/lockfile.rs

impl KleyLock {
    // ... existing methods

    pub fn set_dependencies(&mut self, name: &str, deps: BTreeMap<String, String>) -> Result<()> {
        let pkg_info = self.packages.get_mut(name).ok_or_else(|| anyhow!("Package not in lockfile"))?;
        pkg_info.dependencies = deps;
        Ok(())
    }

    pub fn set_peer_dependencies(&mut self, name: &str, peer_deps: BTreeMap<String, String>) -> Result<()> {
        let pkg_info = self.packages.get_mut(name).ok_or_else(|| anyhow!("Package not in lockfile"))?;
        pkg_info.peer_dependencies = peer_deps;
        Ok(())
    }
}
```

- [ ] **Step 4: Write a failing test for snapshotting**

In `tests/install_command_test.rs`, create a new test that:
1. Installs a package for the first time.
2. Asserts that `kley.lock` now contains the `dependencies` and `peerDependencies` for that package.

- [ ] **Step 5: Run test to verify it fails as expected**

Run the new test. It should fail because the snapshotting logic isn't fully wired up yet.

- [ ] **Step 6: Implement the logic and make the test pass**

Wire up the calls from `install.rs` to `lockfile.rs`.

- [ ] **Step 7: Commit changes**

```bash
git add src/commands/install.rs src/lockfile.rs src/package.rs tests/install_command_test.rs
git commit -m "feat(install): snapshot dependencies on slow path"
```

---

### Task 3: Implement the "Fast Path" Logic

**Files:**
- Modify: `src/commands/install.rs`
- Test: `tests/install_command_test.rs`

- [ ] **Step 1: Write a failing test for the "happy" fast path (Case C)**

In `tests/install_command_test.rs`, create a test that:
1. Installs a package (slow path, creates snapshot).
2. Touches a source file in the library package and re-publishes it.
3. Installs the package again.
4. Asserts that the package manager was **NOT** called this time.

- [ ] **Step 2: Implement the core fast path logic in `install_package`**

```rust
// In src/commands/install.rs, inside install_package

// ... before calling the package manager ...

let (current_deps, current_peer_deps) = package.get_dependencies();
let snapshot = lockfile.get_package_info(package_name);

let should_fast_path = if let Some(info) = snapshot {
    info.dependencies == current_deps && info.peer_dependencies == current_peer_deps
} else {
    false
};

if should_fast_path {
    // FAST PATH LOGIC HERE
    // For now, just copy files (Case C)
    let dest_path = project_dir.join("node_modules").join(package_name);
    fs_extra::dir::copy(&package_path, &dest_path, &fs_extra::dir::CopyOptions::new().overwrite(true))?;
    // ... print success message and return
    return Ok(());
}

// ... existing slow path logic (pm.run()...)
```

- [ ] **Step 3: Make the test pass and commit**

Run the test from Step 1. It should now pass.

```bash
git add src/commands/install.rs tests/install_command_test.rs
git commit -m "feat(install): implement fast path file copy"
```

- [ ] **Step 4: Write failing tests for symlink cases (A and B)**

Add two new tests:
1.  **Test Case A:**
    *   Run `kley link my-lib`.
    *   Re-publish `my-lib`.
    *   Run `kley install my-lib`.
    *   Assert that the PM was **NOT** called and the command succeeded.
2.  **Test Case B:**
    *   Manually create a symlink `node_modules/my-lib` pointing to an unrelated directory.
    *   Run `kley install my-lib`.
    *   Assert that the PM **WAS** called (it fell back to the slow path).

- [ ] **Step 5: Implement the symlink handling logic**

Update the `if should_fast_path` block to handle all three cases.

```rust
// In src/commands/install.rs, inside the fast path block

let dest_path = project_dir.join("node_modules").join(package_name);

if dest_path.is_symlink() {
    let link_target = std::fs::read_link(&dest_path)?;
    if link_target == package_path {
        // Case A: Symlink points to our cache. Do nothing.
        info!("Destination is a correct symlink. Fast path complete.");
    } else {
        // Case B: Unknown symlink. Fall back to slow path.
        info!("Destination is an unknown symlink. Falling back to slow path.");
        // Just let the code fall through to the slow path logic below
    }
} else {
    // Case C: Directory or does not exist. Copy files.
    fs_extra::dir::copy(&package_path, &dest_path, &fs_extra::dir::CopyOptions::new().overwrite(true))?;
}
// Make sure to structure the if/else so that Case B falls through
// and the others return Ok(()).
```

- [ ] **Step 6: Make all tests pass and commit**

Run the full test suite for `install_command_test.rs`. All fast path tests should now pass.

```bash
git add src/commands/install.rs
git commit -m "feat(install): handle symlinks in fast path"
```
