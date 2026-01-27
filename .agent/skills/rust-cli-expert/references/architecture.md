# Vertical Slice Architecture (AVAP-Style)

## Core Principle
**"Features over Layers"**
The application is structured into independent **Vertical Slices** (`src/features/*`), where each slice contains everything needed to implement a specific CLI command (Action, Logic, View).

## Directory Structure
1.  **Features** (`src/features/<name>/*`): Self-contained slices.
    *   `mod.rs`: **Command/Action**. Entry point impl `clap::Parser`. Orchestrates flow.
    *   `logic.rs`: **Domain/UseCase**. Pure business logic, filtering, state manipulation.
    *   `view.rs`: **UI/Presentation**. Formatting, `tabled` integration, console output.
    *   *Rule: Features should be loosely coupled. One feature should rarely import another.*
    
2.  **Core** (`src/core/*`): **Shared Kernel**.
    *   Functionality required by *multiple* features.
    *   `config.rs`: Metadata store, file I/O.
    *   `sso.rs`: AWS SSO authentication utilities.
    *   `error.rs`: Centralized `AppError`.
    *   *Rule: Core MUST NOT import Features.*

## Dependency Flow
`Features (Slice)` -> `Core (Shared)` -> `External Crates`

## Coding Standards
*   **Public API**: Only `mod.rs` in features should be `pub` (exposed to `lib.rs` -> `main.rs`). `logic` and `view` are often private or `pub(crate)`.
*   **Error Handling**: All Actions return `Result<()>` using `anyhow` or `AppError`.