---
name: agent-handoff
description: >-
  Summarizes active working memory, key decisions made, pending tasks, and recent
  file modifications into a persistent handoff file (.agents/HANDOFF.md).
  Use when ending a session, switching tasks, passing context to another agent,
  or before context compaction.
---

# Agent Handoff Skill

Captures transient working memory, architectural decisions, and current execution state into a persistent state file (`.agents/HANDOFF.md`). Enables instant context resumption by future agent turns or different agents without repeating discovery or losing progress.

---

## The Handoff File Location

All handoff states are saved to:
```text
.agents/HANDOFF.md
```

If the `.agents/` folder does not exist, create it automatically.

---

## Standardized Handoff Schema

The `.agents/HANDOFF.md` file must strictly adhere to this template:

```markdown
# Agent Handoff State

**Timestamp**: YYYY-MM-DD HH:MM:SS (Timezone)  
**Session Goal**: <One-sentence summary of the main goal tackled in this session>

---

## 1. Executive Summary & Current Status
- **Overall Status**: [In Progress | Completed | Blocked on External Input]
- Brief paragraph summarizing what was achieved and the current state of the codebase.

---

## 2. Working Memory & Architectural Decisions
- **Decisions Made**:
  - *Decision 1*: Chose X over Y because of Z.
  - *Decision 2*: Kept module A synchronous to avoid cross-thread mutex contention.
- **Discovered Constraints / Gotchas**:
  - E.g., Module B requires specific initialization order before rendering starts.
- **Rejected Approaches**:
  - E.g., Tried dynamic buffer re-allocation, but it caused frame stutter.

---

## 3. Modified & Created Files
- `path/to/file1.ext`: Added feature X; updated struct Y.
- `path/to/file2.ext`: Fixed edge-case bug in calculation Z.
- `path/to/file.test.ext`: Added unit tests covering edge cases.

---

## 4. Build, Test & Lint Health
- **Build Status**: Passing / Failing / Not Run
- **Test Results**: All passing (e.g., 42 tests passed, 0 failures)
- **Known Regressions or Outstanding Bugs**: None / List if any exist

---

## 5. Immediate Next Steps (Prioritized)
1. [ ] **Step 1**: <Exact next action with target file and goal>
2. [ ] **Step 2**: <Subsequent action>
3. [ ] **Step 3**: <Subsequent action>

---

## 6. Prompt for Resuming Agent
Copy and paste this prompt to resume work immediately:
> "Resume work from `.agents/HANDOFF.md`. The primary goal is <Goal>. Proceed immediately to Step 1: <Next Action>."
```

---

## Execution Workflow

When requested to create or update an agent handoff:

1. **Inspect Working Tree & History**:
   - Run `git status -s` and `git diff --stat` to get an accurate list of modified and untracked files.
   - Run `git log -n 3 --oneline` to inspect the latest commits.
2. **Review Health**:
   - Run a rapid sanity build/test check (e.g., `cargo check` or `npm test`) to record the exact build status.
3. **Synthesize Memory**:
   - Extract key decisions and insights that are not obvious from the raw code diff.
4. **Write `.agents/HANDOFF.md`**:
   - Use `write_file` (with `Overwrite: true`) to write the updated handoff document.
5. **Acknowledge**:
   - Present a concise 3-line summary to the user with a link to [`.agents/HANDOFF.md`](file:///d:/false/.agents/HANDOFF.md).
