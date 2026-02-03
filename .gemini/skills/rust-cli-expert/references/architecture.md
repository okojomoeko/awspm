# Vertical Slice Architecture (AVAP-Style)

## Core Principle
**"Features over Layers"**
The application is structured into independent **Vertical Slices** (`src/features/*`), where each slice contains everything needed to implement a specific CLI command.

## Directory Structure
1.  **Features** (`src/features/<name>/*`): Self-contained slices.
    *   `mod.rs`: **Command/Action**. Entry point impl `clap::Parser`. Orchestrates flow and Dependency Injection.
    *   `logic.rs`: **Domain/UseCase**. Pure business logic.
        *   **MUST NOT** import other features.
        *   **MUST** use Traits for side-effects (Process, File, Network).
    *   `view.rs`: **UI/Presentation**. Console output.
    
2.  **Core** (`src/core/*`): **Shared Kernel**.
    *   Functionality required by *multiple* features.
    *   `config.rs`: Metadata store, file I/O.
    *   `sso.rs`: AWS SSO authentication utilities.
    *   `error.rs`: Centralized `AppError`.
    *   *Rule: Core MUST NOT import Features.*

## Dependency Flow
`Features (Slice)` -> `Core (Shared)` -> `External Crates`
**Feature A** -/-> **Feature B** (Forbidden direct dependency)

## Dependency Injection (DI)
*   **Traits**: Define traits for external interactions in `logic.rs` (e.g., `CommandExecutor`).
*   **Injection**: Inject implementations from `mod.rs`.
*   **Testing**: Use Mocks in unit tests to verify logic without side effects.

## Coding Standards
*   **Public API**: Only `mod.rs` in features should be `pub`.
*   **Error Handling**: All Actions return `Result<()>` using `anyhow` or `AppError`.
*   **Safety**: No `unsafe`. No `unwrap()`.
