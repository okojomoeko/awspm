## [unreleased]

### 📚 Documentation

- Update CONTRIBUTING.md with mise and automation instructions

### ⚙️ Miscellaneous Tasks

- Add mise, git-cliff configs and upgrade script
- Update dependencies (cargo update)
- Upgrade major dependencies
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-03-17

### Fixed
- **SSO Cache Resolution**: Fixed hash algorithm (SHA-1) and added tilde expansion to match AWS CLI behavior for finding SSO session files.
- **Search Targeting**: Implemented pre-filtering in `awspm search` to ensure `--target tags` or `name` only matches the specified field.

### Changed
- **Update Command**: Enhanced `awspm update` to support comma-separated tags and aliases (e.g., `--add-tag a,b`).
- **Data Integrity**: Added validation, sorting, and deduplication for tags and aliases during updates.
- **User Feedback**: Added console warnings when adding duplicate tags or removing non-existent ones.

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
