# Workspace Agent Rules & Mandatory Skill Enforcement

This workspace operates under strict development and pair-programming discipline. The agent MUST apply the following 10 workspace skills by default across their respective workflow lifecycle stages.

---

## 1. Architectural Alignment & Planning (Before Any Code is Written)

- **`grill-me` ([skills/grill-me](file:///d:/false/.agents/skills/grill-me/SKILL.md))**:
  - **Policy**: For any non-trivial feature, architectural change, or ambiguous request, HALT before writing code.
  - Conduct an interactive clarifying interview using `ask_question` or targeted inquiries to resolve non-goals, data flow, failure modes, and trade-offs.
  - Proceed with implementation only after alignment is reached.

- **`task-decomposer` ([skills/task-decomposer](file:///d:/false/.agents/skills/task-decomposer/SKILL.md))**:
  - **Policy**: Decompose multi-step features, architectural migrations, or refactoring plans into sequential, testable sub-tasks.
  - Persist the task list directly to `TODO.md` at the workspace root and check off items as they are verified.

---

## 2. Implementation Precision & Code Quality (During Development)

- **`surgical-refactor` ([skills/surgical-refactor](file:///d:/false/.agents/skills/surgical-refactor/SKILL.md))**:
  - **Policy**: Mandatory for modifying existing code.
  - Never rewrite entire files or disturb surrounding boilerplate when changing a localized function or block.
  - Use `edit_file` with precise, anchored replacements.
  - Preserve all existing comments, docstrings, formatting, and naming conventions. Keep the diff minimal and proportional to the task.

- **`minimalist-dark-ui` ([skills/minimalist-dark-ui](file:///d:/false/.agents/skills/minimalist-dark-ui/SKILL.md))**:
  - **Policy**: Mandatory for any UI styling, layout, or design component.
  - Enforce zero-bloat, high-density layouts; 1px subtle borders (`rgba(255, 255, 255, 0.08)`); layered dark palettes (`#09090b`, `#121215`, `#18181b`); tabular monospace numbers (`tnum`); and strictly scoped CSS variables without global style pollution.

- **`component-scaffolder` ([skills/component-scaffolder](file:///d:/false/.agents/skills/component-scaffolder/SKILL.md))**:
  - **Policy**: When requested to build a new UI component, scaffold the complete canonical package:
    - `<ComponentName>.types.ts`: Strict props, variants, and event interfaces.
    - `<ComponentName>.module.css`: Scoped CSS Module styling following dark UI tokens.
    - `<ComponentName>.tsx`: Accessible, semantic component presentation and logic.
    - `<ComponentName>.test.tsx`: Comprehensive unit and interaction tests.
    - `index.ts`: Public barrel export.

- **`webgl-canvas-optimizer` ([skills/webgl-canvas-optimizer](file:///d:/false/.agents/skills/webgl-canvas-optimizer/SKILL.md))**:
  - **Policy**: Mandatory when implementing or reviewing graphics, Canvas 2D, WebGL/WebGPU, or engine render loops.
  - Enforce the 60fps/120fps budget ($\le$10ms CPU render work per frame).
  - Absolute zero heap allocations/closures inside the animation/tick loop.
  - Batch draw calls, minimize GPU pipeline state changes, and clamp High-DPI canvas DPR to $\le$2.0.

---

## 3. Dependency Management & Binary Footprint (Before Adding Dependencies)

- **`bundle-size-auditor` ([skills/bundle-size-auditor](file:///d:/false/.agents/skills/bundle-size-auditor/SKILL.md))**:
  - **Policy**: Before adding or updating packages in `Cargo.toml` or `package.json`, evaluate dependency weight.
  - Always check if crate features can be pruned (`default-features = false`).
  - Run `cargo tree -d` to check for duplicate crates and verify that tree-shaking works for web libraries.
  - Prefer stdlib/zero-dependency solutions over heavy external libraries for simple tasks.

---

## 4. Security Verification & Commit Standards (Before Committing)

- **`security-audit` ([skills/security-audit](file:///d:/false/.agents/skills/security-audit/SKILL.md))**:
  - **Policy**: Before committing or executing system calls, audit diffs for security risks:
    - No hardcoded API keys, secrets, or tokens.
    - Validate inputs and sanitize file paths (prevent path traversal).
    - Avoid unsafe shell execution (`sh -c`, `cmd /C`) with concatenated user input; use discrete argument arrays.
    - Enforce proper permissions and authentication on IPC/WebSocket endpoints.
    - Require `// SAFETY:` justifications on all Rust `unsafe` blocks.

- **`conventional-commits` ([skills/conventional-commits](file:///d:/false/.agents/skills/conventional-commits/SKILL.md))**:
  - **Policy**: Mandatory for creating git commits.
  - Inspect staged changes (`git diff --cached`).
  - Format commit messages strictly according to Conventional Commits: `<type>(<optional-scope>): <lowercase imperative subject>`.
  - Use recognized types: `feat`, `fix`, `refactor`, `perf`, `build`, `test`, `docs`, `style`, `ci`, `chore`.
  - Flag breaking changes with `!` or `BREAKING CHANGE:`.

---

## 5. Context Persistence & Handoff (At Session End / Context Switch)

- **`agent-handoff` ([skills/agent-handoff](file:///d:/false/.agents/skills/agent-handoff/SKILL.md))**:
  - **Policy**: When ending a session, switching to a new major task, or preparing for context compaction, persist active context to [`.agents/HANDOFF.md`](file:///d:/false/.agents/HANDOFF.md).
  - Record the executive summary, architectural decisions, modified files, test health, prioritized next steps, and a resumption prompt.
