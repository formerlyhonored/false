#!/usr/bin/env node
/**
 * False frontend pre-flight checks.
 *
 * Catches the class of bugs that shipped in v0.1.2:
 *   - a $()/#id reference with no matching element in the markup
 *   - an invoke() command with no #[tauri::command] registered in Rust
 *   - a function called from boot() that is never declared
 *
 * Usage:
 *   node scripts/check-frontend.mjs [path/to/index.html]
 *
 * Exits 1 on errors. Run it from build_package.ps1 and CI.
 */

import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const indexPath = path.resolve(root, process.argv[2] || 'crates/false-ui/ui/index.html');
const rustPath = path.resolve(root, process.argv[3] || 'crates/false-ui/src/lib.rs');

const errors = [];
const warnings = [];
const err = (msg) => errors.push(msg);
const warn = (msg) => warnings.push(msg);

if (!fs.existsSync(indexPath)) {
  console.error(`check-frontend: index not found: ${indexPath}`);
  process.exit(1);
}

const html = fs.readFileSync(indexPath, 'utf8');
const lineOf = (index) => html.slice(0, index).split('\n').length;
const relIndex = path.relative(root, indexPath) || indexPath;

// ---------------------------------------------------------------------------
// 1. Every $()/#id reference must resolve to an element in the markup.
// ---------------------------------------------------------------------------
const ids = new Set(
  [...html.matchAll(/\bid\s*=\s*["']([^"']+)["']/g)].map((m) => m[1])
);
const idRefRe = /\$\(\s*['"]#([A-Za-z0-9_-]+)['"]\s*\)/g;
const missingIds = new Map();
for (const m of html.matchAll(idRefRe)) {
  if (!ids.has(m[1])) {
    const line = lineOf(m.index);
    if (!missingIds.has(m[1])) missingIds.set(m[1], line);
  }
}
for (const [id, line] of missingIds) {
  err(`${relIndex}:${line}: $() references missing element #${id}`);
}

// Top-level element bindings that would throw on a null element.
// (These are the ones that abort the whole single-scope script.)
const lines = html.split(/\r?\n/);
let scriptDepth = 0;
for (let i = 0; i < lines.length; i++) {
  const line = lines[i];
  if (/<script\b/.test(line) && !/<\/script>/.test(line)) scriptDepth++;
  if (/<\/script>/.test(line)) scriptDepth = Math.max(0, scriptDepth - 1);
  if (scriptDepth <= 0) continue;
  if (/^\s*\$\('#[^']+'\)\s*\.\s*on[a-z]+\s*=/.test(line) && !/^\s{2,}/.test(line)) {
    err(`${relIndex}:${i + 1}: unguarded top-level binding on a possibly-missing element -> ${line.trim()}`);
  }
}

// ---------------------------------------------------------------------------
// 2. Every invoke('cmd') must exist in the Tauri generate_handler! list.
// ---------------------------------------------------------------------------
// `tryInvoke` is the deliberate graceful-degradation path: it returns false when
// the command is unavailable, so a missing command there is a warning, not a
// build-stopper. Direct `tauriInvoke`/`invoke` calls reject and must exist.
const invokeRe = /\b(tauriInvoke|tryInvoke|invoke)\(\s*['"]([a-z_][a-z0-9_]*)['"]/gi;
const invoked = new Map();
for (const m of html.matchAll(invokeRe)) {
  const [, fn, cmd] = m;
  if (!invoked.has(cmd)) invoked.set(cmd, { line: lineOf(m.index), optional: fn === 'tryInvoke' });
}

let registered = null;
if (fs.existsSync(rustPath)) {
  const rust = fs.readFileSync(rustPath, 'utf8');
  const handlerMatch = rust.match(/generate_handler!\s*\[([\s\S]*?)\]/);
  if (handlerMatch) {
    registered = new Set(
      handlerMatch[1]
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean)
    );
  } else {
    warn(`${path.relative(root, rustPath)}: no generate_handler![] found; skipping invoke check`);
  }
} else {
  warn(`Rust source not found: ${rustPath}; skipping invoke check`);
}

if (registered) {
  for (const [cmd, { line, optional }] of invoked) {
    if (registered.has(cmd)) continue;
    const msg = `${relIndex}:${line}: invoke('${cmd}') is not registered in generate_handler![]`;
    if (optional) warn(msg + ' (tryInvoke: optional feature, shows fallback)');
    else err(msg);
  }
}

// ---------------------------------------------------------------------------
// 3. Functions called from boot()/startup paths must be declared somewhere.
// ---------------------------------------------------------------------------
const declared = new Set();
for (const re of [
  /\bfunction\s+([A-Za-z_$][\w$]*)\s*\(/g,
  /\b(?:const|let|var)\s+([A-Za-z_$][\w$]*)\s*=/g,
  /^\s*([A-Za-z_$][\w$]*)\s*=\s*(?:async\s*)?(?:function|\()/gm,
]) {
  for (const m of html.matchAll(re)) declared.add(m[1]);
}

// A deliberately small allowlist of globals/builtins seen in this script.
const GLOBALS = new Set([
  'if', 'for', 'while', 'switch', 'catch', 'function', 'return', 'typeof', 'new',
  'console', 'addEventListener', 'removeEventListener', 'setTimeout', 'clearTimeout',
  'setInterval', 'clearInterval', 'requestAnimationFrame', 'cancelAnimationFrame',
  'document', 'window', 'localStorage', 'sessionStorage', 'navigator', 'location',
  'JSON', 'Math', 'Date', 'Object', 'Array', 'String', 'Number', 'Boolean', 'RegExp',
  'Map', 'Set', 'Promise', 'Error', 'URL', 'Image', 'MutationObserver', 'ResizeObserver',
  'parseInt', 'parseFloat', 'isNaN', 'isFinite', 'encodeURIComponent', 'decodeURIComponent',
  'encodeURI', 'decodeURI', 'alert', 'confirm', 'prompt', 'fetch', 'matchMedia',
  'getComputedStyle', 'requestIdleCallback', 'queueMicrotask', 'URLSearchParams',
  'tauriInvoke', 'tryInvoke', 'section', 'on', 'onEv', 'void', 'await', 'async',
  'yield', 'delete', 'in', 'of', 'instanceof', 'super', 'this',
]);

function declares(name) {
  return declared.has(name) || GLOBALS.has(name);
}

for (const fnName of ['boot', 'loadProfiles', 'renderProfileBadge', 'renderProfilePop', 'switchProfile']) {
  const fnMatch = new RegExp(`(?:async\\s+)?function\\s+${fnName}\\s*\\([^)]*\\)\\s*\\{`).exec(html);
  if (!fnMatch) {
    if (!declared.has(fnName)) err(`${relIndex}: required startup function ${fnName}() is not declared`);
    continue;
  }
  // Extract the body with a brace counter.
  let depth = 1;
  let j = fnMatch.index + fnMatch[0].length;
  const start = j;
  while (j < html.length && depth > 0) {
    const ch = html[j];
    if (ch === '{') depth++;
    else if (ch === '}') depth--;
    j++;
  }
  const body = html.slice(start, j - 1);
  for (const m of body.matchAll(/(?<![.\w$])([A-Za-z_$][\w$]*)\s*\(/g)) {
    const name = m[1];
    if (declares(name)) continue;
    err(`${relIndex}:${lineOf(start + m.index)}: ${fnName}() calls undeclared function ${name}()`);
  }
}

// ---------------------------------------------------------------------------
// 4. Version consistency across the app metadata.
// ---------------------------------------------------------------------------
const pkgVersion = /^version\s*=\s*"([^"]+)"/m.exec(
  fs.existsSync(path.resolve(root, 'crates/false-ui/Cargo.toml'))
    ? fs.readFileSync(path.resolve(root, 'crates/false-ui/Cargo.toml'), 'utf8')
    : ''
)?.[1];

if (pkgVersion) {
  const versionFiles = [
    ['crates/false-ui/tauri.conf.json', /"version"\s*:\s*"([^"]+)"/],
    ['crates/false-installer/Cargo.toml', /^version\s*=\s*"([^"]+)"/m],
  ];
  for (const [rel, re] of versionFiles) {
    const p = path.resolve(root, rel);
    if (!fs.existsSync(p)) continue;
    const found = re.exec(fs.readFileSync(p, 'utf8'))?.[1];
    if (found && found !== pkgVersion) {
      err(`${rel}: version ${found} does not match crates/false-ui/Cargo.toml (${pkgVersion})`);
    }
  }

  // Static version labels in the shipped HTML.
  const htmlVersion = /Version\s+(\d+\.\d+\.\d+)/.exec(html)?.[1];
  if (htmlVersion && htmlVersion !== pkgVersion) {
    err(`${relIndex}: version ${htmlVersion} does not match crates/false-ui/Cargo.toml (${pkgVersion})`);
  }
  const fallbackVersion = /current_version\s*:\s*['"](\d+\.\d+\.\d+)['"]/.exec(html)?.[1];
  if (fallbackVersion && fallbackVersion !== pkgVersion) {
    err(`${relIndex}: update fallback ${fallbackVersion} does not match crates/false-ui/Cargo.toml (${pkgVersion})`);
  }
  for (const rel of [
    'crates/false-ui/ui/installer.html',
    'crates/false-ui/ui/uninstaller.html',
  ]) {
    const p = path.resolve(root, rel);
    if (!fs.existsSync(p)) continue;
    const found = /Version\s+(\d+\.\d+\.\d+)/.exec(fs.readFileSync(p, 'utf8'))?.[1];
    if (found && found !== pkgVersion) {
      err(`${rel}: version ${found} does not match crates/false-ui/Cargo.toml (${pkgVersion})`);
    }
  }
  // Hard-coded fallbacks/defaults in Rust sources.
  for (const rel of ['crates/false-ui/src/main.rs', 'crates/false-ui/src/flse.rs', 'crates/false-ui/src/installer_ui.rs']) {
    const p = path.resolve(root, rel);
    if (!fs.existsSync(p)) continue;
    const src = fs.readFileSync(p, 'utf8');
    for (const m of src.matchAll(/"(0\.\d+\.\d+)"/g)) {
      if (m[1] !== pkgVersion) {
        warn(`${rel}:${src.slice(0, m.index).split('\n').length}: hard-coded version ${m[1]} (expected ${pkgVersion})`);
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------
const label = relIndex;
for (const w of warnings) console.warn(`WARN  ${w}`);
if (errors.length) {
  for (const e of errors) console.error(`ERROR ${e}`);
  console.error(`\ncheck-frontend: ${errors.length} error(s), ${warnings.length} warning(s) in ${label}`);
  process.exit(1);
}
console.log(`check-frontend: OK (${label}) — ${ids.size} ids, ${invoked.size} commands, ${declared.size} declarations`);
