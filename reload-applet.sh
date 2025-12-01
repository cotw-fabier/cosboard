#!/bin/bash
# Reload the cosboard applet by replacing the binary and restarting the panel

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY_SRC="$SCRIPT_DIR/target/release/cosboard-applet"
BINARY_DST="$HOME/.local/bin/cosboard-applet"

# Check if source binary exists
if [[ ! -f "$BINARY_SRC" ]]; then
    echo "Error: Binary not found at $BINARY_SRC"
    echo "Run 'cargo build --release --bin cosboard-applet' first"
    exit 1
fi

echo "Replacing applet binary..."
rm -f "$BINARY_DST"
cp "$BINARY_SRC" "$BINARY_DST"
chmod +x "$BINARY_DST"

echo "Restarting cosmic-panel..."
pkill cosmic-panel

echo "Done! Panel should auto-restart with new applet."
