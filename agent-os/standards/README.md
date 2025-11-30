# Rust + libcosmic Development Standards

This directory contains development standards for building COSMIC desktop applications using Rust and libcosmic, based on Microsoft's Rust guidelines and libcosmic best practices.

## Architecture Overview

```
┌────────────────────────────────────────────────┐
│          COSMIC Application (Rust)             │
│                                                │
│  ┌──────────────────────────────────────────┐ │
│  │     libcosmic (UI Framework)             │ │
│  │  - Iced rendering                        │ │
│  │  - COSMIC widgets                        │ │
│  │  - Theme system                          │ │
│  └──────────────┬───────────────────────────┘ │
│                 │                              │
│  ┌──────────────▼───────────────────────────┐ │
│  │     Application Logic                    │ │
│  │  - Elm architecture (MVU)                │ │
│  │  - Core state management                 │ │
│  │  - Message handling                      │ │
│  └──────────────┬───────────────────────────┘ │
│                 │                              │
│  ┌──────────────▼───────────────────────────┐ │
│  │     cosmic-config                        │ │
│  │  - Settings persistence                  │ │
│  │  - Live config watching                  │ │
│  └──────────────────────────────────────────┘ │
└────────────────────────────────────────────────┘
```

## Directory Structure

### Global Standards (7 files)
- **[tech-stack.md](./global/tech-stack.md)** - Rust, libcosmic, Tokio, serde technology stack
- **[conventions.md](./global/conventions.md)** - Project structure, crate organization, module patterns
- **[coding-style.md](./global/coding-style.md)** - Rust naming conventions, rustfmt, clippy configuration
- **[error-handling.md](./global/error-handling.md)** - Canonical error structs, panic vs Result
- **[commenting.md](./global/commenting.md)** - Doc comments, 15-word first sentence rule
- **[validation.md](./global/validation.md)** - Strong types, input validation, type safety
- **[performance.md](./global/performance.md)** - Hot path identification, benchmarking, optimization

### Application Standards (4 files)
- **[lifecycle.md](./app/lifecycle.md)** - Application trait, Core state, init/update/view pattern
- **[components.md](./app/components.md)** - libcosmic widgets, buttons, inputs, layouts
- **[theming.md](./app/theming.md)** - cosmic-theme, colors, spacing, dark/light mode
- **[accessibility.md](./app/accessibility.md)** - Keyboard navigation, screen readers, WCAG compliance

### API Design Standards (4 files)
- **[design.md](./api/design.md)** - Strong types, impl AsRef, builder pattern, API guidelines
- **[models.md](./api/models.md)** - Data structures, serde, CosmicConfigEntry
- **[async.md](./api/async.md)** - Tokio, Task-based async, subscriptions
- **[state-management.md](./api/state-management.md)** - Core state, nav_bar, configuration persistence

### Safety Standards (2 files)
- **[memory.md](./safety/memory.md)** - Ownership, unsafe guidelines, soundness requirements
- **[verification.md](./safety/verification.md)** - Compiler lints, clippy, miri, cargo-audit

### Testing Standards (1 file)
- **[test-writing.md](./testing/test-writing.md)** - test-util feature, mockable I/O, unit/integration tests

## Quick Start

### Creating a New COSMIC Application

1. **Use COSMIC App Template:**
   ```bash
   git clone https://github.com/pop-os/cosmic-app-template my-cosmic-app
   cd my-cosmic-app
   ```

2. **Or Start from Scratch:**
   ```bash
   cargo new my-cosmic-app
   cd my-cosmic-app
   ```

3. **Add Dependencies (Cargo.toml):**
   ```toml
   [package]
   name = "my-cosmic-app"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   cosmic = { git = "https://github.com/pop-os/libcosmic" }
   tokio = { version = "1", features = ["rt", "sync"] }
   serde = { version = "1", features = ["derive"] }
   thiserror = "1"

   [profile.release]
   lto = true
   codegen-units = 1
   strip = true
   ```

4. **Create Basic Application Structure:**
   ```
   src/
   ├── main.rs           # Entry point
   ├── app.rs            # Application trait implementation
   ├── message.rs        # Message enum
   └── config.rs         # Configuration
   ```

## Core Principles

### 1. Memory Safety
- **Ownership**: Understand move semantics and borrowing
- **Avoid unsafe**: Only use unsafe when absolutely necessary with SAFETY comments
- **RAII**: Resources cleaned up via Drop trait
- **Test with Miri**: Validate unsafe code

### 2. Error Handling
- **Canonical structs**: Use situation-specific error types
- **Panic for bugs**: Use panic only for programming errors
- **Result for failures**: Return Result for expected errors
- **Backtraces**: Capture backtraces when creating errors

### 3. Type Safety
- **Strong types**: Use PathBuf for paths, not String
- **Newtype pattern**: Wrap primitives in validated types
- **impl AsRef**: Accept flexible inputs in functions
- **Concrete types**: Use concrete types in structs

### 4. Performance
- **Profile first**: Benchmark before optimizing
- **Hot paths**: Identify and optimize critical code
- **Throughput**: Optimize for items per CPU cycle
- **Batch operations**: Process in batches when possible

### 5. Testing
- **test-util feature**: Guard test utilities behind feature flag
- **Mockable I/O**: Use traits for mockable dependencies
- **Unit tests**: Test public API, not implementation
- **Integration tests**: Place in `tests/` directory

## Common Patterns

### Application Pattern

```rust
use cosmic::app::{Core, Task};
use cosmic::Application;

pub struct App {
    core: Core,
    nav_model: nav_bar::Model,
    items: Vec<Item>,
}

impl Application for App {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "org.cosmic.MyApp";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let app = Self {
            core,
            nav_model: Self::init_nav_model(),
            items: Vec::new(),
        };
        (app, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::LoadData => {
                Task::perform(
                    async { load_data().await },
                    |result| Message::DataLoaded(result),
                )
            }
            _ => Task::none(),
        }
    }

    fn view(&self) -> cosmic::Element<Self::Message> {
        cosmic::widget::text("Hello, COSMIC!").into()
    }
}
```

### Configuration Pattern

```rust
use cosmic_config::CosmicConfigEntry;

#[derive(Debug, Clone, CosmicConfigEntry, PartialEq)]
#[version = 1]
pub struct AppConfig {
    pub theme_mode: ThemeMode,
    pub sidebar_visible: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::System,
            sidebar_visible: true,
        }
    }
}
```

### Error Handling Pattern

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
```

### Widget Composition Pattern

```rust
fn view(&self) -> cosmic::Element<Message> {
    let spacing = cosmic::theme::spacing();

    cosmic::widget::column()
        .push(self.view_header())
        .push(self.view_content())
        .push(self.view_footer())
        .spacing(spacing.space_m)
        .into()
}

fn view_header(&self) -> cosmic::Element<Message> {
    cosmic::widget::text::heading("My App").into()
}
```

## References

### Official Documentation
- [libcosmic Documentation](https://pop-os.github.io/libcosmic/cosmic/)
- [libcosmic Book](https://pop-os.github.io/libcosmic-book/)
- [COSMIC App Template](https://github.com/pop-os/cosmic-app-template)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### Microsoft Rust Guidelines
- [Error Handling](https://github.com/AzureMessaging/guidelines-rust)
- [Safety and Soundness](https://github.com/AzureMessaging/guidelines-rust)
- [Testing Patterns](https://github.com/AzureMessaging/guidelines-rust)

### Tools
- [Tokio](https://tokio.rs/)
- [cosmic-config](https://docs.rs/cosmic-config/)
- [Criterion Benchmarking](https://bheisler.github.io/criterion.rs/book/)
- [Miri](https://github.com/rust-lang/miri)

## Contributing

When adding new standards:
1. Follow Microsoft's Rust guidelines
2. Include comprehensive code examples
3. Provide good and bad examples
4. Add relevant cross-references
5. Include best practices checklists

## Version

Standards Version: 2.0
Last Updated: 2025-11-30
For: Rust stable + libcosmic
