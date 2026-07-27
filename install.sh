#!/bin/bash

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ok()   { echo -e "    ${GREEN}✓${NC} $1"; }
fail() { echo -e "    ${RED}✗${NC} $1"; }
warn() { echo -e "    ${YELLOW}⚠${NC} $1"; }
info() { echo -e "    ${CYAN}›${NC} $1"; }

ask_yn() {
    local prompt="$1"
    local default="${2:-y}"
    local yn_hint
    if [ "$default" = "y" ]; then yn_hint="[Y/n]"; else yn_hint="[y/N]"; fi
    while true; do
        echo -ne "    ${CYAN}?${NC} ${prompt} ${yn_hint}: "
        read -r answer
        answer="${answer:-$default}"
        case "$answer" in
            [Yy]*) return 0 ;;
            [Nn]*) return 1 ;;
            *) echo "    Please answer y or n." ;;
        esac
    done
}

ask_obs() {
    local prompt="$1"
    while true; do
        echo -ne "    ${CYAN}?${NC} ${prompt} [o]verwrite / [b]ackup+overwrite / [s]kip: "
        read -r answer
        case "$answer" in
            [Oo]*) return 0 ;;
            [Bb]*) return 1 ;;
            [Ss]*) return 2 ;;
            *) echo "    Please answer o, b, or s." ;;
        esac
    done
}

abort_safe() {
    echo ""
    echo -e "  ${YELLOW}Installation cancelled safely. No changes were made.${NC}"
    echo ""
    exit 0
}

declare -a INSTALLED=()
declare -a SKIPPED=()

echo ""
echo "═══════════════════════════════════════════════════════════"
echo -e "  ${BOLD}edbookmark Installer${NC}"
echo "  TUI Bookmark Manager for Arch Linux + Hyprland"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo -e "  ${YELLOW}Tested on:${NC} Omarchy · Arch Linux · Hyprland · Wayland · Kitty · Bash"
echo ""

# ──────────────────────────────────────────────────
# STAGE 1: Platform check
# ──────────────────────────────────────────────────
echo -e "${CYAN}━━ Stage 1: Platform Check ━━${NC}"
echo ""

if [ -f /etc/arch-release ]; then
    ok "Arch Linux detected"
else
    warn "Not Arch Linux — automatic dependency installation unavailable"
fi

if [ -n "$WAYLAND_DISPLAY" ]; then
    ok "Wayland session detected (WAYLAND_DISPLAY=$WAYLAND_DISPLAY)"
else
    warn "Wayland session not detected"
fi

if [ "$XDG_CURRENT_DESKTOP" = "Hyprland" ] || pgrep -x Hyprland >/dev/null 2>&1; then
    ok "Hyprland detected"
else
    warn "Hyprland not detected"
fi

if [ -n "$BASH_VERSION" ]; then
    ok "Bash shell ($BASH_VERSION)"
else
    warn "Not running in Bash"
fi

echo ""

# ──────────────────────────────────────────────────
# STAGE 2: Package manager check
# ──────────────────────────────────────────────────
echo -e "${CYAN}━━ Stage 2: Package Manager Check ━━${NC}"
echo ""

HAS_PACMAN=false
CAN_SUDO=false

if command -v pacman &>/dev/null; then
    ok "pacman found: $(which pacman)"
    HAS_PACMAN=true

    if command -v sudo &>/dev/null; then
        if sudo -n true 2>/dev/null || sudo -v 2>/dev/null; then
            ok "sudo available"
            CAN_SUDO=true
        else
            warn "sudo requires password — will prompt when needed"
            CAN_SUDO=true
        fi
    else
        warn "sudo not found — cannot install packages automatically"
    fi
else
    warn "pacman not found"
    warn "Automatic dependency installation is only supported on Arch Linux with pacman"
    warn "You must install missing dependencies manually"
fi

echo ""

# ──────────────────────────────────────────────────
# STAGE 3: Requirements check
# ──────────────────────────────────────────────────
echo -e "${CYAN}━━ Stage 3: Requirements Check ━━${NC}"
echo ""

declare -a MISSING_PKGS=()
declare -a MISSING_NAMES=()

check_required() {
    local cmd="$1"
    local pkg="$2"
    local label="$3"

    if command -v "$cmd" &>/dev/null; then
        ok "$label: $(which "$cmd")"
    else
        fail "$label: NOT FOUND"
        MISSING_PKGS+=("$pkg")
        MISSING_NAMES+=("$label ($cmd)")
    fi
}

echo -e "  ${BOLD}Required:${NC}"
check_required "rustc"        "rust"          "Rust compiler"
check_required "cargo"        "rust"          "Cargo"
check_required "kitty"        "kitty"         "Kitty terminal"
check_required "wl-copy"      "wl-clipboard"  "wl-copy (clipboard)"
check_required "setsid"       "util-linux"    "setsid"
check_required "systemd-run"  "systemd"       "systemd-run"
check_required "xdg-settings" "xdg-utils"     "xdg-settings"
check_required "chromium"     "chromium"       "Chromium browser"

if command -v rustc &>/dev/null; then
    RUST_VER=$(rustc --version | awk '{print $2}')
    MIN_VER="1.70.0"
    if [ "$(printf '%s\n' "$MIN_VER" "$RUST_VER" | sort -V | head -n1)" = "$MIN_VER" ]; then
        ok "Rust version $RUST_VER >= $MIN_VER"
    else
        fail "Rust $RUST_VER too old (need >= $MIN_VER)"
        MISSING_PKGS+=("rust")
        MISSING_NAMES+=("Rust >= $MIN_VER")
    fi
fi

echo ""
echo -e "  ${BOLD}Optional:${NC}"

check_optional() {
    local cmd="$1"
    if command -v "$cmd" &>/dev/null; then
        ok "$cmd: $(which "$cmd")"
    else
        warn "$cmd: not found (optional)"
    fi
}

check_optional "uwsm-app"
check_optional "walker"
check_optional "hyprctl"

echo ""

# ──────────────────────────────────────────────────
# STAGE 4: Install missing dependencies
# ──────────────────────────────────────────────────
if [ ${#MISSING_PKGS[@]} -gt 0 ]; then
    echo -e "${CYAN}━━ Stage 4: Missing Dependencies ━━${NC}"
    echo ""

    UNIQUE_PKGS=($(echo "${MISSING_PKGS[@]}" | tr ' ' '\n' | sort -u))

    echo "  The following packages are missing:"
    for name in "${MISSING_NAMES[@]}"; do
        echo -e "    ${RED}•${NC} $name"
    done
    echo ""
    echo "  Packages to install via pacman:"
    for pkg in "${UNIQUE_PKGS[@]}"; do
        echo -e "    ${CYAN}•${NC} $pkg"
    done
    echo ""

    if $HAS_PACMAN && $CAN_SUDO; then
        if ask_yn "Install missing packages using pacman?" "y"; then
            echo ""
            echo "  Running: sudo pacman -S --needed ${UNIQUE_PKGS[*]}"
            echo ""
            if sudo pacman -S --needed "${UNIQUE_PKGS[@]}"; then
                ok "Dependencies installed successfully"
                echo ""

                STILL_MISSING=false
                for cmd in rustc cargo kitty wl-copy setsid systemd-run xdg-settings chromium; do
                    if ! command -v "$cmd" &>/dev/null; then
                        fail "$cmd still not found after installation"
                        STILL_MISSING=true
                    fi
                done

                if $STILL_MISSING; then
                    echo ""
                    fail "Some dependencies are still missing after installation"
                    abort_safe
                fi
            else
                echo ""
                fail "pacman installation failed"
                abort_safe
            fi
        else
            abort_safe
        fi
    else
        echo -e "  ${YELLOW}Automatic installation unavailable (no pacman or no sudo).${NC}"
        echo ""
        echo "  Please install these packages manually:"
        for pkg in "${UNIQUE_PKGS[@]}"; do
            echo "    sudo pacman -S $pkg"
        done
        echo ""
        echo "  Then run this installer again."
        abort_safe
    fi
else
    echo -e "  ${GREEN}All required dependencies are installed.${NC}"
fi

echo ""

# ──────────────────────────────────────────────────
# STAGE 5: Build
# ──────────────────────────────────────────────────
echo -e "${CYAN}━━ Stage 5: Build ━━${NC}"
echo ""

info "Building edbookmark (release mode)..."
info "This may take 1-2 minutes on first build..."
echo ""

cd "$SCRIPT_DIR"

if cargo build --release 2>&1; then
    BINARY="$SCRIPT_DIR/target/release/edbookmark"
    BIN_SIZE=$(du -h "$BINARY" | awk '{print $1}')
    ok "Build successful ($BIN_SIZE)"
    echo ""
else
    echo ""
    fail "Build failed"
    echo "  Check the error output above."
    echo "  You can also try: cd $SCRIPT_DIR && cargo build --release 2>&1 | less"
    abort_safe
fi

# ──────────────────────────────────────────────────
# STAGE 6: Install (with permission per target)
# ──────────────────────────────────────────────────
echo -e "${CYAN}━━ Stage 6: Install ━━${NC}"
echo ""

BINARY_SRC="$SCRIPT_DIR/target/release/edbookmark"
BINARY_DST="$HOME/.local/bin/edbookmark"
WRAPPER_DST="$HOME/.local/bin/edbookmark-walker"
CONFIG_SRC="$SCRIPT_DIR/config/default.toml"
CONFIG_DST="$HOME/.config/edbookmark/config.toml"
DESKTOP_DST="$HOME/.local/share/applications/edbookmark.desktop"
DATA_DIR="$HOME/.local/share/edbookmark"
STATE_DIR="$HOME/.local/state/edbookmark"
HYPR_CONF="$HOME/.config/hypr/hyprland.conf"

# ── 6.1 Binary ──
echo -e "  ${BOLD}[1/7] Binary${NC}"
if [ -f "$BINARY_DST" ] || [ -L "$BINARY_DST" ]; then
    info "Already exists: $BINARY_DST"
    ask_obs "Binary already exists"
    OBS_RESULT=$?
    if [ $OBS_RESULT -eq 0 ]; then
        mkdir -p "$(dirname "$BINARY_DST")"
        ln -sf "$BINARY_SRC" "$BINARY_DST"
        ok "Binary symlink overwritten: $BINARY_DST"
        INSTALLED+=("Binary: $BINARY_DST")
    elif [ $OBS_RESULT -eq 1 ]; then
        cp -L "$BINARY_DST" "${BINARY_DST}.bak.$(date +%Y%m%d%H%M%S)" 2>/dev/null || true
        ln -sf "$BINARY_SRC" "$BINARY_DST"
        ok "Binary backed up and overwritten: $BINARY_DST"
        INSTALLED+=("Binary: $BINARY_DST (backed up)")
    else
        warn "Binary skipped"
        SKIPPED+=("Binary: $BINARY_DST")
    fi
else
    if ask_yn "Install binary symlink to $BINARY_DST ?" "y"; then
        mkdir -p "$(dirname "$BINARY_DST")"
        ln -sf "$BINARY_SRC" "$BINARY_DST"
        ok "Binary installed: $BINARY_DST"
        INSTALLED+=("Binary: $BINARY_DST")
    else
        warn "Binary skipped"
        SKIPPED+=("Binary: $BINARY_DST")
    fi
fi
echo ""

# ── 6.2 Walker wrapper ──
echo -e "  ${BOLD}[2/7] Walker Wrapper${NC}"

write_wrapper() {
    mkdir -p "$(dirname "$WRAPPER_DST")"
    cat > "$WRAPPER_DST" << 'WRAPEOF'
#!/bin/bash
# Managed by edbookmark installer

export PATH="$HOME/.local/bin:/usr/local/bin:/usr/bin:/bin:${PATH:-}"
export XDG_CURRENT_DESKTOP="${XDG_CURRENT_DESKTOP:-Hyprland}"

exec kitty --title edbookmark -e bash -lc 'edbookmark'
WRAPEOF
    chmod +x "$WRAPPER_DST"
}

if [ -f "$WRAPPER_DST" ]; then
    info "Already exists: $WRAPPER_DST"
    ask_obs "Walker wrapper already exists"
    OBS_RESULT=$?
    if [ $OBS_RESULT -eq 0 ]; then
        write_wrapper
        ok "Walker wrapper overwritten"
        INSTALLED+=("Wrapper: $WRAPPER_DST")
    elif [ $OBS_RESULT -eq 1 ]; then
        cp "$WRAPPER_DST" "${WRAPPER_DST}.bak.$(date +%Y%m%d%H%M%S)"
        write_wrapper
        ok "Walker wrapper backed up and overwritten"
        INSTALLED+=("Wrapper: $WRAPPER_DST (backed up)")
    else
        warn "Walker wrapper skipped"
        SKIPPED+=("Wrapper: $WRAPPER_DST")
    fi
else
    if ask_yn "Install Walker wrapper to $WRAPPER_DST ?" "y"; then
        write_wrapper
        ok "Walker wrapper installed"
        INSTALLED+=("Wrapper: $WRAPPER_DST")
    else
        warn "Walker wrapper skipped"
        SKIPPED+=("Wrapper: $WRAPPER_DST")
    fi
fi
echo ""

# ── 6.3 Config ──
echo -e "  ${BOLD}[3/7] Configuration${NC}"

if [ -f "$CONFIG_DST" ]; then
    info "Already exists: $CONFIG_DST"
    ask_obs "Configuration file already exists"
    OBS_RESULT=$?
    if [ $OBS_RESULT -eq 0 ]; then
        cp "$CONFIG_SRC" "$CONFIG_DST"
        ok "Configuration overwritten"
        INSTALLED+=("Config: $CONFIG_DST")
    elif [ $OBS_RESULT -eq 1 ]; then
        cp "$CONFIG_DST" "${CONFIG_DST}.bak.$(date +%Y%m%d%H%M%S)"
        cp "$CONFIG_SRC" "$CONFIG_DST"
        ok "Configuration backed up and overwritten"
        INSTALLED+=("Config: $CONFIG_DST (backed up)")
    else
        warn "Configuration skipped"
        SKIPPED+=("Config: $CONFIG_DST")
    fi
else
    if ask_yn "Create configuration file at $CONFIG_DST ?" "y"; then
        mkdir -p "$(dirname "$CONFIG_DST")"
        cp "$CONFIG_SRC" "$CONFIG_DST"
        ok "Configuration installed"
        INSTALLED+=("Config: $CONFIG_DST")
    else
        warn "Configuration skipped"
        SKIPPED+=("Config: $CONFIG_DST")
    fi
fi
echo ""

# ── 6.4 Desktop entry ──
echo -e "  ${BOLD}[4/7] Desktop Entry${NC}"

write_desktop() {
    mkdir -p "$(dirname "$DESKTOP_DST")"
    cat > "$DESKTOP_DST" << DSKEOF
[Desktop Entry]
Version=1.0
Name=edbookmark
Comment=TUI Bookmark Manager
Exec=$HOME/.local/bin/edbookmark-walker
Terminal=false
Type=Application
Categories=Utility;
StartupNotify=true
DSKEOF
}

if [ -f "$DESKTOP_DST" ]; then
    info "Already exists: $DESKTOP_DST"
    ask_obs "Desktop entry already exists"
    OBS_RESULT=$?
    if [ $OBS_RESULT -eq 0 ]; then
        write_desktop
        ok "Desktop entry overwritten"
        INSTALLED+=("Desktop: $DESKTOP_DST")
    elif [ $OBS_RESULT -eq 1 ]; then
        cp "$DESKTOP_DST" "${DESKTOP_DST}.bak.$(date +%Y%m%d%H%M%S)"
        write_desktop
        ok "Desktop entry backed up and overwritten"
        INSTALLED+=("Desktop: $DESKTOP_DST (backed up)")
    else
        warn "Desktop entry skipped"
        SKIPPED+=("Desktop: $DESKTOP_DST")
    fi
else
    if ask_yn "Create desktop entry at $DESKTOP_DST ?" "y"; then
        write_desktop
        ok "Desktop entry installed"
        INSTALLED+=("Desktop: $DESKTOP_DST")
    else
        warn "Desktop entry skipped"
        SKIPPED+=("Desktop: $DESKTOP_DST")
    fi
fi

if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
fi
echo ""

# ── 6.5 Data directory ──
echo -e "  ${BOLD}[5/7] Data Directory${NC}"

if [ -d "$DATA_DIR" ]; then
    ok "Data directory already exists: $DATA_DIR"
    INSTALLED+=("Data dir: $DATA_DIR (existing)")
else
    if ask_yn "Create data directory at $DATA_DIR ?" "y"; then
        mkdir -p "$DATA_DIR"
        mkdir -p "$STATE_DIR"
        ok "Data directory created"
        INSTALLED+=("Data dir: $DATA_DIR")
    else
        warn "Data directory skipped"
        SKIPPED+=("Data dir: $DATA_DIR")
    fi
fi
echo ""

# ── 6.6 Hyprland rules ──
echo -e "  ${BOLD}[6/7] Hyprland Window Rules${NC}"

if [ -f "$HYPR_CONF" ]; then
    if grep -q "BEGIN edbookmark rules" "$HYPR_CONF"; then
        ok "Hyprland rules already present"
        INSTALLED+=("Hyprland rules: existing")
    else
        info "Hyprland config found: $HYPR_CONF"
        echo ""
        echo "    Rules to add:"
        echo "      windowrule = float on, match:title ^(edbookmark)\$"
        echo "      windowrule = center on, match:title ^(edbookmark)\$"
        echo "      windowrule = size 800 500, match:title ^(edbookmark)\$"
        echo ""
        if ask_yn "Add edbookmark window rules to Hyprland config?" "y"; then
            cp "$HYPR_CONF" "${HYPR_CONF}.bak.$(date +%Y%m%d%H%M%S)"
            cat >> "$HYPR_CONF" << 'HYPREOF'

# BEGIN edbookmark rules
windowrule = float on, match:title ^(edbookmark)$
windowrule = center on, match:title ^(edbookmark)$
windowrule = size 800 500, match:title ^(edbookmark)$
# END edbookmark rules
HYPREOF
            ok "Hyprland rules added (backup created)"
            INSTALLED+=("Hyprland rules: added")

            if command -v hyprctl &>/dev/null; then
                if ask_yn "Reload Hyprland config now?" "y"; then
                    hyprctl reload 2>/dev/null
                    ok "Hyprland config reloaded"
                fi
            else
                info "Run 'hyprctl reload' or re-login to apply window rules"
            fi
        else
            warn "Hyprland rules skipped"
            SKIPPED+=("Hyprland rules")
        fi
    fi
else
    warn "Hyprland config not found at $HYPR_CONF"
    info "Add these rules manually to your Hyprland config:"
    echo "      windowrule = float on, match:title ^(edbookmark)\$"
    echo "      windowrule = center on, match:title ^(edbookmark)\$"
    echo "      windowrule = size 800 500, match:title ^(edbookmark)\$"
    SKIPPED+=("Hyprland rules: config not found")
fi
echo ""

# ── 6.7 PATH ──
echo -e "  ${BOLD}[7/7] PATH Configuration${NC}"

if echo "$PATH" | grep -q "$HOME/.local/bin"; then
    ok "~/.local/bin is already in PATH"
    INSTALLED+=("PATH: already configured")
else
    warn "~/.local/bin is NOT in PATH"
    info "edbookmark will not be accessible from terminal without this"
    echo ""
    if ask_yn "Add ~/.local/bin to PATH in ~/.bashrc?" "y"; then
        echo '' >> "$HOME/.bashrc"
        echo '# BEGIN edbookmark PATH' >> "$HOME/.bashrc"
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
        echo '# END edbookmark PATH' >> "$HOME/.bashrc"
        ok "PATH added to ~/.bashrc"
        info "Run: source ~/.bashrc (or open a new terminal)"
        INSTALLED+=("PATH: added to ~/.bashrc")
    else
        warn "PATH update skipped"
        info "You may need to add ~/.local/bin to your PATH manually"
        SKIPPED+=("PATH update")
    fi
fi
echo ""

# ──────────────────────────────────────────────────
# STAGE 7: Verification
# ──────────────────────────────────────────────────
echo -e "${CYAN}━━ Stage 7: Verification ━━${NC}"
echo ""

export PATH="$HOME/.local/bin:$PATH"

if command -v edbookmark &>/dev/null; then
    VER=$(edbookmark --version 2>&1)
    ok "edbookmark accessible: $VER"
else
    warn "edbookmark not found in PATH (you may need to open a new terminal)"
fi

if [ -f "$BINARY_DST" ] || [ -L "$BINARY_DST" ]; then
    ok "Binary exists"
else
    warn "Binary not installed"
fi

if [ -f "$DESKTOP_DST" ]; then
    ok "Desktop entry exists"
else
    warn "Desktop entry not installed"
fi

if [ -f "$CONFIG_DST" ]; then
    ok "Configuration exists"
else
    warn "Configuration not installed (built-in defaults will be used)"
fi

echo ""

# ──────────────────────────────────────────────────
# STAGE 8: Summary
# ──────────────────────────────────────────────────
echo "═══════════════════════════════════════════════════════════"
echo -e "  ${GREEN}${BOLD}✓ Installation complete!${NC}"
echo "═══════════════════════════════════════════════════════════"
echo ""

if [ ${#INSTALLED[@]} -gt 0 ]; then
    echo -e "  ${GREEN}Installed:${NC}"
    for item in "${INSTALLED[@]}"; do
        echo -e "    ${GREEN}✓${NC} $item"
    done
    echo ""
fi

if [ ${#SKIPPED[@]} -gt 0 ]; then
    echo -e "  ${YELLOW}Skipped:${NC}"
    for item in "${SKIPPED[@]}"; do
        echo -e "    ${YELLOW}⚠${NC} $item"
    done
    echo ""
fi

echo "  Usage:"
echo "    edbookmark                     Open TUI"
echo "    Walker → edbookmark            Open from launcher"
echo "    edbookmark --import chromium   Import bookmarks"
echo "    edbookmark --export html -o bookmarks.html"
echo ""
echo "  Config:    ~/.config/edbookmark/config.toml"
echo "  Data:      ~/.local/share/edbookmark/bookmarks.json"
echo ""
echo "  Uninstall: bash uninstall-edbookmark.sh"
echo ""
echo "═══════════════════════════════════════════════════════════"
