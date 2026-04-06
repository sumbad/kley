# 📦 kley

[![Release](https://github.com/sumbad/kley/actions/workflows/release.yml/badge.svg)](https://github.com/sumbad/kley/releases)

English | [Русский](./README_RU.md)

**A simple local package manager for npm (JS/TS)**

> Like **`npm link`**, but with a more convenient workflow. Like **`yalc`**, but without the dependency on Node.js.

**kley** is a command-line tool that simplifies local development of npm packages. It provides a robust alternative to `npm link` by managing a local package store. It saves packages to a central cache on your machine when you "publish" and lets you quickly install them into your local projects via direct file copying or symlinks, without needing to connect to a remote repository.

## Key Features

- **Fast and Efficient**: All operations are local, with no network latency or unnecessary publishing of intermediate versions.
- **Reliable and Independent**: Avoids `npm link` issues and works even if your library and projects use different versions of Node.js.
- **Safe**: Avoids running `npm` scripts, only performing basic file copy and link operations.
- **Simple API**: Six core commands to get started: `publish`, `unpublish`, `add`, `link`, `update`, and `remove`.
- **Cross-Platform**: Works on macOS, Linux, and Windows.

## Installation

### Quick Install (recommended)

You can install `kley` with a single command using the installer script:

```bash
# Linux / macOS
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/sumbad/kley/releases/latest/download/kley-installer.sh | sh
```
```bash
# Windows
powershell -ExecutionPolicy Bypass -c "irm https://github.com/sumbad/kley/releases/latest/download/kley-installer.ps1 | iex"
```

### Manual Installation

Alternatively, you can install `kley` by downloading a pre-compiled binary from the [**Releases page**](https://github.com/sumbad/kley/releases).

1.  Download the appropriate archive for your system (e.g., `kley-x86_64-apple-darwin.tar.gz`).
2.  Unpack the archive.
3.  Move the `kley` binary to a directory in your system's `PATH` (e.g., `/usr/local/bin` on macOS/Linux).


### Install via Cargo (crates.io)
If you have Rust and Cargo installed, you can install `kley` directly from crates.io:

```bash
cargo install kley
```

### Install via npm (kley-cli)
You can install `kley-cli` globally. Use it only if you have the same Node.js versions for your library and host a project, otherwise you should install it to all Node.js versions:

```bash
npm install -g kley-cli
```

## Usage

### 1. `kley publish`
Run this command in the directory of the package you want to share locally. Kley copies all necessary files to a central store at `~/.kley/packages/<your-package-name>`.

- Use the `--push` flag to automatically update the package in all projects where it has been added or linked. This is the primary command for a fast, iterative workflow.

### 2. `kley unpublish`
Run this command in the directory of a published package to remove it from the kley store.

- By default, it performs a "soft" unpublish, removing the package from the store but leaving your projects intact until the next install.
- Use the `--push` flag to perform a "hard" unpublish, which also removes the package from all projects that use it.

### 3. `kley add <package-name>`
Run this command in the project where you want to use your local package. Kley copies the package into a local `./.kley/` directory, then automatically updates your `package.json` and `kley.lock`.

- Use the `--dev` flag to add the package to `devDependencies`.

> **Note:** For the changes to appear in `node_modules`, you must run `npm install` (or `yarn`, `pnpm`) after `kley add`.

### 4. `kley link <package-name>`
This command provides a flexible workflow that avoids modifying `package.json`. It copies the package to a local `.kley` cache and then creates a symbolic link from that cache to your project's `node_modules` directory.

> **Warning:** Because `package.json` is not modified, running `npm install` (or `yarn`, `pnpm`) will likely delete the symlink from `node_modules`. To restore it, simply run `kley link <package-name>` again. This is a fast operation because the local cache is preserved.

### 5. `kley update [package-name]`
This command updates installed packages to the latest version from the kley store.

- If you provide a package name, only that specific package will be updated.
- If you run it without arguments, `kley` will update all packages listed in `kley.lock`.

### 6. `kley remove [package-name]`
Run this command to cleanly remove a kley-managed dependency from your project. It will update `package.json` and `kley.lock`, and delete the package files from the `./.kley/` directory.

- Use the `--all` flag to remove all kley-managed packages from the project.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## About

This project is inspired by great tools like [yalc](https://github.com/wclr/yalc). The main advantage of `kley` is that it is a single, self-contained binary with **no dependency on Node.js**. This means you can manage packages regardless of your current Node.js version or any issues with `npm` itself.

> **Note:** This project is in active development and currently supports only the basic commands. More features are coming soon!

## License

This project is licensed under the MIT License - see the LICENSE file for details.
