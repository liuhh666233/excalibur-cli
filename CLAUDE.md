# Excalibur CLI

A unified TUI command-line interface built in Rust with ratatui, integrating multiple tools via a module system.

## Build & Run

```bash
cd excalibur
cargo build --release
cargo run                    # main menu
cargo run -- history         # direct module entry
cargo run -- process-tracer  # direct module entry
cargo run -- settings        # direct module entry
```

## Architecture

```
excalibur/src/
├── main.rs              # CLI entry (clap), terminal setup
├── app.rs               # Event loop, key dispatch, ModuleAction handling
├── event.rs             # Background event thread (Tick/Crossterm/AppEvent)
├── view.rs              # View enum (MainMenu / Module)
├── ui.rs                # Main menu rendering
└── modules/
    ├── mod.rs           # Module trait, ModuleId, ModuleAction
    ├── manager.rs       # ModuleManager (registry, routing)
    ├── history/         # Fish shell history browser
    ├── proctrace/       # Process tracer/analyzer
    └── settings/        # Claude Code settings switcher
```

## Modules

| Module | Rules file | Description |
|--------|-----------|-------------|
| core | `.claude/rules/core.md` | App framework: event loop, module system, main menu |
| history | `.claude/rules/history.md` | Fish shell history browser with search, sort, clipboard |
| proctrace | `.claude/rules/proctrace.md` | Query-driven process inspector (name/PID/port) |
| settings | `.claude/rules/settings.md` | Claude Code settings profile switcher |

## Adding a New Module

1. Add variant to `ModuleId` in `excalibur/src/modules/mod.rs`
2. Create `excalibur/src/modules/<name>/` with `mod.rs` implementing the `Module` trait
3. Register in `ModuleManager::new()` in `excalibur/src/modules/manager.rs`
4. Add CLI subcommand in `main.rs` `Commands` enum
5. Create `.claude/rules/<module>.md` with `paths:` frontmatter listing all related source files
6. Add an entry to the Modules table above
