# Kley 🚀

[![Rust](https://github.com/sumbad/kley/actions/workflows/rust.yml/badge.svg)](https://github.com/sumbad/kley/actions/workflows/rust.yml)

**Fast and reliable local package manager for npm (JS/TS), written in Rust.**

[Русская версия (Russian Version)](README_RU.md)

Kley is a command-line tool that simplifies local development of npm packages. It provides a robust alternative to `npm link` by managing a local package store, allowing you to "publish" packages to a central cache on your machine and "add" them to your projects via direct file copying. This avoids the common pitfalls of symbolic links.

## Key Features

- **Blazing Fast**: Built with Rust for maximum performance.
- **Reliable**: Avoids symlink issues by copying files directly.
- **Simple API**: Just two commands to get started: `publish` and `add`.
- **Cross-Platform**: Works on macOS, Linux, and Windows.

## Installation

_Coming soon... (Pre-compiled binaries will be available in Releases)_

For now, you can build from source:
```bash
git clone https://github.com/sumbad/kley.git
cd kley
cargo build --release
# The binary will be in ./target/release/kley
```

## How It Works

### 1. `kley publish`

Run this command in the directory of the package you want to share locally. Kley will:
1.  Read your `package.json`.
2.  Copy all necessary files into a central store located at `~/.kley/packages/<your-package-name>`.
3.  Overwrite any previous version of the same package in the store.

Example:
```bash
# In your library project (/path/to/my-lib)
kley publish
```

### 2. `kley add <package-name>`

Run this command in the project where you want to use your local package. Kley will:
1.  Find the package in the central store.
2.  Copy the package files into a `./.kley/<your-package-name>` directory within your project.
3.  Provide you with instructions to add the local dependency to your `package.json`.

Example:
```bash
# In your main application (/path/to/my-app)
kley add my-lib

# Then, add the following to your my-app/package.json:
# "dependencies": {
#   "my-lib": "file:.kley/my-lib"
# }
```
Now you can `npm install` and `import` your library as usual!

## Build/Test Commands

### Build
```bash
cargo build --release
```

### Test
```bash
cargo test
```

### Lint & Format
```bash
cargo fmt
cargo clippy -- -D warnings
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
