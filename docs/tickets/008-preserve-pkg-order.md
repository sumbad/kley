# Ticket 008: Preserve package.json Order on `kley add`

**Epic**: VI (DX/UX Improvements (General))
**Complexity**: Medium

## Problem

The `kley add <package>` command currently rewrites the `package.json` file in a way that reorders its top-level properties alphabetically. This creates unnecessary noise in git diffs, making it harder to see the actual change (the new dependency).

An integration test (`tests/add_command_preserves_order_test.rs`) has been added to confirm this behavior.

## Proposed Solution

The root cause is likely that the `package.json` is being deserialized into a struct or a `BTreeMap` (which sorts its keys) and then serialized back.

To fix this, we should handle the JSON modification in a way that preserves the original order of properties. The recommended approach is:

1.  **Read `package.json` into `serde_json::Value`**: Instead of a strongly-typed struct, parse the file into a generic `serde_json::Value`. This will preserve the order of the JSON object's members.

2.  **Mutate the `Value`**:
    *   Navigate to the `dependencies` field. If it doesn't exist, create it as a new `Value::Object`.
    *   Get a mutable reference to the `dependencies` object (`as_object_mut`).
    *   Insert the new package and its version into the `dependencies` map.

3.  **Write the `Value` back to the file**:
    *   Serialize the modified `serde_json::Value` back into a string. `serde_json` will respect the original order when serializing from a `Value`.
    *   Write the string back to `package.json`.

This will ensure that only the `dependencies` object is modified, and the rest of the file remains untouched, preserving the original formatting and property order.
