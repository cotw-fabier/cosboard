# Test Writing

## Overview

This document defines testing standards for Rust and COSMIC applications following Microsoft's guidelines. Comprehensive testing ensures reliability and prevents regressions.

## Testing Philosophy

**Test Pyramid:**
```
        /\
       /  \        Integration Tests (Few)
      /____\       - Critical workflows
     /      \      - Multi-component interaction
    /        \
   /__________\    Unit Tests (Many)
  /            \   - Individual functions
 /              \  - Business logic
/________________\ - Pure functions
```

## Unit Tests

### Basic Unit Test

```rust
// src/calculator.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_add_overflow() {
        add(i32::MAX, 1);
    }
}
```

### Testing with Results

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() -> Result<(), ConfigError> {
        let config = parse_config("key=value")?;
        assert_eq!(config.get("key"), Some("value"));
        Ok(())
    }
}
```

### Test Organization

```rust
// src/user.rs
pub struct User { ... }

impl User {
    pub fn new(name: String) -> Self { ... }
    pub fn is_valid(&self) -> bool { ... }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Group related tests in modules
    mod new {
        use super::*;

        #[test]
        fn creates_user() {
            let user = User::new("Alice".into());
            assert_eq!(user.name(), "Alice");
        }

        #[test]
        fn handles_empty_name() {
            let user = User::new("".into());
            assert!(!user.is_valid());
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn valid_user_passes() {
            let user = User::new("Alice".into());
            assert!(user.is_valid());
        }
    }
}
```

## Test-Util Feature (M-TEST-UTIL)

Guard testing utilities behind a feature flag:

```toml
# Cargo.toml
[features]
test-util = []

[dependencies]
# Testing utilities only in test builds
```

### Test Utilities

```rust
// src/testing.rs
#[cfg(feature = "test-util")]
pub mod testing {
    use super::*;

    /// Creates a test user with default values
    pub fn create_test_user() -> User {
        User {
            id: UserId::new(1),
            name: "Test User".into(),
            email: Email::new("test@example.com").unwrap(),
        }
    }

    /// Mock database for testing
    pub struct MockDatabase {
        users: Vec<User>,
    }

    impl MockDatabase {
        pub fn new() -> Self {
            Self { users: Vec::new() }
        }

        pub fn add_user(&mut self, user: User) {
            self.users.push(user);
        }
    }
}
```

### Using Test Utilities

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "test-util")]
    use crate::testing::*;

    #[test]
    #[cfg(feature = "test-util")]
    fn test_with_mock_db() {
        let mut db = MockDatabase::new();
        let user = create_test_user();
        db.add_user(user);
        assert_eq!(db.users.len(), 1);
    }
}
```

## Mockable I/O (M-MOCKABLE-SYSCALLS)

Make I/O operations mockable for testing:

### Trait-Based Abstraction

```rust
// Define trait for I/O operations
pub trait FileSystem {
    fn read(&self, path: &Path) -> Result<String, IoError>;
    fn write(&self, path: &Path, contents: &str) -> Result<(), IoError>;
}

// Real implementation
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read(&self, path: &Path) -> Result<String, IoError> {
        std::fs::read_to_string(path).map_err(IoError::from)
    }

    fn write(&self, path: &Path, contents: &str) -> Result<(), IoError> {
        std::fs::write(path, contents).map_err(IoError::from)
    }
}

// Use trait in production code
pub struct ConfigManager<F: FileSystem> {
    fs: F,
    path: PathBuf,
}

impl<F: FileSystem> ConfigManager<F> {
    pub fn load(&self) -> Result<Config, ConfigError> {
        let contents = self.fs.read(&self.path)?;
        parse_config(&contents)
    }
}
```

### Mock Implementation

```rust
#[cfg(feature = "test-util")]
pub mod testing {
    use super::*;
    use std::collections::HashMap;

    pub struct MockFileSystem {
        files: HashMap<PathBuf, String>,
    }

    impl MockFileSystem {
        pub fn new() -> Self {
            Self {
                files: HashMap::new(),
            }
        }

        pub fn add_file(&mut self, path: impl Into<PathBuf>, contents: impl Into<String>) {
            self.files.insert(path.into(), contents.into());
        }
    }

    impl FileSystem for MockFileSystem {
        fn read(&self, path: &Path) -> Result<String, IoError> {
            self.files
                .get(path)
                .cloned()
                .ok_or(IoError::NotFound)
        }

        fn write(&self, path: &Path, contents: &str) -> Result<(), IoError> {
            Ok(())  // Mock: don't actually write
        }
    }
}
```

### Testing with Mocks

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;

    #[test]
    #[cfg(feature = "test-util")]
    fn test_load_config() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs.add_file("/config.toml", "theme = \"dark\"");

        let manager = ConfigManager {
            fs: mock_fs,
            path: PathBuf::from("/config.toml"),
        };

        let config = manager.load().unwrap();
        assert_eq!(config.theme, "dark");
    }
}
```

## Integration Tests

Place in `tests/` directory:

```rust
// tests/integration_test.rs
use my_app::*;

#[test]
fn test_full_workflow() {
    let app = App::new();
    let user = app.create_user("Alice").unwrap();
    let retrieved = app.get_user(user.id()).unwrap();
    assert_eq!(user.name(), retrieved.name());
}
```

### Integration Test Organization

```
tests/
├── common/
│   └── mod.rs          # Shared test utilities
├── config_tests.rs     # Config loading tests
├── user_tests.rs       # User workflow tests
└── database_tests.rs   # Database integration tests
```

```rust
// tests/common/mod.rs
pub fn setup_test_db() -> TestDatabase {
    // Setup code
}

pub fn cleanup() {
    // Cleanup code
}

// tests/user_tests.rs
mod common;

#[test]
fn test_user_creation() {
    let db = common::setup_test_db();
    // Test code
    common::cleanup();
}
```

## Benchmarking

Use Criterion for benchmarks:

```rust
// benches/benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use my_app::*;

fn benchmark_parse(c: &mut Criterion) {
    let input = "large input data...";

    c.bench_function("parse_data", |b| {
        b.iter(|| parse_data(black_box(input)))
    });
}

fn benchmark_with_setup(c: &mut Criterion) {
    let mut group = c.benchmark_group("processing");

    group.bench_function("small", |b| {
        let data = vec![1, 2, 3];
        b.iter(|| process(black_box(&data)))
    });

    group.bench_function("large", |b| {
        let data = vec![0; 10000];
        b.iter(|| process(black_box(&data)))
    });

    group.finish();
}

criterion_group!(benches, benchmark_parse, benchmark_with_setup);
criterion_main!(benches);
```

## Property-Based Testing

Use proptest for property-based tests:

```rust
// Add to Cargo.toml
// [dev-dependencies]
// proptest = "1.0"

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_roundtrip(s in "\\PC*") {
        let encoded = encode(&s);
        let decoded = decode(&encoded)?;
        prop_assert_eq!(s, decoded);
    }

    #[test]
    fn test_add_commutative(a in 0..1000i32, b in 0..1000i32) {
        prop_assert_eq!(add(a, b), add(b, a));
    }
}
```

## Testing COSMIC Applications

### Testing Application State

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_handling() {
        let mut app = App::test_instance();

        // Simulate message
        let task = app.update(Message::AddItem("Test".into()));

        // Verify state change
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0], "Test");
    }

    #[test]
    fn test_navigation() {
        let mut app = App::test_instance();

        app.on_nav_select(nav_bar::Id::new(1));

        assert_eq!(app.current_page, Page::Settings);
    }
}
```

### Testing View Functions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_renders_items() {
        let app = App {
            items: vec!["Item 1".into(), "Item 2".into()],
            ..Default::default()
        };

        let element = app.view();
        // View testing is limited - focus on state
    }
}
```

## Test Coverage

### Running with Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/
```

### Coverage in CI

```yaml
# .github/workflows/ci.yml
- name: Test with coverage
  run: |
    cargo install cargo-tarpaulin
    cargo tarpaulin --out Xml

- name: Upload to codecov
  uses: codecov/codecov-action@v3
```

## Test Naming Conventions

```rust
#[test]
fn test_<functionality>_<scenario>_<expected_result>() {
    // Examples:
}

#[test]
fn test_user_creation_with_valid_data_succeeds() { }

#[test]
fn test_user_creation_with_empty_name_fails() { }

#[test]
fn test_database_query_with_invalid_sql_returns_error() { }
```

## Common Test Patterns

### Setup and Teardown

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> TestContext {
        TestContext::new()
    }

    #[test]
    fn test_something() {
        let ctx = setup();
        // Use ctx
    }  // Automatic cleanup via Drop
}
```

### Table-Driven Tests

```rust
#[test]
fn test_validation() {
    let test_cases = vec![
        ("valid@email.com", true),
        ("invalid", false),
        ("no@domain", false),
        ("valid.email@example.com", true),
    ];

    for (input, expected) in test_cases {
        assert_eq!(
            is_valid_email(input),
            expected,
            "Failed for input: {}",
            input
        );
    }
}
```

### Async Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_load() {
        let result = load_data().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_with_timeout() {
        let result = tokio::time::timeout(
            Duration::from_secs(1),
            slow_operation(),
        ).await;

        assert!(result.is_ok());
    }
}
```

## Best Practices Checklist

**Unit Tests:**
- [ ] Test public API, not private implementation
- [ ] One assertion per test when possible
- [ ] Use descriptive test names
- [ ] Test edge cases and error paths

**Test Organization:**
- [ ] Group related tests in modules
- [ ] Use `#[cfg(test)]` for test modules
- [ ] Place integration tests in `tests/`
- [ ] Guard test utilities with `test-util` feature

**Mocking:**
- [ ] Use traits for mockable dependencies
- [ ] Provide mock implementations in `test-util`
- [ ] Test with both real and mock implementations
- [ ] Mock non-deterministic operations

**Coverage:**
- [ ] Aim for >80% coverage of critical paths
- [ ] Don't chase 100% coverage
- [ ] Focus on business logic coverage
- [ ] Test error handling paths

## References

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Proptest](https://proptest-rs.github.io/proptest/)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
