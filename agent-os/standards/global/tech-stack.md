# Technology Stack

## Overview

This document defines the core technology stack for building COSMIC desktop applications. All technologies are chosen for their reliability, performance, and alignment with the COSMIC desktop ecosystem.

## Core Stack

### Language: Rust

**Version**: Stable (latest)

Rust is the foundation of all COSMIC applications. Key benefits:
- Memory safety without garbage collection
- Zero-cost abstractions
- Fearless concurrency
- Excellent tooling ecosystem

**Toolchain Components**:
- `rustup` - Toolchain management
- `cargo` - Package manager and build system
- `rustfmt` - Code formatting
- `clippy` - Linting and static analysis

### UI Framework: libcosmic

**Repository**: pop-os/libcosmic

libcosmic is the official UI toolkit for COSMIC desktop applications, built on top of Iced.

**Key Features**:
- Native COSMIC look and feel
- Integrated theming system
- Navigation patterns (nav_bar, header_bar)
- Rich widget library
- Configuration system integration

**Core Dependencies**:
```toml
[dependencies]
cosmic = { git = "https://github.com/pop-os/libcosmic" }
```

### Underlying Framework: Iced

libcosmic is built on Iced, a cross-platform GUI library for Rust.

**Iced Fundamentals**:
- Elm-inspired architecture (Model-View-Update)
- Reactive UI with messages
- GPU-accelerated rendering
- Async-first design

### Theming: cosmic-theme

Provides the COSMIC design system:
- Color palette (light/dark/high-contrast modes)
- Spacing system (space_xxs through space_xxl)
- Typography scales
- Border radius standards
- Semantic color components

### Configuration: cosmic-config

Persistent application settings:
- XDG-compliant configuration storage
- Type-safe configuration entries
- Live configuration watching
- Automatic serialization/deserialization

### Async Runtime: Tokio

**Version**: Latest stable

Tokio provides the async runtime for I/O operations:
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

**Use Cases**:
- File I/O operations
- Network requests
- Background processing
- Timer/interval operations

### Serialization: serde

**Version**: Latest stable

Standard serialization framework:
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**Use Cases**:
- Configuration files
- Data persistence
- IPC communication
- API payloads

## Project Structure

Standard COSMIC application structure:

```
my-cosmic-app/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point, app::run()
│   ├── app.rs            # Application trait implementation
│   ├── config.rs         # CosmicConfigEntry definitions
│   ├── message.rs        # Message enum
│   ├── pages/            # Multi-page navigation views
│   │   ├── mod.rs
│   │   ├── home.rs
│   │   └── settings.rs
│   └── widgets/          # Custom widgets
│       └── mod.rs
├── i18n/                 # Internationalization
│   └── en/
│       └── app.ftl
└── res/                  # Resources
    └── icons/
```

## Cargo.toml Template

```toml
[package]
name = "my-cosmic-app"
version = "0.1.0"
edition = "2021"
license = "GPL-3.0"

[dependencies]
# Core COSMIC
cosmic = { git = "https://github.com/pop-os/libcosmic" }

# Async runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }

# Error handling
thiserror = "1"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

[profile.release]
lto = true
codegen-units = 1
strip = true
```

## Feature Flags

libcosmic provides optional features:

| Feature | Description |
|---------|-------------|
| `multi-window` | Multiple window support |
| `tokio` | Tokio async runtime integration |
| `smol` | Smol async runtime (alternative) |
| `wayland` | Wayland-specific features |
| `xdg-portal` | XDG portal integration |
| `winit` | Winit window management |

Enable features as needed:
```toml
cosmic = { git = "...", features = ["tokio", "wayland"] }
```

## Development Tools

### Required
- `rustfmt` - Format code before commits
- `clippy` - Lint code for common issues
- `cargo-audit` - Check dependencies for vulnerabilities

### Recommended
- `cargo-watch` - Auto-rebuild on file changes
- `cargo-expand` - View macro expansions
- `cargo-flamegraph` - Performance profiling

## Best Practices Checklist

- [ ] Use latest stable Rust toolchain
- [ ] Pin cosmic dependency to specific commit for reproducibility
- [ ] Enable appropriate libcosmic features only
- [ ] Configure release profile optimizations
- [ ] Include tracing/logging for debugging
- [ ] Follow COSMIC app project structure
- [ ] Use cosmic-config for persistent settings

## References

- [libcosmic Documentation](https://pop-os.github.io/libcosmic/cosmic/)
- [libcosmic Book](https://pop-os.github.io/libcosmic-book/)
- [Iced Documentation](https://docs.iced.rs/)
- [COSMIC App Template](https://github.com/pop-os/cosmic-app-template)
- [Tokio Documentation](https://tokio.rs/)
