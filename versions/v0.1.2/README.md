# False Browser — Version v0.1.2

- **Status**: Released
- **Release Date**: November 7, 2026
- **Architecture**: Windows x64 (Tauri v2 + WebView2 + Pure Rust Engine)

---

### Key Features & Enhancements

#### 1. Native Main Menu & Navigation Overhaul
- Introduced a dedicated browser Main Menu popup triggered via the new toolbar Menu button (`#menuBtn`) or the sidebar Menu button (`#setSide`).
- Quick navigation shortcuts:
  - New tab (`Ctrl + T`), New window (`Ctrl + N`), New private window (`Ctrl + Shift + N`).
  - History (`Ctrl + H`), Bookmarks (`Ctrl + Shift + O`), Downloads (`Ctrl + J`).
  - Inline Bookmarks bar toggle (`Ctrl + Shift + B`).
  - Zoom controls with inline percentage stepper (`+`, `-`, reset).
  - Print (`Ctrl + P`), Find in page (`Ctrl + F`), Reader view, and DevTools (`F12`).
  - Direct Profiles & Accounts manager link.

#### 2. Native Context Menus
- Full context menu support inside native webviews for:
  - **Links**: Open link in new tab, copy link address.
  - **Images**: Open image in new tab, copy image address.
  - **Editable inputs**: Cut, Copy, Paste, Select all.
  - **Selected text**: Search with active search engine, Copy.
  - **Page background**: Back, Forward, Reload, Bookmark, Copy link, Print, Inspect element.

#### 3. Extensions Dock Popover
- Added top toolbar Extensions button (`#extBtn`) opening a clean popover listing active and installed extensions.

#### 4. Address-Bar Bookmark Star & Refreshed Quick Tiles
- Integrated a fast toggle star (`#star`) directly into the omnibar address input for instant one-click tab bookmarking.
- Updated new-tab quick link tiles with high-density rounded glass icons and titles underneath.

#### 5. Seamless Multi-Profile Continuity
- Fully retained multi-profile management (Personal, Work, Custom) and Google account connection in the top toolbar avatar pill (`#profileTop`).

---

### Release Artifacts
- **Installer**: `FalseSetup.exe` (Standalone Windows setup)
- **Hot-Patch Package**: `false-v0.1.2.flse` (Modular update package)
