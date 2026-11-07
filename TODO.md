# Project TODO

## Current Goal: Git Setup, GitHub Repository, Landing Page & Changelog
> Objective: Install Git on the system, set up GitHub integration, create GitHub repository documentation, then build a high-performance minimalist dark landing page with an integrated changelog.

### Phase 1: Git Installation & Authentication
- [ ] Task 1.1: Install Git using `winget install --id Git.Git -e --source winget --accept-source-agreements --accept-package-agreements --silent`.
  - Verification: `git --version` executes cleanly and outputs valid Git version.
- [ ] Task 1.2: Refresh environment PATH and verify Git CLI is accessible.
- [ ] Task 1.3: Guide user to authenticate with GitHub (`gh auth login` or git credentials).

### Phase 2: GitHub Repository Setup & GitHub Presence
- [ ] Task 2.1: Initialize git repository in workspace (`git init`) if not already initialized.
- [ ] Task 2.2: Ensure `.gitignore` is comprehensive (ignoring `/target`, temp files, test lnks, etc.).
- [ ] Task 2.3: Craft polished, minimalist README.md showcasing False Browser, architecture, features, and installer links.
- [ ] Task 2.4: Stage and commit workspace files adhering to `conventional-commits` (`feat: initial release v0.1.0`).

### Phase 3: Landing Page (`minimalist-dark-ui`)
- [ ] Task 3.1: Design and build the False Browser landing page (zero-bloat dark palette, live glacier loop preview, quick download CTAs for `FalseSetup.exe` and `.flse`).
- [ ] Task 3.2: Verify responsive layout, 60fps animations, and fast load times.

### Phase 4: Integrated Changelog Page
- [ ] Task 4.1: Build the changelog page inside the landing page tracking `v0.1.0` release notes and future update logs.
- [ ] Task 4.2: Link changelog navigation and test switching between landing page and changelog.
