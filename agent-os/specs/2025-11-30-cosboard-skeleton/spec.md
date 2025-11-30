# Specification: Cosboard Skeleton

## Goal
Create the foundational window framework for the Cosboard soft keyboard application, including a floating always-on-top window using `cosmic::Application`, a separate system tray applet, and state persistence for window position and size.

## User Stories
- As a user, I want a soft keyboard window that floats above other applications so I can type without switching focus
- As a user, I want to show/hide the keyboard from the system tray so I can access it conveniently without cluttering my screen

## Specific Requirements

**Application Window Framework**
- Implement `cosmic::Application` trait for the main keyboard window
- Use APP_ID following RDNN format: `io.github.cosboard.Cosboard`
- Initialize with `cosmic::app::run::<AppModel>(settings, ())` pattern from cosmic-app-template
- Include `cosmic::Core` in AppModel struct for runtime state management
- Set `core.window.show_headerbar = false` for chromeless appearance

**Borderless Window with Resize Handles**
- Configure `Settings::default().client_decorations(true)` for client-side decorations
- Set `resizable: Some(8.0)` in Settings to enable resize border of 8 pixels
- Disable window decorations via `window_settings.decorations = false`
- Window should have no title bar, borders, or standard window controls

**Always-on-Top via Layer-Shell**
- Use Wayland layer-shell protocol (`zwlr_layer_shell_v1`) for overlay behavior
- Leverage `cctk` (cosmic-client-toolkit) for layer-shell integration
- Set layer to overlay layer to ensure keyboard stays above normal windows
- Implement as layer surface rather than standard XDG toplevel window

**Default Window Dimensions**
- Default size: 800x300 pixels via `Settings::default().size(iced::Size::new(800.0, 300.0))`
- Minimum constraints via `size_limits(Limits::NONE.min_width(400.0).min_height(150.0))`
- Size limits should be configurable and eventually sourced from keyboard JSON layout file
- Store default dimensions in centralized `app_settings.rs` module

**State Persistence for Window Position/Size**
- Use `cosmic_config::Config::new_state(APP_ID, VERSION)` for state persistence
- Define State struct with `CosmicConfigEntry` derive macro containing x, y, width, height fields
- Save state on window move/resize using `on_window_resize` callback from Application trait
- Load saved state on application init to restore previous window position
- Use atomic writes via `ConfigSet::set()` for real-time state updates

**System Tray Applet**
- Create separate applet module using `cosmic::applet::run::<AppletModel>(flags)` pattern
- Applet implements `cosmic::Application` trait with applet-specific settings
- Use `cosmic::applet::Context::default()` for applet configuration
- Left-click: Send D-Bus message to show/toggle main keyboard window visibility
- Right-click: Display popup context menu with show/hide and quit options

**D-Bus Communication Between Components**
- Use `zbus` for D-Bus interface between applet and main application
- Define interface at path `/io/github/cosboard/Cosboard` for window control
- Expose methods: `Show()`, `Hide()`, `Toggle()`, `Quit()`
- Main application registers as D-Bus service; applet acts as client

**Centralized Configuration Module**
- Create `src/app_settings.rs` for application-wide constants and defaults
- Define default window dimensions, minimum constraints, APP_ID
- Configuration values should be easily adjustable in one location
- Future: Support loading size constraints from keyboard JSON files

**Project Structure**
- Follow cosmic-app-template structure: `src/main.rs`, `src/app.rs`, `src/config.rs`
- Add `src/app_settings.rs` for centralized settings
- Add `src/applet/mod.rs` for system tray applet implementation
- Add `src/state.rs` for window state persistence struct
- Include `resources/` directory with desktop entry and icons
- Include `i18n/` directory for future localization support

## Visual Design
No visual assets provided for this skeleton specification. The window should appear as a transparent, empty container with invisible resize handles until keyboard layout rendering is implemented in future specifications.

## Existing Code to Leverage

**cosmic-app-template (`~/Documents/libraries/cosmic-app-template`)**
- Use as base project template for file structure and build configuration
- Reference `src/app.rs` for Application trait implementation pattern
- Reference `src/config.rs` for CosmicConfigEntry usage with derive macro
- Copy `Cargo.toml` structure and libcosmic feature configuration
- Use justfile for build and install recipes

**libcosmic Application Module (`~/Documents/libraries/libcosmic/src/app/`)**
- Reference `mod.rs` for Application trait definition and required methods
- Use `Settings` struct from `settings.rs` for window configuration
- Leverage `on_window_resize` callback for position/size change detection
- Use `core.window.*` fields for headerbar and decoration control

**libcosmic Applet Module (`~/Documents/libraries/libcosmic/src/applet/`)**
- Reference `mod.rs` for applet::Context and applet-specific settings
- Use `applet::run()` function to launch applet with proper panel integration
- Leverage popup and tooltip helpers for context menu implementation
- Reference icon_button and menu_button widget helpers

**cosmic-config (`~/Documents/libraries/libcosmic/cosmic-config/`)**
- Use `Config::new_state()` for runtime state persistence (not config)
- Implement `CosmicConfigEntry` trait via derive macro for State struct
- Use `watch_state()` subscription for config change notifications
- Reference `ConfigSet::set()` for atomic state updates

**libcosmic Core Module (`~/Documents/libraries/libcosmic/src/core.rs`)**
- Reference `Core` struct for window state and focus management
- Use `Window` struct fields for headerbar visibility control
- Leverage `watch_config` and `watch_state` for configuration subscriptions

## Out of Scope
- Keyboard layout definitions and JSON layout file parsing
- Key rendering, buttons, and visual keyboard display
- Input handling, key press events, and virtual keyboard protocol
- Long-press, swipe, or gesture detection for keys
- Word prediction, autocomplete, or text suggestion features
- Speech-to-text integration
- Theming or custom keyboard appearance settings
- Sound or haptic feedback for key presses
- Multi-language or alternative keyboard layout support
- Settings UI or preferences panel
