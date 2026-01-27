# Development Workflow

## Phase 1: Research
*   **Context**: Use `context7` for crate research.
*   **Architecture**: Refer to `references/architecture.md` (Vertical Slices).

## Phase 2: Implementation (TDD)
1.  **Red**: Write failing unit test.
2.  **Green**: Implement logic in `src/features/<feature>/logic.rs`.
3.  **Refactor**: Clean up and optimize.

## Phase 3: Verification (Mandatory)
Before finishing any task, you **MUST** run the full verification suite:
*   **Command**: `cargo clippy --all-targets --all-features -- -D warnings`
*   **Command**: `cargo test`
*   **Command**: `./e2e_test.sh` (End-to-End Shell Tests)

*Tip: Use the skill script `scripts/verify_all.sh` to run all of these at once.*

## Phase 4: Release
*   **Tagging**: Push `v0.1.0` tag to trigger GitHub release workflow.
*   **Publish**: (Disabled in Cargo.toml for safety).