<div align="center">

# false

**A Windows browser shell built on Tauri 2 and WebView2, with a custom Rust backend and a hand-built HTML/CSS/JS interface.**

[![Version](https://img.shields.io/badge/version-0.1.3-blue?style=flat-square)](https://github.com/formerlyhonored/false/releases/latest)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%20%2F%2011-0078d4?style=flat-square)](https://github.com/formerlyhonored/false/releases/latest)
[![License](https://img.shields.io/badge/license-Proprietary-6b7280?style=flat-square)](LICENSE)

</div>

---

## Status

False is a small, actively developed browser for Windows 10 and 11. The current
release is **v0.1.3**.

> v0.1.2 shipped a frontend that crashed during startup. It is superseded and
> should not be installed. See the [v0.1.3 release notes](versions/v0.1.3/README.md).

False renders web pages with the installed WebView2 runtime. The Rust backend
manages windows, tabs, storage, local filtering, and signed updates. It is not a
from-scratch HTML engine; the `false-*` crates outside `false-ui` are an
experimental rendering workbench, described below.

## Features

- **Tabs** with session restore, background unloading ("sleeping" tabs), mute,
  pin, drag reorder, and a compact tab strip.
- **Omnibox** with URL/search handling, autocomplete, and keyboard shortcuts.
- **Bookmarks and history** stored locally in SQLite, scoped per profile.
- **Profiles** with separate history, bookmarks, and browser storage.
- **Reader view**, find-in-page, zoom controls, print, and a downloads panel.
- **Ad and tracker filtering** through a local HTTP/HTTPS proxy with per-site
  allow rules and live counters.
- **Themes** with a dark-first, high-density interface.
- **Discord Rich Presence** (optional; it does not send visited URLs).
- **Signed updates** delivered as `.flse` packages. See [Updates](#updates).

## Install

Download the latest installer from the
[Releases page](https://github.com/formerlyhonored/false/releases/latest).

1. Run `FalseSetup.exe`.
2. Choose an install folder and shortcut options.
3. The installer registers False in the Start Menu, Desktop, and Windows
   "Apps & features".

Release artifacts:

| File | Purpose |
| :--- | :--- |
| `FalseSetup.exe` | GUI installer / uninstaller |
| `false.exe` | Standalone executable |
| `false-v0.1.3.flse` | Signed update package |

## Build from source

Requirements:

- Rust 1.80 or newer (edition 2021)
- Node.js 20 or newer (frontend checks and manifest signing)
- Windows 10/11 with the WebView2 runtime

```powershell
git clone https://github.com/formerlyhonored/false.git
cd false
.\build_package.ps1 -Version "0.1.3"
```

The script runs the frontend guardrail, Clippy, a release build, signs the
package manifest, and verifies the staged UI before producing `FalseSetup.exe`
and `false-v0.1.3.flse`.

Useful commands:

```powershell
node scripts/check-frontend.mjs          # DOM/command/version checks
cargo test -p false-ui --lib             # package and security tests
cargo clippy -p false-ui --all-targets --no-deps -- -D warnings
```

## Updates

Every `.flse` package contains `manifest.json` and `manifest.sig`. The signature
is verified against a public key compiled into the binary before any file is
extracted, and unsigned or tampered packages are rejected. Opening a `.flse`
only previews the verified publisher, version, and description; the user must
confirm before it is applied.

The release public key is committed at
[`crates/false-ui/keys/update-signing.pub`](crates/false-ui/keys/update-signing.pub).
Its SHA-256 fingerprint is:

```
52:e1:13:c6:57:5c:37:b4:f3:42:d3:fc:8d:0f:78:a1:dd:6e:9a:43:37:1d:a1:98:6d:78:17:32:e1:d4:45:8b
```

The private key is never committed. Maintainers sign with:

```powershell
node scripts/sign-manifest.mjs <manifest.json> <manifest.sig>
node scripts/verify-manifest.mjs <manifest.json> <manifest.sig>
```

## Repository layout

```
crates/
  false-ui/          Shipping app: Tauri shell, installer, signed updates, UI
  false-core/        Experimental engine: data structures and config
  false-net/         Experimental engine: networking helpers
  false-dom/         Experimental engine: HTML parser and DOM
  false-css/         Experimental engine: CSS parser and cascade
  false-layout/      Experimental engine: box layout
  false-render/      Experimental engine: rasterizer
  false-engine/      Experimental engine: pipeline coordinating the above
  false-installer/   Command-line installer helper
scripts/             Frontend checks and release signing tools
versions/            Per-release notes
website/             Static project page
```

The `false-ui` crate is the browser users run. The other `false-*` crates are a
separate, incomplete rendering workbench used for experiments and the
`false_engine_render` command. They are not the engine behind normal browsing.

## Repository hygiene

- Release artifacts are published on GitHub Releases, not committed.
- `Cargo.lock` is tracked because this is an application workspace.
- Dead legacy code is removed from the current tree; history remains in Git.

## License

Copyright &copy; 2026 **zaidi**. All rights reserved.
See [LICENSE](LICENSE) for the full terms.
