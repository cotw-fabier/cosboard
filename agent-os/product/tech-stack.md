# Tech Stack

## Core Technologies

### Programming Language

**Rust**
- Primary and only language for the entire codebase
- Chosen for performance, memory safety, and ecosystem compatibility with Cosmic desktop
- Enables direct integration with libcosmic without FFI overhead

### UI Framework

**LibCosmic (built on Iced)**
- Native UI framework for the Cosmic desktop environment
- Built on top of Iced, a cross-platform GUI library for Rust
- Provides consistent theming, accessibility support, and desktop integration
- Applet architecture for system tray and panel integration

### Data Format

**JSON**
- Used for declarative keyboard layout definitions
- Human-readable and editable without specialized tools
- Enables layout sharing and community contributions
- Schema will define: key positions, sizes, labels, key codes, modifiers, actions, and gesture bindings

## Future Technologies

### Speech-to-Text (Phase 5)

**ONNX Runtime**
- Cross-platform machine learning inference engine
- CPU-powered inference (no GPU required)
- Rust bindings via `ort` crate

**Parakeet v3 Model**
- NVIDIA's automatic speech recognition model
- State-of-the-art accuracy with efficient inference
- Optimized for real-time transcription
- All processing done locally for privacy

### Scripting Engine (Phase 3)

**Rhai**
- Embedded scripting language designed for Rust
- Safe sandboxed execution
- Easy to learn syntax similar to JavaScript
- Enables user-defined key actions and macros without recompilation

## Architecture Decisions

### Input Handling

- Integration with system input layer for key event emission
- Support for standard key codes and Unicode input
- Modifier key state management (Shift, Ctrl, Alt, Super)

### Layout System

- JSON schema for layout definitions
- Runtime layout loading and hot-reloading
- Support for multiple layout layers (base, shift, symbols, etc.)
- Extensible action system for future gesture and script support

### Applet Modes

- Docked mode: Fixed position at screen edge
- Floating mode: Draggable, resizable window (Phase 3)
- Size presets for different use cases and screen sizes

### Performance Considerations

- Efficient rendering through Iced's retained-mode architecture
- Lazy loading of layouts and resources
- Optimized STT inference for real-time performance on CPU

## Dependencies (Planned)

| Category | Crate | Purpose |
|----------|-------|---------|
| UI | `libcosmic` | Cosmic desktop integration |
| UI | `iced` | Underlying GUI framework |
| Data | `serde` | JSON serialization/deserialization |
| Data | `serde_json` | JSON parsing |
| Scripting | `rhai` | Embedded scripting (Phase 3) |
| ML | `ort` | ONNX runtime bindings (Phase 5) |
| Audio | `cpal` | Cross-platform audio capture (Phase 5) |

## Build and Distribution

- Standard Rust toolchain (cargo)
- Targeting Linux with Cosmic desktop
- Package formats: Flatpak, native packages for Pop!_OS and other distributions
- Layout files distributed separately from binary for easy customization
