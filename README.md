# nm-remover 🗑️

A fast, interactive TUI tool to find and remove `node_modules` folders, written in Rust.

[![Build](https://github.com/supostat/node-modules-remover/actions/workflows/build.yml/badge.svg)](https://github.com/supostat/node-modules-remover/actions/workflows/build.yml)
[![Release](https://github.com/supostat/node-modules-remover/actions/workflows/release.yml/badge.svg)](https://github.com/supostat/node-modules-remover/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

![Demo](docs/demo.gif)

## Features

- 🔍 **Smart Scanning** - Finds only first-level `node_modules` (doesn't traverse into nested ones)
- 📊 **Size Display** - Shows folder sizes and last modified times
- ✅ **Multi-select** - Select multiple folders for batch deletion
- ⚡ **Fast** - Parallel directory scanning with Rayon
- 🎨 **Beautiful TUI** - Interactive terminal UI with Ratatui
- 🖥️ **Welcome Screen** - Run without arguments for an interactive path input
- 📈 **Progress Display** - Visual progress bar during deletion

## Installation

### From crates.io (Recommended)

```bash
cargo install nm-remover
```

### From GitHub Releases

Pre-built binaries for Linux, macOS, and Windows are available on the [Releases](https://github.com/supostat/node-modules-remover/releases) page.

### Build from Source

```bash
git clone https://github.com/supostat/node-modules-remover.git
cd node-modules-remover
cargo install --path .
```

## Usage

### Interactive Mode (Default)

```bash
# Launch with welcome screen (enter path interactively)
nm-remover

# Scan specific directory directly
nm-remover /path/to/projects

# Use ~ for home directory
nm-remover ~/Projects
```

## Workflow

### 1. Launch the App

Run `nm-remover` without arguments to see the welcome screen with ASCII logo, or provide a path directly.

```
  ███╗   ██╗███╗   ███╗      ██████╗ ███████╗███╗   ███╗ ██████╗ ██╗   ██╗███████╗██████╗
  ████╗  ██║████╗ ████║      ██╔══██╗██╔════╝████╗ ████║██╔═══██╗██║   ██║██╔════╝██╔══██╗
  ██╔██╗ ██║██╔████╔██║█████╗██████╔╝█████╗  ██╔████╔██║██║   ██║██║   ██║█████╗  ██████╔╝
  ██║╚██╗██║██║╚██╔╝██║╚════╝██╔══██╗██╔══╝  ██║╚██╔╝██║██║   ██║╚██╗ ██╔╝██╔══╝  ██╔══██╗
  ██║ ╚████║██║ ╚═╝ ██║      ██║  ██║███████╗██║ ╚═╝ ██║╚██████╔╝ ╚████╔╝ ███████╗██║  ██║
  ╚═╝  ╚═══╝╚═╝     ╚═╝      ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝ ╚═════╝   ╚═══╝  ╚══════╝╚═╝  ╚═╝
```

### 2. Enter Path or Scan

- If launched without arguments: Type the path to scan and press `Enter`
- Supports tilde expansion (`~/Projects` → `/Users/you/Projects`)

### 3. Browse Results

Navigate through the list of found `node_modules` folders:

```
Found 5 node_modules | Total: 1.18 GB | Selected: 0 (0 B)
┌────────────────────────────────────────────────────────────────┐
│ ► [ ] /Users/dev/projects/app1/node_modules [245 MB] (3d ago) │
│   [ ] /Users/dev/projects/app2/node_modules [189 MB] (1w ago) │
│   [ ] /Users/dev/projects/old-project/node_modules [512 MB]   │
│   [ ] /Users/dev/projects/api/node_modules [78 MB] (1d ago)   │
│   [ ] /Users/dev/projects/website/node_modules [156 MB]       │
└────────────────────────────────────────────────────────────────┘
```

### 4. Select Folders

- Press `Space` to toggle selection on current item
- Press `a` to select all
- Press `n` to deselect all

### 5. Delete Selected

- Press `d` to delete selected folders
- Confirm with `Y` or cancel with `N` in the warning dialog
- Watch the progress bar as folders are deleted

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Space` | Toggle selection |
| `a` | Select all |
| `n` | Deselect all |
| `d` | Delete selected |
| `?` | Show help |
| `q` / `Esc` | Quit |

### Non-interactive Mode

```bash
# List all node_modules (no TUI)
nm-remover --list /path/to/projects

# Delete all without confirmation (⚠️ dangerous!)
nm-remover --delete-all /path/to/projects
```

## Development

### Prerequisites

- Rust 1.70+

### Local Development

```bash
# Run in development
cargo run -- /path/to/scan

# Run without path (shows welcome screen)
cargo run

# Run tests
cargo test

# Run with hot reload (requires cargo-watch)
cargo watch -x run
```

### Building Release Binaries

```bash
# Build for current platform
cargo build --release

# Build static binary (Linux)
cargo build --release --target x86_64-unknown-linux-musl
```

## Project Structure

```
nm-remover/
├── src/
│   ├── main.rs      # Entry point, CLI parsing
│   ├── app.rs       # App state, selection, scan/delete states
│   ├── tui.rs       # Event loop, terminal management
│   ├── scanner.rs   # Directory scanning logic
│   └── ui/
│       ├── mod.rs   # Screen rendering (list, welcome)
│       ├── popups.rs # Popup widgets (help, confirm, progress)
│       └── input.rs # Keyboard input handling
├── Cargo.toml       # Dependencies
└── .github/
    └── workflows/   # CI/CD workflows
```

## How It Works

1. **Welcome Screen**: If no path provided, displays logo and path input
2. **Scanning**: Recursively walks the directory tree using parallel processing
3. **Smart Detection**: When a `node_modules` folder is found, it's added to the list and scanning stops at that level (avoids nested `node_modules`)
4. **Size Calculation**: Calculates total size of each `node_modules` folder
5. **Interactive Selection**: TUI allows you to navigate and select folders
6. **Confirmation**: Warning dialog with Yes/No buttons before deletion
7. **Progress Display**: Shows deletion progress with a visual progress bar
8. **Cleanup**: Removes selected folders and updates the list

## GitHub Actions Workflows

This project includes CI/CD workflows:

### Build Workflow (`.github/workflows/build.yml`)
- Runs on every push and pull request
- Tests on multiple platforms (Linux, macOS, Windows)
- Runs clippy linting and tests

### Release Workflow (`.github/workflows/release.yml`)
- Triggered on version tags (`v*`)
- Builds binaries for all platforms
- Creates GitHub release with artifacts
- Publishes to crates.io (stable releases only)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI library
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal library
- [WalkDir](https://github.com/BurntSushi/walkdir) - Directory traversal
- [Rayon](https://github.com/rayon-rs/rayon) - Parallel processing
