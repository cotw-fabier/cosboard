# Cosboard

**A powerful, customizable soft keyboard for the COSMIC desktop**

## Not yet Ready for Use

This library is under development and doesn't yet fill its role. This is a hobby project with the intention to create a software keyboard in Cosmic Desktop Environment with the same capabilities as most commercial mobile keyboards.

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

- System tray applet with integrated keyboard layer surface
- Docked mode (exclusive zone - pushes windows up) and floating mode
- Drag and resize support in floating mode with preview surface
- Window state persistence (size, position, mode)
- Left-click to toggle keyboard, right-click for popup menu

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

# Run the applet (manages keyboard surface directly)
cargo run --release --bin cosboard-applet
```

### After Building

```bash
./target/release/cosboard-applet
```

## Architecture

Cosboard runs as a single-process applet that manages an integrated keyboard layer surface:

```
┌───────────────────────────────────────────────────┐
│                 cosboard-applet                   │
│  ┌─────────────────┐    ┌──────────────────────┐  │
│  │  Panel Icon     │    │  Keyboard Surface    │  │
│  │  (System Tray)  │───►│  (Layer Shell)       │  │
│  └─────────────────┘    └──────────────────────┘  │
└───────────────────────────────────────────────────┘
         │                         │
         ▼                         ▼
    ┌─────────┐            ┌─────────────────┐
    │  Panel  │            │ Desktop Surface │
    └─────────┘            │ (Overlay Layer) │
                           └─────────────────┘
```

The applet uses Wayland layer-shell protocol for the keyboard surface:
- **Docked mode**: Anchored to bottom, exclusive zone pushes windows up
- **Floating mode**: Anchored to bottom-right with margins, draggable/resizable

### D-Bus Interface (Planned)

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

```bash
# Development (release mode recommended for performance)
cargo run --release --bin cosboard-applet

# After building
./target/release/cosboard-applet
```

## Installation

### User Installation (Recommended)

```bash
./install-user.sh
```

This installs to user directories (no sudo required):
- Binary to `~/.local/bin/cosboard-applet`
- Desktop entry to `~/.local/share/applications/`
- Icon to `~/.local/share/icons/hicolor/scalable/apps/`
- AppStream metadata to `~/.local/share/metainfo/`

### System Installation

```bash
just install
```

This installs to system directories (requires sudo):
- Binary to `/usr/bin/cosboard-applet`
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

# Install binary
sudo install -Dm0755 target/release/cosboard-applet /usr/bin/cosboard-applet

# Install desktop entry
sudo install -Dm0644 resources/io.github.cosboard.Cosboard.Applet.desktop \
  /usr/share/applications/io.github.cosboard.Cosboard.Applet.desktop

# Install icon
sudo install -Dm0644 resources/icons/hicolor/scalable/apps/io.github.cosboard.Cosboard.svg \
  /usr/share/icons/hicolor/scalable/apps/io.github.cosboard.Cosboard.svg

# Install appstream metadata
sudo install -Dm0644 resources/io.github.cosboard.Cosboard.metainfo.xml \
  /usr/share/appdata/io.github.cosboard.Cosboard.metainfo.xml
```

## Uninstallation

### System Uninstall

```bash
just uninstall
```

### User Uninstall

```bash
rm -f ~/.local/bin/cosboard-applet
rm -f ~/.local/share/applications/io.github.cosboard.Cosboard.Applet.desktop
rm -f ~/.local/share/icons/hicolor/scalable/apps/io.github.cosboard.Cosboard.svg
rm -f ~/.local/share/metainfo/io.github.cosboard.Cosboard.metainfo.xml
```

## Usage

### From Panel

After installation:
1. Right-click on the COSMIC panel
2. Add the Cosboard applet to your panel
3. Left-click the keyboard icon to toggle visibility
4. Right-click for options (show/hide, toggle mode, quit)

### From Command Line

```bash
cosboard-applet
```

### Development Workflow

After making changes, reload the applet without restarting your session:

```bash
# Build and reload in one step
cargo build --release --bin cosboard-applet && ./reload-applet.sh
```

## Configuration

### State Persistence

Window state is automatically saved using COSMIC's configuration system:

```
~/.config/cosmic/io.github.cosboard.Cosboard.Applet/v1/
```

### Default Window Settings

| Setting | Value |
|---------|-------|
| Default Size | 800x300 pixels |
| Min/Max Width | 300-1920 pixels |
| Min/Max Height | 150-500 pixels |
| Resize Zone | 16 pixels |

## Project Structure

```
cosboard/
├── Cargo.toml           # Package configuration
├── justfile             # Build automation recipes
├── install-user.sh      # User installation script
├── reload-applet.sh     # Development reload script
├── src/
│   ├── lib.rs           # Library crate with shared modules
│   ├── app_settings.rs  # Centralized constants
│   ├── config.rs        # User configuration
│   ├── state.rs         # Window state persistence
│   ├── layer_shell.rs   # Wayland layer-shell utilities
│   ├── i18n.rs          # Localization support
│   ├── applet/
│   │   └── mod.rs       # System tray applet with keyboard surface
│   └── bin/
│       └── applet.rs    # Applet binary entry point
├── i18n/
│   └── en/
│       └── cosboard.ftl # English translations
└── resources/
    ├── io.github.cosboard.Cosboard.Applet.desktop
    ├── io.github.cosboard.Cosboard.metainfo.xml
    └── icons/hicolor/scalable/apps/
        └── io.github.cosboard.Cosboard.svg
```

## Roadmap

Cosboard development is organized into phases:

### Phase 1: MVP - Core Keyboard Applet (Current)
- [x] Keyboard applet shell with window management
- [x] System tray toggle with popup menu
- [x] Docked and floating modes with drag/resize
- [x] Window state persistence
- [ ] JSON layout parser
- [ ] Layout renderer
- [ ] Basic key input (virtual keyboard protocol)
- [ ] Default QWERTY layout

### Phase 2: Enhanced Key Actions
- Long-press detection and actions
- Swipe gesture support
- Key repeat behavior
- Haptic/audio feedback

### Phase 3: Scripting and Advanced Behavior
- Rhai scripting engine integration
- Script-bound keys
- Theming support
- D-Bus interface for external control

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
| `libcosmic` | COSMIC widget toolkit with applet and layer-shell support |
| `tokio` | Async runtime |
| `futures` | Async utilities |
| `serde` | Configuration serialization |
| `i18n-embed` | Internationalization |
| `tracing` | Logging |

## License

GPL-3.0-only

## Contributing

Contributions are welcome! Please see the project's issue tracker for current tasks and feature requests.
