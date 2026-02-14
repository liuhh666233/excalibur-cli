---
paths:
  - "excalibur/src/modules/settings/mod.rs"
  - "excalibur/src/modules/settings/state.rs"
  - "excalibur/src/modules/settings/ui.rs"
---

# Settings

Claude Code settings profile switcher. Lists `~/.claude/settings*.json` files, previews JSON content, and swaps the active `settings.json` with a selected profile.

## Key Files

| File | Role |
|------|------|
| `mod.rs` | `SettingsModule` — implements `Module`, routes SelectProfile/RenameCurrent input modes, executes file swap |
| `state.rs` | `SettingsState` — profile scanning, selection, preview content, rename input buffer |
| `ui.rs` | Horizontal split layout: profile list (left) + JSON preview (right), rename input bar, notification |

## Architecture / Data Flow

```
init() -> load_profiles() -> scan ~/.claude/settings*.json
  -> Vec<ProfileEntry> { name, path, is_active }
  -> update_preview() reads selected file content

Enter on non-active profile:
  -> InputMode::RenameCurrent (user types backup suffix)
  -> Enter confirms -> execute_swap():
      1. rename settings.json -> settings_{suffix}.json
      2. copy selected -> settings.json
      3. reload profiles
```

## Dependencies

- dirs 5.0 — resolve home directory
- serde_json 1.0 — pretty-print JSON preview
