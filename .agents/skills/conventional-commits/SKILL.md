---
name: conventional-commits
description: >-
  Enforces strict Conventional Commits specification (feat:, fix:, refactor:, etc.)
  by analyzing staged git diffs before creating or executing git commit commands.
  Use whenever staging changes, preparing commit messages, or running git commit workflows.
---

# Conventional Commits Skill

Enforces strict standardization of commit messages based on the **Conventional Commits v1.0.0** specification. It prevents vague or malformed commits by inspecting staged diffs and generating precise, semantic commit messages.

---

## Commit Message Format

Every commit message must strictly follow this structure:

```
<type>(<scope>): <subject>

[optional body]

[optional footer(s)]
```

### 1. Types (Strict Whitelist)

| Type | Purpose | SemVer Impact |
| :--- | :--- | :--- |
| `feat` | Introduces a new feature or functionality to the codebase | Minor (unless breaking) |
| `fix` | Patches a bug or addresses an unintended defect | Patch (unless breaking) |
| `refactor` | Code change that neither fixes a bug nor adds a feature | Patch |
| `perf` | Code change that improves performance | Patch |
| `docs` | Documentation changes only (e.g., README, API docs, comments) | None |
| `style` | Formatting, whitespace, semicolon changes; no production code behavior change | None |
| `test` | Adding missing tests, refactoring test suites, or updating mocks | None |
| `build` | Changes affecting build systems, packaging, or external dependencies | Patch |
| `ci` | Changes to CI configuration files and automation scripts | None |
| `chore` | Maintenance tasks, repository housekeeping, tooling adjustments | None |
| `revert` | Reverts a previous commit | Depends |

### 2. Scope Guidelines
- Scopes must be lowercase, enclosed in parentheses, and represent the module, package, or subsystem affected.
- Examples: `feat(false-render):`, `fix(dom):`, `refactor(parser):`, `build(cargo):`.
- If a change spans the entire project or a distinct scope cannot be isolated, omit the scope (e.g., `chore: update license year`).

### 3. Subject Line Rules
- Use imperative, present tense: `"add"`, not `"added"` or `"adds"`.
- Lowercase first letter after the colon.
- No trailing period (`.`) at the end of the subject.
- Keep the subject under 72 characters.

### 4. Breaking Changes
- Indicate breaking changes either by appending a `!` before the colon:
  ```
  feat(api)!: change payload schema for websocket events
  ```
- And/or by including a `BREAKING CHANGE:` footer in the commit body:
  ```
  BREAKING CHANGE: The `render_node` method now takes a context reference instead of a raw pointer.
  ```

---

## Workflow: Pre-Commit Diff Analysis

Before executing any `git commit`, execute the following procedure:

### Step 1: Inspect Staged Diffs
Run:
```powershell
git diff --cached
```
If no changes are staged, check:
```powershell
git status --short
```
Stage only the relevant files intended for this atomic unit of work:
```powershell
git add <files>
```

### Step 2: Classify the Change
Analyze the diff output:
1. Did we add new behavior or capability? $\rightarrow$ `feat`
2. Did we resolve an issue, crash, or logic flaw? $\rightarrow$ `fix`
3. Did we restructure code without changing behavior? $\rightarrow$ `refactor`
4. Did we optimize speed, memory, or draw calls? $\rightarrow$ `perf`
5. Did we alter dependencies, Cargo.toml, or build scripts? $\rightarrow$ `build`

### Step 3: Validate Against Anti-Patterns
- **Never** create multi-concern commits (e.g., combining a feature addition and an unrelated formatting cleanup). Split them into separate commits.
- **Never** use generic messages like `fix: fix bugs`, `feat: update code`, or `chore: misc changes`.
- **Never** include staging instructions or AI conversation references in the commit message.

### Step 4: Execute Commit
Construct the command using the properly quoted message:
```powershell
git commit -m "feat(false-render): add batching for canvas draw calls"
```
Or for multi-line messages:
```powershell
git commit -m "fix(css): correctly compute specificity for pseudo-classes" -m "Resolves an issue where :hover rules were prematurely discarded during cascading resolution."
```
