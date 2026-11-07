---
name: task-decomposer
description: >-
  Converts high-level feature requests, specifications, and refactoring plans
  into sequential, checkable sub-tasks persisted directly to TODO.md.
  Use when planning new features, breaking down complex tasks, or tracking multi-step execution.
---

# Task Decomposer Skill

Breaks down broad, complex engineering requirements into small, sequential, verifiable sub-tasks. Persists the task plan directly into `TODO.md` at the workspace root to ensure visibility, progress tracking, and execution discipline.

---

## Task Decomposition Principles

1. **Atomicity**: Each sub-task must represent a single, focused operation (e.g., "Add `RenderConfig` struct in `false-render/src/config.rs`") rather than broad buckets ("Build rendering engine").
2. **Sequential Dependency**: Order tasks logically so foundational work (data types, schemas) precedes logic, which precedes UI/integration, which precedes tests and benchmarks.
3. **Verifiability**: Every sub-task must have a concrete verification criterion (e.g., a command that compiles, a unit test that passes, or a diff to inspect).
4. **Persistence in `TODO.md`**: All tasks must be written to or updated in `TODO.md` in the workspace root.

---

## Canonical `TODO.md` Structure

```markdown
# Project TODO

## Current Goal: <Short Title of Goal>
> Objective: <One sentence description of the desired outcome>

### Phase 1: Architecture & Data Models
- [ ] Task 1.1: Define core data structures and interfaces in `<path/to/types>`.
  - Verification: `cargo check` / `tsc --noEmit` compiles without errors.
- [ ] Task 1.2: Add validation functions and unit tests for models.
  - Verification: `cargo test <module>` / `npm test <module>`.

### Phase 2: Core Implementation
- [ ] Task 2.1: Implement primary business logic / algorithm in `<path/to/source>`.
  - Verification: Core logic passes isolated unit tests.
- [ ] Task 2.2: Integrate module into existing pipeline or callers.
  - Verification: Integration tests run cleanly.

### Phase 3: Polish, Edge Cases & Verification
- [ ] Task 3.1: Add error handling for unexpected or malformed inputs.
- [ ] Task 3.2: Run end-to-end integration tests and performance benchmarks.
  - Verification: Full test suite passes.
```

---

## Execution Workflow

### Step 1: Read Existing `TODO.md`
Check if `TODO.md` exists in the workspace root.
- If it exists, read it with `read_file` to understand current progress and avoid overwriting existing unfinished work.
- If it does not exist, initialize it.

### Step 2: Decompose the Feature
Break the request down into 3–5 logical phases, each containing 2–4 atomic sub-tasks. Include exact file paths and verification criteria.

### Step 3: Write / Update `TODO.md`
Use `write_file` or `edit_file` to write the structured tasks to `TODO.md`.

### Step 4: Execute & Check Off Tasks
As tasks are completed during development:
- Mark tasks as completed by changing `- [ ]` to `- [x]`.
- For tasks currently in progress, mark as `[/]` (in-progress) if desired.
- Keep the user updated on phase progression.
