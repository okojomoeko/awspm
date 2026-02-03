# Development Workflow

## Phase 0: Preparation
1.  **Context**: Use `context7` or MCP tools for research.
2.  **Branching**: Create a focused topic branch or worktree.
    *   `git checkout -b feature/my-feature` or `git worktree add ...`

## Phase 1: Research & Plan
*   **Architecture**: Refer to `references/architecture.md`.
*   **Design**: Plan the Vertical Slice. Identify necessary traits for DI (e.g., `CommandExecutor`).

## Phase 2: Implementation (TDD)
1.  **Red**: Write a failing **Unit Test**.
    *   Mock external dependencies (FS, Command, Network).
    *   Ensure the test fails for the right reason.
2.  **Green**: Implement the minimal logic to pass the test.
3.  **Refactor**: Clean up.
    *   Remove duplication.
    *   Improve naming.
    *   **Verify**: Ensure tests still pass.

## Phase 3: Verification (Mandatory)
Before finishing any task, you **MUST** run the full verification suite:
*   **Command**: `.gemini/skills/rust-cli-expert/scripts/verify_all.sh`
    *   Runs `cargo fmt`
    *   Runs `cargo clippy -- -D warnings`
    *   Runs `cargo test`
    *   Runs `./e2e_test.sh`

## Phase 4: Commit
*   **Atomic Commits**: One logical change per commit.
*   **Message**: clear and descriptive (e.g., "Feat: Add exec logic with DI").
