# False Browser Development Handoff

## Executive Summary
Delivered **v0.1.1** release addressing user-reported onboarding reoccurrence bugs, introducing full multi-profile and Google account management, deploying intelligent tab sleeping (RAM idle unloader), instant cosmetic ad and cookie wall blocking, and generating distribution packages (`FalseSetup.exe` and `false-v0.1.1.flse`).

## Key Changes in v0.1.1
1. **Onboarding Persistence Fix**:
   - Fixed the recurring onboarding popup across restarts by persisting `onboarding_done: 'true'` to SQLite `settings` table via `save_state` rather than volatile custom-protocol `localStorage`.
   - Updated `hydrateState` and `restoreSession` in `crates/false-ui/ui/index.html` to reliably verify and sync state.

2. **Multi-Profile & Google Account Management**:
   - Added `profiles` table to SQLite in `crates/false-ui/src/lib.rs` with automatic migration for default `personal` and `work` profiles.
   - Migrated `bookmarks` and `history` tables to include `profile_id TEXT NOT NULL DEFAULT 'personal'` with indexes.
   - Added Tauri commands: `profiles_list`, `profile_save`, `profile_delete`, `profile_set_active`, `profile_get_active`.
   - Built UI Profile Switcher popover (`#profilePop`) and Profile Editor modal (`#profileModalBackdrop`) in `crates/false-ui/ui/index.html`.
   - Supported linking Google account identity (name, email, Google avatar URL) with real-time UI updates to top toolbar avatar pill.
   - Child webview freeze/unfreeze layering implemented via `freezePageForPopover()` to prevent native WebView2 clipping over HTML popovers.

3. **Performance & Calming Blockers**:
   - Tab sleeping sweep: Automatically sleeps background tabs after 5 minutes of idle inactivity, unloading native child webviews to conserve system RAM while preserving tab state and instant restore cards.
   - Instant cosmetic blocker: Pure CSS and DOM `MutationObserver` injected via `init_script()` in `crates/false-ui/src/lib.rs` to purge ad slots (`ins.adsbygoogle`, `.ad-container`) and dismiss intrusive cookie consent overlays (`#onetrust-banner-sdk`, `.cookie-banner`, `.qc-cmp2-container`, `#didomi-host`) without network latency.

4. **Release & Packaging**:
   - Bumped version to `0.1.1` across `Cargo.toml`, `tauri.conf.json`, `index.html`, `installer.html`, `uninstaller.html`, and `installer_ui.rs`.
   - Executed `build_package.ps1 -Version "0.1.1"` generating:
     - `FalseSetup.exe` (18.5 MB)
     - `false-v0.1.1.flse` (7.07 MB)
   - Updated `versions/v0.1.1/README.md` with complete release notes.
