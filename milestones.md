# CmdDeck Implementation Milestones

- `[x]` **Milestone 1: Core CLI & Storage**
  - `[x]` Initialize Rust project
  - `[x]` Configure dependencies (`clap`, `serde`, `toml`, `dirs`)
  - `[x]` Define command data model
  - `[x]` Implement local TOML storage (read/write)
  - `[x]` Implement CLI parser setup (`cm <name>`, subcommands)
  - `[x]` Implement basic CLI direct execution

- `[x]` **Milestone 2: TUI Foundation**
  - `[x]` Add `ratatui` and `crossterm` dependencies
  - `[x]` Setup basic TUI event loop
  - `[x]` Build layout (list pane, details pane, footer)
  - `[x]` Implement keyboard navigation (and mouse support!)
  - `[x]` Handle graceful exit

- `[x]` **Milestone 3: Execution Integration (TUI)**
  - `[x]` Hook `Enter` key on selected command
  - `[x]` Implement TUI suspension (reset to canonical mode, clear screen)
  - `[x]` Execute interactive command
  - `[x]` Restore TUI (re-enter raw mode) after exit

- `[x]` **Milestone 4: Modals & CRUD**
  - `[x]` Create Add/New command modal
  - `[x]` Create Edit command modal
  - `[x]` Create Delete confirmation modal
  - `[x]` Persist changes from modals to storage

- `[x]` **Milestone 5: Polish & Search**
  - `[x]` Implement fuzzy search overlay (`skim` or `nucleo`)
  - `[x]` Filter by category and favorites
  - `[x]` Responsive two-pane / single-pane switching
  - `[x]` Empty state onboarding screen
