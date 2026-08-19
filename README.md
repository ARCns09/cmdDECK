# cmdDECK

A lightweight, fast, terminal-based command manager and launcher for Linux.

CmdDeck (`cm`) allows you to save long or frequently used shell commands under short, memorable names, browse them through a polished Terminal User Interface (TUI), and launch them instantly without needing to remember complex syntax.

## Features
- **TUI & CLI Mode**: Launch the beautiful TUI or execute directly via CLI (`cm <alias>`).
- **Interactive Execution**: Safely suspends the TUI, executes your command (even interactive apps like SSH or Vim), and perfectly restores the TUI when done.
- **Fuzzy Search**: Instantly filter your commands by name, description, or category.
- **Favorites & Categories**: Keep your most-used commands at the top.
- **CRUD Operations**: Add, Edit, and Delete commands completely from within the TUI.

## Installation
Ensure you have Rust installed, then clone and build:
```bash
cargo build --release
```
The binary will be located in `target/release/cm`.

## Configuration
Commands are safely stored in human-readable TOML format at:
`~/.config/cmddeck/commands.toml`
