#!/bin/bash

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

ok()   { echo -e "    ${GREEN}✓${NC} $1"; }
fail() { echo -e "    ${RED}✗${NC} $1"; }
warn() { echo -e "    ${YELLOW}⚠${NC} $1"; }
info() { echo -e "    ${CYAN}›${NC} $1"; }

ask_yn() {
    local prompt="$1"
    local default="${2:-n}"
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

BINARY_DST="$HOME/.local/bin/edbookmark"
WRAPPER_DST="$HOME/.local/bin/edbookmark-walker"
CONFIG_DIR="$HOME/.config/edbookmark"
CONFIG_DST="$CONFIG_DIR/config.toml"
DATA_DIR="$HOME/.local/share/edbookmark"
DESKTOP_DST="$HOME/.local/share/applications/edbookmark.desktop"
STATE_DIR="$HOME/.local/state/edbookmark"
HYPR_CONF="$HOME/.config/hypr/hyprland.conf"

echo ""
echo "═══════════════════════════════════════════════════════════"
echo -e "  ${BOLD}edbookmark Uninstaller${NC}"
echo "═══════════════════════════════════════════════════════════"
echo ""

# ──────────────────────────────────────────────────
# STAGE 1: Detect installed components
# ──────────────────────────────────────────────────
echo -e "${CYAN}━━ Stage 1: Detecting Installed Components ━━${NC}"
echo ""

declare -a FOUND_ITEMS=()
declare -a FOUND_LABELS=()

if [ -f "$BINARY_DST" ] || [ -L "$BINARY_DST" ]; then
    ok "Binary: $BINARY_DST"
    FOUND_ITEMS+=("binary")
    FOUND_LABELS+=("Binary: $BINARY_DST")
else
    info "Binary not found (already removed?)"
fi

if [ -f "$WRAPPER_DST" ]; then
    ok "Wrapper: $WRAPPER_DST"
    FOUND_ITEMS+=("wrapper")
    FOUND_LABELS+=("Wrapper: $WRAPPER_DST")
else
    info "Wrapper not found"
fi

if [ -f "$CONFIG_DST" ]; then
    ok "Config: $CONFIG_DST"
    FOUND_ITEMS+=("config")
    FOUND_LABELS+=("Config: $CONFIG_DIR/")
else
    info "Config not found"
fi

if [ -d "$DATA_DIR" ]; then
    BM_COUNT=0
    if [ -f "$DATA_DIR/bookmarks.json" ]; then
        BM_COUNT=$(python3 -c "
import json
try:
    with open('$DATA_DIR/bookmarks.json') as f:
        print(len(json.load(f).get('bookmarks',[])))
except:
    print(0)
" 2>/dev/null || echo "0")
    fi
    ok "Data: $DATA_DIR ($BM_COUNT bookmarks)"
    FOUND_ITEMS+=("data")
    FOUND_LABELS+=("Data: $DATA_DIR/ ($BM_COUNT bookmarks)")
else
    info "Data directory not found"
fi

if [ -f "$DESKTOP_DST" ]; then
    ok "Desktop: $DESKTOP_DST"
    FOUND_ITEMS+=("desktop")
    FOUND_LABELS+=("Desktop: $DESKTOP_DST")
else
    info "Desktop entry not found"
fi

if [ -d "$STATE_DIR" ]; then
    ok "Logs/State: $STATE_DIR"
    FOUND_ITEMS+=("state")
    FOUND_LABELS+=("Logs/State: $STATE_DIR/")
else
    info "State directory not found"
fi

HAS_HYPR_RULES=false
if [ -f "$HYPR_CONF" ] && grep -q "BEGIN edbookmark rules" "$HYPR_CONF"; then
    ok "Hyprland rules in $HYPR_CONF"
    FOUND_ITEMS+=("hyprland")
    FOUND_LABELS+=("Hyprland rules in $HYPR_CONF")
    HAS_HYPR_RULES=true
else
    info "No edbookmark Hyprland rules found"
fi

HAS_PATH_ENTRY=false
if [ -f "$HOME/.bashrc" ] && grep -q "BEGIN edbookmark PATH" "$HOME/.bashrc"; then
    ok "PATH entry in ~/.bashrc"
    FOUND_ITEMS+=("path")
    FOUND_LABELS+=("PATH entry in ~/.bashrc")
    HAS_PATH_ENTRY=true
else
    info "No edbookmark PATH entry in .bashrc"
fi

echo ""

if [ ${#FOUND_ITEMS[@]} -eq 0 ]; then
    echo -e "  ${YELLOW}Nothing to uninstall. edbookmark is not installed.${NC}"
    echo ""
    exit 0
fi

# ──────────────────────────────────────────────────
# STAGE 2: Ask permission per item
# ──────────────────────────────────────────────────
echo -e "${CYAN}━━ Stage 2: Select Items to Remove ━━${NC}"
echo ""

declare -a REMOVE_LIST=()
declare -a REMOVE_LABELS=()
declare -a PRESERVE_LABELS=()

for i in "${!FOUND_ITEMS[@]}"; do
    item="${FOUND_ITEMS[$i]}"
    label="${FOUND_LABELS[$i]}"

    case "$item" in
        data)
            if ask_yn "Remove $label ?"; then
                REMOVE_LIST+=("$item")
                REMOVE_LABELS+=("$label")
            else
                PRESERVE_LABELS+=("$label")
            fi
            ;;
        config)
            if ask_yn "Remove $label ?"; then
                REMOVE_LIST+=("$item")
                REMOVE_LABELS+=("$label")
            else
                PRESERVE_LABELS+=("$label")
            fi
            ;;
        path)
            if ask_yn "Remove $label ?"; then
                REMOVE_LIST+=("$item")
                REMOVE_LABELS+=("$label")
            else
                PRESERVE_LABELS+=("$label")
            fi
            ;;
        *)
            if ask_yn "Remove $label ?"; then
                REMOVE_LIST+=("$item")
                REMOVE_LABELS+=("$label")
            else
                PRESERVE_LABELS+=("$label")
            fi
            ;;
    esac
done

echo ""

if [ ${#REMOVE_LIST[@]} -eq 0 ]; then
    echo -e "  ${YELLOW}Nothing selected for removal. Uninstall cancelled.${NC}"
    echo ""
    exit 0
fi

# ──────────────────────────────────────────────────
# STAGE 3: Final confirmation
# ──────────────────────────────────────────────────
echo -e "${CYAN}━━ Stage 3: Confirmation ━━${NC}"
echo ""

echo "  The following will be ${RED}removed${NC}:"
for label in "${REMOVE_LABELS[@]}"; do
    echo -e "    ${RED}•${NC} $label"
done

if [ ${#PRESERVE_LABELS[@]} -gt 0 ]; then
    echo ""
    echo "  The following will be ${GREEN}preserved${NC}:"
    for label in "${PRESERVE_LABELS[@]}"; do
        echo -e "    ${GREEN}•${NC} $label"
    done
fi

echo ""
if ! ask_yn "Proceed with uninstall?"; then
    echo ""
    echo -e "  ${YELLOW}Uninstall cancelled. No changes were made.${NC}"
    echo ""
    exit 0
fi

echo ""

# ──────────────────────────────────────────────────
# STAGE 4: Remove
# ──────────────────────────────────────────────────
echo -e "${CYAN}━━ Stage 4: Removing ━━${NC}"
echo ""

for item in "${REMOVE_LIST[@]}"; do
    case "$item" in
        binary)
            rm -f "$BINARY_DST"
            ok "Binary removed"
            ;;
        wrapper)
            rm -f "$WRAPPER_DST"
            ok "Wrapper removed"
            ;;
        config)
            rm -rf "$CONFIG_DIR"
            ok "Config directory removed"
            ;;
        data)
            rm -rf "$DATA_DIR"
            ok "Data directory removed"
            ;;
        desktop)
            rm -f "$DESKTOP_DST"
            if command -v update-desktop-database &>/dev/null; then
                update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
            fi
            ok "Desktop entry removed"
            ;;
        state)
            rm -rf "$STATE_DIR"
            ok "Logs/State directory removed"
            ;;
        hyprland)
            if [ -f "$HYPR_CONF" ]; then
                cp "$HYPR_CONF" "${HYPR_CONF}.bak.$(date +%Y%m%d%H%M%S)"
                sed -i '/# BEGIN edbookmark rules/,/# END edbookmark rules/d' "$HYPR_CONF"
                # Clean trailing empty lines
                sed -i -e :a -e '/^\n*$/{$d;N;ba' -e '}' "$HYPR_CONF"
                ok "Hyprland rules removed (backup created)"
                if command -v hyprctl &>/dev/null; then
                    if ask_yn "Reload Hyprland config now?"; then
                        hyprctl reload 2>/dev/null
                        ok "Hyprland config reloaded"
                    fi
                fi
            fi
            ;;
        path)
            if [ -f "$HOME/.bashrc" ]; then
                cp "$HOME/.bashrc" "$HOME/.bashrc.bak.$(date +%Y%m%d%H%M%S)"
                sed -i '/# BEGIN edbookmark PATH/,/# END edbookmark PATH/d' "$HOME/.bashrc"
                ok "PATH entry removed from .bashrc (backup created)"
            fi
            ;;
    esac
done

echo ""

# ──────────────────────────────────────────────────
# STAGE 5: Summary
# ──────────────────────────────────────────────────
echo "═══════════════════════════════════════════════════════════"
echo -e "  ${GREEN}${BOLD}✓ edbookmark uninstalled${NC}"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo -e "  ${RED}Removed:${NC}"
for label in "${REMOVE_LABELS[@]}"; do
    echo -e "    ${RED}✓${NC} $label"
done

if [ ${#PRESERVE_LABELS[@]} -gt 0 ]; then
    echo ""
    echo -e "  ${GREEN}Preserved:${NC}"
    for label in "${PRESERVE_LABELS[@]}"; do
        echo -e "    ${GREEN}•${NC} $label"
    done
fi

echo ""
echo "  To reinstall:"
echo "    git clone git@github.com:CuedNub/edBookmark.git"
echo "    cd edBookmark && bash install.sh"
echo ""
echo "═══════════════════════════════════════════════════════════"
