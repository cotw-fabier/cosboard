# Data Models

## Overview

This document defines patterns for data structures, serialization, and domain models in Rust applications.

## Struct Design

### Basic Structure

```rust
/// Represents a user in the system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    id: UserId,
    username: Username,
    email: Email,
    created_at: DateTime<Utc>,
}

impl User {
    pub fn new(id: UserId, username: Username, email: Email) -> Self {
        Self {
            id,
            username,
            email,
            created_at: Utc::now(),
        }
    }

    // Getters
    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
```

### Common Derive Macros

```rust
// Most data types should derive these
#[derive(Debug, Clone)]
pub struct Document { ... }

// Add when equality comparison is needed
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config { ... }

// Add Hash for HashMap keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(u64);

// Add Default when sensible
#[derive(Debug, Clone, Default)]
pub struct Settings {
    theme: Theme,
    notifications: bool,
}
```

## Serialization with Serde

### Basic Serialization

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub app_name: String,
    pub version: String,
    pub settings: Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: Theme,
    pub auto_save: bool,
    pub interval_seconds: u64,
}
```

### Serde Attributes

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]  // JSON style
pub struct ApiResponse {
    pub user_name: String,    // Serializes as "userName"
    pub created_at: String,   // Serializes as "createdAt"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,

    #[serde(default)]  // Use Default if missing
    pub active: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    #[serde(skip)]  // Never serialize
    pub password_hash: String,

    #[serde(rename = "type")]  // Rename field
    pub user_type: UserType,
}
```

### Enum Serialization

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

// Serializes as: "active", "inactive", "pending"

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]  // Tagged union
pub enum Message {
    Text { content: String },
    Image { url: String, width: u32, height: u32 },
    File { path: String, size: u64 },
}

// Serializes as: {"type": "text", "content": "..."}
```

## cosmic-config Integration

### CosmicConfigEntry

```rust
use cosmic_config::CosmicConfigEntry;

#[derive(Debug, Clone, CosmicConfigEntry, PartialEq)]
#[version = 1]
pub struct AppConfig {
    pub theme_mode: ThemeMode,
    pub show_sidebar: bool,
    pub recent_files: Vec<String>,
    pub window_size: (u32, u32),
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::System,
            show_sidebar: true,
            recent_files: Vec::new(),
            window_size: (800, 600),
        }
    }
}
```

### Loading Configuration

```rust
use cosmic_config::{Config, ConfigGet};

fn load_config() -> Result<AppConfig, cosmic_config::Error> {
    let config = Config::new("org.cosmic.MyApp", 1)?;
    let app_config = AppConfig::get_entry(&config)?;
    Ok(app_config)
}
```

## DTOs vs Domain Models

### Data Transfer Objects

```rust
/// DTO for API requests - minimal validation
#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// DTO for API responses - serialization focused
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

impl From<&User> for UserResponse {
    fn from(user: &User) -> Self {
        Self {
            id: user.id().value(),
            username: user.username().as_str().to_string(),
            email: user.email().as_str().to_string(),
            created_at: user.created_at().to_rfc3339(),
        }
    }
}
```

### Domain Models

```rust
/// Domain model - validated, type-safe
pub struct User {
    id: UserId,
    username: Username,  // Validated type
    email: Email,        // Validated type
    created_at: DateTime<Utc>,
}

impl User {
    /// Create from DTO with validation
    pub fn from_request(
        id: UserId,
        request: CreateUserRequest,
    ) -> Result<Self, ValidationError> {
        let username = Username::new(&request.username)?;
        let email = Email::new(&request.email)?;

        Ok(Self {
            id,
            username,
            email,
            created_at: Utc::now(),
        })
    }
}
```

## Newtype Pattern

Wrap primitive types for type safety:

```rust
/// Strongly-typed user ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(u64);

impl UserId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

### Validated Newtypes

```rust
/// Validated email address
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Email(String);

impl Email {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        if !value.contains('@') || !value.contains('.') {
            return Err(ValidationError::InvalidEmail);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Custom deserialize with validation
impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}
```

## Optional Fields

Handle optional data consistently:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub username: String,

    // Optional with default
    #[serde(default)]
    pub bio: String,

    // Optional nullable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    // Optional with custom default
    #[serde(default = "default_theme")]
    pub theme: Theme,
}

fn default_theme() -> Theme {
    Theme::System
}
```

## Collection Types

Choose appropriate collections:

```rust
use std::collections::{HashMap, HashSet, BTreeMap};

pub struct AppState {
    // Fast lookup by key
    users: HashMap<UserId, User>,

    // Unique items, no order
    active_sessions: HashSet<SessionId>,

    // Sorted keys
    events: BTreeMap<DateTime<Utc>, Event>,

    // Simple list
    recent_files: Vec<PathBuf>,
}
```

## Immutability

Prefer immutable data when possible:

```rust
/// Immutable configuration
#[derive(Debug, Clone)]
pub struct Config {
    settings: Settings,
}

impl Config {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    /// Return new config with updated settings
    pub fn with_settings(self, settings: Settings) -> Self {
        Self { settings }
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}
```

## Update Patterns

For mutable state, use builder-style updates:

```rust
#[derive(Debug, Clone)]
pub struct Document {
    title: String,
    content: String,
    modified_at: DateTime<Utc>,
}

impl Document {
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.modified_at = Utc::now();
    }

    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
        self.modified_at = Utc::now();
    }
}
```

## Best Practices Checklist

**Derive Macros:**
- [ ] Debug for all types
- [ ] Clone when type should be cloneable
- [ ] PartialEq, Eq for comparable types
- [ ] Hash for HashMap keys
- [ ] Serialize, Deserialize for persistence

**Serde:**
- [ ] Use `#[serde(rename_all)]` for style consistency
- [ ] Use `#[serde(default)]` for optional fields
- [ ] Use `#[serde(skip)]` for sensitive data
- [ ] Use `#[serde(tag)]` for tagged enums

**Type Safety:**
- [ ] Use newtype pattern for IDs
- [ ] Validate newtypes on construction
- [ ] Separate DTOs from domain models
- [ ] Use Option for nullable fields

## References

- [Serde Documentation](https://serde.rs/)
- [cosmic-config Documentation](https://docs.rs/cosmic-config/)
- [Rust API Guidelines - Types](https://rust-lang.github.io/api-guidelines/type-safety.html)
