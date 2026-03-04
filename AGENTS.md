# AGENTS.md - Kley Package Manager

## Project Overview

Kley is a fast and reliable local package manager for npm (JS/TS) written in Rust. It allows publishing packages to a local store and adding them to projects via file references.

## Build/Lint/Test Commands

### Build
```bash
cargo build          # Debug build
cargo build --release  # Release build (optimized)
```

### Run
```bash
cargo run -- publish  # Run publish command
cargo run -- add <package-name>  # Run add command
```

### Test
```bash
cargo test           # Run all tests
cargo test <test_name>  # Run a specific test by name
cargo test --test <test_file>  # Run tests in a specific file
cargo test -- --nocapture  # Run tests with stdout visible
```

### Lint & Format
```bash
cargo clippy         # Run linter (recommended before commits)
cargo clippy -- -D warnings  # Treat warnings as errors
cargo fmt            # Format code
cargo fmt -- --check  # Check formatting without changes
```

### Complete Check
```bash
cargo fmt && cargo clippy -- -D warnings && cargo test
```

## Code Style Guidelines

### Imports

```rust
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
```

- Group imports by external crates first, then the standard library.
- Use curly braces `{}` for multiple items from the same crate.
- Order: external dependencies → standard library.

### Error Handling

- Use `anyhow::Result<T>` as the return type for fallible functions.
- Use `.context("description")` to add human-readable context to errors.
- Use `anyhow::bail!("message")` for early returns with errors.
- Example:
```rust
fn some_function() -> Result<()> {
    let data = fs::read_to_string(path)
        .context("Failed to read file")?;
    
    if !valid {
        anyhow::bail!("Invalid input");
    }
    
    Ok(())
}
```

### Naming Conventions

- **Structs/Enums**: PascalCase (`PackageJson`, `Commands`)
- **Functions/Methods**: snake_case (`publish()`, `add()`)
- **Variables/Parameters**: snake_case (`pkg_path`, `store_path`)
- **Constants**: SCREAMING_SNAKE_CASE
- **Modules**: snake_case

### Structs and Enums

- Use derive macros for standard traits (`Debug`, `Parser`, `Subcommand`, `Serialize`, `Deserialize`).
- Place documentation comments with `///` above public-facing items to explain their purpose.

```rust
#[derive(Parser)]
#[command(name = "kley")]
#[command(about = "Description", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
```

### Functions

- Keep functions focused on a single responsibility.
- Use the `?` operator for concise error propagation.
- Match on enums for command dispatch in `main`.

### Code Organization

- The main entry point is `src/main.rs`.
- Extract logic into separate modules as the project grows to maintain clarity.
- Prefer small, focused functions over large, monolithic ones.

### Output Formatting

- Use the `colored` crate for terminal output to improve readability.
- Use Unicode emojis for visual feedback (e.g., 🚀, ✅, ⚠️, 📎).
- Use color methods like `.cyan()`, `.magenta()`, `.green()`, and `.yellow()`.

### Comments

- Use English for all inline comments and documentation.
- Use `//` for single-line inline comments.
- Use `///` for documentation comments (Doc-comments).

### File Operations

- Check for file existence with `.exists()` before performing operations.
- Use `Path` and `PathBuf` for all path handling to ensure cross-platform compatibility.
- Clean up directories before copying with `fs::remove_dir_all()` where appropriate.

### Patterns Used

1. **CLI with Clap**: Utilize derive macros for command and argument definitions.
2. **Error Propagation**: Rely on `anyhow` with the `context` method for clear, traceable errors.
3. **Path Handling**: Use `Path`/`PathBuf` and avoid string manipulation for paths.
4. **Home Directory**: Use `home::home_dir()` for cross-platform home directory detection.

## Project Structure

```
kley/
├── Cargo.toml        # Dependencies and project metadata
├── Cargo.lock        # Dependency lock file
├── SPEC.md           # Project specification
├── TODO.md           # Project tasks and roadmap
├── src/
│   └── main.rs       # Main application entry point
└── target/           # Build artifacts (gitignored)
```

## Dependencies

- `anyhow`: Flexible error handling.
- `clap`: Powerful CLI argument parsing (with `derive` feature).
- `colored`: Terminal colorization.
- `fs_extra`: Extra file system operations (like directory copying).
- `home`: Cross-platform home directory discovery.
- `ignore`: Gitignore-style pattern matching for file traversal (planned).
- `serde`/`serde_json`: Serialization and deserialization for JSON.
- `walkdir`: Efficient directory traversal (planned).

## Important Notes

- The project uses Rust Edition 2021.
- Strive to keep the Minimum Supported Rust Version (MSRV) stable.
- Avoid `unsafe` code unless absolutely necessary and clearly justified.
- All warnings from `cargo clippy` should be resolved before committing code.
- All new functionality should be accompanied by tests.
