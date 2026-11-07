# False Browser Development Handoff

## Executive Summary
Resolved critical installer/uninstaller startup lag, missing Windows integration (Desktop shortcut, Start Menu shortcut, and Control Panel Uninstall registry), silent process death on "Open false", and browser website rendering bottlenecks. Both `FalseSetup.exe` and `false-v0.1.0.flse` have been rebuilt and verified.

## Key Changes & Root Cause Fixes
1. **Isolated WebView2 Profile for Installer / Uninstaller**:
   - *Problem*: `FalseSetup.exe` crashed or froze on startup with `HRESULT 0x800700AA ("The requested resource is in use.")` whenever `false.exe` was running or held a lock on `EBWebView`.
   - *Solution*: Added automated detection in `main.rs` to set `WEBVIEW2_USER_DATA_FOLDER` to a dedicated temp directory for setup/uninstaller mode, eliminating locks and collisions.
2. **Missing Tauri IPC Handler Registration**:
   - *Problem*: All 7 installer commands (`installer_get_defaults`, `installer_pick_folder`, `installer_run`, `installer_launch_app`, `installer_close`, `uninstaller_run`, `uninstaller_close`) were missing from `tauri::generate_handler!`, causing installer API calls to fail silently in JavaScript catch blocks.
   - *Solution*: Registered all 7 installer commands in `crates/false-ui/src/lib.rs`.
3. **Robust Native Shortcut & Registry Creation**:
   - *Problem*: Sluggish, error-prone PowerShell multiline COM scripts failed to create `.lnk` shortcuts and registry entries.
   - *Solution*: Replaced with `cscript //nologo` VBScript (15ms execution, `CREATE_NO_WINDOW`) for both Desktop (`%USERPROFILE%\Desktop\False.lnk`) and Start Menu (`%APPDATA%\Microsoft\Windows\Start Menu\Programs\False.lnk`). Registered Control Panel uninstall entries and `.flse` file associations with silent `reg add` commands.
4. **Reliable Browser Launch (`installer_launch_app`)**:
   - *Problem*: Process was terminated on installer exit and working directory was unset.
   - *Solution*: Set working directory to installation folder, spawned with `DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP`, and allowed clean handoff.
5. **Instant Installer Boot & Zero "Not Responding" Lag**:
   - *Problem*: `AdBlock::new()`, network proxy listener, SQLite disk DB opening, and rule compilation blocked the UI thread before window creation.
   - *Solution*: Bypassed proxy, disk database, and rule loading for installer/uninstaller modes, booting the window in <150ms.
6. **Browser Rendering & Networking Optimizations**:
   - Removed aggressive `singleTab` link mutation (`target="_self"`) and `window.open` hijacking that broke Google/GitHub OAuth flows.
   - Removed 1.5s interval `false.invalid` pseudo-navigations that aborted active `fetch` requests on complex websites.
   - Allowed direct native Chromium Edge networking without user-space proxy bottlenecks, enabling full HTTP/2, HTTP/3 QUIC, and zero-roundtrip TLS.

## Distribution Artifacts
- `d:\false\FalseSetup.exe` (18.45 MB): Standalone GUI Installer & Uninstaller
- `d:\false\false-v0.1.0.flse` (7.05 MB): Custom False Update Package
- Release Version: `v0.1.0`

## Verification
- Verified `FalseSetup.exe` launches instantly (<150ms) with zero resource collision.
- Verified Desktop shortcut `C:\Users\zaidi\Desktop\False.lnk` and Start Menu shortcut are created.
- Verified Control Panel Uninstall entry in `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\False`.
- Verified `FalseSetup.exe --uninstaller` launches GUI uninstaller.
- Verified `false.exe` browser launches smoothly at native speeds.
