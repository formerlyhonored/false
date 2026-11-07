# False Browser — Version v0.1.3

- **Status**: Released
- **Release Date**: November 7, 2026
- **Architecture**: Windows x64 (Tauri v2 + WebView2)
- **Update signing key SHA-256**: `52:e1:13:c6:57:5c:37:b4:f3:42:d3:fc:8d:0f:78:a1:dd:6e:9a:43:37:1d:a1:98:6d:78:17:32:e1:d4:45:8b`

---

### Why this release exists

v0.1.2 shipped a frontend that threw during startup: `boot()` called profile
functions that were not present in the packaged `ui/index.html`. The shell never
created a tab. v0.1.3 replaces that build and adds the guardrail that should have
caught it.

### Fixes

- Fixed the startup crash and removed the stale `#historyTop` binding.
- Every top-level DOM binding is guarded; independent init sections fail locally
  instead of aborting the whole shell.
- Added a visible startup error overlay with Ctrl+C copy support.
- Added `scripts/check-frontend.mjs`, which fails the build when:
  - a `$('#id')` reference has no matching element,
  - a direct `invoke()` command is missing from `generate_handler![]`,
  - a startup function is undeclared,
  - version literals drift across crates and HTML.

### Security

- `.flse` packages are signed with Ed25519. The public key is pinned in the
  binary and unsigned/tampered packages are rejected before extraction.
- Opening a `.flse` no longer auto-applies it. The app verifies the manifest and
  shows the publisher, version, and description before asking for consent.
- Package extraction rejects traversal, absolute paths, drive prefixes,
  alternate data streams, Windows device names, and oversized entries.
- Update downloads no longer interpolate URLs/paths into PowerShell commands.
- Installer paths are validated before being interpolated into VBScript/registry.
- DevTools are release-gated and the app ships with a restrictive CSP.
- Binary auto-update was removed; all updates go through the signed `.flse` path.

### Release artifacts

- **Installer**: `FalseSetup.exe`
- **Standalone binary**: `false.exe`
- **Signed update package**: `false-v0.1.3.flse`
- **Public signing key**: `crates/false-ui/keys/update-signing.pub`

> **v0.1.2 is superseded and should not be installed.** Its `.flse` package
> contains the broken startup script.
