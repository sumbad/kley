# Ticket 019: Strip `devDependencies` from Consumed Packages

- **Epic:** III (Streamlined Local Package Workflow)
- **Complexity:** Low
- **Depends On:** None

## 1. Problem Statement

When `kley install` (or `kley add`) copies a package from the global store into the project's `.kley/<pkg>/` directory, the package's `package.json` retains its `devDependencies`. When the native package manager then installs this local package (e.g. `npm install ./.kley/my-lib`), it installs **all** listed dependencies — including `devDependencies` of the library.

This is problematic because:

- The package in `.kley/<pkg>/` is a **consumed** copy, not a development source. Its `devDependencies` (test runners, build tools, linters) are irrelevant to the consumer.
- Installing them adds unnecessary packages to the consumer's `node_modules`, increasing install time and disk usage.
- In real-world testing, `kley install @mail/features` caused `rollup-plugin-postcss` (a devDependency of the library) to be installed in the host project.

**Why `--omit=dev` doesn't work:** This flag tells npm to skip devDependencies across the **entire dependency tree of the host project**, not just the added package. Using it causes the host project to lose its own devDependencies.

## 2. Proposed Solution

Strip `devDependencies` from `<project>/.kley/<pkg>/package.json` **after** copying from the registry. This way:

1. The original package in the registry (`~/.kley/packages/<pkg>/`) retains its full `package.json` — needed for the library author.
2. The local copy in the consuming project (`<project>/.kley/<pkg>/`) has `devDependencies` removed — the PM won't install them.

### Implementation

Modify `copy_from_registry()` in `src/utils.rs` to call a `strip_dev_dependencies()` function after the `fs_extra::dir::copy()` call:

```rust
fn strip_dev_dependencies(pkg_dir: &Path) -> Result<()> {
    let pkg_json_path = pkg_dir.join("package.json");
    let content = fs::read_to_string(&pkg_json_path)?;
    let mut value: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(obj) = value.as_object_mut() {
        if obj.remove("devDependencies").is_some() {
            // Write back with preserved formatting
            let mut buf = Vec::new();
            let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
            value.serialize(&mut ser)?;
            fs::write(&pkg_json_path, buf)?;
        }
    }

    Ok(())
}
```

This function should be called in `copy_from_registry()` right after the directory copy succeeds.

## 3. Acceptance Criteria

- After `kley add <pkg>` or `kley install <pkg>`, the `package.json` in `.kley/<pkg>/` does **not** contain a `devDependencies` key.
- The original `package.json` in `~/.kley/packages/<pkg>/` is **unchanged** — it still has `devDependencies`.
- If a package has no `devDependencies`, the function is a no-op (no unnecessary file write).
- The `package.json` formatting (indentation) is preserved after the modification.
