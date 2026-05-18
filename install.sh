#!/bin/bash
set -e

APP_ID="com.leaf.app"
PREFIX="${HOME}/.local"

case "${1:-}" in
    uninstall)
        echo "Uninstalling Leaf..."
        rm -f "${PREFIX}/bin/leaf"
        rm -f "${PREFIX}/share/applications/${APP_ID}.desktop"
        for size in 48x48 128x128 256x256 1024x1024; do
            rm -f "${PREFIX}/share/icons/hicolor/${size}/apps/${APP_ID}.png"
        done
        echo "Leaf uninstalled."
        exit 0
        ;;
    system)
        if [ "$(id -u)" -ne 0 ]; then
            echo "System install requires root (run with sudo)."
            exit 1
        fi
        PREFIX="/usr/local"
        ;;
esac

# ------------------------------------------------------------------
# Dependency detection and installation
# ------------------------------------------------------------------

detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    else
        echo "unknown"
    fi
}

MISSING_BUILD=()
MISSING_RUNTIME=()

if ! command -v rustc &> /dev/null; then MISSING_BUILD+=("rustc"); fi
if ! command -v cargo &> /dev/null; then MISSING_BUILD+=("cargo"); fi

if ! command -v pkg-config &> /dev/null || ! pkg-config --exists gtk4 2>/dev/null; then
    MISSING_BUILD+=("gtk4 development headers (includes pango, gdk-pixbuf, cairo)")
fi

if ! command -v pdftoppm &> /dev/null; then
    MISSING_RUNTIME+=("pdftoppm (part of poppler-utils or poppler-tools, needed for PDF covers)")
fi

if [ ${#MISSING_BUILD[@]} -gt 0 ] || [ ${#MISSING_RUNTIME[@]} -gt 0 ]; then
    echo ""
    echo "Some dependencies are missing."
    echo ""

    DISTRO=$(detect_distro)
    PKG_CMD=""

    case "$DISTRO" in
        arch|endeavouros|artix|manjaro|cachyos)
            PKG_CMD="sudo pacman -S --needed gtk4 poppler"
            PKG_LABEL="Arch or Arch derivative (EndeavourOS, CachyOS, Manjaro, etc.)"
            ;;
        debian|ubuntu|pop|linuxmint|zorin|elementary|trisquel|neon)
            PKG_CMD="sudo apt install libgtk-4-dev poppler-utils"
            PKG_LABEL="Debian, Ubuntu, or derivative (Pop!_OS, Linux Mint, Zorin, etc.)"
            ;;
        fedora)
            PKG_CMD="sudo dnf install gtk4-devel poppler-utils"
            PKG_LABEL="Fedora or derivative"
            ;;
        opensuse*|suse*)
            PKG_CMD="sudo zypper install gtk4-devel poppler-tools"
            PKG_LABEL="openSUSE or derivative"
            ;;
    esac

    if [ -n "$PKG_CMD" ]; then
        echo "Detected: $PKG_LABEL"
        [ ${#MISSING_BUILD[@]} -gt 0 ] && echo "  Missing build deps: ${MISSING_BUILD[*]}"
        [ ${#MISSING_RUNTIME[@]} -gt 0 ] && echo "  Missing runtime deps: ${MISSING_RUNTIME[*]}"
        echo ""
        echo "Run the following to install them?"
        echo "  $PKG_CMD"
        read -rp "Run now? [Y/n] " REPLY
        case "$REPLY" in
            [Nn]*)
                echo "Install the dependencies manually, then re-run this script."
                exit 1
                ;;
            *)
                eval "$PKG_CMD"
                ;;
        esac
    else
        echo "Could not detect your distro from /etc/os-release."
        echo "Please install these dependencies manually, then re-run this script:"
        echo "  - gtk4 development headers (including pango, gdk-pixbuf, cairo)"
        echo "  - poppler-utils or poppler-tools (for pdftoppm)"
        echo "  - rustc and cargo (from https://rustup.rs)"
        echo ""
        read -rp "Press Enter after you have installed the dependencies (or Ctrl+C to abort)."
    fi
fi

# ------------------------------------------------------------------
# Build and install
# ------------------------------------------------------------------

echo ""
echo "Building Leaf..."
cargo build --release

echo ""
echo "Installing binary..."
mkdir -p "${PREFIX}/bin"
cp "target/release/leaf" "${PREFIX}/bin/leaf"

echo "Installing desktop entry..."
mkdir -p "${PREFIX}/share/applications"
cp "data/${APP_ID}.desktop" "${PREFIX}/share/applications/"

echo "Installing icons..."
for size in 48x48 128x128 256x256 1024x1024; do
    icon_src="data/icons/hicolor/${size}/apps/${APP_ID}.png"
    if [ -f "$icon_src" ]; then
        mkdir -p "${PREFIX}/share/icons/hicolor/${size}/apps"
        cp "$icon_src" "${PREFIX}/share/icons/hicolor/${size}/apps/"
    fi
done

echo "Updating icon cache..."
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t "${PREFIX}/share/icons/hicolor" 2>/dev/null || true
fi

echo ""
echo "Leaf installed to ${PREFIX}."
echo "Run 'leaf --gui' or launch from your app menu."
echo "For the CLI version, run 'leaf' in your terminal."
echo "To uninstall: ./install.sh uninstall"
