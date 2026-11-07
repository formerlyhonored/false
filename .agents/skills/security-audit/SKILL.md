---
name: security-audit
description: >-
  Scans code diffs, source files, and dependencies for hardcoded credentials,
  unvalidated inputs, unsafe system command execution, weak IPC endpoints, and
  memory safety violations. Use before committing changes, integrating external data,
  or writing sensitive network, IPC, or systems code.
---

# Security Audit Skill

Provides comprehensive vulnerability scanning guidelines and heuristics for code diffs and newly authored modules. Ensures applications remain resilient against injection attacks, secret exposure, insecure execution, and IPC vulnerabilities.

---

## Vulnerability Scan Matrix

When reviewing or writing code, audit against the following 5 vulnerability vectors:

### 1. Hardcoded Credentials & Secrets
- **Patterns**:
  - API keys, OAuth client secrets, JWT secrets, RSA/Ed25519 private keys.
  - Hardcoded tokens in test files or configuration fixtures.
  - High-entropy strings assigned to variables named `key`, `secret`, `token`, `password`, `auth`.
- **Heuristics & Checks**:
  - Are secrets pulled strictly from environment variables or secure credential storage (e.g., OS keychain, secrets manager)?
  - Are sensitive values excluded from logs, error messages, and telemetry payloads?

### 2. Input Validation & Sanitization
- **Patterns**:
  - Path Traversal: Constructing file paths directly from user input without canonicalization or directory escaping checks (`../`, `..\\`).
  - Injection Attacks: Direct string concatenation into SQL queries, template engines, HTML markup (XSS), or regex compilers (ReDoS).
  - Unbounded Memory/Payload Allocation: Reading streams into memory without size limits (causing Denial of Service / OOM).
- **Remediation**:
  - Use parameterized queries, strongly-typed domain objects, and safe path joining with normalization verification (`canonicalize()`).
  - Enforce explicit payload size caps before buffer allocation.

### 3. Unsafe System Execution Calls
- **Patterns**:
  - Calling shell environments directly:
    - Node: `child_process.exec()`, `execSync()` without argument arrays.
    - Rust: `std::process::Command::new("sh").arg("-c")` or `Command::new("cmd.exe").arg("/C")` passing concatenated user strings.
    - Python: `os.system()`, `subprocess.Popen(..., shell=True)`.
- **Remediation**:
  - Always execute binaries directly by absolute path or sanitized executable name with distinct, un-concatenated argument lists.
  - Never allow user input to specify the binary name or control shell interpolation flags.

### 4. Weak IPC Endpoints & Protocol Boundaries
- **Patterns**:
  - Local IPC: Named pipes, Unix domain sockets, or local TCP ports created without access control lists (ACLs) or permission masks.
  - WebSockets / IPC Messages: Deserializing untrusted JSON or binary payloads without schema validation or type checks.
  - Missing Origin / Authentication: Allowing arbitrary processes or web origins to send commands over IPC channels without handshake tokens or validation.
- **Remediation**:
  - Restrict IPC permissions to current user (`chmod 0600` or Windows Security Descriptors).
  - Validate and authenticate every IPC packet with strict schema validators (e.g., `serde` with exhaustive enum matching).

### 5. Memory Safety & Unsafe Blocks (Rust / Native)
- **Patterns**:
  - Unchecked `unsafe { ... }` blocks in Rust.
  - Raw pointer dereferencing without null / alignment checks.
  - Exposing mutable references across FFI boundaries without lifetime guarantees.
  - Calling C libraries without checking buffer lengths or null termination.
- **Remediation**:
  - Ensure every `unsafe` block has an explicit `// SAFETY: ...` comment explaining why invariants are upheld.
  - Minimize the scope of `unsafe` blocks to the exact operation requiring it.

---

## Step-by-Step Security Audit Workflow

1. **Diff Extraction**:
   Review all staged or modified code:
   ```powershell
   git diff HEAD
   ```
2. **Automated Keyword Scan**:
   Search for critical keywords in modified files:
   - `secret`, `password`, `token`, `api_key`
   - `exec`, `shell`, `spawn`, `system`
   - `unsafe`, `transmute`, `from_raw_parts`
   - `eval`, `innerHTML`, `dangerouslySetInnerHTML`
3. **Boundary Verification**:
   Inspect every point where data crosses a trust boundary (network input, CLI args, file reads, IPC messages).
4. **Triage Severity**:
   - **Critical**: Secret leaked, remote code execution, unauthenticated root/admin IPC endpoint. $\rightarrow$ *Must be blocked immediately.*
   - **High**: Path traversal, unchecked memory bounds, ReDoS. $\rightarrow$ *Must be patched before commit.*
   - **Medium**: Permissive CORS, missing payload size limits. $\rightarrow$ *Flag and address.*
   - **Low / Informational**: Redundant sanitization, defensive programming suggestions.
