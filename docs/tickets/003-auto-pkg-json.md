# Ticket 003: Automate package.json modification

**Epic**: [I: Core Publishing & Linking](../epics/I.md)
**Complexity**: High

## 1. Goal
The `kley add <pkg>` command must automatically modify the host project's `package.json` to add or update a `file:` dependency. This removes the need for manual user intervention and is a core feature for a smooth workflow.

## 2. Implementation Plan ("Update in Place" approach)

### Step 1: Read and Parse `package.json`
- In the `add` command logic, find and read `package.json` into a string.
- **Detect Indentation**: Before parsing, analyze the string to detect the indentation style (e.g., 2 spaces, 4 spaces, tab). This will be used later for writing.
- **Parse**: Parse the JSON string into a mutable `serde_json::Value`. The `Value` type preserves the original key order.

### Step 2: Modify the Dependency Tree
- Define a list of dependency keys to check: `["dependencies", "devDependencies", "peerDependencies"]`.
- **Search**: Iterate through the keys. Check if the package exists within any of these sections.
- **If Found**: Update the package's value to `file:.kley/my-package` in the section where it was found.
- **If Not Found**: Add the package to the appropriate section (`dependencies` or `devDependencies` based on the `--dev` flag).

### Step 3: Write the Modified `package.json`
- Create a buffer (e.g., `Vec<u8>`) to write the output to.
- Create a `serde_json::ser::PrettyFormatter` configured with the indentation style detected in Step 1.
- Create a `serde_json::Serializer` using the formatter and the buffer.
- Serialize the modified `serde_json::Value` into the serializer.
- Write the buffer's content back to the `package.json` file, overwriting it.
- This "Smart `serde_json`" approach ensures key order and indentation are preserved, providing a non-disruptive user experience.

## 3. Code Structure (in `src/main.rs`)

```rust
// In `add` function

// ... (copy files logic remains)

// New logic starts here
let pkg_json_path = Path::new("package.json");
// ... handle path not existing ...

let content = fs::read_to_string(&pkg_json_path)?;

// --- Step 1: Detect Indent & Parse ---
let indent = detect_indent(&content); // A helper function we'll write
let mut value: serde_json::Value = serde_json::from_str(&content)
    .context("Failed to parse package.json")?;

// --- Step 2: Modify Tree (logic as previously discussed) ---
// ... (search and update/add logic) ...

// --- Step 3: Write with Preserved Formatting ---
let mut buf = Vec::new();
let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
value.serialize(&mut ser)?;

fs::write(&pkg_json_path, buf)?;

println!("{}", "✅ package.json has been updated!".green());
```

## 4. Open Questions
- **(Resolved)** **Dependency Type**: The "Update in Place" approach is chosen. We update the dependency where we find it. If not found, we add it to `dependencies` or `devDependencies` based on the `--dev` flag.
- **(Resolved)** **Formatting Preservation**: The "Smart `serde_json`" approach of detecting indentation and using a custom `PrettyFormatter` will be used. This provides a robust solution without new dependencies.
- **(Resolved)** **Error Handling**: What if `package.json` is malformed? The `.context()` call will handle this gracefully.
