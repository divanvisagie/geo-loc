#!/bin/bash
set -e

# geo-loc uninstallation script
# Removes the binary and desktop file

BINARY_NAME="geo-loc"
DESKTOP_FILE="geo-loc.desktop"
INSTALL_PREFIX="/usr/local"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

error() {
    echo -e "${RED}Error: $1${NC}" >&2
    exit 1
}

info() {
    echo -e "${GREEN}Info: $1${NC}"
}

warn() {
    echo -e "${YELLOW}Warning: $1${NC}"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    error "Do not run this script as root. Use sudo only when prompted."
fi

info "Uninstalling $BINARY_NAME..."

# Remove binary
if [[ -f "$INSTALL_PREFIX/bin/$BINARY_NAME" ]]; then
    info "Removing binary from $INSTALL_PREFIX/bin/$BINARY_NAME"
    sudo rm -f "$INSTALL_PREFIX/bin/$BINARY_NAME"
else
    warn "Binary not found at $INSTALL_PREFIX/bin/$BINARY_NAME"
fi

# Remove desktop file
if [[ -f "/usr/share/applications/$DESKTOP_FILE" ]]; then
    info "Removing desktop file from /usr/share/applications/$DESKTOP_FILE"
    sudo rm -f "/usr/share/applications/$DESKTOP_FILE"

    # Update desktop database
    if command -v update-desktop-database >/dev/null 2>&1; then
        info "Updating desktop database..."
        sudo update-desktop-database /usr/share/applications/
    else
        warn "update-desktop-database not found"
    fi
else
    warn "Desktop file not found at /usr/share/applications/$DESKTOP_FILE"
fi

# Check if completely removed
if command -v "$BINARY_NAME" >/dev/null 2>&1; then
    warn "$BINARY_NAME still found in PATH - may be installed elsewhere"
else
    info "✓ $BINARY_NAME successfully removed from system"
fi

echo
info "Uninstallation complete!"
echo
info "Note: GeoClue configuration entry was NOT removed."
info "To remove it manually:"
info "  sudo sed -i '/^\\[geo-loc\\]/,+3d' /etc/geoclue/geoclue.conf"
echo
