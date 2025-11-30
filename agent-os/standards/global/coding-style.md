# Coding Style

## Overview

This document defines coding style conventions for Rust and libcosmic applications, ensuring consistency and readability across the codebase.

## Naming Conventions

### Files and Modules

```rust
// Files: snake_case.rs
// user_service.rs
// config_manager.rs
// main.rs

// Modules: snake_case
mod user_service;
mod config_manager;
```

### Types

```rust
// Structs: PascalCase
struct UserProfile { ... }
struct AppConfig { ... }

// Enums: PascalCase with PascalCase variants
enum Message {
    ButtonPressed,
    InputChanged(String),
    NavigationSelected(nav_bar::Id),
}

// Traits: PascalCase
trait Renderable { ... }

// Type aliases: PascalCase
type Result<T> = std::result::Result<T, AppError>;
```

### Functions and Methods

```rust
// Functions: snake_case
fn load_user_profile() -> UserProfile { ... }
fn calculate_total(items: &[Item]) -> f64 { ... }

// Methods: snake_case
impl UserService {
    fn get_current_user(&self) -> &User { ... }
    fn update_profile(&mut self, profile: UserProfile) { ... }
}
```

### Variables and Constants

```rust
// Variables: snake_case
let current_user = get_user();
let is_authenticated = true;
let item_count = items.len();

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRIES: u32 = 3;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const APP_ID: &str = "org.cosmic.MyApp";

// Static: SCREAMING_SNAKE_CASE (avoid when possible)
static GLOBAL_CONFIG: Lazy<Config> = Lazy::new(Config::default);
```

### libcosmic Specific

```rust
// Application ID: reverse domain notation
const APP_ID: &str = "org.cosmic.MyAppName";

// Message enum: PascalCase with descriptive variants
enum Message {
    // User actions
    OpenFile,
    SaveDocument,

    // State updates
    ConfigUpdated(AppConfig),
    ThemeChanged(cosmic::theme::ThemeType),

    // Navigation
    NavSelect(nav_bar::Id),

    // Window operations
    Surface(cosmic::surface::Action),
}
```

## Automated Formatting

### rustfmt

Always format code with rustfmt before committing:

```bash
# Format all files
cargo fmt

# Check formatting in CI
cargo fmt -- --check
```

### rustfmt.toml (Optional)

```toml
# Optional customizations (defaults are usually fine)
edition = "2021"
max_width = 100
tab_spaces = 4
use_small_heuristics = "Default"
```

## Static Analysis (M-STATIC-VERIFICATION)

### Compiler Lints

Enable in `Cargo.toml`:

```toml
[lints.rust]
# Warn on common issues
missing_debug_implementations = "warn"
trivial_numeric_casts = "warn"
unsafe_op_in_unsafe_fn = "warn"
unused_lifetimes = "warn"
redundant_lifetimes = "warn"
```

### Clippy Configuration

```toml
[lints.clippy]
# Enable major lint categories
cargo = { level = "warn", priority = -1 }
complexity = { level = "warn", priority = -1 }
correctness = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
perf = { level = "warn", priority = -1 }
style = { level = "warn", priority = -1 }
suspicious = { level = "warn", priority = -1 }

# Specific lints
clone_on_ref_ptr = "warn"
dbg_macro = "warn"
print_stdout = "warn"
todo = "warn"
unimplemented = "warn"

# Allow some pedantic lints that reduce readability
module_name_repetitions = "allow"
too_many_lines = "allow"
```

Run clippy:

```bash
cargo clippy --all-targets --all-features
```

## File Organization

### Import Order

```rust
// 1. Standard library
use std::collections::HashMap;
use std::path::PathBuf;

// 2. External crates
use cosmic::app::{Core, Task};
use cosmic::iced::Length;
use cosmic::widget::{button, text};
use serde::{Deserialize, Serialize};
use tokio::fs;

// 3. Crate modules
use crate::config::AppConfig;
use crate::message::Message;

// 4. Super/self imports
use super::widgets::CustomButton;
```

### Module Layout

```rust
// src/app.rs

//! Application module documentation.

// Imports
use cosmic::app::{Core, Task};
use cosmic::Application;

// Constants
const DEFAULT_WIDTH: f32 = 800.0;

// Type definitions
pub type Result<T> = std::result::Result<T, AppError>;

// Main struct
pub struct App {
    core: Core,
    // ...
}

// Trait implementations
impl Application for App {
    // ...
}

// Inherent implementations
impl App {
    pub fn new(core: Core) -> Self {
        // ...
    }
}

// Private helpers
fn helper_function() {
    // ...
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }
}
```

## Code Patterns

### Match Expressions

```rust
// Exhaustive matching
match message {
    Message::ButtonPressed => {
        // Handle
        Task::none()
    }
    Message::InputChanged(value) => {
        self.input = value;
        Task::none()
    }
    Message::NavSelect(id) => {
        self.nav_model.activate(id);
        Task::none()
    }
}

// Single-line for simple cases
let result = match option {
    Some(v) => v,
    None => default_value,
};
```

### Error Handling with `?`

```rust
// Use ? for error propagation
fn load_config() -> Result<AppConfig> {
    let path = get_config_path()?;
    let contents = std::fs::read_to_string(&path)?;
    let config = serde_json::from_str(&contents)?;
    Ok(config)
}
```

### Builder Pattern

```rust
// Method chaining for widget construction
cosmic::widget::button::text("Click me")
    .on_press(Message::ButtonPressed)
    .width(Length::Fixed(200.0))
    .height(Length::Fixed(40.0))
```

## Widget Code Style

### View Functions

```rust
fn view(&self) -> Element<Message> {
    let header = self.view_header();
    let content = self.view_content();
    let footer = self.view_footer();

    cosmic::widget::column()
        .push(header)
        .push(content)
        .push(footer)
        .spacing(cosmic::theme::spacing().space_m)
        .into()
}

fn view_header(&self) -> Element<Message> {
    cosmic::widget::text::heading("My App")
        .into()
}
```

### Avoid Deep Nesting

```rust
// BAD: Deeply nested
fn view(&self) -> Element<Message> {
    column()
        .push(
            row()
                .push(
                    container(
                        column()
                            .push(text("Title"))
                            .push(text("Subtitle"))
                    )
                )
        )
        .into()
}

// GOOD: Extract into helper functions
fn view(&self) -> Element<Message> {
    column()
        .push(self.view_header())
        .into()
}

fn view_header(&self) -> Element<Message> {
    let title_section = column()
        .push(text("Title"))
        .push(text("Subtitle"));

    row()
        .push(container(title_section))
        .into()
}
```

## Modern Rust Features

### Use These

```rust
// ? operator for error handling
let data = file.read_to_string()?;

// Pattern matching
if let Some(value) = option {
    // use value
}

// Iterator methods
let names: Vec<_> = users.iter().map(|u| &u.name).collect();

// Closures
items.filter(|item| item.is_active())

// impl Trait
fn get_items() -> impl Iterator<Item = Item> {
    // ...
}

// const fn for compile-time evaluation
const fn default_timeout() -> Duration {
    Duration::from_secs(30)
}
```

## Remove Dead Code

```rust
// BAD: Commented code
// fn old_function() {
//     // ...
// }

// BAD: Unused imports
use std::collections::HashMap;  // If not used

// GOOD: Remove entirely, use version control for history
```

## Best Practices Checklist

- [ ] All code formatted with `cargo fmt`
- [ ] All clippy warnings addressed
- [ ] Consistent naming conventions
- [ ] Imports organized by category
- [ ] No dead code or commented-out code
- [ ] Deeply nested code extracted into functions
- [ ] Modern Rust idioms used

## References

- [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
