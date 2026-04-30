#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const args = process.argv.slice(2);

// Log the command
fs.appendFileSync('pm.log', `pnpm ${args.join(' ')}\n`);

const cmd = args[0];
if (cmd !== 'add') {
  process.exit(0);
}

let mode = 'dependencies';
let pkgPath = null;

for (let i = 1; i < args.length; i++) {
  const arg = args[i];
  if (arg === '-D' || arg === '--save-dev') {
    mode = 'devDependencies';
  } else if (arg === '--no-save') {
    mode = 'none';
  } else if (arg.startsWith('-')) {
    // skip other flags
  } else {
    pkgPath = arg;
  }
}

if (!pkgPath) {
  process.exit(0);
}

const pkgName = path.basename(pkgPath);

if (mode !== 'none') {
  const pkgJsonPath = path.resolve('package.json');
  const pkg = JSON.parse(fs.readFileSync(pkgJsonPath, 'utf8'));
  pkg[mode] = pkg[mode] || {};
  pkg[mode][pkgName] = `file:${pkgPath.replace(/\\/g, '/')}`;
  fs.writeFileSync(pkgJsonPath, JSON.stringify(pkg, null, 2));
}

// Create node_modules/.pnpm and copy package
const nodeModules = path.resolve('node_modules');
const pnpmStore = path.join(nodeModules, '.pnpm');
fs.mkdirSync(pnpmStore, { recursive: true });

const dest = path.join(pnpmStore, pkgName);
if (fs.existsSync(dest)) {
  fs.rmSync(dest, { recursive: true, force: true });
}
fs.cpSync(pkgPath, dest, { recursive: true, force: true });

// Create symlink (junction on Windows, symlink on Unix)
const symlinkPath = path.join(nodeModules, pkgName);
if (fs.existsSync(symlinkPath)) {
  fs.rmSync(symlinkPath, { recursive: true, force: true });
}
fs.symlinkSync(dest, symlinkPath, 'junction');

// Create lock file marker
fs.writeFileSync(path.resolve('pnpm-lock.yaml'), '');
