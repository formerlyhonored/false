---
name: surgical-refactor
description: >-
  Restricts the AI from rewriting large boilerplate sections or whole files.
  Forces minimal, surgical, targeted diffs using exact character replacements
  to preserve existing architecture, invariants, and comments.
  Use when modifying existing code, fixing bugs, or executing refactor tasks.
---

# Surgical Refactor Skill

Enforces minimal, high-precision code modifications. Prevents unprompted whole-file rewrites, loss of comments, boilerplate disruption, and unintentional architectural drift.

---

## Core Directive: Surgical Precision Over Sweeping Rewrites

When modifying any existing source file:
1. **Never overwrite an entire file** if only a function, method, conditional block, or a few lines need to change.
2. **Favor `edit_file` over `write_file` (with `Overwrite: true`)**.
3. **Preserve surrounding context**: Retain all existing variable names, function signatures, comments, docstrings, imports, and formatting unless explicitly instructed to modify them.
4. **Zero-drift principle**: The total lines modified should be proportional to the actual logic change requested.

---

## The Surgical Refactoring Rules

### 1. The Diff Budget Principle
- Before making an edit, calculate the minimum number of lines required to achieve the goal.
- If a change requires altering 5 lines, the diff must not touch 50 lines.
- Reject automatic formatting passes (e.g. running full file prettier/rustfmt) across entire existing files when making a small bug fix or targeted feature enhancement, as it obscures git blame.

### 2. Targeting Anchors
When preparing an edit using `edit_file`:
- Select a unique, stable anchor block (5–10 lines above and below if needed) to ensure the replacement matches exactly once.
- Ensure indentation, line endings, and whitespace in `Target` match the existing file exactly.
- Verify that replacing `Target` with `Replacement` leaves no orphan braces, dangling commas, or syntax errors.

### 3. Preserving Invariants and Architecture
- Do not introduce new abstractions (helper classes, factory methods, wrapper types) unless the architectural pattern is already established in that module or explicitly requested.
- Maintain existing error handling paradigms (e.g., if the module returns `Result<T, CustomError>`, do not introduce `.unwrap()` or panic).
- Do not reorder functions or methods in an existing file. Add new methods adjacent to related functionality.

---

## Step-by-Step Refactoring Procedure

1. **Read and Map**:
   Read the targeted region of the file using `read_file` with specific line ranges (`StartLine`, `EndLine`).
2. **Isolate Scope**:
   Identify the smallest self-contained block that requires modification.
3. **Formulate Minimal Replacement**:
   Draft the exact replacement block. Ensure all surrounding untouched code remains identical.
4. **Apply Surgical Edit**:
   Use `edit_file` to replace only the isolated target string.
5. **Verify Syntax & Regression**:
   Immediately run a targeted compiler/linter check or unit test to verify that the change compiled cleanly without side effects:
   - For Rust: `cargo check --lib` or `cargo test <test_name>`
   - For TypeScript/JavaScript: `npx tsc --noEmit` or `npm test -- <test_name>`
6. **Diff Review**:
   Check `git diff` to confirm that only the intended lines were touched.

---

## Prohibited Behaviors (Anti-Patterns)

- ❌ Rewriting a 400-line file from top to bottom to change one function.
- ❌ Stripping out existing comments, license headers, or TODOs.
- ❌ Re-arranging import statements or sorting keys across untouched sections.
- ❌ Renaming variables or functions outside the immediate scope of the change.
- ❌ Changing coding style (e.g., converting tabs to spaces, changing bracket style) outside the edited block.
