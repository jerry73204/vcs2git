# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Tracing-based logging system for better debugging and monitoring
- Native git2 API operations for submodule removal (no more shell commands)
- Early return optimization when no operations need to be performed
- Improved error context and messages throughout the codebase

### Changed
- Replaced all `println!` and `eprintln!` macros with structured tracing logs
- Progress reporting is now automatic (removed `--progress` flag)
- Migrated `remove_submodule()` and `remove_submodule_rollback()` to use native git2 API
- Improved `.gitmodules` file handling with proper index updates
- Better handling of partially initialized submodules during rollback

### Removed
- `--progress` flag (progress bars are now shown automatically when needed)
- Dependency on external git commands for submodule removal

### Fixed
- Fixed `--sync-selection` hanging issue when removing submodules
- Improved test reliability by removing dependency on progress bar output capture
- Better cleanup of `.git/modules` directory during submodule removal

## [0.4.0] - 2025-01-13

### Added
- New `--only` flag to process only specific repositories (replaces `--select`)
- New `--ignore` flag to exclude specific repositories (replaces `--skip`)
- New `--skip-existing` flag to skip updating existing submodules
- New `--sync-selection` flag to remove submodules not in current selection
- New `--dry-run` flag to preview operations without making changes
- New `--progress` flag to show progress bars during operations
- Progress reporting with indicatif for better user feedback
- Comprehensive rollback mechanism that restores original state on failure
- Support for removing submodules to keep repository in sync
- Extensive test suite for new features
- CI/CD workflows for GitHub Actions

### Changed
- **BREAKING**: Updates are now done by default (previously required `--update` flag)
- **BREAKING**: Renamed `--select` to `--only` for clarity
- **BREAKING**: Renamed `--skip` to `--ignore` for clarity
- Improved error messages and user feedback
- Better handling of partial failures with atomic operations

### Deprecated
- `--select` flag (use `--only` instead)
- `--skip` flag (use `--ignore` instead)
- `--update` flag (updates are now default behavior)

### Fixed
- Proper rollback when operations fail midway
- Better validation of repository state before operations
- Correct handling of submodule removal

## [0.3.1] - Previous

### Fixed
- Fixed inverted logic in disjoint check
- Fixed error messages
- Added submodule state validation
- Added input validation for duplicates

## [0.3.0] - Initial stable release

### Added
- Basic VCS repos to Git submodules conversion
- Support for `--select` and `--skip` options
- SSH agent authentication
- Version/branch/tag checkout support