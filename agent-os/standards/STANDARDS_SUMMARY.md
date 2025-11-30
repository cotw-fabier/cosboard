# Rust + libcosmic Standards Summary

## Overview

This directory contains **20 comprehensive standards documents** for building COSMIC desktop applications with Rust and libcosmic, based on Microsoft's Rust guidelines and libcosmic best practices.

## Complete Standards List

### 📋 README and Documentation (2 files)
- **README.md** - Architecture overview, quick start guide, core principles
- **STANDARDS_SUMMARY.md** - This index and reference guide

### 🌍 Global Standards (7 files)
1. **global/tech-stack.md** - Rust, libcosmic, Iced, cosmic-theme, cosmic-config, Tokio, serde
2. **global/conventions.md** - Project structure, crate organization (M-SMALLER-CRATES), module patterns
3. **global/coding-style.md** - Rust naming conventions, rustfmt, clippy configuration (M-STATIC-VERIFICATION)
4. **global/error-handling.md** - Canonical error structs (M-ERRORS-CANONICAL-STRUCTS), panic vs Result
5. **global/commenting.md** - Doc comments, 15-word first sentence rule (M-FIRST-DOC-SENTENCE)
6. **global/validation.md** - Strong types (M-STRONG-TYPES), input validation, type safety
7. **global/performance.md** - Hot path identification (M-HOTPATH), throughput optimization (M-THROUGHPUT)

### 🎨 Application Standards (4 files)
8. **app/lifecycle.md** - cosmic::Application trait, Core state, init/update/view pattern, Task-based async
9. **app/components.md** - libcosmic widget catalog (buttons, inputs, layouts, navigation, specialized widgets)
10. **app/theming.md** - cosmic-theme system, ThemeType, colors, spacing, dark/light mode, high contrast
11. **app/accessibility.md** - Keyboard navigation, screen readers, WCAG compliance, focus management

### 🔧 API Design Standards (4 files)
12. **api/design.md** - Strong types, impl AsRef (M-IMPL-ASREF), builder pattern (M-INIT-BUILDER), API guidelines
13. **api/models.md** - Data structures, serde serialization, DTOs, CosmicConfigEntry, newtype pattern
14. **api/async.md** - Tokio runtime, Task-based async, subscriptions, cancellation, channel communication
15. **api/state-management.md** - Core state, nav_bar::Model, config persistence, UI state patterns

### 🛡️ Safety Standards (2 files)
16. **safety/memory.md** - Ownership, unsafe guidelines (M-UNSAFE), soundness (M-UNSOUND), Arc/Rc patterns
17. **safety/verification.md** - Compiler lints, clippy configuration, cargo-audit, miri, CI integration

### 🧪 Testing Standards (1 file)
18. **testing/test-writing.md** - test-util feature (M-TEST-UTIL), mockable I/O (M-MOCKABLE-SYSCALLS), unit/integration tests

## Key Features

### Microsoft Rust Guidelines Integration
- **M-SMALLER-CRATES**: Prefer smaller, focused crates
- **M-STATIC-VERIFICATION**: Comprehensive linting configuration
- **M-ERRORS-CANONICAL-STRUCTS**: Situation-specific error types with backtraces
- **M-FIRST-DOC-SENTENCE**: 15-word first sentence rule for docs
- **M-STRONG-TYPES**: Use PathBuf for paths, domain types for concepts
- **M-IMPL-ASREF**: Flexible function parameters
- **M-INIT-BUILDER**: Builder pattern for 3+ optional parameters
- **M-HOTPATH**: Identify and optimize performance-critical code
- **M-THROUGHPUT**: Optimize for items per CPU cycle
- **M-UNSAFE**: Strict unsafe code guidelines
- **M-UNSOUND**: Never write unsound safe code
- **M-TEST-UTIL**: Feature-gate testing utilities
- **M-MOCKABLE-SYSCALLS**: Trait-based mockable dependencies

### libcosmic Specific
- **Application trait**: Complete lifecycle management
- **Core state**: Window, theme, and system integration
- **Navigation**: nav_bar::Model for sidebar navigation
- **Widgets**: Comprehensive COSMIC widget catalog
- **Theming**: cosmic-theme with semantic colors
- **Configuration**: cosmic-config with live watching
- **Async**: Task-based operations with Tokio

### Production-Ready
- Based on Microsoft's Rust guidelines
- Integrated with libcosmic patterns
- Comprehensive error handling
- Memory safety emphasis
- Performance optimization guidance
- Complete testing strategies

## Standards by Concern

### Memory Safety
- safety/memory.md (Ownership, unsafe, soundness)
- global/conventions.md (Avoid statics)
- api/design.md (Strong types)

### Error Handling
- global/error-handling.md (Canonical structs, backtraces)
- api/design.md (Return types)
- app/lifecycle.md (Error display in UI)

### Performance
- global/performance.md (Hot paths, benchmarking)
- app/components.md (View optimization)
- api/async.md (Task batching)

### UI Development
- app/lifecycle.md (Application trait)
- app/components.md (Widget patterns)
- app/theming.md (COSMIC design system)
- app/accessibility.md (A11y patterns)

### Testing
- testing/test-writing.md (All test types)
- safety/verification.md (Static analysis)
- Each standard includes test examples

## Quick Reference

### For New Developers
Start with:
1. README.md - Architecture overview
2. global/tech-stack.md - Technology stack
3. global/conventions.md - Project structure
4. app/lifecycle.md - Application basics

### For Application Developers
Focus on:
1. app/lifecycle.md - Application trait
2. app/components.md - Widget catalog
3. app/theming.md - COSMIC design system
4. api/state-management.md - State patterns

### For Library Developers
Focus on:
1. api/design.md - API patterns
2. global/error-handling.md - Error types
3. safety/memory.md - Unsafe code
4. testing/test-writing.md - Testing utilities

### For Performance
Focus on:
1. global/performance.md - Optimization
2. api/async.md - Async patterns
3. app/components.md - View efficiency

### For Safety/Security
Focus on:
1. safety/memory.md - Memory safety
2. safety/verification.md - Static analysis
3. global/validation.md - Input validation
4. global/error-handling.md - Error handling

## Microsoft Guideline Coverage

This standards collection implements the following Microsoft Rust guidelines:

**Project Organization:**
- M-SMALLER-CRATES - Prefer focused crates
- M-NO-GLOB-REEXPORTS - Individual re-exports
- M-DOC-INLINE - Inline re-exported docs
- M-AVOID-STATICS - Dependency injection over statics
- M-MODULE-DOCS - Module documentation required
- M-OOBE - Build without prerequisites

**Error Handling:**
- M-ERRORS-CANONICAL-STRUCTS - Error struct pattern
- M-PANIC-IS-STOP - Panics terminate programs
- M-PANIC-ON-BUG - Panic for programming errors

**API Design:**
- M-STRONG-TYPES - Domain types over primitives
- M-IMPL-ASREF - Flexible parameters
- M-INIT-BUILDER - Builder for 3+ options
- M-CONCISE-NAMES - Clear, concise naming

**Documentation:**
- M-FIRST-DOC-SENTENCE - 15-word summary rule
- M-MODULE-DOCS - Comprehensive module docs

**Safety:**
- M-UNSAFE - Strict unsafe guidelines
- M-UNSOUND - Soundness requirements
- M-UNSAFE-IMPLIES-UB - Unsafe means UB possible

**Performance:**
- M-HOTPATH - Identify critical paths
- M-THROUGHPUT - Optimize for throughput

**Testing:**
- M-TEST-UTIL - Feature-gate test utilities
- M-MOCKABLE-SYSCALLS - Mockable I/O

**Verification:**
- M-STATIC-VERIFICATION - Comprehensive linting

## Maintenance

### Updating Standards
When modifying standards:
1. Follow Microsoft Rust guidelines
2. Include comprehensive code examples
3. Provide good and bad examples
4. Add relevant cross-references
5. Update this summary if structure changes
6. Test examples in real code

### Version History
- **Version 2.0** (2025-11-30): Complete rewrite for Rust + libcosmic
  - 20 standards documents
  - Based on Microsoft Rust guidelines
  - Integrated with libcosmic patterns
  - Removed Flutter/Dart content
  - Added performance guidelines
  - Added safety/verification standards

- **Version 1.0** (2025-11-15): Initial Flutter-Rust standards (deprecated)

## Contributing

All standards should:
- Follow Microsoft Rust guidelines
- Include practical code examples
- Show correct and incorrect patterns
- Provide implementation checklists
- Reference related standards
- Be tested in real projects

## File Size Reference

Total documentation coverage across:
- Global standards: 7 files covering foundation
- Application standards: 4 files covering UI
- API design: 4 files covering Rust patterns
- Safety: 2 files covering memory and verification
- Testing: 1 file covering all test types
- Documentation: 2 files (README, this summary)

**Total: 20 comprehensive files**
