#!/bin/bash
# Cosboard User Uninstallation Script
# Removes cosboard and applet from user directories

set -e

APP_ID="io.github.cosboard.Cosboard"

echo "=== Cosboard User Uninstallation ==="
echo ""

echo "Removing binaries..."
rm -f ~/.local/bin/cosboard
rm -f ~/.local/bin/cosboard-applet

echo "Removing desktop entries..."
rm -f ~/.local/share/applications/$APP_ID.desktop
rm -f ~/.local/share/applications/$APP_ID.Applet.desktop

echo "Removing icon..."
rm -f ~/.local/share/icons/hicolor/scalable/apps/$APP_ID.svg

echo "Removing D-Bus service file..."
rm -f ~/.local/share/dbus-1/services/$APP_ID.service

echo "Removing appstream metadata..."
rm -f ~/.local/share/metainfo/$APP_ID.metainfo.xml

echo "Updating icon cache..."
gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor 2>/dev/null || true

echo "Updating desktop database..."
update-desktop-database ~/.local/share/applications 2>/dev/null || true

echo ""
echo "=== Uninstallation Complete ==="
echo ""
echo "Note: You may need to remove the applet from your panel manually"
echo "and log out/in for changes to fully take effect."
echo ""
