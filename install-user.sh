#!/bin/bash
# Cosboard User Installation Script
# Installs cosboard-applet to user directories (no sudo required)
# The applet binary now directly manages the keyboard window (no separate main app needed)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_ID="io.github.cosboard.Cosboard"

echo "=== Cosboard User Installation ==="
echo ""

# Check if applet binary exists
if [[ ! -f "$SCRIPT_DIR/target/release/cosboard-applet" ]]; then
    echo "Building release binary..."
    cargo build --release --bin cosboard-applet --manifest-path="$SCRIPT_DIR/Cargo.toml"
fi

echo "Installing applet binary..."
mkdir -p ~/.local/bin
cp "$SCRIPT_DIR/target/release/cosboard-applet" ~/.local/bin/
chmod +x ~/.local/bin/cosboard-applet

echo "Installing desktop entries..."
mkdir -p ~/.local/share/applications
cp "$SCRIPT_DIR/resources/$APP_ID.Applet.desktop" ~/.local/share/applications/

# Update desktop entry Exec path to use ~/.local/bin
sed -i "s|Exec=cosboard-applet|Exec=$HOME/.local/bin/cosboard-applet|g" ~/.local/share/applications/$APP_ID.Applet.desktop

echo "Installing icon..."
mkdir -p ~/.local/share/icons/hicolor/scalable/apps
cp "$SCRIPT_DIR/resources/icons/hicolor/scalable/apps/$APP_ID.svg" ~/.local/share/icons/hicolor/scalable/apps/

echo "Installing appstream metadata..."
mkdir -p ~/.local/share/metainfo
cp "$SCRIPT_DIR/resources/$APP_ID.metainfo.xml" ~/.local/share/metainfo/

echo "Updating icon cache..."
gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor 2>/dev/null || true

echo "Updating desktop database..."
update-desktop-database ~/.local/share/applications 2>/dev/null || true

echo ""
echo "=== Installation Complete ==="
echo ""
echo "Installed to:"
echo "  - Binary:       ~/.local/bin/cosboard-applet"
echo "  - Desktop:      ~/.local/share/applications/"
echo "  - Icon:         ~/.local/share/icons/hicolor/scalable/apps/"
echo ""
echo "IMPORTANT: Make sure ~/.local/bin is in your PATH."
echo "Add this to your ~/.bashrc or ~/.zshrc if needed:"
echo '  export PATH="$HOME/.local/bin:$PATH"'
echo ""
echo "NEXT STEPS:"
echo "  1. Log out and log back in (or restart your session)"
echo "  2. Right-click on the COSMIC panel"
echo "  3. Select 'Panel Settings' or 'Add Applet'"
echo "  4. Find 'Cosboard' and add it to your panel"
echo ""
echo "Or test immediately by running:"
echo "  ~/.local/bin/cosboard-applet"
echo ""
