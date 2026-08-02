
# edbookmark

A keyboard-centric TUI (Terminal User Interface) bookmark manager built with Rust and Ratatui. Designed for Arch Linux + Hyprland (Wayland) with Omarchy.

> 🇮🇩 [Baca dalam Bahasa Indonesia](README.id.md)

## Features

- **Vim-like keyboard navigation** — `j/k` to navigate, `/` to search, `a` to add, `e` to edit, `d` to delete
- **Fuzzy multi-word search** — type `pano rama` to find bookmarks containing both words
- **Cursor-aware text editing** — move cursor with `← →`, `Home/End`, `Ctrl+A/E` in forms and search bar
- **Import bookmarks** — from Chromium (JSON), Firefox (HTML export), and XLSX spreadsheet
- **Export bookmarks** — to JSON, HTML, and XLSX format (compatible with Chrome/Firefox/Excel)
- **Frameless Chromium webapp** — bookmarks open as frameless Chromium windows via `systemd-run`
- **Customizable Ayu theme** — all colors can be changed through config file
- **Transparent background** — follows your terminal background
- **Colorful borders** — each border side can have a different color
- **Walker integration** — can be launched from Walker as a floating window
- **Clipboard support** — copy URL to clipboard with `y` (using `wl-copy`)
- **Multi-select** — select multiple bookmarks with `Space`, bulk delete with `D`

## Screenshot

> *Coming soon*

## System Requirements

### Tested On

| Component | Version |
|---|---|
| OS | Arch Linux (Omarchy) |
| Window Manager | Hyprland |
| Display Protocol | Wayland |
| Terminal | Kitty |
| Shell | Bash |
| Browser | Chromium |

### Required Dependencies

| Package | Purpose |
|---|---|
| `rust` | Compiler and Cargo (build from source) |
| `kitty` | Terminal emulator to run the TUI |
| `chromium` | Browser to open bookmarks |
| `wl-clipboard` | Copy URL to clipboard (`wl-copy`) |
| `xdg-utils` | Detect default browser (`xdg-settings`) |
| `util-linux` | Detach process (`setsid`) |
| `systemd` | Run browser as separate service (`systemd-run`) |

### Optional Dependencies

| Package | Purpose |
|---|---|
| `uwsm-app` | Alternative launcher for Wayland session |
| `walker` | Application launcher for desktop integration |
| `hyprland` | Auto-reload window rules (`hyprctl`) |

## Installation

### Clone and Install

```bash
git clone git@github.com:CuedNub/edBookmark.git
cd edBookmark
bash install.sh
```

The installer will:
1. Check all system requirements
2. Offer automatic installation of missing packages (via `pacman`)
3. Build an optimized release binary
4. Ask permission before writing each file
5. Configure Hyprland window rules
6. Verify the installation

### File Locations After Install

| File | Location |
|---|---|
| Binary | `~/.local/bin/edbookmark` |
| Walker wrapper | `~/.local/bin/edbookmark-walker` |
| Configuration | `~/.config/edbookmark/config.toml` |
| Bookmark data | `~/.local/share/edbookmark/bookmarks.json` |
| Desktop entry | `~/.local/share/applications/edbookmark.desktop` |
| Launcher log | `~/.local/state/edbookmark/launcher.log` |

## Uninstall

```bash
cd edBookmark
bash uninstall-edbookmark.sh
```

The uninstaller will:
1. Detect all installed components
2. Ask permission before removing each item
3. Show a final confirmation before execution
4. Create backups of important files before removal

### Reinstall

If something goes wrong, simply uninstall and reinstall:

```bash
bash uninstall-edbookmark.sh
bash install.sh
```

## Usage

### Opening the Application

```bash
# From terminal
edbookmark

# From Walker (application launcher)
# Type "edbookmark" → Enter
```

### Workflow

```
Walker → "edbookmark" → Enter
    ↓
Kitty opens (floating 800x500, centered)
    ↓
Select bookmark with j/k or /search
    ↓
Enter → edbookmark closes + frameless Chromium opens
```

### Import and Export

```bash
# Import from Chromium
edbookmark --import chromium

# Import from HTML file (Firefox export)
edbookmark --import-file bookmarks.html

# Export to JSON
edbookmark --export json -o backup.json

# Export to HTML (importable to Chrome/Firefox)
edbookmark --export html -o bookmarks.html

# Export to XLSX (Excel spreadsheet)
edbookmark --export xlsx -o bookmarks.xlsx

# Import from XLSX file
edbookmark --import-file bookmarks.xlsx
```

## Keybindings

### Normal Mode

| Key | Action |
|---|---|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `g` | Go to top |
| `G` | Go to bottom |
| `Enter` | Open bookmark in Chromium (frameless) |
| `/` | Enter search mode |
| `a` | Add new bookmark |
| `e` | Edit selected bookmark |
| `d` | Delete selected bookmark |
| `Space` | Toggle select (multi-select) |
| `D` | Bulk delete selected |
| `y` | Yank (copy) URL to clipboard |
| `?` | Show keybinding help |
| `q` / `Esc` | Quit |

### Search Mode

| Key | Action |
|---|---|
| *Any character* | Type search query |
| `Enter` | Confirm search |
| `Esc` | Cancel search |
| `↓` / `Ctrl+N` | Navigate results down |
| `↑` / `Ctrl+P` | Navigate results up |
| `←` / `Ctrl+B` | Move cursor left |
| `→` / `Ctrl+F` | Move cursor right |
| `Home` / `Ctrl+A` | Move cursor to start |
| `End` / `Ctrl+E` | Move cursor to end |
| `Delete` | Delete character at cursor |
| `Backspace` | Delete character before cursor |
| `Ctrl+W` | Delete word |
| `Ctrl+U` | Clear entire input |

### Form Mode (Add / Edit)

| Key | Action |
|---|---|
| *Any character* | Type in active field |
| `Tab` | Move to next field |
| `Shift+Tab` | Move to previous field |
| `Ctrl+S` | Save |
| `Esc` | Cancel |
| `←` / `Ctrl+B` | Move cursor left |
| `→` / `Ctrl+F` | Move cursor right |
| `Home` / `Ctrl+A` | Move cursor to start |
| `End` / `Ctrl+E` | Move cursor to end |
| `Delete` | Delete character at cursor |
| `Backspace` | Delete character before cursor |
| `Ctrl+W` | Delete word |
| `Ctrl+U` | Clear entire field |

### Delete Confirm

| Key | Action |
|---|---|
| `y` / `Enter` | Confirm delete |
| `n` / `Esc` | Cancel |

## Configuration

The configuration file is located at `~/.config/edbookmark/config.toml`.

### Example Configuration

```toml
[window]
width = 100
height = 30

[launcher]
command = "omarchy-launch-webapp"
args = ["--isolate"]

[paths]
bookmarks = "~/.local/share/edbookmark/bookmarks.json"

[theme]
preset = "ayu-dark"
transparent_bg = true

[theme.colors]
# Background & foreground
bg = "reset"           # "reset" = transparent (follows terminal)
fg = "#BFBDB6"

# Bookmark components
name = "#E6E1CF"       # Bookmark name color
url = "#95E6CB"        # URL color
folder = "#D2A6FF"     # Folder color

# Table header
header = "#FF8F40"
header_bg = "reset"

# Selected row
selected_fg = "#E6E1CF"
selected_bg = "#2D4F67"

# Multi-select
multiselect_fg = "#E6E1CF"
multiselect_bg = "#3E4B59"

# Search bar
search_fg = "#E6E1CF"
search_border = "#39BAE6"
match_highlight = "#F07178"

# Status bar
status_fg = "#AAD94C"
status_bg = "reset"

# Accent and muted
accent = "#E6B450"
muted = "#565B66"

# Colorful borders (4 sides)
border_top = "#39BAE6"
border_right = "#AAD94C"
border_bottom = "#FF8F40"
border_left = "#D2A6FF"

# Form fields
field_active_border = "#39BAE6"
field_inactive_border = "#565B66"
field_text = "#E6E1CF"
field_placeholder = "#565B66"

# Delete dialog
delete_border = "#F07178"
delete_text = "#F07178"

# Buttons
button_save_fg = "#0D1017"
button_save_bg = "#AAD94C"
button_cancel_fg = "#BFBDB6"
button_cancel_bg = "#3E4B59"
button_delete_fg = "#0D1017"
button_delete_bg = "#F07178"
```

### Supported Color Formats

| Format | Example |
|---|---|
| Hex | `"#39BAE6"` |
| Named | `"red"`, `"green"`, `"blue"`, `"cyan"`, `"magenta"`, `"yellow"`, `"white"`, `"black"` |
| Transparent | `"reset"` or `"transparent"` |

## Hyprland Window Rules

The installer automatically adds the following rules to `~/.config/hypr/hyprland.conf`:

```ini
# BEGIN edbookmark rules
windowrule = float on, match:title ^(edbookmark)$
windowrule = center on, match:title ^(edbookmark)$
windowrule = size 800 500, match:title ^(edbookmark)$
# END edbookmark rules
```

Adjust the window size by editing the `size 800 500` value to your preference.

## Project Structure

```
edBookmark/
├── Cargo.toml              # Rust project configuration
├── Cargo.lock              # Dependency lock file
├── LICENSE                 # MIT License
├── install.sh              # Interactive installer
├── uninstall-edbookmark.sh # Interactive uninstaller
├── config/
│   └── default.toml        # Default configuration
├── dist/
│   └── edbookmark.desktop  # Desktop entry template
├── src/
│   ├── main.rs             # Entry point + CLI
│   ├── app.rs              # Application state & event loop
│   ├── bookmark.rs         # Bookmark struct & logic
│   ├── config.rs           # Configuration parser
│   ├── import_export.rs    # Chrome, Firefox & XLSX import/export
│   ├── keybinding.rs       # Mode & keybinding mapping
│   ├── launcher.rs         # Browser execution (systemd-run)
│   ├── search.rs           # Fuzzy search engine
│   ├── storage.rs          # JSON file read/write
│   ├── theme.rs            # Theme & color parser
│   └── ui/
│       ├── mod.rs
│       ├── main_view.rs    # Main layout
│       ├── search_bar.rs   # Search bar widget
│       ├── bookmark_list.rs # Bookmark table widget
│       ├── form_view.rs    # Add/edit form widget
│       ├── delete_dialog.rs # Delete dialog widget
│       ├── status_bar.rs   # Status bar widget
│       └── help_popup.rs   # Help popup widget
├── data/
│   └── .gitkeep
└── assets/
    └── .gitkeep
```

## Tech Stack

| Component | Technology |
|---|---|
| Language | Rust |
| TUI Framework | Ratatui + Crossterm |
| Fuzzy Search | fuzzy-matcher (Skim algorithm) |
| Serialization | serde + serde_json |
| Configuration | toml |
| CLI | clap |
| ID | uuid v4 |
| Timestamp | chrono |

## Troubleshooting

### edbookmark not found after installation

Make sure `~/.local/bin` is in your PATH:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Chromium does not open when pressing Enter

Check the launcher log:

```bash
cat ~/.local/state/edbookmark/launcher.log
```

### Chromium does not open from Walker

Make sure the desktop entry uses the walker wrapper:

```bash
cat ~/.local/share/applications/edbookmark.desktop
# Exec should point to: ~/.local/bin/edbookmark-walker
```

### Window is not floating in Hyprland

Make sure window rules are added:

```bash
grep "edbookmark" ~/.config/hypr/hyprland.conf
```

If not present, run the installer again or add them manually.

### Reset bookmark data

```bash
echo '{"version":"1.0","bookmarks":[]}' > ~/.local/share/edbookmark/bookmarks.json
```

### Reinstall if something goes wrong

```bash
bash uninstall-edbookmark.sh
bash install.sh
```

## License

[MIT License](LICENSE)

## Author

[CuedNub](https://github.com/CuedNub)

---
