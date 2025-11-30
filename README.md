# Cosboard

**A powerful, customizable soft keyboard for the COSMIC desktop**

## Introduction

Cosboard is the first native soft keyboard solution for the COSMIC desktop environment. Built with libcosmic/Iced, it provides touch keyboard users with a fully customizable, JSON-driven layout system designed for accessibility, extensibility, and ease of use.

### Why Cosboard?

- **Native Integration**: Purpose-built for COSMIC using libcosmic - no third-party workarounds
- **Declarative Customization**: JSON-based layouts let you create, share, and modify keyboards without coding
- **Accessibility First**: Flexible architecture supporting custom layouts, adjustable key sizes, and alternative input methods
- **Future-Ready**: Designed to support advanced features including speech-to-text, word prediction, and touch input emulation

### Who Is It For?

- **Touch Device Users**: Operating COSMIC on tablets, 2-in-1 laptops, or touchscreen monitors
- **Accessibility Users**: Those requiring customizable layouts and alternative input methods
- **Power Users**: Technical users wanting fine-grained control, custom actions, and scripting
- **Multi-Language Users**: Those working across languages who need easy layout switching

## Current Features

- Floating, always-on-top keyboard window
- System tray applet for quick show/hide access
- D-Bus interface for external control
- Window position and size persistence
- Chromeless (borderless) design with resizable borders

## Quick Start

### Prerequisites

- Rust (2024 edition)
- COSMIC desktop environment (or libcosmic development libraries)
- Wayland compositor (X11 fallback available)
- D-Bus session bus

### Build and Run

```bash
# Clone the repository
git clone https://github.com/user/cosboard.git
cd cosboard

# Build in release mode
cargo build --release

# Run the main keyboard application
cargo run --release

# In a separate terminal, run the system tray applet
cargo run --release --bin cosboard-applet
```

### One-Liner (after building)

```bash
# Run both components
./target/release/cosboard & ./target/release/cosboard-applet
```

## Architecture

Cosboard consists of two components that communicate via D-Bus:

```
┌─────────────────────┐     D-Bus      ┌─────────────────────┐
│   cosboard-applet   │◄──────────────►│      cosboard       │
│   (System Tray)     │                │  (Keyboard Window)  │
└─────────────────────┘                └─────────────────────┘
        │                                       │
        │ Click to toggle                       │ Floating window
        ▼                                       ▼
   ┌─────────┐                          ┌─────────────────┐
   │  Panel  │                          │ Desktop Surface │
   └─────────┘                          └─────────────────┘
```

### D-Bus Interface

- **Service**: `io.github.cosboard.Cosboard`
- **Object Path**: `/io/github/cosboard/Cosboard`
- **Methods**: `Show()`, `Hide()`, `Toggle()`, `Quit()`
- **Signals**: `VisibilityChanged(visible: bool)`

## Building

### Debug Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

### Using just (if installed)

```bash
just build-release
```

### Run Tests

```bash
cargo test
```

## Running

### Main Application

```bash
# Development
cargo run --release

# After building
./target/release/cosboard
```

### System Tray Applet

```bash
# Development
cargo run --release --bin cosboard-applet

# After building
./target/release/cosboard-applet
```

## Installation

### Using just

```bash
just install
```

This installs:
- Binaries to `/usr/bin/cosboard` and `/usr/bin/cosboard-applet`
- Desktop entries to `/usr/share/applications/`
- AppStream metadata to `/usr/share/appdata/`
- Icon to `/usr/share/icons/hicolor/scalable/apps/`
- D-Bus service file to `/usr/share/dbus-1/services/`

### Adding the Applet to the Panel

After installation, add the Cosboard applet to your COSMIC panel:

1. Right-click on the COSMIC panel
2. Select "Panel Settings" or "Add Applet"
3. Find "Cosboard Applet" in the list
4. Add it to your desired panel location

The applet will appear as a keyboard icon in your system tray.

### Manual Installation

```bash
# Build release
cargo build --release

# Install binaries
sudo install -Dm0755 target/release/cosboard /usr/bin/cosboard
sudo install -Dm0755 target/release/cosboard-applet /usr/bin/cosboard-applet

# Install desktop entries
sudo install -Dm0644 resources/io.github.cosboard.Cosboard.desktop \
  /usr/share/applications/io.github.cosboard.Cosboard.desktop
sudo install -Dm0644 resources/io.github.cosboard.Cosboard.Applet.desktop \
  /usr/share/applications/io.github.cosboard.Cosboard.Applet.desktop

# Install icon
sudo install -Dm0644 resources/icons/hicolor/scalable/apps/io.github.cosboard.Cosboard.svg \
  /usr/share/icons/hicolor/scalable/apps/io.github.cosboard.Cosboard.svg

# Install appstream metadata
sudo install -Dm0644 resources/io.github.cosboard.Cosboard.metainfo.xml \
  /usr/share/appdata/io.github.cosboard.Cosboard.metainfo.xml

# Install D-Bus service file (enables auto-start)
sudo install -Dm0644 resources/io.github.cosboard.Cosboard.service \
  /usr/share/dbus-1/services/io.github.cosboard.Cosboard.service
```

## Uninstallation

### Using just

```bash
just uninstall
```

### Manual Uninstallation

```bash
sudo rm -f /usr/bin/cosboard /usr/bin/cosboard-applet
sudo rm -f /usr/share/applications/io.github.cosboard.Cosboard.desktop
sudo rm -f /usr/share/applications/io.github.cosboard.Cosboard.Applet.desktop
sudo rm -f /usr/share/icons/hicolor/scalable/apps/io.github.cosboard.Cosboard.svg
sudo rm -f /usr/share/appdata/io.github.cosboard.Cosboard.metainfo.xml
sudo rm -f /usr/share/dbus-1/services/io.github.cosboard.Cosboard.service
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

Control the keyboard programmatically:

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

## Configuration

### State Persistence

Window position and size are automatically saved using COSMIC's configuration system:

```
~/.config/cosmic/io.github.cosboard.Cosboard/v1/
```

### Default Window Settings

| Setting | Value |
|---------|-------|
| Default Size | 800x300 pixels |
| Minimum Size | 400x150 pixels |
| Resize Border | 8 pixels |

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
    └── icons/hicolor/scalable/apps/
        └── io.github.cosboard.Cosboard.svg
```

## Roadmap

Cosboard development is organized into phases:

### Phase 1: MVP - Core Keyboard Applet (Current)
- [x] Keyboard applet shell with window management
- [x] System tray toggle
- [ ] JSON layout parser
- [ ] Layout renderer
- [ ] Basic key input
- [ ] Default QWERTY layout

### Phase 2: Enhanced Key Actions
- Long-press detection and actions
- Swipe gesture support
- Key repeat behavior
- Haptic/audio feedback

### Phase 3: Scripting and Advanced Behavior
- Rhai scripting engine integration
- Script-bound keys
- Floating/draggable mode
- Theming support

### Phase 4: Prediction and Dictionary
- Word prediction engine
- Autocomplete integration
- User dictionary
- Learning mode

### Phase 5: Speech-to-Text
- ONNX runtime integration
- Parakeet v3 STT model
- Real-time transcription

### Phase 6: Touch Input Emulation
- Virtual mouse mode
- Gesture-to-mouse mapping
- Touchpad overlay

## Dependencies

| Crate | Purpose |
|-------|---------|
| `libcosmic` | COSMIC widget toolkit |
| `zbus` | D-Bus implementation |
| `tokio` | Async runtime |
| `futures` | Async utilities |
| `serde` | Serialization |
| `i18n-embed` | Internationalization |
| `tracing` | Logging |

## License

GPL-3.0-only

## Contributing

Contributions are welcome! Please see the project's issue tracker for current tasks and feature requests.
