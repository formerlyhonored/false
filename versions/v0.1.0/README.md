# False Browser — Release v0.1.0

- **Status**: Released
- **Release Date**: November 2026
- **Architecture**: Windows 64-bit (x86_64)

### Deliverables
- `FalseSetup.exe`: Standalone GUI Installer & Uninstaller
- `false-v0.1.0.flse`: Modular runtime update package

### Changelog
- Single-binary GUI installer with zero CMD popups and isolated setup profile.
- Native Windows integration: Desktop shortcut, Start Menu shortcut, and Control Panel Uninstall key.
- Modular runtime `.flse` packages with drag-and-drop import.
- Edge Chromium hardware-accelerated core (QUIC, HTTP/3, GPU rasterization, multi-threaded raster threads).
- Pure-Rust layout & render engine workbench (`false-dom`, `false-css`, `false-layout`).
- Built-in privacy-preserving Discord Rich Presence.
