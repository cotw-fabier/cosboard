# Documentation and Comments

## Overview

This document defines documentation standards for Rust code following Microsoft's guidelines. Well-documented code improves maintainability, enables better tooling, and helps new contributors understand the codebase.

## Documentation Philosophy

- **Self-documenting code**: Write clear code with descriptive names; comments explain "why", not "what"
- **Doc comments for public APIs**: Always document public-facing items
- **First sentence matters**: Summary line appears in module listings
- **Examples help**: Include usage examples for non-trivial APIs
- **No stale comments**: Remove outdated comments immediately

## Doc Comments

### Item Documentation (`///`)

```rust
/// Loads the application configuration from disk.
///
/// Configuration is read from the XDG config directory, typically
/// `~/.config/cosmic/org.cosmic.MyApp/`. If no configuration exists,
/// returns the default configuration.
///
/// # Errors
///
/// Returns `ConfigError::IoError` if the configuration file exists
/// but cannot be read.
///
/// # Examples
///
/// ```rust
/// let config = AppConfig::load()?;
/// println!("Theme: {:?}", config.theme_mode);
/// ```
pub fn load() -> Result<AppConfig, ConfigError> {
    // Implementation
}
```

### Module Documentation (`//!`)

Every public module requires documentation (M-MODULE-DOCS):

```rust
//! Application configuration module.
//!
//! This module provides type-safe configuration using `cosmic-config`.
//! Configuration is automatically persisted to the XDG config directory
//! and supports live watching for external changes.
//!
//! # Usage
//!
//! ```rust
//! use my_app::config::AppConfig;
//!
//! // Load configuration
//! let config = AppConfig::load()?;
//!
//! // Watch for changes
//! let watcher = config.watch(|new_config| {
//!     println!("Config changed!");
//! });
//! ```
//!
//! # Configuration Location
//!
//! - Linux: `~/.config/cosmic/org.cosmic.MyApp/`
//! - macOS: `~/Library/Application Support/org.cosmic.MyApp/`

use cosmic_config::CosmicConfigEntry;
```

## First Sentence Rule (M-FIRST-DOC-SENTENCE)

The first sentence becomes the summary in module listings. Keep it:
- **Single line** (approximately 15 words max)
- **Complete sentence** ending with a period
- **Action-oriented** for functions

```rust
// GOOD: Concise first sentence
/// Loads user preferences from the configuration file.
///
/// This function reads the TOML configuration from the standard
/// XDG config directory and deserializes it into a Preferences struct.
pub fn load_preferences() -> Result<Preferences> {
    // ...
}

// BAD: First sentence too long
/// This function is responsible for loading the user's preferences from
/// the configuration file located in the XDG config directory.
pub fn load_preferences() -> Result<Preferences> {
    // ...
}

// BAD: Missing period
/// Loads user preferences from the configuration file
pub fn load_preferences() -> Result<Preferences> {
    // ...
}
```

## Section Headers

Use standard section headers for comprehensive documentation:

```rust
/// Creates a new database connection pool.
///
/// # Arguments
///
/// * `config` - Database configuration including connection string
/// * `max_connections` - Maximum number of pooled connections
///
/// # Returns
///
/// A configured connection pool ready for use.
///
/// # Errors
///
/// * `DbError::ConnectionFailed` - If initial connection cannot be established
/// * `DbError::ConfigInvalid` - If configuration values are invalid
///
/// # Panics
///
/// Panics if `max_connections` is zero.
///
/// # Safety
///
/// This function is safe to call from multiple threads.
///
/// # Examples
///
/// ```rust
/// let pool = ConnectionPool::new(&config, 10)?;
/// let conn = pool.get().await?;
/// ```
pub fn new(config: &DbConfig, max_connections: u32) -> Result<Self> {
    assert!(max_connections > 0, "max_connections must be positive");
    // ...
}
```

## What to Document

### Always Document

- **Public functions and methods**
- **Public structs and enums**
- **Public modules**
- **Complex algorithms**
- **Safety requirements for unsafe code**
- **Non-obvious behavior**

### When to Add Implementation Comments

```rust
impl DataProcessor {
    fn process(&mut self, data: &[u8]) -> Result<ProcessedData> {
        // Use a ring buffer here instead of Vec to avoid allocations
        // in the hot path. Benchmarks showed 15% improvement.
        let buffer = RingBuffer::with_capacity(1024);

        // SAFETY: We verified data.len() >= HEADER_SIZE above
        let header = unsafe {
            std::ptr::read_unaligned(data.as_ptr() as *const Header)
        };

        // Skip processing if data is stale (older than 5 minutes)
        // This prevents replay attacks - see security review #123
        if header.timestamp < self.cutoff_time {
            return Ok(ProcessedData::empty());
        }

        // ...
    }
}
```

### When NOT to Comment

```rust
// BAD: Redundant comment
// Increment the counter
counter += 1;

// BAD: Obvious from code
// Check if user is null
if user.is_none() {
    return;
}

// BAD: Commented-out code (use version control)
// fn old_implementation() {
//     // ...
// }

// GOOD: No comment needed - code is self-explanatory
let active_users = users.iter().filter(|u| u.is_active()).count();
```

## libcosmic-Specific Documentation

### Application Trait

```rust
/// Main application state and logic.
///
/// Implements the COSMIC application lifecycle including initialization,
/// message handling, and view rendering.
pub struct App {
    /// COSMIC runtime core - manages window, theme, and system integration.
    core: Core,

    /// Navigation model for sidebar navigation.
    nav_model: nav_bar::Model,

    /// Current page being displayed.
    current_page: Page,
}
```

### Message Enum

```rust
/// Messages that trigger state updates in the application.
///
/// Messages follow the Elm architecture pattern - user actions and
/// system events are represented as variants, processed in `update()`.
#[derive(Debug, Clone)]
pub enum Message {
    /// User clicked a navigation item.
    NavSelect(nav_bar::Id),

    /// Theme changed (system or user preference).
    ThemeChanged(cosmic::theme::ThemeType),

    /// Configuration was updated externally.
    ConfigUpdated(AppConfig),

    /// Async data load completed.
    DataLoaded(Result<Vec<Item>, String>),

    /// Window surface action (minimize, maximize, etc.).
    Surface(cosmic::surface::Action),
}
```

### Widget Functions

```rust
/// Renders the main content area.
///
/// Content changes based on the currently selected navigation item.
/// Returns a loading indicator while data is being fetched.
fn view_content(&self) -> Element<Message> {
    // ...
}
```

## TODO Comments

Use issue tracking instead of TODO comments when possible:

```rust
// ACCEPTABLE: Temporary, with issue reference
// TODO(#123): Replace with async implementation

// AVOID: Permanent TODO without tracking
// TODO: Add error handling here
```

## Best Practices Checklist

- [ ] All public items have doc comments
- [ ] First sentence is concise (< 15 words)
- [ ] All modules have `//!` documentation
- [ ] Examples included for complex APIs
- [ ] No commented-out code
- [ ] No redundant comments
- [ ] Safety sections for unsafe code
- [ ] Error conditions documented

## Generating Documentation

```bash
# Generate and open documentation
cargo doc --open

# Include private items
cargo doc --document-private-items

# Check for documentation warnings
RUSTDOCFLAGS="-D warnings" cargo doc
```

## References

- [Rust Documentation Guide](https://doc.rust-lang.org/rustdoc/)
- [RFC 1574 - API Documentation Conventions](https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html)
- [Microsoft Rust Guidelines - Documentation](https://github.com/AzureMessaging/guidelines-rust)
