# Workspace Rules: Mandatory Skills Enforcement

This workspace enforces 10 specialized agent skills by default. See [AGENTS.md](file:///d:/false/AGENTS.md) for full policy specifications.

## Mandatory Skill Lifecycles:
1. **Planning**: Apply `grill-me` (interactive interview) and `task-decomposer` (breakdown in `TODO.md`) before writing implementation code.
2. **Implementation**: Enforce `surgical-refactor` (targeted diffs, no full-file rewrites), `minimalist-dark-ui` (zero-bloat, dark tokens), `component-scaffolder` (5-file atomic component layout), and `webgl-canvas-optimizer` (zero-allocation render loops, <10ms frame budget).
3. **Dependencies**: Apply `bundle-size-auditor` before adding packages to `Cargo.toml` or `package.json`.
4. **Pre-Commit**: Apply `security-audit` (vulnerability & secret scanning) and `conventional-commits` (strict semantic commit format).
5. **Handoff**: Apply `agent-handoff` to write session state to `.agents/HANDOFF.md`.
