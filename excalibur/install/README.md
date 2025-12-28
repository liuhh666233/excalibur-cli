# Fish Shell Integration

This directory contains the Fish shell integration for Excalibur.

## Installation

### Step 1: Build and Install Excalibur

```bash
cd ..
cargo build --release
cargo install --path .
```

### Step 2: Install Fish Function

```bash
# Copy the function file
cp exh.fish ~/.config/fish/functions/

# Reload Fish configuration
source ~/.config/fish/config.fish
```

The function is automatically named `exh` (Excalibur History) and bound to `Ctrl+R`.

## Usage

### Method 1: Command

```fish
exh
```

### Method 2: Keybinding

Press `Ctrl+R` (automatically bound) to launch Excalibur.

### Method 3: Direct Binary

```bash
# Run standalone (without auto-insert)
excalibur
```

### In Excalibur

1. **Navigate**: Use `↑/↓` or `j/k`
2. **Search**: Press `/` and type to filter
3. **Sort**: Press `s` to cycle sort modes
4. **Select**: Press `Enter` to insert command into shell (you can edit it)
5. **Execute**: Press `Ctrl+O` to insert and execute immediately
6. **Cancel**: Press `Esc` or `q` to exit

## How It Works

```
User presses Ctrl+R
    ↓
Fish calls excalibur function
    ↓
Rust program launches TUI
    ↓
User selects command and presses:
  - Enter: Command is output with exit code 0
  - Ctrl+O: Command is output with exit code 10
    ↓
Fish function captures output and exit code
    ↓
If exit code 0:
  Command is inserted into command line via commandline -r
  User can edit and execute
    ↓
If exit code 10:
  Command is inserted and automatically executed via commandline -f execute
```

## Uninstallation

```bash
rm ~/.config/fish/functions/excalibur.fish
# Remove keybinding from config.fish if added
```

## Troubleshooting

### Command not found: excalibur

Make sure the binary is in your PATH:

```bash
which excalibur
# If not found, reinstall:
cargo install --path /path/to/excalibur-cli/excalibur
```

### Function doesn't work

1. Check if function is loaded:
   ```fish
   functions excalibur
   ```

2. Reload Fish configuration:
   ```fish
   source ~/.config/fish/config.fish
   ```

3. Check if function file exists:
   ```bash
   ls ~/.config/fish/functions/excalibur.fish
   ```
