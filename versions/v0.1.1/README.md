# False Browser — Version v0.1.1

- **Status**: Released
- **Release Date**: November 7, 2026
- **Architecture**: Windows x64 (Tauri v2 + WebView2 + Pure Rust Engine)

---

### Key Features & Enhancements

#### 1. Multi-Profile & Account Management
- Create and switch between multiple isolated profiles (Personal, Work, Custom).
- Profile-scoped bookmarks and browsing history, isolated directly within SQLite.
- Seamless active profile switcher popover and profile editor modal with custom color tags.

#### 2. Google Account Connection
- Connect Google identity (name, email, avatar URL) directly to your profile.
- Display custom avatars or color initials on the main top toolbar and profile menus.
- Integrated into the first-run onboarding setup and settings modal.

#### 3. Intelligent Tab Sleeping
- Automated background tab suspension after 5 minutes of idle inactivity.
- Deep RAM reduction: sleeps offscreen child webviews and reawakens seamlessly upon selection.

#### 4. Instant Cosmetic Blocker & Cookie Wall Dismissal
- Pure CSS and MutationObserver cosmetic ad-filtering rules injected at document startup.
- Zero network proxy latency: hides banner ads, Google Ads containers, and invasive cookie consent walls (`#onetrust-banner-sdk`, `.qc-cmp2-container`, `.cookie-banner`).
- Restores frozen body scrolling automatically on consent-blocked websites.

#### 5. Onboarding Persistence Fix
- Fixed recurring onboarding modal on every startup by persisting completion flag directly into SQLite settings.

---

### Release Artifacts
- **Installer**: `FalseSetup.exe` (Self-extracting standalone setup)
- **Hot-Patch Package**: `false-v0.1.1.flse` (Zero-downtime delta update package)
