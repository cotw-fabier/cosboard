# Project Conventions

## Overview

This document defines project organization, crate structure, and development conventions for COSMIC desktop applications built with Rust and libcosmic.

## Project Structure

Standard COSMIC application structure:

```
my-cosmic-app/
├── Cargo.toml              # Project manifest
├── Cargo.lock              # Locked dependencies (commit this)
├── src/
│   ├── main.rs             # Entry point, app::run()
│   ├── app.rs              # Application trait implementation
│   ├── config.rs           # CosmicConfigEntry definitions
│   ├── message.rs          # Message enum for state updates
│   ├── core_state.rs       # Application state struct
│   ├── pages/              # Multi-page navigation views
│   │   ├── mod.rs
│   │   ├── home.rs
│   │   └── settings.rs
│   ├── widgets/            # Custom widget implementations
│   │   └── mod.rs
│   └── localize.rs         # i18n helper functions
├── i18n/                   # Internationalization
│   ├── en/
│   │   └── app.ftl
│   └── es/
│       └── app.ftl
├── res/                    # Resources
│   ├── icons/
│   │   └── app-icon.svg
│   └── app.desktop         # Desktop entry file
├── tests/                  # Integration tests
│   └── integration_test.rs
└── benches/                # Benchmarks
    └── benchmark.rs
```

## Crate Organization (M-SMALLER-CRATES)

Prefer smaller, focused crates over monolithic ones:

```
# Good: Split by domain
my-cosmic-app/              # Main application
my-cosmic-app-core/         # Core business logic
my-cosmic-app-widgets/      # Custom reusable widgets
my-cosmic-app-config/       # Configuration types

# Workspace Cargo.toml
[workspace]
members = [
    "my-cosmic-app",
    "my-cosmic-app-core",
    "my-cosmic-app-widgets",
    "my-cosmic-app-config",
]
```

**Benefits:**
- Faster incremental compilation
- Prevents cyclic dependencies
- Clearer API boundaries
- Easier testing

**When to Split:**
- Submodule can be used independently
- Build times exceed acceptable threshold
- Clear domain boundary exists

## Module Organization

### Public API Pattern

```rust
// src/lib.rs - Re-export public API
pub mod app;
pub mod config;
pub mod message;

// Individual re-exports (M-NO-GLOB-REEXPORTS)
pub use app::App;
pub use config::AppConfig;
pub use message::Message;

// BAD: Glob re-exports leak internal types
// pub use app::*;
```

### Module Documentation (M-MODULE-DOCS)

Every public module requires documentation:

```rust
//! Application configuration module.
//!
//! This module provides type-safe configuration using cosmic-config.
//! Configuration is automatically persisted to `~/.config/cosmic/`.
//!
//! # Usage
//!
//! ```rust
//! use my_app::config::AppConfig;
//!
//! let config = AppConfig::default();
//! ```

use cosmic_config::CosmicConfigEntry;

#[derive(CosmicConfigEntry)]
pub struct AppConfig {
    // ...
}
```

## Version Control

### Commit Messages

Use conventional commits:

```
feat: add dark mode toggle
fix: resolve navigation crash on empty state
refactor: extract widget into separate module
docs: update README with build instructions
perf: optimize list rendering
test: add integration tests for config loading
chore: update dependencies
```

### What to Commit

**Always commit:**
- `Cargo.lock` (for applications, ensures reproducible builds)
- Source code changes
- Configuration files
- Documentation

**Never commit:**
- `target/` directory
- IDE settings (`.idea/`, `.vscode/` unless shared)
- Environment files with secrets (`.env`)
- Build artifacts

### .gitignore

```gitignore
# Build artifacts
/target/

# IDE
.idea/
.vscode/
*.swp
*.swo

# Environment
.env
.env.local

# OS files
.DS_Store
Thumbs.db

# Coverage
coverage/
*.profraw
*.profdata
```

## Dependency Management

### Cargo.toml Best Practices

```toml
[dependencies]
# Pin to git commit for reproducibility
cosmic = { git = "https://github.com/pop-os/libcosmic", rev = "abc123" }

# Use minimal features
tokio = { version = "1", default-features = false, features = ["rt", "sync"] }
serde = { version = "1", features = ["derive"] }

# Document why each dependency exists
thiserror = "1"  # Error handling derive macros
tracing = "0.1"  # Structured logging

[dev-dependencies]
criterion = "0.5"  # Benchmarking
```

### Feature Flags

```toml
[features]
default = []
test-util = []  # Testing utilities (M-TEST-UTIL)
multi-window = ["cosmic/multi-window"]

# Document features
# test-util: Enables mocking and test helpers
# multi-window: Support for multiple application windows
```

## Avoid Static State (M-AVOID-STATICS)

Prefer dependency injection over static/global state:

```rust
// BAD: Static state
static CONFIG: Lazy<AppConfig> = Lazy::new(|| AppConfig::default());

// GOOD: Pass state through
pub struct App {
    config: AppConfig,
    core: Core,
}

impl App {
    pub fn new(config: AppConfig, core: Core) -> Self {
        Self { config, core }
    }
}
```

**Why avoid statics:**
- Multiple crate versions can duplicate state
- Interferes with testing
- Makes dependencies implicit
- Complicates multi-threading

## Re-exports (M-DOC-INLINE)

```rust
// Mark inline re-exports for better docs
#[doc(inline)]
pub use config::AppConfig;

// Don't inline external types - make origin clear
pub use cosmic::widget::button;
```

## Build Configuration

### Release Profile

```toml
[profile.release]
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Smaller binary
panic = "abort"      # Smaller binary, faster

[profile.dev]
# Keep debug symbols for better error messages
debug = true

[profile.bench]
debug = 1  # Enable symbols for profiling
```

### Platform Support (M-OOBE)

Application should build on Tier 1 platforms without extra setup:

```rust
// Use conditional compilation for platform differences
#[cfg(target_os = "linux")]
fn get_config_dir() -> PathBuf {
    // XDG standard
}

#[cfg(target_os = "macos")]
fn get_config_dir() -> PathBuf {
    // macOS standard
}
```

## Code Review Checklist

- [ ] Follows module documentation requirements
- [ ] No glob re-exports (`pub use foo::*`)
- [ ] No unnecessary static state
- [ ] Dependencies documented
- [ ] Features properly gated
- [ ] Builds on all target platforms
- [ ] Commit message follows convention

## Best Practices Checklist

- [ ] Use workspace for multi-crate projects
- [ ] Document all public modules
- [ ] Pin dependencies to specific versions/commits
- [ ] Use minimal feature sets
- [ ] Avoid global state
- [ ] Follow conventional commit format
- [ ] Configure release profile optimizations

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Cargo Book - Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [COSMIC App Template](https://github.com/pop-os/cosmic-app-template)
