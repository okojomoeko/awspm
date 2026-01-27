# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-01-30

### Added
- Initial release of `awspm` (AWS Profile Manager).
- Interactive profile search and fuzzy selection (`awspm search`, `awspm select`).
- Intelligent profile switching with shell integration (`awspm switch`).
- Secure command execution with environment isolation (`awspm exec`).
- Metadata management (tags, aliases, notes) without modifying `~/.aws/config`.
- Region override support in profiles.
- Cross-platform support (Linux, macOS, Windows).
- GitHub Actions CI/CD pipeline.
