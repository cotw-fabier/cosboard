# API Design

## Overview

This document defines API design patterns for Rust libraries and applications following Microsoft's guidelines. Well-designed APIs are intuitive, consistent, and hard to misuse.

## Strong Types (M-STRONG-TYPES)

Use appropriate types to encode meaning and prevent errors:

```rust
// BAD: Generic types lose meaning
fn connect(host: String, port: i32, timeout: i32) { ... }

// GOOD: Strong types encode intent
fn connect(endpoint: Endpoint, timeout: Duration) { ... }

pub struct Endpoint {
    host: Host,
    port: Port,
}
```

### Path Handling

```rust
// BAD: String for paths
fn load_config(path: String) -> Config { ... }

// GOOD: Use Path/PathBuf
fn load_config(path: impl AsRef<Path>) -> Result<Config, ConfigError> {
    let path = path.as_ref();
    // ...
}
```

### Domain Types

```rust
// Create types for domain concepts
pub struct UserId(u64);
pub struct Email(String);
pub struct Username(String);

// Functions use domain types
fn get_user(id: UserId) -> Option<User> { ... }
fn send_email(to: &Email, subject: &str) { ... }
```

## Accept impl AsRef<T> (M-IMPL-ASREF)

Accept flexible input in function parameters:

```rust
// Flexible string input
pub fn process(name: impl AsRef<str>) {
    let name = name.as_ref();
    // Works with &str, String, Cow<str>, etc.
}

// Flexible path input
pub fn read_file(path: impl AsRef<Path>) -> Result<String> {
    std::fs::read_to_string(path.as_ref())
}

// Flexible byte input
pub fn hash(data: impl AsRef<[u8]>) -> [u8; 32] {
    // Works with &[u8], Vec<u8>, String, etc.
}
```

**Important**: Use concrete types in struct fields:

```rust
// BAD: Generics in structs
pub struct Config<P: AsRef<Path>> {
    path: P,  // Leaks generic everywhere
}

// GOOD: Concrete types in structs
pub struct Config {
    path: PathBuf,  // Clear ownership
}
```

## Builder Pattern (M-INIT-BUILDER)

### When to Use Builders

- **0-2 optional parameters**: Use inherent methods
- **3+ optional parameters**: Use builder pattern

### Simple Case (No Builder)

```rust
pub struct Connection {
    endpoint: Endpoint,
    timeout: Option<Duration>,
}

impl Connection {
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            timeout: None,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

// Usage
let conn = Connection::new(endpoint).with_timeout(Duration::from_secs(30));
```

### Builder Pattern

```rust
pub struct Client {
    endpoint: Endpoint,
    timeout: Duration,
    retry_count: u32,
    user_agent: String,
    // ... many options
}

pub struct ClientBuilder {
    endpoint: Endpoint,
    timeout: Duration,
    retry_count: u32,
    user_agent: String,
}

impl ClientBuilder {
    // Required parameter in builder creation
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            timeout: Duration::from_secs(30),
            retry_count: 3,
            user_agent: "cosmic-app/1.0".into(),
        }
    }

    // Optional setters - named without "set_" prefix
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn retry_count(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = agent.into();
        self
    }

    pub fn build(self) -> Client {
        Client {
            endpoint: self.endpoint,
            timeout: self.timeout,
            retry_count: self.retry_count,
            user_agent: self.user_agent,
        }
    }
}

impl Client {
    // Convenience method for starting builder
    pub fn builder(endpoint: Endpoint) -> ClientBuilder {
        ClientBuilder::new(endpoint)
    }
}

// Usage
let client = Client::builder(endpoint)
    .timeout(Duration::from_secs(60))
    .retry_count(5)
    .build();
```

### Builder with Validation

```rust
impl ClientBuilder {
    pub fn build(self) -> Result<Client, BuildError> {
        if self.retry_count == 0 {
            return Err(BuildError::InvalidRetryCount);
        }
        if self.user_agent.is_empty() {
            return Err(BuildError::EmptyUserAgent);
        }

        Ok(Client {
            endpoint: self.endpoint,
            timeout: self.timeout,
            retry_count: self.retry_count,
            user_agent: self.user_agent,
        })
    }
}
```

## Essential Functions in Inherent Impl

Place frequently-used methods in inherent impl, not traits:

```rust
pub struct User {
    id: UserId,
    name: String,
    email: Email,
}

impl User {
    // Core functionality in inherent impl
    pub fn new(id: UserId, name: String, email: Email) -> Self {
        Self { id, name, email }
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &Email {
        &self.email
    }
}

// Trait implementations are secondary
impl Display for User {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} <{}>", self.name, self.email.as_str())
    }
}
```

## Getter/Setter Conventions

Follow Rust API conventions:

```rust
impl Config {
    // Getter: field name as method
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    // Getter for reference: field name
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    // Setter: set_* prefix
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    // Builder-style setter (consumes self)
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
```

## Conversion Conventions

Use standard naming for conversions:

```rust
impl User {
    // as_* - Cheap reference conversion
    pub fn as_str(&self) -> &str {
        &self.name
    }

    // to_* - Expensive conversion (may allocate)
    pub fn to_string(&self) -> String {
        self.name.clone()
    }

    // into_* - Consumes self
    pub fn into_name(self) -> String {
        self.name
    }
}
```

## Public Debug and Display

Implement standard traits for all public types:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    id: UserId,
    name: String,
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "User({})", self.name)
    }
}
```

## Common Trait Implementations

Derive or implement common traits when appropriate:

```rust
// Derivable traits
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(u64);

// When to implement each:
// - Debug: Almost always (required for error messages)
// - Clone: When type should be copyable
// - Copy: For small, trivially copyable types
// - PartialEq, Eq: When equality comparison makes sense
// - Hash: When type will be used as HashMap key
// - Default: When there's a sensible default value
// - Serialize, Deserialize: For configuration/persistence
```

## Error Types

Follow error handling guidelines from `error-handling.md`:

```rust
pub fn connect(endpoint: &Endpoint) -> Result<Connection, ConnectionError> {
    // ...
}

// Return specific error types, not Box<dyn Error>
pub struct ConnectionError {
    kind: ConnectionErrorKind,
    message: String,
}

impl ConnectionError {
    pub fn is_timeout(&self) -> bool {
        matches!(self.kind, ConnectionErrorKind::Timeout)
    }
}
```

## Iterator Patterns

Return iterators for collections:

```rust
impl UserList {
    // Return iterator, not Vec
    pub fn iter(&self) -> impl Iterator<Item = &User> {
        self.users.iter()
    }

    // For owned iteration
    pub fn into_iter(self) -> impl Iterator<Item = User> {
        self.users.into_iter()
    }
}
```

## API Documentation

Document all public items:

```rust
/// Creates a new connection to the specified endpoint.
///
/// # Arguments
///
/// * `endpoint` - The server endpoint to connect to
///
/// # Errors
///
/// Returns `ConnectionError::Timeout` if connection times out.
/// Returns `ConnectionError::Refused` if server refuses connection.
///
/// # Examples
///
/// ```rust
/// let endpoint = Endpoint::new("localhost", 8080);
/// let conn = Connection::connect(&endpoint)?;
/// ```
pub fn connect(endpoint: &Endpoint) -> Result<Connection, ConnectionError> {
    // ...
}
```

## Best Practices Checklist

**Type Safety:**
- [ ] Use domain types instead of primitives
- [ ] Accept `impl AsRef<T>` in functions
- [ ] Use concrete types in structs
- [ ] Validate at construction time

**Builders:**
- [ ] Use builder for 3+ optional parameters
- [ ] Required params in builder constructor
- [ ] Setters named without "set_" prefix
- [ ] Provide `Type::builder()` convenience method

**Traits:**
- [ ] Derive Debug for all public types
- [ ] Implement Display when appropriate
- [ ] Derive Clone, PartialEq, Eq when useful
- [ ] Essential methods in inherent impl

**Conventions:**
- [ ] Follow getter naming (field name)
- [ ] Follow conversion naming (as_, to_, into_)
- [ ] Return specific error types
- [ ] Document all public items

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust API Guidelines Checklist](https://rust-lang.github.io/api-guidelines/checklist.html)
- [Microsoft Rust Guidelines](https://github.com/AzureMessaging/guidelines-rust)
