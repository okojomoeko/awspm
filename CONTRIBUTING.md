# Contributing to `awspm`

Thank you for your interest in contributing to AWS Profile Manager (`awspm`)!
We aim for a high standard of code quality ("shame-free release"), so please follow these guidelines.

## Development Setup

1.  **Prerequisites**:
    Ensure you have the latest stable Rust toolchain installed:
    ```bash
    rustup update stable
    ```
    We strongly recommend installing [mise](https://mise.jdx.dev/) to manage project-specific tools (`git-cliff`, `cargo-deny`, etc.). Once `mise` is installed, simply run:
    ```bash
    mise install
    ```

2.  **Clone & Build**:
    ```bash
    git clone https://github.com/okojomoeko/awspm.git
    cd awspm
    cargo build
    # Install git hooks
    ./scripts/setup_hooks.sh
    ```


## Testing & Quality Assurance

Before submitting a Pull Request, please ensure all checks pass. We enforce strict linting to maintain a clean codebase.

### 1. Formatting
We use `rustfmt` to ensure consistent code style.
```bash
cargo fmt --all -- --check
```

### 2. Linting (Clippy)
We treat all warnings as errors.
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### 3. Tests
Run the unit and integration tests.
```bash
cargo test
```

### 4. Dependency Updates & Automation
When updating dependencies, please use our automated script which runs tests, linters, license generation, and security audits in one go:
```bash
./scripts/upgrade_deps.sh --commit
```
This script ensures `Cargo.lock` is updated securely, tests pass, and `CHANGELOG.md` is automatically updated via `git-cliff`.

## Project Structure

- `src/main.rs`: Entry point and CLI command dispatch.
- `src/features/`: Vertical slices required for each feature.
- `src/core/`: Shared kernel logic (config, error, sso, policy).
- `tests/`: Integration tests.

## Contribution Flow

1.  Fork the repository.
2.  Create a feature branch (`git checkout -b feature/my-amazing-feature`).
3.  Commit your changes.
4.  Run checks (`fmt`, `clippy`, `test`).
5.  Push to the branch.
6.  Open a Pull Request.

Happy coding!
