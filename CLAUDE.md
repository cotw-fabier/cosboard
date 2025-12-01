# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Cosboard is a native soft keyboard for the COSMIC desktop environment, built with libcosmic/Iced. It provides a system tray applet that manages a Wayland layer-shell keyboard surface.

## Build Commands

```bash
# Debug build
cargo build

# Release build
cargo build --release
# or
just build-release

# Run the applet (release mode recommended for performance)
cargo run --release --bin cosboard-applet

# Run clippy with pedantic lints
just check

# Run tests
cargo test

# Install to system (requires sudo)
just install

# Install to user directories (no sudo)
./install-user.sh

# Reload applet after rebuilding (replaces binary, restarts panel)
./reload-applet.sh

# Uninstall from system
just uninstall
```

## Architecture

### Two-Component Design (Current: Single Process)

The current implementation runs as a single process:
- **cosboard-applet**: System tray applet that manages an integrated layer-shell keyboard surface

Key architectural decisions:
- Uses Wayland layer-shell protocol for overlay behavior (anchored to bottom of screen)
- Supports docked mode (exclusive zone, pushes windows up) and floating mode (draggable, resizable)
- State persistence via cosmic-config
- Localization via i18n-embed with fluent translations

### Key Modules

- `src/applet/mod.rs`: Main applet logic implementing `cosmic::Application` trait
- `src/layer_shell.rs`: Layer-shell configuration utilities for overlay behavior
- `src/state.rs`: Window state persistence (width, height, position, mode)
- `src/config.rs`: User configuration with cosmic_config
- `src/i18n.rs`: Localization support

### libcosmic Application Pattern

The applet implements `cosmic::Application` with:
- `init()`: Initialize state, load persisted config
- `update()`: Handle messages (Toggle, Show, Hide, drag/resize events)
- `view()`: Render applet icon button
- `view_window()`: Render keyboard layer surface and preview surfaces
- `subscription()`: Only subscribe to mouse events when dragging/resizing (critical for performance)
- `on_close_requested()`: Handle surface closes

### Layer Surface Management

Uses libcosmic's layer-shell commands:
- `get_layer_surface()`: Create new layer surface
- `destroy_layer_surface()`: Remove layer surface
- `set_anchor()`, `set_size()`, `set_margin()`, `set_exclusive_zone()`: Configure positioning

## Standards

The project follows standards documented in `agent-os/standards/`:
- Microsoft Rust guidelines (M-SMALLER-CRATES, M-ERRORS-CANONICAL-STRUCTS, etc.)
- libcosmic patterns for Application trait, theming, state management
- See `agent-os/standards/STANDARDS_SUMMARY.md` for complete reference

## Key Patterns

### Performance-Critical Subscription Pattern

The applet must return `Subscription::none()` when idle. Only subscribe to mouse events during active drag/resize operations to maintain instant responsiveness:

```rust
fn subscription(&self) -> Subscription<Message> {
    if self.is_dragging || self.resize_edge.is_some() {
        event::listen_with(...)
    } else {
        Subscription::none()
    }
}
```

### Preview Surface Pattern

During drag/resize, a lightweight preview surface shows the target bounds while the actual keyboard surface remains unchanged. Final values are applied only when the operation ends.

### D-Bus Interface (Planned)

- Service: `io.github.cosboard.Cosboard`
- Methods: `Show()`, `Hide()`, `Toggle()`, `Quit()`
- Signal: `VisibilityChanged(visible: bool)`

## Dependencies

- `libcosmic`: COSMIC widget toolkit with applet and layer-shell support
- `tokio`: Async runtime
- `serde`: Configuration serialization
- `i18n-embed`: Internationalization
- `tracing`: Logging

## Rust Edition

This project uses Rust 2024 edition.
