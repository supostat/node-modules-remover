# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com),
and this project adheres to [Semantic Versioning](https://semver.org).

## [1.1.0] - 2026-03-17

### Added
- Sort scan results by size (largest first)
- Publish to crates.io on release

### Changed
- Modular architecture: split monolithic ui.rs into app, tui, ui/ modules

### Removed
- Docker support (Dockerfile, docker-compose.yml)
- install.sh script (use `cargo install dev-clean` instead)

## [1.0.0] - Initial release

### Added
- Interactive TUI with welcome screen and path input
- Smart scanning: finds first-level node_modules only
- Parallel directory scanning with Rayon
- Multi-select with batch deletion
- Progress display during scanning and deletion
- CLI modes: --list, --delete-all
- Cross-platform support (Linux, macOS, Windows)
- GitHub Actions CI/CD with multi-platform builds
