# Task Breakdown: Cosboard Skeleton

## Overview
Total Tasks: 44 (across 7 task groups)

This specification creates the foundational window framework for the Cosboard soft keyboard application, including:
- Floating always-on-top window using `cosmic::Application`
- System tray applet for show/hide control
- D-Bus communication between components
- State persistence for window position and size

## Task List

### Project Setup

#### Task Group 1: Project Structure and Configuration
**Dependencies:** None

- [ ] 1.0 Complete project setup and structure
  - [ ] 1.1 Initialize Rust project with Cargo
    - Create `/home/fabier/Documents/code/cosboard/Cargo.toml` with project metadata
    - Package name: `cosboard`
    - Edition: 2024
    - Add repository URL and license
  - [ ] 1.2 Configure libcosmic dependencies in Cargo.toml
    - Add libcosmic git dependency with features:
      - `a11y` (accessibility)
      - `about` (about widget)
      - `dbus-config` (config watching via D-Bus)
      - `multi-window` (multiple windows support)
      - `single-instance` (focus existing instance)
      - `tokio` (async runtime)
      - `winit` (windowing)
      - `wayland` (Wayland support)
      - `wgpu` (GPU rendering)
      - `applet` (system tray applet support)
    - Add zbus for D-Bus communication
    - Add futures-util, tokio, serde dependencies
  - [ ] 1.3 Create source directory structure
    - `/home/fabier/Documents/code/cosboard/src/main.rs`
    - `/home/fabier/Documents/code/cosboard/src/app.rs`
    - `/home/fabier/Documents/code/cosboard/src/config.rs`
    - `/home/fabier/Documents/code/cosboard/src/state.rs`
    - `/home/fabier/Documents/code/cosboard/src/app_settings.rs`
    - `/home/fabier/Documents/code/cosboard/src/i18n.rs`
    - `/home/fabier/Documents/code/cosboard/src/applet/mod.rs`
    - `/home/fabier/Documents/code/cosboard/src/dbus/mod.rs`
  - [ ] 1.4 Create resources directory
    - `/home/fabier/Documents/code/cosboard/resources/io.github.cosboard.Cosboard.desktop`
    - `/home/fabier/Documents/code/cosboard/resources/io.github.cosboard.Cosboard.metainfo.xml`
    - `/home/fabier/Documents/code/cosboard/resources/icons/hicolor/scalable/apps/io.github.cosboard.Cosboard.svg`
  - [ ] 1.5 Create i18n directory and base translations
    - `/home/fabier/Documents/code/cosboard/i18n/en/cosboard.ftl`
    - `/home/fabier/Documents/code/cosboard/i18n.toml`
  - [ ] 1.6 Create justfile for build automation
    - `/home/fabier/Documents/code/cosboard/justfile`
    - Include: build-debug, build-release, run, install, uninstall recipes
    - Configure APP_ID as `io.github.cosboard.Cosboard`
  - [ ] 1.7 Create .gitignore file
    - `/home/fabier/Documents/code/cosboard/.gitignore`
    - Ignore: target/, .cargo/, vendor/, *.tar
  - [ ] 1.8 Verify project compiles (empty main)
    - Run `cargo check` to verify dependencies resolve
    - Ensure no compilation errors

**Acceptance Criteria:**
- Project structure matches cosmic-app-template pattern
- All required directories and placeholder files exist
- `cargo check` passes without errors
- justfile contains all standard build recipes

---

### Centralized Configuration

#### Task Group 2: App Settings and Constants Module
**Dependencies:** Task Group 1

- [ ] 2.0 Complete centralized configuration module
  - [ ] 2.1 Create app_settings.rs with application constants
    - File: `/home/fabier/Documents/code/cosboard/src/app_settings.rs`
    - Define `APP_ID: &str = "io.github.cosboard.Cosboard"`
    - Define `APP_VERSION: u64 = 1`
    - Define `DEFAULT_WIDTH: f32 = 800.0`
    - Define `DEFAULT_HEIGHT: f32 = 300.0`
    - Define `MIN_WIDTH: f32 = 400.0`
    - Define `MIN_HEIGHT: f32 = 150.0`
    - Define `RESIZE_BORDER: f64 = 8.0`
    - Define D-Bus path: `/io/github/cosboard/Cosboard`
    - Define D-Bus interface name
  - [ ] 2.2 Create config.rs for user configuration
    - File: `/home/fabier/Documents/code/cosboard/src/config.rs`
    - Implement `Config` struct with `CosmicConfigEntry` derive
    - Version field using `#[version = 1]`
    - Placeholder fields for future keyboard settings
  - [ ] 2.3 Create state.rs for window state persistence
    - File: `/home/fabier/Documents/code/cosboard/src/state.rs`
    - Implement `WindowState` struct with `CosmicConfigEntry` derive
    - Fields: `x: i32`, `y: i32`, `width: f32`, `height: f32`
    - Version field using `#[version = 1]`
    - Implement Default trait with values from app_settings
  - [ ] 2.4 Create i18n.rs for localization support
    - File: `/home/fabier/Documents/code/cosboard/src/i18n.rs`
    - Use `rust_embed` for embedding translations
    - Implement `fl!` macro for localized strings
    - Reference pattern from cosmic-app-template
  - [ ] 2.5 Create base English translations
    - File: `/home/fabier/Documents/code/cosboard/i18n/en/cosboard.ftl`
    - Define: `app-title = Cosboard`
    - Define: `show-keyboard = Show Keyboard`
    - Define: `hide-keyboard = Hide Keyboard`
    - Define: `quit = Quit`
    - Define: `about = About`
  - [ ] 2.6 Verify configuration modules compile
    - Run `cargo check`
    - Ensure CosmicConfigEntry derives work correctly

**Acceptance Criteria:**
- All constants centralized in app_settings.rs
- Config and State structs derive CosmicConfigEntry
- i18n module properly embeds translations
- `cargo check` passes

---

### Core Application Window

#### Task Group 3: Main Application Window Framework
**Dependencies:** Task Group 2

- [ ] 3.0 Complete main application window
  - [ ] 3.1 Write 4-6 focused tests for core window functionality
    - Test: Application initializes with correct default dimensions
    - Test: State loads from cosmic_config on init
    - Test: State saves on window resize
    - Test: Window settings match app_settings values
    - Test: Headerbar is hidden (chromeless mode)
  - [ ] 3.2 Create main.rs application entry point
    - File: `/home/fabier/Documents/code/cosboard/src/main.rs`
    - Initialize i18n with requested languages
    - Configure `Settings::default()`:
      - `.size(iced::Size::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))`
      - `.size_limits(Limits::NONE.min_width(MIN_WIDTH).min_height(MIN_HEIGHT))`
      - `.resizable(Some(RESIZE_BORDER))`
      - `.client_decorations(true)`
      - `.transparent(true)`
    - Call `cosmic::app::run::<app::AppModel>(settings, ())`
  - [ ] 3.3 Create AppModel struct in app.rs
    - File: `/home/fabier/Documents/code/cosboard/src/app.rs`
    - Include `core: cosmic::Core` field
    - Include `config: Config` field
    - Include `window_state: WindowState` field
    - Include `state_config: Option<cosmic_config::Config>` for state persistence
  - [ ] 3.4 Implement cosmic::Application trait for AppModel
    - Set `const APP_ID: &'static str` from app_settings
    - Implement `type Executor = cosmic::executor::Default`
    - Implement `type Flags = ()`
    - Implement `type Message` enum
    - Implement `core()` and `core_mut()` methods
  - [ ] 3.5 Implement init() method
    - Load config using `cosmic_config::Config::new(APP_ID, Config::VERSION)`
    - Load state using `cosmic_config::Config::new_state(APP_ID, WindowState::VERSION)`
    - Restore window position/size from loaded state
    - Set `core.window.show_headerbar = false` for chromeless appearance
    - Return initialized AppModel and startup tasks
  - [ ] 3.6 Implement view() method
    - Return minimal transparent container
    - Use `widget::container` with no content (skeleton only)
    - Set width and height to `Length::Fill`
  - [ ] 3.7 Implement Message enum and update() method
    - Message variants: `WindowResized(f32, f32)`, `SaveState`, `UpdateConfig(Config)`
    - Handle window resize to update internal state
    - Trigger state save on resize
  - [ ] 3.8 Implement on_window_resize() callback
    - Update window_state with new dimensions
    - Save state using `state_config.set()` for atomic writes
    - Log resize events for debugging
  - [ ] 3.9 Implement subscription() method
    - Watch for config changes using `core.watch_config::<Config>(APP_ID)`
    - Map config updates to UpdateConfig message
  - [ ] 3.10 Ensure core window tests pass
    - Run only the tests from 3.1
    - Verify window initializes correctly
    - Do NOT run entire test suite

**Acceptance Criteria:**
- The 4-6 tests from 3.1 pass
- Application window appears borderless with resize handles
- Window dimensions match defaults from app_settings
- Headerbar is hidden (chromeless)
- State saves on resize
- Application initializes without errors

---

### Layer-Shell Integration

#### Task Group 4: Always-on-Top via Layer-Shell
**Dependencies:** Task Group 3

- [ ] 4.0 Complete layer-shell integration for overlay behavior
  - [ ] 4.1 Write 2-4 focused tests for layer-shell behavior
    - Test: Window configured as layer surface
    - Test: Layer set to overlay layer
    - Test: Window stays above normal windows
  - [ ] 4.2 Research cctk layer-shell integration
    - Review libcosmic's use of `cctk::sctk` for layer surfaces
    - Identify required Wayland protocols (`zwlr_layer_shell_v1`)
    - Document approach for overlay layer configuration
  - [ ] 4.3 Configure window as layer surface
    - Modify Settings configuration in main.rs
    - Use cctk layer-shell APIs if available in libcosmic
    - Set layer to overlay (highest z-order)
    - Configure anchor and exclusive zone as needed
  - [ ] 4.4 Handle layer-shell fallback for non-Wayland
    - Implement graceful fallback for X11 environments
    - Use standard window hints for always-on-top where layer-shell unavailable
    - Log warnings when layer-shell not available
  - [ ] 4.5 Ensure layer-shell tests pass
    - Run only tests from 4.1
    - Verify overlay behavior on Wayland
    - Do NOT run entire test suite

**Acceptance Criteria:**
- The 2-4 tests from 4.1 pass
- Window floats above other applications on Wayland
- Graceful fallback on non-Wayland systems
- No crashes when layer-shell unavailable

---

### D-Bus Communication

#### Task Group 5: D-Bus Interface for Window Control
**Dependencies:** Task Group 3

- [ ] 5.0 Complete D-Bus interface implementation
  - [ ] 5.1 Write 3-5 focused tests for D-Bus interface
    - Test: D-Bus service registers successfully
    - Test: Show method triggers window visibility
    - Test: Hide method triggers window hide
    - Test: Toggle method alternates visibility
    - Test: Quit method triggers application exit
  - [ ] 5.2 Create dbus/mod.rs module
    - File: `/home/fabier/Documents/code/cosboard/src/dbus/mod.rs`
    - Import zbus crate
    - Define D-Bus interface path from app_settings
  - [ ] 5.3 Define D-Bus interface trait
    - Use `#[zbus::interface]` attribute
    - Interface name: `io.github.cosboard.Cosboard`
    - Methods: `Show()`, `Hide()`, `Toggle()`, `Quit()`
    - Signal: `VisibilityChanged(visible: bool)`
  - [ ] 5.4 Implement D-Bus server in main application
    - Register D-Bus service on session bus
    - Create interface implementation struct
    - Connect methods to application message channel
    - Handle connection errors gracefully
  - [ ] 5.5 Create D-Bus client helper for applet
    - Implement async methods: `show()`, `hide()`, `toggle()`, `quit()`
    - Handle connection failures with retries
    - Provide blocking variants for simple use cases
  - [ ] 5.6 Integrate D-Bus with AppModel
    - Add Message variants for D-Bus commands
    - Update `update()` to handle Show/Hide/Toggle/Quit
    - Emit D-Bus signals on visibility changes
  - [ ] 5.7 Ensure D-Bus tests pass
    - Run only tests from 5.1
    - Verify service registration
    - Verify method calls work correctly

**Acceptance Criteria:**
- The 3-5 tests from 5.1 pass
- D-Bus service registers on session bus
- All four methods (Show, Hide, Toggle, Quit) function correctly
- Signals emit on visibility changes
- Graceful handling of D-Bus errors

---

### System Tray Applet

#### Task Group 6: System Tray Applet Component
**Dependencies:** Task Groups 3, 5

- [ ] 6.0 Complete system tray applet
  - [ ] 6.1 Write 3-5 focused tests for applet functionality
    - Test: Applet initializes with correct icon
    - Test: Left-click sends toggle D-Bus message
    - Test: Right-click opens popup menu
    - Test: Menu items trigger correct D-Bus calls
  - [ ] 6.2 Create applet/mod.rs module
    - File: `/home/fabier/Documents/code/cosboard/src/applet/mod.rs`
    - Import cosmic::applet module
    - Import D-Bus client from dbus module
  - [ ] 6.3 Create AppletModel struct
    - Include `core: cosmic::Core` field
    - Include `popup_open: bool` field
    - Include `dbus_client: Option<DbusClient>` field
    - Include `applet_helper: cosmic::applet::Context` field
  - [ ] 6.4 Implement cosmic::Application trait for AppletModel
    - Set `const APP_ID: &'static str = "io.github.cosboard.Cosboard.Applet"`
    - Implement `type Executor = cosmic::executor::Default`
    - Implement `type Flags = ()`
    - Configure applet-specific settings
  - [ ] 6.5 Implement init() for applet
    - Initialize applet context with `cosmic::applet::Context::default()`
    - Set `core.window.show_headerbar = false`
    - Initialize D-Bus client connection
    - Return AppletModel and startup tasks
  - [ ] 6.6 Implement view() for applet icon button
    - Use `applet_helper.icon_button("keyboard-symbolic")`
    - Attach left-click handler for toggle
    - Use `applet_helper.applet_tooltip()` for hover tooltip
  - [ ] 6.7 Implement popup menu for right-click
    - Use `applet_helper.popup_container()` for menu
    - Menu items: "Show Keyboard", "Hide Keyboard", separator, "Quit"
    - Use `applet::menu_button()` for menu items
    - Connect items to D-Bus client methods
  - [ ] 6.8 Implement Message enum and update() for applet
    - Message variants: `Toggle`, `Show`, `Hide`, `Quit`, `PopupOpen`, `PopupClose`
    - Handle D-Bus client calls in update
    - Manage popup state
  - [ ] 6.9 Create applet entry point
    - Separate binary or feature-gated main function
    - Use `cosmic::applet::run::<AppletModel>(())` to launch
    - Configure for panel integration
  - [ ] 6.10 Ensure applet tests pass
    - Run only tests from 6.1
    - Verify icon displays correctly
    - Verify D-Bus integration works

**Acceptance Criteria:**
- The 3-5 tests from 6.1 pass
- Applet icon appears in system tray
- Left-click toggles keyboard visibility via D-Bus
- Right-click displays popup context menu
- Menu items trigger correct actions
- Proper panel integration

---

### Integration and Testing

#### Task Group 7: Test Review and Final Integration
**Dependencies:** Task Groups 1-6

- [ ] 7.0 Review existing tests and fill critical gaps
  - [ ] 7.1 Review tests from Task Groups 3-6
    - Review 4-6 tests from core window (Task 3.1)
    - Review 2-4 tests from layer-shell (Task 4.1)
    - Review 3-5 tests from D-Bus (Task 5.1)
    - Review 3-5 tests from applet (Task 6.1)
    - Total existing tests: approximately 12-20 tests
  - [ ] 7.2 Analyze test coverage gaps for this feature only
    - Identify critical integration points lacking coverage
    - Focus on end-to-end workflows:
      - Applet click -> D-Bus -> Main app visibility toggle
      - Window resize -> State save -> State restore on restart
      - Layer-shell overlay behavior verification
    - Do NOT assess entire application test coverage
  - [ ] 7.3 Write up to 8 additional integration tests
    - Test: Full toggle workflow (applet -> D-Bus -> window)
    - Test: State persistence across application restart
    - Test: Multiple rapid resize events debounce correctly
    - Test: D-Bus service cleanup on application exit
    - Test: Applet reconnects if main app restarts
    - Test: Window position restoration accuracy
    - Add tests only for critical gaps identified
    - Maximum of 8 additional tests
  - [ ] 7.4 Create integration test for full workflow
    - Test complete user flow: launch app, toggle from applet, resize, quit
    - Verify all components communicate correctly
    - Test on Wayland compositor if available
  - [ ] 7.5 Run feature-specific tests only
    - Run ALL tests from Task Groups 3-7
    - Expected total: approximately 20-28 tests
    - Verify all critical workflows pass
    - Do NOT run unrelated application tests
  - [ ] 7.6 Create README.md for the project
    - Document build instructions
    - Document installation steps
    - Document usage (main app and applet)
    - List dependencies and requirements
  - [ ] 7.7 Verify desktop entry and installation
    - Test `just install` recipe
    - Verify desktop entry appears in application menu
    - Verify icon displays correctly
    - Test `just uninstall` recipe

**Acceptance Criteria:**
- All feature-specific tests pass (approximately 20-28 total)
- Full toggle workflow operates correctly
- State persistence works across restarts
- Project documentation is complete
- Installation/uninstallation works correctly
- No more than 8 additional tests added in gap analysis

---

## Execution Order

Recommended implementation sequence:

1. **Project Setup (Task Group 1)** - Foundation for all other work
2. **Centralized Configuration (Task Group 2)** - Required by all components
3. **Core Application Window (Task Group 3)** - Main window framework
4. **D-Bus Communication (Task Group 5)** - Can proceed in parallel with 4
5. **Layer-Shell Integration (Task Group 4)** - Can proceed in parallel with 5
6. **System Tray Applet (Task Group 6)** - Requires D-Bus to be complete
7. **Integration and Testing (Task Group 7)** - Final verification

## File Structure Summary

```
/home/fabier/Documents/code/cosboard/
|-- Cargo.toml
|-- justfile
|-- i18n.toml
|-- .gitignore
|-- src/
|   |-- main.rs
|   |-- app.rs
|   |-- config.rs
|   |-- state.rs
|   |-- app_settings.rs
|   |-- i18n.rs
|   |-- applet/
|   |   |-- mod.rs
|   |-- dbus/
|       |-- mod.rs
|-- i18n/
|   |-- en/
|       |-- cosboard.ftl
|-- resources/
    |-- io.github.cosboard.Cosboard.desktop
    |-- io.github.cosboard.Cosboard.metainfo.xml
    |-- icons/
        |-- hicolor/
            |-- scalable/
                |-- apps/
                    |-- io.github.cosboard.Cosboard.svg
```

## Reference Materials

- **Template:** `/home/fabier/Documents/libraries/cosmic-app-template`
- **Library:** `/home/fabier/Documents/libraries/libcosmic`
- **Key Files:**
  - `libcosmic/src/app/mod.rs` - Application trait
  - `libcosmic/src/app/settings.rs` - Settings struct
  - `libcosmic/src/applet/mod.rs` - Applet module
  - `libcosmic/src/core.rs` - Core struct
  - `libcosmic/cosmic-config/src/lib.rs` - Config persistence
