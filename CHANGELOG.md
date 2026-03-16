# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.4.3] - 2026-03-16

### Added

- `--version` flag

## [0.4.2] - 2026-03-16

### Fixed

- Homebrew formula auto-update now correctly sets sha256 for all platforms

## [0.4.1] - 2026-03-16

### Changed

- Refactored internal state management for better maintainability

### Added

- CI workflow (fmt, clippy, test)
- CHANGELOG, CONTRIBUTING guide, issue/PR templates
- GitHub Releases with pre-built binaries (macOS, Linux)
- Homebrew tap support (`brew install tonegawa07/tap/ndir`)

## [0.4.0] - 2026-03-11

### Added

- `Ctrl+F` toggle to show files for path copying

## [0.3.0] - 2026-03-05

### Added

- `Y` key to copy selected path to clipboard

## [0.2.0] - 2026-03-04

### Added

- `--init` flag for shell setup (`eval "$(ndir --init)"`)

## [0.1.0] - 2026-03-04

### Added

- Inline directory navigation with arrow keys
- Fuzzy filtering by typing
- `Ctrl+H` to toggle hidden files
- Zsh integration
