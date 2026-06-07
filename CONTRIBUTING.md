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

## Release Process

When acting as a maintainer, here is how and when to cut a new release.

### When to cut a release (Tagging Policy)

We follow [Semantic Versioning](https://semver.org/). You should consider cutting a new release tag when:
- **Major (`v1.0.0`)**: Breaking changes are introduced to the CLI interface or configuration format.
- **Minor (`v0.2.0`)**: New backward-compatible features are added (e.g., a new command or flag).
- **Patch (`v0.1.2`)**: Bug fixes or minor internal refactoring are merged.

As a general cadence for solo or small-team development:
- Accumulate a few non-critical bug fixes before cutting a patch release.
- Cut a minor release immediately if a highly anticipated feature is merged to `main`.
- Always ensure `CHANGELOG.md` is updated (via `git-cliff` or `./scripts/upgrade_deps.sh`) and CI passes on `main` before tagging.

### Releasing and Publishing

1. **Tag the release**:
   Run the automated release script to bump the version in `Cargo.toml`, update `CHANGELOG.md`, commit, and push the tag to the `main` branch.
   ```bash
   ./scripts/release.sh 0.1.2
   ```

2. **GitHub Actions**:
   Pushing a `v*` tag automatically triggers the `release.yml` workflow. Wait for the workflow to finish building the binaries and creating the GitHub Release.

3. **Update Homebrew Tap (Semi-automated)**:
   After the GitHub Release is successfully published and assets are uploaded, you must update the Homebrew formula in the `homebrew-awspm` repository. Use the provided script:
   ```bash
   ./scripts/update_brew.sh 0.1.2 ~/work/homebrew-awspm
   ```
   This script will safely verify the local tap directory, fetch the latest SHA256 checksums from the GitHub Release, and automatically rewrite the formula. Finally, follow the script's instructions to commit and push the updated tap repository.
