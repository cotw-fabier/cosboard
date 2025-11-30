# Async Patterns

## Overview

This document defines async programming patterns for COSMIC applications using Tokio and libcosmic's Task system.

## Tokio Runtime

### Setup

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1", features = ["rt", "sync", "time", "fs", "net"] }
```

### Feature Selection

Choose minimal features:

```toml
# Full runtime (heavyweight)
tokio = { version = "1", features = ["full"] }

# Minimal for COSMIC apps (recommended)
tokio = { version = "1", features = ["rt", "sync", "time"] }

# Add as needed:
# "fs" - async file operations
# "net" - async networking
# "process" - async process spawning
```

## Task-Based Async in COSMIC

libcosmic uses `Task<Message>` for async operations:

### Basic Async Operation

```rust
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::LoadData => {
            Task::perform(
                async {
                    load_data_from_disk().await
                },
                |result| match result {
                    Ok(data) => Message::DataLoaded(data),
                    Err(e) => Message::Error(e.to_string()),
                },
            )
        }
        Message::DataLoaded(data) => {
            self.data = data;
            Task::none()
        }
        Message::Error(msg) => {
            self.error = Some(msg);
            Task::none()
        }
        _ => Task::none(),
    }
}
```

### Async Function Pattern

```rust
async fn load_data_from_disk() -> Result<Vec<Item>, LoadError> {
    let path = get_data_path();
    let contents = tokio::fs::read_to_string(&path).await?;
    let items: Vec<Item> = serde_json::from_str(&contents)?;
    Ok(items)
}

async fn save_data_to_disk(items: &[Item]) -> Result<(), SaveError> {
    let path = get_data_path();
    let contents = serde_json::to_string_pretty(items)?;
    tokio::fs::write(&path, contents).await?;
    Ok(())
}
```

### Chained Tasks

```rust
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::SaveAndClose => {
            let data = self.data.clone();
            Task::perform(
                async move {
                    save_data(&data).await?;
                    Ok::<_, SaveError>(())
                },
                |result| match result {
                    Ok(()) => Message::CloseWindow,
                    Err(e) => Message::Error(e.to_string()),
                },
            )
        }
        _ => Task::none(),
    }
}
```

### Batch Tasks

```rust
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::RefreshAll => {
            Task::batch([
                Task::perform(
                    async { load_users().await },
                    |r| Message::UsersLoaded(r.map_err(|e| e.to_string())),
                ),
                Task::perform(
                    async { load_settings().await },
                    |r| Message::SettingsLoaded(r.map_err(|e| e.to_string())),
                ),
            ])
        }
        _ => Task::none(),
    }
}
```

## Subscriptions

For continuous async operations:

```rust
impl Application for App {
    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        let mut subscriptions = vec![];

        // Timer subscription
        if self.auto_refresh {
            subscriptions.push(
                cosmic::iced::time::every(Duration::from_secs(30))
                    .map(|_| Message::Refresh)
            );
        }

        // Config watching
        subscriptions.push(
            self.core()
                .watch_config::<AppConfig>(Self::APP_ID)
                .map(Message::ConfigUpdated)
        );

        cosmic::iced::Subscription::batch(subscriptions)
    }
}
```

## Cancellation

### With Tokio Select

```rust
use tokio::select;
use tokio::sync::oneshot;

struct CancellableOperation {
    cancel_tx: Option<oneshot::Sender<()>>,
}

impl CancellableOperation {
    async fn run_with_cancel(
        cancel_rx: oneshot::Receiver<()>,
    ) -> Result<Data, OperationError> {
        select! {
            result = long_running_operation() => result,
            _ = cancel_rx => Err(OperationError::Cancelled),
        }
    }
}
```

### In COSMIC App

```rust
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::StartOperation => {
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.cancel_handle = Some(tx);

            Task::perform(
                async move {
                    CancellableOperation::run_with_cancel(rx).await
                },
                |result| Message::OperationComplete(result),
            )
        }
        Message::Cancel => {
            if let Some(tx) = self.cancel_handle.take() {
                let _ = tx.send(());
            }
            Task::none()
        }
        _ => Task::none(),
    }
}
```

## Channel Communication

### One-shot Channel

```rust
use tokio::sync::oneshot;

// For single response
let (tx, rx) = oneshot::channel();

// Send result
tx.send(result).ok();

// Receive (async)
let result = rx.await?;
```

### MPSC Channel

```rust
use tokio::sync::mpsc;

// Multiple producer, single consumer
let (tx, mut rx) = mpsc::channel(100);

// Send
tx.send(message).await?;

// Receive
while let Some(msg) = rx.recv().await {
    process(msg);
}
```

### Watch Channel

For state updates:

```rust
use tokio::sync::watch;

// Single value that can be watched
let (tx, rx) = watch::channel(initial_value);

// Update value
tx.send(new_value)?;

// Watch for changes
let mut rx = rx.clone();
while rx.changed().await.is_ok() {
    let value = rx.borrow().clone();
    // Handle updated value
}
```

## Error Handling in Async

### Map Errors for Messages

```rust
Task::perform(
    async {
        load_data().await.map_err(|e| e.to_string())
    },
    Message::DataLoaded,  // Expects Result<Data, String>
)
```

### Custom Error Types

```rust
#[derive(Debug, Clone)]
pub enum AsyncError {
    Io(String),
    Parse(String),
    Network(String),
}

impl From<std::io::Error> for AsyncError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

// In update
Task::perform(
    async {
        load_data().await
            .map_err(AsyncError::from)
    },
    |result| match result {
        Ok(data) => Message::Loaded(data),
        Err(e) => Message::Error(e),
    },
)
```

## Timeout Handling

```rust
use tokio::time::timeout;

async fn load_with_timeout() -> Result<Data, LoadError> {
    timeout(Duration::from_secs(10), load_data())
        .await
        .map_err(|_| LoadError::Timeout)?
}
```

## Debouncing

For search inputs:

```rust
use tokio::time::{sleep, Duration};

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::SearchInput(query) => {
            self.search_query = query.clone();
            self.search_version += 1;
            let version = self.search_version;

            Task::perform(
                async move {
                    // Debounce: wait before searching
                    sleep(Duration::from_millis(300)).await;
                    search(&query).await
                },
                move |result| Message::SearchResults(version, result),
            )
        }
        Message::SearchResults(version, results) => {
            // Only apply if this is the latest search
            if version == self.search_version {
                self.results = results.unwrap_or_default();
            }
            Task::none()
        }
        _ => Task::none(),
    }
}
```

## File Operations

### Async File I/O

```rust
use tokio::fs;

async fn load_file(path: &Path) -> Result<String, std::io::Error> {
    fs::read_to_string(path).await
}

async fn save_file(path: &Path, contents: &str) -> Result<(), std::io::Error> {
    fs::write(path, contents).await
}

async fn create_directory(path: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(path).await
}
```

### File Watching

```rust
// In subscription, use cosmic-config's watch
fn subscription(&self) -> cosmic::iced::Subscription<Message> {
    self.core()
        .watch_config::<AppConfig>(Self::APP_ID)
        .map(Message::ConfigUpdated)
}
```

## Best Practices

### Do

```rust
// Use Task::perform for async in COSMIC apps
Task::perform(async { ... }, |result| Message::Done(result))

// Handle errors in message mapping
.map_err(|e| e.to_string())

// Use timeouts for network operations
timeout(Duration::from_secs(30), operation).await

// Batch independent operations
Task::batch([task1, task2, task3])
```

### Don't

```rust
// DON'T: Block in update()
fn update(&mut self, message: Message) -> Task<Message> {
    // BAD: This blocks the UI
    let result = std::fs::read_to_string("file.txt");
    Task::none()
}

// DON'T: Spawn unbounded tasks
tokio::spawn(async { ... });  // Can't track or cancel

// DON'T: Ignore errors
let _ = operation().await;  // Error silently dropped
```

## Best Practices Checklist

- [ ] Use `Task::perform` for all async operations
- [ ] Handle errors by mapping to message variants
- [ ] Use timeouts for network/IO operations
- [ ] Batch independent operations with `Task::batch`
- [ ] Use subscriptions for continuous events
- [ ] Implement cancellation for long operations
- [ ] Debounce rapid user input
- [ ] Use async file operations (`tokio::fs`)

## References

- [Tokio Documentation](https://tokio.rs/)
- [Iced Task Documentation](https://docs.iced.rs/iced/task/struct.Task.html)
- [Async Rust Book](https://rust-lang.github.io/async-book/)
