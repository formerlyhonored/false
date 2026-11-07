---
name: grill-me
description: >-
  Forces the agent to immediately halt before writing any code and conduct an
  interactive clarifying interview. Uses targeted architectural questions to resolve
  ambiguities, trade-offs, and edge cases before implementation begins.
  Use when receiving new complex feature requests, architecture refactors, or ambiguous prompts.
---

# Grill-Me Skill

Enforces an intentional pause before code implementation. Halts spontaneous coding to interview the user on core architectural decisions, non-obvious edge cases, and technical trade-offs.

---

## The Golden Rule: Zero Code Until Aligned

When this skill is activated:
1. **DO NOT write code** or modify files immediately.
2. **DO NOT generate speculative implementations**.
3. **DO ask targeted, high-signal questions** that uncover hidden assumptions, boundaries, and trade-offs.

---

## Interrogation Framework

Formulate 2 to 4 focused questions addressing key architectural dimensions:

### 1. Scope & Non-Goals
- What is explicitly *out of scope* for this change?
- Is this a temporary MVP/prototype or a permanent production architectural foundation?

### 2. Architecture & Data Flow
- Where should the state live (global store, local component, server/IPC, or database)?
- What is the data schema and how do data models cross system boundaries?
- What are the synchronization or caching strategies?

### 3. Error Handling & Edge Cases
- What happens when network/IPC disconnects or payloads fail validation?
- Should errors fail fast (panic/throw), silently recover with fallbacks, or display user-facing alerts?
- How should race conditions or concurrent mutations be resolved?

### 4. Technical Trade-Offs
Present explicit trade-offs with recommendations:
- *Option A (Simpler, synchronous, lower initial overhead)* vs *Option B (Scalable, asynchronous, higher complexity)*.

---

## Tool Usage & Interaction Flow

1. **Leverage `ask_question`**:
   Whenever possible, format questions using the `ask_question` tool with clear, multiple-choice options. Prefix your recommended choice with `(Recommended)`.
   ```json
   {
     "questions": [
       {
         "question": "How should state synchronization between the worker and main thread be handled?",
         "options": [
           "(Recommended) Transferable ArrayBuffers with zero-copy postMessage",
           "Structured cloning of JSON state objects",
           "SharedArrayBuffer with Atomics (requires cross-origin isolation headers)"
         ]
       }
     ]
   }
   ```
2. **If Free-Form Text Is Needed**:
   Present concise, bulleted prompts directly in chat and wait for user reply.
3. **Synthesis & Alignment Confirmation**:
   Once the user replies:
   - Summarize the decisions into a crisp bulleted specification.
   - Confirm readiness before touching any code.
