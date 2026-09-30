# Agent Operational Guidelines

## 1. Operational Principles & Tooling

- Operate as a rigorous technical co-pilot: no speculative code, no unsolicited refactoring, and no conversational filler in codebase files.
- **NEVER invoke raw `git commit`. `git push` or `git tag` directly.** All commits, linting, and releases must be executed exclusively through `cccp` (via `cccp commit`, `cccp lint`, `cccp push` and `cccp tag`).
- **Language Policy:**
  - Code, commit messages, docstrings, technical documentation, and task summaries must be written strictly in **portuguese**.
  - Direct communication with the user in conversation follows Brazilian Portuguese.

---

## 2. Ticket Ingestion Policy (`.antigravity/tickets.md`)

The local file `.antigravity/tickets.md` serves as the authoritative source of truth for pending tasks, bug reports, and roadmap criteria.

1. **Size Boundary (10KB Ceiling):**
   - Read at most the first **10 KB** of `.antigravity/tickets.md` (approximately 10 to 15 full tickets).
   - **No Truncation Rule:** Never process an issue cut mid-sentence at the 10KB cut-off. If a ticket is truncated by the boundary, stop reading at the preceding `---` delimiter and discard the incomplete ticket.
2. **Issue Binding:**
   - Every implementation must map to an explicit issue identifier (`#<id>`). The identifier MUST be on commit text.
   - If requirements or acceptance criteria in the ticket are ambiguous, halt execution immediately and request clarification rather than inferring scope.
3. **Closed Scope:**
   - Implement only the direct requirements defined in the active ticket. Do not preempt future roadmap phases.

---

## 3. Atomic Commits & Real-Time Documentation Parity

Every unit of work must be broken into discrete, atomic commits. Whenever an implementation modifies software behavior, public interfaces, configuration options, or architectural flows, **code and documentation must be updated in immediate, sequential parity**.

### The Two-Step Paired Commit Cycle

1. **Step 1 — Implementation Commit:**
   - Stage exclusively the source code, tests, or configurations resolving the ticket.
   - Execute the commit via `cccp`:

     ```bash
     git add src/ tests/
     cccp commit "<type>(<scope>): <imperative summary> (#<id>)"
     ```

   - If this commit fixes or finalizes the ticket, it MUST finish with one of the keywords [close, closes, closed, fix, fixes, fixed, resolve, resolves, resolved] followed by the #<id> of the ticket closed.
2. **Step 2 — Immediate Documentation Parity Commit:**
   - Must immediately follow the code commit before addressing any other task or issue.
   - Stage exclusively the documentation files (`README.md`, docstrings, specifications, manpages).
   - Execute the documentation commit via `cccp`:

     ```bash
     git add README.md docs/
     cccp commit "docs(<scope>): update specifications for <feature/fix> (#<id>)"
     ```

> **Strict Isolation Rule:** Never combine code modifications and documentation updates within the same commit. They must remain separate, atomic, and consecutive.

---

## 4. `cccp` Command Reference & Conventions

### Commit Format

```text
<type>(<scope>): <subject> (#<id>)
```

*For breaking changes, append `!` before the colon (e.g., `feat(api)!: drop legacy endpoint (#12)`).*

### Permitted Commit Types

- `feat`: New feature or user-facing functionality (triggers SemVer minor bump).
- `fix`: Bug fix (triggers SemVer patch bump).
- `perf`: Performance improvement (triggers SemVer patch bump).
- `refactor`: Internal restructuring without altering observable behavior (triggers SemVer patch bump).
- `docs`: Documentation additions or revisions only.
- `test`: Adding or correcting tests.
- `chore`: Tooling, maintenance, dependency, or repository configuration tasks.
- `build`: Build system or packaging changes.
- `ci`: Continuous integration pipeline configurations.

### Diagnostic & Release Commands

- **Inspect Working Tree, Hooks & Target Version:**

  ```bash
  cccp status
  ```

- **Validate Commit History Compliance:**

  ```bash
  cccp lint
  ```
    
- **Push Commit History to online repo:**

  ```bash
  cccp push
  ```

- **Tag a Release (only on explicit user request):**

  ```bash
  cccp tag -t "<Release Title>"
  ```

---

## 5. Standard Execution Sequence

For every assigned ticket:

1. **Contextualize:** Extract the target ticket and requirements from `.antigravity/tickets.md`.
2. **Implement:** Write minimal, focused code to satisfy the issue.
3. **Verify:** Run local test suites and linters.
4. **Commit Code:** Stage code files and run `cccp commit "<type>(<scope>): <msg> (#<id>)"`. Make atomic commits, with every step in the process. Do not make cumulative commits.
5. **Update Docs:** Synchronize documentation to reflect all behavioral changes.
6. **Commit Docs:** Stage doc files and run `cccp commit "docs(<scope>): <msg> (#<id>)"`.
