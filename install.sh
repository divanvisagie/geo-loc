#!/bin/bash
set -e

# geo-loc installation script
# Installs the binary and required desktop file for GeoClue integration

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

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -f "resources/$DESKTOP_FILE" ]]; then
    error "Please run this script from the geo-loc project root directory"
fi

# Build the binary if it doesn't exist
if [[ ! -f "target/release/$BINARY_NAME" ]]; then
    info "Building $BINARY_NAME..."
    cargo build --release
fi

# Verify binary exists
if [[ ! -f "target/release/$BINARY_NAME" ]]; then
    error "Failed to build $BINARY_NAME binary"
fi

info "Installing $BINARY_NAME..."

# Install binary
info "Installing binary to $INSTALL_PREFIX/bin/$BINARY_NAME"
sudo cp "target/release/$BINARY_NAME" "$INSTALL_PREFIX/bin/"
sudo chmod +x "$INSTALL_PREFIX/bin/$BINARY_NAME"

# Install desktop file (REQUIRED for GeoClue)
info "Installing desktop file to /usr/share/applications/$DESKTOP_FILE"
sudo cp "resources/$DESKTOP_FILE" "/usr/share/applications/"
sudo chmod 644 "/usr/share/applications/$DESKTOP_FILE"

# Update desktop database
if command -v update-desktop-database >/dev/null 2>&1; then
    info "Updating desktop database..."
    sudo update-desktop-database /usr/share/applications/
else
    warn "update-desktop-database not found - desktop file may not be recognized immediately"
fi

# Verify installation
if command -v "$BINARY_NAME" >/dev/null 2>&1; then
    info "Installation successful!"
    info "You can now run: $BINARY_NAME"

    # Test the installation
    info "Testing installation..."
    if "$BINARY_NAME" >/dev/null 2>&1; then
        info "✓ geo-loc is working correctly"
    else
        warn "geo-loc installed but returned an error - this may be normal if location services are unavailable"
    fi
else
    error "Installation failed - $BINARY_NAME not found in PATH"
fi

echo
info "Installation complete!"
echo
info "Requirements for GeoClue support:"
info "  - GeoClue 2 service: sudo apt install geoclue-2.0"
info "  - Desktop file installed: ✓ (done)"
info "  - Location permissions: May be requested on first run"
echo
info "The tool will automatically fall back to IP-based location if GeoClue is unavailable."
