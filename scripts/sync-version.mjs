#!/usr/bin/env node
/**
 * Keep every user-visible version literal in sync with
 * `crates/false-ui/Cargo.toml`.
 *
 * Rust code already reads `CARGO_PKG_VERSION`. The remaining literals are
 * metadata consumed before Rust runs (Tauri config, installer HTML) or static
 * fallbacks in the shell.
 *
 * Usage:
 *   node scripts/sync-version.mjs           # rewrite files
 *   node scripts/sync-version.mjs --check   # exit 1 if anything is stale
 */

import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const checkOnly = process.argv.includes('--check');

const cargoPath = path.resolve(root, 'crates/false-ui/Cargo.toml');
if (!fs.existsSync(cargoPath)) {
  console.error(`sync-version: missing ${cargoPath}`);
  process.exit(1);
}

const version = /^version\s*=\s*"([^"]+)"/m.exec(fs.readFileSync(cargoPath, 'utf8'))?.[1];
if (!version) {
  console.error('sync-version: cannot read version from crates/false-ui/Cargo.toml');
  process.exit(1);
}

/** @type {Array<{file: string, pattern: RegExp, replace: string}>} */
const targets = [
  {
    file: 'crates/false-ui/tauri.conf.json',
    pattern: /("version"\s*:\s*")[^"]+(")/,
    replace: `$1${version}$2`,
  },
  {
    file: 'crates/false-installer/Cargo.toml',
    pattern: /^(version\s*=\s*")[^"]+(")/m,
    replace: `$1${version}$2`,
  },
  {
    file: 'crates/false-installer/src/main.rs',
    pattern: /(DisplayVersion"\s* -Value\s*")[^"]+(")/,
    replace: `$1${version}$2`,
  },
  {
    file: 'crates/false-ui/ui/installer.html',
    pattern: /(Version\s+)\d+\.\d+\.\d+/,
    replace: `$1${version}`,
  },
  {
    file: 'crates/false-ui/ui/uninstaller.html',
    pattern: /(Version\s+)\d+\.\d+\.\d+/,
    replace: `$1${version}`,
  },
  {
    file: 'crates/false-ui/ui/index.html',
    pattern: /(Version\s+)\d+\.\d+\.\d+/,
    replace: `$1${version}`,
  },
  {
    file: 'crates/false-ui/ui/index.html',
    pattern: /(current_version\s*:\s*['"])\d+\.\d+\.\d+(['"])/,
    replace: `$1${version}$2`,
  },
];

const stale = [];
for (const target of targets) {
  const file = path.resolve(root, target.file);
  if (!fs.existsSync(file)) continue;
  const before = fs.readFileSync(file, 'utf8');
  const after = before.replace(target.pattern, target.replace);
  if (after !== before) {
    stale.push(target.file);
    if (!checkOnly) fs.writeFileSync(file, after);
  }
}

if (checkOnly) {
  if (stale.length) {
    console.error(`sync-version: ${stale.length} file(s) out of sync with ${version}:`);
    for (const file of [...new Set(stale)]) console.error(`  - ${file}`);
    console.error('Run `node scripts/sync-version.mjs` to fix.');
    process.exit(1);
  }
  console.log(`sync-version: OK (${version})`);
} else if (stale.length) {
  console.log(`sync-version: updated ${[...new Set(stale)].length} file(s) to ${version}`);
} else {
  console.log(`sync-version: already at ${version}`);
}
