# Error Handling

## Overview

This document defines error handling patterns for Rust applications following Microsoft's guidelines. The key principle is using canonical error structs with backtraces and clearly distinguishing between recoverable errors (Result) and programming bugs (panic).

## Core Philosophy

- **Errors are for expected failures** - Return `Result<T, E>` for operations that can fail
- **Panics are for bugs** - Use panic only for programming errors and invariant violations
- **Backtraces for debugging** - Capture backtraces when errors are created
- **Specific error types** - Use situation-specific structs, not catch-all enums

## Canonical Error Structs (M-ERRORS-CANONICAL-STRUCTS)

### Error Structure

Define error types as structs, not enums:

```rust
use std::backtrace::Backtrace;
use std::fmt;

/// Error returned from configuration operations.
#[derive(Debug)]
pub struct ConfigError {
    kind: ConfigErrorKind,
    message: String,
    backtrace: Backtrace,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

// Internal enum for categorization
#[derive(Debug, Clone, Copy)]
enum ConfigErrorKind {
    NotFound,
    ParseError,
    IoError,
    ValidationError,
}

impl ConfigError {
    /// Creates a new configuration error.
    fn new(kind: ConfigErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            backtrace: Backtrace::capture(),
            source: None,
        }
    }

    /// Creates error with source cause.
    fn with_source(
        kind: ConfigErrorKind,
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            backtrace: Backtrace::capture(),
            source: Some(Box::new(source)),
        }
    }

    // Expose is_xxx() methods instead of the enum
    pub fn is_not_found(&self) -> bool {
        matches!(self.kind, ConfigErrorKind::NotFound)
    }

    pub fn is_parse_error(&self) -> bool {
        matches!(self.kind, ConfigErrorKind::ParseError)
    }

    pub fn is_io_error(&self) -> bool {
        matches!(self.kind, ConfigErrorKind::IoError)
    }
}
```

### Display and Error Implementation

```rust
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Configuration error: {}", self.message)?;

        if let Some(source) = &self.source {
            writeln!(f, "Caused by: {}", source)?;
        }

        writeln!(f, "\nBacktrace:\n{}", self.backtrace)?;

        Ok(())
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|s| s.as_ref() as _)
    }
}
```

### From Implementations

```rust
impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        Self::with_source(
            ConfigErrorKind::IoError,
            "I/O operation failed",
            err,
        )
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        Self::with_source(
            ConfigErrorKind::ParseError,
            "Failed to parse configuration",
            err,
        )
    }
}
```

## Using thiserror (Simplified Approach)

For simpler error types, use `thiserror`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("{0}")]
    Other(String),
}
```

## Panic vs Error (M-PANIC-IS-STOP, M-PANIC-ON-BUG)

### When to Panic

Panic for **programming errors** - bugs that indicate incorrect code:

```rust
// Contract violation - caller's bug
fn get_element(index: usize, slice: &[i32]) -> i32 {
    slice[index]  // Panics if out of bounds - this is correct!
}

// Invariant violation
fn process(state: &State) {
    assert!(state.is_valid(), "Invalid state - this should never happen");
}

// Unrecoverable initialization failure
fn init() {
    let config = Config::load()
        .expect("Failed to load required configuration");
}

// Const context
const VALUE: i32 = some_option.unwrap();  // OK in const
```

### When to Return Error

Return `Result` for **expected failures** - things that can legitimately fail:

```rust
// User input
fn parse_user_input(input: &str) -> Result<i32, ParseError> {
    input.parse().map_err(ParseError::from)
}

// I/O operations
fn read_config(path: &Path) -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    let config = serde_json::from_str(&contents)?;
    Ok(config)
}

// Network operations
async fn fetch_data(url: &str) -> Result<Data, NetworkError> {
    // ...
}

// Resource acquisition
fn open_database(path: &Path) -> Result<Database, DatabaseError> {
    // ...
}
```

### Never Panic For

```rust
// BAD: Panicking for recoverable errors
fn read_file(path: &Path) -> String {
    std::fs::read_to_string(path)
        .expect("File must exist")  // Wrong! File might not exist
}

// GOOD: Return error
fn read_file(path: &Path) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}
```

## Error Handling in COSMIC Apps

### In Application Update

```rust
impl Application for App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadData => {
                Task::perform(
                    async { load_data().await },
                    |result| match result {
                        Ok(data) => Message::DataLoaded(data),
                        Err(e) => Message::Error(e.to_string()),
                    },
                )
            }
            Message::DataLoaded(data) => {
                self.data = Some(data);
                Task::none()
            }
            Message::Error(msg) => {
                self.error = Some(msg);
                // Optionally show toast
                Task::none()
            }
            _ => Task::none(),
        }
    }
}
```

### Error Display in UI

```rust
fn view(&self) -> Element<Message> {
    let content = if let Some(error) = &self.error {
        self.view_error(error)
    } else if let Some(data) = &self.data {
        self.view_data(data)
    } else {
        self.view_loading()
    };

    content.into()
}

fn view_error(&self, error: &str) -> Element<Message> {
    let error_text = cosmic::widget::text(error)
        .style(cosmic::theme::Text::Destructive);

    let retry_button = cosmic::widget::button::suggested("Retry")
        .on_press(Message::LoadData);

    cosmic::widget::column()
        .push(error_text)
        .push(retry_button)
        .spacing(cosmic::theme::spacing().space_m)
        .into()
}
```

## Result Type Alias

Define a crate-level Result alias:

```rust
// src/error.rs
pub type Result<T> = std::result::Result<T, AppError>;

// Usage
pub fn do_something() -> Result<()> {
    // ...
    Ok(())
}
```

## Bail Macro Helper

Create a helper macro for common error creation:

```rust
macro_rules! bail {
    ($msg:expr) => {
        return Err(AppError::Other($msg.into()))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err(AppError::Other(format!($fmt, $($arg)*)))
    };
}

// Usage
fn validate(input: &str) -> Result<()> {
    if input.is_empty() {
        bail!("Input cannot be empty");
    }
    if input.len() > 100 {
        bail!("Input too long: {} characters (max 100)", input.len());
    }
    Ok(())
}
```

## Context Extension

Add context to errors using anyhow-style pattern:

```rust
pub trait ResultExt<T> {
    fn context(self, msg: &str) -> Result<T>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> ResultExt<T>
    for std::result::Result<T, E>
{
    fn context(self, msg: &str) -> Result<T> {
        self.map_err(|e| AppError::with_context(msg, e))
    }
}

// Usage
fn load_config() -> Result<Config> {
    let path = get_config_path()
        .context("Failed to determine config path")?;

    let contents = std::fs::read_to_string(&path)
        .context("Failed to read config file")?;

    let config = serde_json::from_str(&contents)
        .context("Failed to parse config")?;

    Ok(config)
}
```

## Best Practices Checklist

**Error Design:**
- [ ] Use structs for error types, not catch-all enums
- [ ] Capture backtraces at error creation
- [ ] Expose `is_xxx()` methods instead of error kind enums
- [ ] Implement Display and Error traits
- [ ] Include source error when wrapping

**Panic vs Error:**
- [ ] Panic only for programming bugs
- [ ] Return Result for expected failures
- [ ] Use `.expect()` with descriptive messages
- [ ] Never panic for user input errors

**In Application:**
- [ ] Handle errors in update() method
- [ ] Display user-friendly error messages
- [ ] Provide recovery options (retry buttons)
- [ ] Log errors for debugging

## References

- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [thiserror](https://docs.rs/thiserror/)
- [Microsoft Rust Guidelines - Errors](https://github.com/AzureMessaging/guidelines-rust)
