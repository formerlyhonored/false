# Workspace Rule: Default Skills Enforcement

All 10 workspace skills in `.agents/skills/` are mandatory standard operating procedures for this project:

- `grill-me`: Stop to ask clarifying architectural questions before implementing ambiguous or architectural tasks.
- `task-decomposer`: Plan and decompose multi-step tasks into `TODO.md`.
- `surgical-refactor`: Keep diffs minimal and targeted; do not rewrite whole files or perturb untouched boilerplate.
- `minimalist-dark-ui`: Adhere to zero-bloat layouts, 1px subtle borders, and scoped CSS variables.
- `component-scaffolder`: Scaffold components with complete types, module styles, tests, and barrel exports.
- `webgl-canvas-optimizer`: Eliminate render loop allocations and maintain 60fps/120fps frame budgets.
- `bundle-size-auditor`: Audit dependencies and binary footprint before introducing external crates/packages.
- `security-audit`: Scan diffs for secrets, injection vulnerabilities, and unsafe execution before committing.
- `conventional-commits`: Enforce strict semantic commit messages based on staged git diffs.
- `agent-handoff`: Persist active session memory and prioritized next steps to `.agents/HANDOFF.md`.
