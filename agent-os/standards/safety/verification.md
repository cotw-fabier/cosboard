# Static Verification

## Overview

This document defines static verification requirements for Rust applications following Microsoft's guidelines. Use compiler lints, Clippy, and security tools to catch issues before runtime.

## Compiler Lints (M-STATIC-VERIFICATION)

### Recommended Lints

Add to `Cargo.toml`:

```toml
[lints.rust]
# Warn on common issues
ambiguous_negative_literals = "warn"
missing_debug_implementations = "warn"
redundant_imports = "warn"
redundant_lifetimes = "warn"
trivial_numeric_casts = "warn"
unsafe_op_in_unsafe_fn = "warn"
unused_lifetimes = "warn"
```

### Lint Configuration

```toml
# In Cargo.toml
[lints.rust]
# Deny dangerous patterns
unsafe_code = "warn"                    # Warn on any unsafe usage
missing_docs = "warn"                   # Warn on undocumented public items

# Allow specific patterns when justified
dead_code = "allow"                     # During development
```

### Per-Item Lint Control

```rust
// Allow specific lint for one item
#[allow(dead_code)]
fn experimental_feature() { ... }

// Require explanation
#[allow(clippy::unwrap_used, reason = "Infallible in this context")]
let value = option.unwrap();
```

## Clippy Configuration

### Cargo.toml Configuration

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

# Specific restriction lints
clone_on_ref_ptr = "warn"
dbg_macro = "warn"
empty_drop = "warn"
fn_to_numeric_cast_any = "warn"
print_stdout = "warn"
print_stderr = "warn"
todo = "warn"
unimplemented = "warn"
unwrap_used = "warn"

# Allow some pedantic lints that reduce readability
module_name_repetitions = "allow"
too_many_lines = "allow"
must_use_candidate = "allow"
```

### Running Clippy

```bash
# Basic clippy run
cargo clippy

# All targets and features
cargo clippy --all-targets --all-features

# Treat warnings as errors (CI)
cargo clippy --all-targets -- -D warnings

# Fix automatically where possible
cargo clippy --fix
```

### Common Clippy Fixes

```rust
// clippy::unwrap_used - Use expect or handle error
// BAD
let value = option.unwrap();
// GOOD
let value = option.expect("value should exist after validation");
// BETTER
let value = option.ok_or(Error::Missing)?;

// clippy::clone_on_ref_ptr - Be explicit about Arc/Rc cloning
// BAD
let arc2 = arc.clone();
// GOOD
let arc2 = Arc::clone(&arc);

// clippy::map_unwrap_or - Use map_or or map_or_else
// BAD
option.map(|x| x + 1).unwrap_or(0)
// GOOD
option.map_or(0, |x| x + 1)
```

## Rustfmt

### Configuration

Create `rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
```

### Running Rustfmt

```bash
# Format all files
cargo fmt

# Check formatting (CI)
cargo fmt -- --check
```

## Security Tools

### cargo-audit

Check dependencies for security vulnerabilities:

```bash
# Install
cargo install cargo-audit

# Run audit
cargo audit

# With JSON output (CI)
cargo audit --json

# Ignore specific advisories
cargo audit --ignore RUSTSEC-2020-0001
```

### Advisory Database

Create `.cargo/audit.toml` for configuration:

```toml
[advisories]
ignore = [
    # "RUSTSEC-2020-0001",  # Reason for ignoring
]

[database]
path = "~/.cargo/advisory-db"
```

### cargo-deny

Comprehensive dependency checking:

```bash
# Install
cargo install cargo-deny

# Initialize config
cargo deny init

# Run checks
cargo deny check
```

Create `deny.toml`:

```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
confidence-threshold = 0.93

[bans]
multiple-versions = "warn"
deny = [
    # { name = "openssl" },  # Prefer rustls
]
```

## Additional Tools

### cargo-hack

Validate feature combinations:

```bash
# Install
cargo install cargo-hack

# Check all feature combinations
cargo hack check --feature-powerset

# Check each feature individually
cargo hack check --each-feature
```

### cargo-udeps

Find unused dependencies:

```bash
# Install (requires nightly)
cargo install cargo-udeps --locked

# Run
cargo +nightly udeps
```

### cargo-machete

Faster unused dependency detection:

```bash
# Install
cargo install cargo-machete

# Run
cargo machete
```

## Miri

Validate unsafe code for undefined behavior:

```bash
# Install miri
rustup +nightly component add miri

# Run tests under miri
cargo +nightly miri test

# Run specific test
cargo +nightly miri test test_name
```

### Miri Configuration

```rust
// Skip miri-incompatible tests
#[cfg_attr(miri, ignore)]
#[test]
fn test_with_external_io() {
    // ...
}
```

## CI Integration

### GitHub Actions

```yaml
name: CI

on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: Build
        run: cargo build --release

      - name: Test
        run: cargo test

      - name: Security audit
        run: |
          cargo install cargo-audit
          cargo audit
```

### Pre-commit Hooks

Create `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt -- --check
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false
```

## Best Practices Checklist

**Compiler Lints:**
- [ ] Enable recommended lints in Cargo.toml
- [ ] Fix or justify all warnings
- [ ] Use `#[allow(...)]` sparingly with reasons

**Clippy:**
- [ ] Enable major lint categories
- [ ] Run `cargo clippy --all-targets`
- [ ] Treat warnings as errors in CI

**Formatting:**
- [ ] Configure rustfmt.toml
- [ ] Run `cargo fmt` before commits
- [ ] Check formatting in CI

**Security:**
- [ ] Run `cargo audit` regularly
- [ ] Consider `cargo deny` for comprehensive checks
- [ ] Update dependencies for security fixes

**Unsafe:**
- [ ] Test unsafe code with Miri
- [ ] Document all SAFETY comments
- [ ] Minimize unsafe scope

## References

- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [cargo-audit](https://rustsec.org/)
- [cargo-deny](https://embarkstudios.github.io/cargo-deny/)
- [Miri](https://github.com/rust-lang/miri)
- [RustSec Advisory Database](https://rustsec.org/advisories/)
