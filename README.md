# Cosboard

A soft keyboard for the COSMIC desktop environment.

## Overview

Cosboard is a libcosmic-based software keyboard with a focus on script-defined layouts, scripting, STT (speech-to-text), and swipe gestures. It is designed to provide keyboard interactions on touch-enabled devices running the COSMIC desktop.

## Features

- Floating, always-on-top keyboard window
- System tray applet for quick show/hide access
- D-Bus interface for external control
- Window position and size persistence
- Chromeless (no title bar) design with resizable borders

## Architecture

Cosboard consists of two components:

1. **Main Application** (`cosboard`): The keyboard window that displays the layout and handles input
2. **System Tray Applet** (`cosboard-applet`): A panel applet for toggling keyboard visibility

These components communicate via D-Bus on the session bus.

### D-Bus Interface

- **Service**: `io.github.cosboard.Cosboard`
- **Object Path**: `/io/github/cosboard/Cosboard`
- **Methods**: `Show()`, `Hide()`, `Toggle()`, `Quit()`
- **Signals**: `VisibilityChanged(visible: bool)`

## Requirements

- Rust 2024 edition
- COSMIC desktop environment (or libcosmic)
- Wayland (with fallback support for X11)
- D-Bus session bus

### Dependencies

The project uses the following main dependencies:

- `libcosmic` - COSMIC widget toolkit
- `zbus` - D-Bus implementation for Rust
- `tokio` - Async runtime
- `futures` - Async utilities
- `serde` - Serialization
- `i18n-embed` - Internationalization

## Building

### Debug Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

Or using just (if installed):

```bash
just build-release
```

## Running

### Main Application

```bash
cargo run --release
```

Or after building:

```bash
./target/release/cosboard
```

### System Tray Applet

```bash
cargo run --release --bin cosboard-applet
```

Or after building:

```bash
./target/release/cosboard-applet
```

## Installation

Installation requires `just` (a command runner). If just is not available, you can manually copy the files.

### Using just

```bash
just install
```

This installs:
- Binary to `/usr/bin/cosboard`
- Desktop entry to `/usr/share/applications/`
- AppStream metadata to `/usr/share/appdata/`
- Icon to `/usr/share/icons/hicolor/scalable/apps/`

### Manual Installation

```bash
# Build release
cargo build --release

# Install binary
sudo install -Dm0755 target/release/cosboard /usr/bin/cosboard

# Install desktop entry
sudo install -Dm0644 resources/io.github.cosboard.Cosboard.desktop /usr/share/applications/io.github.cosboard.Cosboard.desktop

# Install icon
sudo install -Dm0644 resources/icons/hicolor/scalable/apps/io.github.cosboard.Cosboard.svg /usr/share/icons/hicolor/scalable/apps/io.github.cosboard.Cosboard.svg

# Install appstream metadata
sudo install -Dm0644 resources/io.github.cosboard.Cosboard.metainfo.xml /usr/share/appdata/io.github.cosboard.Cosboard.metainfo.xml
```

## Uninstallation

### Using just

```bash
just uninstall
```

### Manual Uninstallation

```bash
sudo rm -f /usr/bin/cosboard
sudo rm -f /usr/share/applications/io.github.cosboard.Cosboard.desktop
sudo rm -f /usr/share/icons/hicolor/scalable/apps/io.github.cosboard.Cosboard.svg
sudo rm -f /usr/share/appdata/io.github.cosboard.Cosboard.metainfo.xml
```

## Usage

### From Application Menu

After installation, Cosboard appears in your application menu under Utilities/Accessibility.

### From Command Line

```bash
# Start the keyboard
cosboard

# Start the applet (typically auto-started with the desktop)
cosboard-applet
```

### D-Bus Control

You can control the keyboard via D-Bus:

```bash
# Toggle visibility
dbus-send --session --type=method_call \
  --dest=io.github.cosboard.Cosboard \
  /io/github/cosboard/Cosboard \
  io.github.cosboard.Cosboard.Toggle

# Show keyboard
dbus-send --session --type=method_call \
  --dest=io.github.cosboard.Cosboard \
  /io/github/cosboard/Cosboard \
  io.github.cosboard.Cosboard.Show

# Hide keyboard
dbus-send --session --type=method_call \
  --dest=io.github.cosboard.Cosboard \
  /io/github/cosboard/Cosboard \
  io.github.cosboard.Cosboard.Hide

# Quit application
dbus-send --session --type=method_call \
  --dest=io.github.cosboard.Cosboard \
  /io/github/cosboard/Cosboard \
  io.github.cosboard.Cosboard.Quit
```

## Testing

Run the test suite:

```bash
cargo test
```

## Project Structure

```
cosboard/
├── Cargo.toml           # Package configuration
├── justfile             # Build automation recipes
├── i18n.toml            # i18n configuration
├── src/
│   ├── main.rs          # Main application entry point
│   ├── lib.rs           # Library crate for shared code
│   ├── app.rs           # Main application model
│   ├── app_settings.rs  # Centralized constants
│   ├── config.rs        # User configuration
│   ├── state.rs         # Window state persistence
│   ├── layer_shell.rs   # Wayland layer-shell support
│   ├── i18n.rs          # Localization support
│   ├── dbus/
│   │   └── mod.rs       # D-Bus interface
│   ├── applet/
│   │   └── mod.rs       # System tray applet
│   └── bin/
│       └── applet.rs    # Applet binary entry point
├── i18n/
│   └── en/
│       └── cosboard.ftl # English translations
└── resources/
    ├── io.github.cosboard.Cosboard.desktop
    ├── io.github.cosboard.Cosboard.metainfo.xml
    └── icons/
        └── hicolor/
            └── scalable/
                └── apps/
                    └── io.github.cosboard.Cosboard.svg
```

## Configuration

### State Persistence

Window position and size are automatically saved using COSMIC's configuration system. State is stored in:

```
~/.config/cosmic/io.github.cosboard.Cosboard/v1/
```

### Default Window Settings

- **Default Size**: 800x300 pixels
- **Minimum Size**: 400x150 pixels
- **Resize Border**: 8 pixels

## License

GPL-3.0-only

## Future Features

See the project vision document for planned features including:
- JSON-defined keyboard layouts
- Long-press and swipe gesture support
- Scripting engine for custom key actions
- Word prediction and autocomplete
- Speech-to-text integration
