---
name: bundle-size-auditor
description: >-
  Analyzes package.json, Cargo.toml/Cargo.lock, and system dependencies to identify
  and flag bloated dependencies, transitive bloat, and oversized binaries before
  hitting production. Use when adding new packages, updating dependencies, or optimizing
  build artifacts.
---

# Bundle Size Auditor Skill

Provides strict auditing procedures for tracking and optimizing dependency weight, binary footprint, and asset bundle sizes across Cargo (Rust) and Node.js/TypeScript (npm/yarn/pnpm) ecosystems.

---

## The Dependency Cost Philosophy

Every added dependency carries a long-term tax:
- Increased compilation/bundle time
- Expanded attack surface (supply chain risk)
- Binary footprint inflation (slower cold starts, larger downloads, cache pressure)
- Transitive dependency explosion

Before adding any dependency, evaluate whether standard library primitives or a small, zero-dependency helper can solve the problem instead.

---

## Ecosystem Auditing Guidelines

### 1. Cargo (Rust) Auditing

#### Key Commands & Inspection Tools
- **Inspect Dependency Tree**:
  ```powershell
  cargo tree -d                     # Find duplicate crates in dependency graph
  cargo tree --edges no-dev --depth 2 # Inspect top-level dependency tree
  ```
- **Analyze Binary Size Breakdown** (with `cargo-bloat`):
  ```powershell
  cargo bloat --release --crates   # Shows crate footprint in final binary
  cargo bloat --release -n 20      # Shows the 20 largest functions
  ```

#### Cargo Optimization Checklist
1. **Disable Default Features**:
   Always check if the crate provides `default-features = false`. Many crates bundle heavy HTTP clients, TLS suites, or serialization frameworks by default.
   ```toml
   # Bad: Pulls in hyper, openssl, all features
   reqwest = "0.12"

   # Good: Minimal, specific features only
   reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }
   ```
2. **Release Profile Optimizations (`Cargo.toml`)**:
   Ensure release profile enables Link-Time Optimization (LTO) and symbol stripping:
   ```toml
   [profile.release]
   opt-level = 3          # or "z" / "s" for size-sensitive binaries
   lto = true             # Whole-program link time optimization
   codegen-units = 1      # Improves optimization at the cost of compile time
   panic = "abort"        # Removes unwinding tables
   strip = true           # Automatically strip symbols from the binary
   ```

---

### 2. Frontend / Node.js (`package.json`) Auditing

#### Key Checks & Heuristics
1. **Tree-Shaking Support**:
   - Verify if the package exports ES Modules (`"module"`, `"type": "module"`, or `exports` field in `package.json`).
   - Avoid packages that only export CommonJS bundles if you only need a fraction of their utilities.
2. **Flag Common Bloat Culprits**:
   - 🚫 `moment` $\rightarrow$ Use `date-fns` (modular imports) or native `Intl.DateTimeFormat`.
   - 🚫 `lodash` (monolithic) $\rightarrow$ Use individual packages (`lodash.debounce`) or native modern JS methods (`Array.prototype.flat`, `structuredClone`).
   - 🚫 Full icon libraries (e.g., importing all icons from `@lucide/react` or `react-icons`) $\rightarrow$ Import individual SVGs or use tree-shaken icon paths.
3. **Lockfile Audit**:
   - Check `package-lock.json` or `pnpm-lock.yaml` for duplicate major versions of core libraries (e.g. two different versions of `react`, `typescript`, or `three`).

---

## Step-by-Step Auditing Procedure

When requested to add or update dependencies:

1. **Pre-Check Weight**:
   - Query or estimate the dependency's footprint and transitive graph size.
   - For web packages, verify unpacked size and whether tree-shaking works.
2. **Alternative Evaluation**:
   Ask: *Can this be implemented with fewer than 50 lines of idiomatic code using already-imported libraries or std primitives?*
   - If yes: Implement directly without introducing external dependencies.
   - If no: Select the lightest, actively-maintained alternative.
3. **Audit Transitive Dependencies**:
   After modifying `Cargo.toml` or `package.json`, inspect the diff in the lockfile (`Cargo.lock` or `package-lock.json`).
   - If a single package adds 30+ transitive crates/packages, stop and investigate whether feature flags can prune them.
4. **Binary / Bundle Delta Verification**:
   Compare the built artifact size before and after the change:
   - For Rust: Compare release binary file size.
   - For Web: Run bundle analyzer / build command and check chunk sizes.
