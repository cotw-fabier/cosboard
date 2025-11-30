# Validation and Type Safety

## Overview

This document defines validation patterns and type safety guidelines for Rust applications. Strong typing and validation at system boundaries prevents bugs and security vulnerabilities.

## Strong Types (M-STRONG-TYPES)

Use the appropriate type for your task as early as possible:

### Path Handling

```rust
// BAD: Using String for file paths
fn read_config(path: String) -> Result<Config> {
    std::fs::read_to_string(&path)?  // Error-prone
}

// GOOD: Using PathBuf/Path
fn read_config(path: impl AsRef<Path>) -> Result<Config> {
    let path = path.as_ref();
    let contents = std::fs::read_to_string(path)?;
    // ...
}
```

### Numeric Types

```rust
// BAD: Using i32 for everything
fn process_age(age: i32) { ... }

// GOOD: Use appropriate unsigned type with validation
fn process_age(age: u8) -> Result<(), ValidationError> {
    if age > 150 {
        return Err(ValidationError::out_of_range("age", 0, 150));
    }
    // ...
}
```

### Newtype Pattern

```rust
/// A validated email address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    /// Creates a new validated email.
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        if !Self::is_valid(&value) {
            return Err(ValidationError::invalid_email(&value));
        }
        Ok(Self(value))
    }

    fn is_valid(value: &str) -> bool {
        value.contains('@') && value.contains('.') && value.len() <= 255
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Usage: Type system prevents invalid emails
fn send_notification(email: &Email) {
    // email is guaranteed to be valid
}
```

### ID Types

```rust
use std::marker::PhantomData;

/// Type-safe ID wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id<T> {
    value: u64,
    _marker: PhantomData<T>,
}

impl<T> Id<T> {
    pub fn new(value: u64) -> Self {
        Self { value, _marker: PhantomData }
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}

// Different ID types cannot be mixed
pub struct User;
pub struct Document;

type UserId = Id<User>;
type DocumentId = Id<Document>;

// Compile error: expected UserId, found DocumentId
fn get_user(id: UserId) { ... }
```

## Accept impl AsRef<T> (M-IMPL-ASREF)

Accept flexible input types in function signatures:

```rust
// Instead of &str or String
fn process_name(name: impl AsRef<str>) {
    let name = name.as_ref();
    // Works with &str, String, Cow<str>, etc.
}

// Instead of &Path or PathBuf
fn load_file(path: impl AsRef<Path>) -> Result<String> {
    std::fs::read_to_string(path.as_ref())
}

// Instead of &[u8] or Vec<u8>
fn hash_data(data: impl AsRef<[u8]>) -> [u8; 32] {
    // Works with &[u8], Vec<u8>, &Vec<u8>, etc.
}
```

**Important**: Use concrete types in struct definitions:

```rust
// BAD: Generic bounds in struct
struct Config<S: AsRef<str>> {
    name: S,  // Leaks generics throughout codebase
}

// GOOD: Concrete type in struct
struct Config {
    name: String,  // Clear ownership
}
```

## Validation at Boundaries

Validate all external input at system boundaries:

```rust
/// User input from form
#[derive(Debug)]
pub struct UserInput {
    pub username: String,
    pub email: String,
    pub age: String,
}

/// Validated user data
#[derive(Debug)]
pub struct ValidatedUser {
    pub username: Username,
    pub email: Email,
    pub age: u8,
}

impl UserInput {
    /// Validates raw input and returns validated struct.
    pub fn validate(self) -> Result<ValidatedUser, Vec<ValidationError>> {
        let mut errors = Vec::new();

        let username = match Username::new(&self.username) {
            Ok(u) => Some(u),
            Err(e) => { errors.push(e); None }
        };

        let email = match Email::new(&self.email) {
            Ok(e) => Some(e),
            Err(e) => { errors.push(e); None }
        };

        let age = match self.age.parse::<u8>() {
            Ok(a) if a <= 150 => Some(a),
            Ok(a) => {
                errors.push(ValidationError::out_of_range("age", 0, 150));
                None
            }
            Err(_) => {
                errors.push(ValidationError::invalid_format("age"));
                None
            }
        };

        if errors.is_empty() {
            Ok(ValidatedUser {
                username: username.unwrap(),
                email: email.unwrap(),
                age: age.unwrap(),
            })
        } else {
            Err(errors)
        }
    }
}
```

## Validation Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("{field} is required")]
    Required { field: &'static str },

    #[error("{field} must be between {min} and {max}")]
    OutOfRange {
        field: &'static str,
        min: i64,
        max: i64
    },

    #[error("{field} has invalid format")]
    InvalidFormat { field: &'static str },

    #[error("{field} must be at least {min} characters")]
    TooShort { field: &'static str, min: usize },

    #[error("{field} must be at most {max} characters")]
    TooLong { field: &'static str, max: usize },

    #[error("Invalid email: {value}")]
    InvalidEmail { value: String },
}

impl ValidationError {
    pub fn required(field: &'static str) -> Self {
        Self::Required { field }
    }

    pub fn out_of_range(field: &'static str, min: i64, max: i64) -> Self {
        Self::OutOfRange { field, min, max }
    }

    pub fn invalid_format(field: &'static str) -> Self {
        Self::InvalidFormat { field }
    }
}
```

## Common Validation Patterns

### String Validation

```rust
/// Validated username (3-20 alphanumeric + underscore).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Username(String);

impl Username {
    const MIN_LEN: usize = 3;
    const MAX_LEN: usize = 20;

    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();

        if value.is_empty() {
            return Err(ValidationError::required("username"));
        }

        if value.len() < Self::MIN_LEN {
            return Err(ValidationError::TooShort {
                field: "username",
                min: Self::MIN_LEN
            });
        }

        if value.len() > Self::MAX_LEN {
            return Err(ValidationError::TooLong {
                field: "username",
                max: Self::MAX_LEN
            });
        }

        if !value.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ValidationError::invalid_format("username"));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### URL Validation

```rust
use url::Url;

/// A validated HTTPS URL.
#[derive(Debug, Clone)]
pub struct SecureUrl(Url);

impl SecureUrl {
    pub fn new(value: &str) -> Result<Self, ValidationError> {
        let url = Url::parse(value)
            .map_err(|_| ValidationError::invalid_format("url"))?;

        if url.scheme() != "https" {
            return Err(ValidationError::InvalidFormat {
                field: "url"
            });
        }

        Ok(Self(url))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
```

### Range Validation Helper

```rust
/// Validates that a value is within an inclusive range.
pub fn validate_range<T>(
    value: T,
    min: T,
    max: T,
    field: &'static str,
) -> Result<T, ValidationError>
where
    T: PartialOrd + Into<i64> + Copy,
{
    if value < min || value > max {
        return Err(ValidationError::OutOfRange {
            field,
            min: min.into(),
            max: max.into(),
        });
    }
    Ok(value)
}

// Usage
let age = validate_range(input_age, 0_u8, 150, "age")?;
let quantity = validate_range(input_qty, 1_u32, 1000, "quantity")?;
```

## Validation in COSMIC Apps

### Form Validation Pattern

```rust
#[derive(Debug, Clone)]
pub struct FormState {
    pub name: String,
    pub email: String,
    pub errors: HashMap<String, String>,
}

impl FormState {
    pub fn validate(&mut self) -> bool {
        self.errors.clear();

        // Validate name
        if self.name.trim().is_empty() {
            self.errors.insert("name".into(), "Name is required".into());
        } else if self.name.len() > 100 {
            self.errors.insert("name".into(), "Name too long".into());
        }

        // Validate email
        match Email::new(&self.email) {
            Ok(_) => {}
            Err(e) => {
                self.errors.insert("email".into(), e.to_string());
            }
        }

        self.errors.is_empty()
    }

    pub fn get_error(&self, field: &str) -> Option<&str> {
        self.errors.get(field).map(|s| s.as_str())
    }
}
```

### In Message Handling

```rust
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::SubmitForm => {
            if self.form.validate() {
                // Form is valid, proceed with submission
                Task::perform(
                    async move { submit_form(&self.form).await },
                    Message::FormSubmitted,
                )
            } else {
                // Form has errors, stay on page
                Task::none()
            }
        }
        _ => Task::none(),
    }
}
```

## Best Practices Checklist

**Type Safety:**
- [ ] Use PathBuf/Path for file paths
- [ ] Use newtype pattern for validated strings
- [ ] Use type-safe ID wrappers
- [ ] Accept `impl AsRef<T>` in functions
- [ ] Use concrete types in structs

**Validation:**
- [ ] Validate at system boundaries
- [ ] Return structured validation errors
- [ ] Collect all errors, don't stop at first
- [ ] Provide clear error messages
- [ ] Never trust external input

## References

- [Rust API Guidelines - Type Safety](https://rust-lang.github.io/api-guidelines/type-safety.html)
- [Parse, Don't Validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
- [Newtype Pattern](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
