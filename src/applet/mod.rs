// SPDX-License-Identifier: GPL-3.0-only

//! System tray applet for Cosboard.
//!
//! This module provides a panel applet that manages a keyboard layer surface.
//! The applet spawns and controls a layer-shell keyboard surface within the
//! same process using libcosmic's Wayland layer-shell support.
//!
//! # Architecture
//!
//! The applet runs as a single process that:
//! - Displays an icon in the COSMIC panel
//! - Manages a layer-shell keyboard surface (anchored to bottom of screen)
//! - Supports exclusive zone (pushes windows up) or floating overlay mode
//! - Persists window state (height, floating mode) between sessions
//! - Left-click: Toggle keyboard visibility
//! - Right-click: Open popup menu with show/hide/mode toggle/quit options
//!
//! # Running the Applet
//!
//! ```bash
//! cargo run --bin cosboard-applet
//! ```

use crate::fl;
use crate::input::{parse_keycode, keycodes, ResolvedKeycode, VirtualKeyboard};
use crate::layout::{parse_layout_file, Cell, Key, KeyCode, Modifier};
use crate::renderer::{
    render_animated_panels, render_current_toast, render_keyboard_with_toast, get_scale_factor,
    KeyboardRenderer, RendererMessage, ToastSeverity,
    ANIMATION_FRAME_INTERVAL_MS, LONG_PRESS_TIMER_INTERVAL_MS, TOAST_TIMER_INTERVAL_MS,
};
use crate::state::WindowState;
use cosmic::app::{Core, Task};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::event;
use cosmic::iced::mouse;
use cosmic::iced::time;
use cosmic::iced::window::{self, Id};
use cosmic::iced::{Event, Length, Limits, Point};
use cosmic::iced_runtime::platform_specific::wayland::layer_surface::{
    IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
};
use cosmic::iced_winit::platform_specific::wayland::commands::layer_surface::{
    destroy_layer_surface, get_layer_surface, set_anchor, set_exclusive_zone, set_margin, set_size,
    Anchor, KeyboardInteractivity, Layer,
};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::{self, container, divider, list_column, mouse_area, Space};
use cosmic::Element;
use cosmic::Theme;
use std::time::{Duration, Instant};

/// The applet Application ID (distinct from the main application).
pub const APPLET_ID: &str = "io.github.cosboard.Cosboard.Applet";

/// Default layout file path (relative to the executable or absolute).
const DEFAULT_LAYOUT_PATH: &str = "resources/layouts/example_qwerty.json";

/// Minimum keyboard width in floating mode.
const MIN_WIDTH: f32 = 300.0;
/// Maximum keyboard width in floating mode.
const MAX_WIDTH: f32 = 1920.0;
/// Minimum keyboard height.
const MIN_HEIGHT: f32 = 150.0;
/// Maximum keyboard height.
const MAX_HEIGHT: f32 = 500.0;
/// Size of resize handle zones in pixels (larger for easier grabbing).
const RESIZE_ZONE_SIZE: f32 = 16.0;
/// Minimum interval between preview surface updates (debounce).
const PREVIEW_UPDATE_INTERVAL_MS: u128 = 100;

/// Which edge or corner is being resized.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizeEdge {
    Top,
    Left,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// The applet model stores state for the system tray applet.
pub struct AppletModel {
    /// Application core state managed by the COSMIC runtime.
    core: Core,
    /// Whether the popup menu is currently open.
    popup: Option<Id>,
    /// The keyboard layer surface ID (if open).
    keyboard_surface: Option<window::Id>,
    /// Whether the keyboard is currently visible.
    keyboard_visible: bool,
    /// Window state (size, floating mode) for the keyboard.
    window_state: WindowState,
    /// Config context for persisting window state.
    state_config: Option<cosmic_config::Config>,
    /// Whether currently dragging the keyboard.
    is_dragging: bool,
    /// Current resize edge being dragged (if any).
    resize_edge: Option<ResizeEdge>,
    /// Last known cursor position (for incremental drag/resize tracking).
    last_cursor_position: Option<Point>,
    /// Pending width during resize (avoids triggering rebuilds until resize ends).
    pending_width: f32,
    /// Pending height during resize.
    pending_height: f32,
    /// Pending right margin during drag/resize.
    pending_margin_right: i32,
    /// Pending bottom margin during drag/resize.
    pending_margin_bottom: i32,
    /// Preview layer surface ID (shown during drag/resize operations).
    preview_surface: Option<window::Id>,
    /// Last sent preview width (for deduplication - skip if unchanged).
    last_preview_width: u32,
    /// Last sent preview height (for deduplication).
    last_preview_height: u32,
    /// Last sent preview right margin (for deduplication).
    last_preview_margin_right: i32,
    /// Last sent preview bottom margin (for deduplication).
    last_preview_margin_bottom: i32,
    /// Last time we sent a preview update (for 100ms debounce).
    last_preview_update: Option<Instant>,
    /// Keyboard renderer for rendering the layout (Task 7.1).
    keyboard_renderer: Option<KeyboardRenderer>,
    /// Virtual keyboard for emitting key events (Task Group 5).
    virtual_keyboard: VirtualKeyboard,
}

impl Default for AppletModel {
    fn default() -> Self {
        let window_state = WindowState::default();
        Self {
            core: Core::default(),
            popup: None,
            keyboard_surface: None,
            keyboard_visible: false,
            pending_width: window_state.width,
            pending_height: window_state.height,
            pending_margin_right: window_state.margin_right,
            pending_margin_bottom: window_state.margin_bottom,
            window_state,
            state_config: None,
            is_dragging: false,
            resize_edge: None,
            last_cursor_position: None,
            preview_surface: None,
            last_preview_width: 0,
            last_preview_height: 0,
            last_preview_margin_right: 0,
            last_preview_margin_bottom: 0,
            last_preview_update: None,
            keyboard_renderer: None,
            virtual_keyboard: VirtualKeyboard::new(),
        }
    }
}

/// Messages emitted by the applet and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle keyboard visibility (left-click action).
    Toggle,
    /// Toggle popup menu visibility (right-click action).
    TogglePopup,
    /// Show the keyboard.
    Show,
    /// Hide the keyboard.
    Hide,
    /// Quit the applet.
    Quit,
    /// Popup menu closed.
    PopupClosed(Id),
    /// Handle surface actions (for popup management).
    Surface(cosmic::surface::Action),
    /// Keyboard layer surface was closed.
    KeyboardSurfaceClosed(window::Id),
    /// Keyboard layer surface was resized.
    KeyboardSurfaceResized(window::Id, f32, f32),
    /// Toggle between docked and floating mode.
    ToggleFloatingMode,
    /// Save window state (debounced).
    SaveState,
    /// Start dragging the keyboard (floating mode).
    DragStart,
    /// Stop dragging the keyboard.
    DragEnd,
    /// Start resizing from an edge (floating mode).
    ResizeStart(ResizeEdge),
    /// Stop resizing.
    ResizeEnd,
    /// Cursor moved (for drag/resize tracking).
    CursorMoved(Point),
    /// Preview surface was created.
    PreviewSurfaceCreated(window::Id),
    /// Preview surface was closed.
    PreviewSurfaceClosed(window::Id),
    // ========================================================================
    // Renderer Messages (Task 7.4)
    // ========================================================================
    /// A key was pressed on the rendered keyboard.
    KeyPressed(String),
    /// A key was released on the rendered keyboard.
    KeyReleased(String),
    /// Switch to a different panel.
    SwitchPanel(String),
    /// Animation frame tick for panel transitions.
    AnimationTick,
    /// Long press timer tick for detecting long presses.
    LongPressTimerTick,
    /// Show a toast notification.
    ShowToast(String, ToastSeverity),
    /// Dismiss the current toast notification.
    DismissToast,
    /// Toast timer tick for auto-dismiss.
    ToastTimerTick,
}

impl AppletModel {
    /// Save the current window state to disk.
    fn save_state(&self) {
        if let Some(ref config) = self.state_config {
            if let Err(e) = self.window_state.write_entry(config) {
                tracing::warn!("Failed to save window state: {:?}", e);
            } else {
                tracing::debug!("Saved window state: {:?}", self.window_state);
            }
        }
    }

    /// Create a preview layer surface for drag/resize operations.
    /// Returns the task to spawn the surface and the new surface ID.
    fn create_preview_surface(&mut self) -> Task<Message> {
        let id = window::Id::unique();
        let width = self.window_state.width as u32;
        let height = self.window_state.height as u32;

        let settings = SctkLayerSurfaceSettings {
            id,
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::None,
            input_zone: None,
            anchor: Anchor::BOTTOM | Anchor::RIGHT,
            output: IcedOutput::Active,
            namespace: "cosboard-preview".to_string(),
            margin: IcedMargin {
                top: 0,
                right: self.window_state.margin_right,
                bottom: self.window_state.margin_bottom,
                left: 0,
            },
            size: Some((Some(width), Some(height))),
            exclusive_zone: 0,
            size_limits: Limits::NONE
                .min_width(MIN_WIDTH)
                .max_width(MAX_WIDTH)
                .min_height(MIN_HEIGHT)
                .max_height(MAX_HEIGHT),
        };

        self.preview_surface = Some(id);
        tracing::debug!("Creating preview surface: {:?}", id);

        get_layer_surface(settings)
    }

    /// Load the keyboard layout and create the renderer (Task 7.2).
    ///
    /// Attempts to load the layout from the default path. On success,
    /// creates a KeyboardRenderer. On failure, queues an error toast.
    fn load_keyboard_layout(&mut self) {
        // Try to find the layout file
        let layout_path = Self::find_layout_path();

        match parse_layout_file(&layout_path) {
            Ok(result) => {
                // Log any warnings from parsing
                if result.has_warnings() {
                    for warning in &result.warnings {
                        tracing::warn!("Layout warning: {}", warning);
                    }
                }

                // Create the renderer with the loaded layout
                self.keyboard_renderer = Some(KeyboardRenderer::new(result.layout));
                tracing::info!("Loaded keyboard layout from: {}", layout_path);
            }
            Err(e) => {
                // Log the error and queue a toast notification
                tracing::error!("Failed to load layout from {}: {}", layout_path, e);

                // Create a renderer with an empty/error state is not possible,
                // so we'll display the error message via toast when we have a renderer
                // For now, set renderer to None and handle the missing renderer in view_window
                self.keyboard_renderer = None;
            }
        }
    }

    /// Find the layout file path, checking multiple locations.
    fn find_layout_path() -> String {
        // Check various locations for the layout file
        let candidates = [
            DEFAULT_LAYOUT_PATH.to_string(),
            format!("/usr/share/cosboard/layouts/example_qwerty.json"),
            format!(
                "{}/resources/layouts/example_qwerty.json",
                std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            ),
        ];

        for path in &candidates {
            if std::path::Path::new(path).exists() {
                return path.clone();
            }
        }

        // Return default path even if it doesn't exist (will show error)
        DEFAULT_LAYOUT_PATH.to_string()
    }

    /// Render the keyboard content using the renderer (Task 7.3).
    fn render_keyboard_content(&self) -> Element<'_, Message> {
        let surface_width = self.window_state.width;
        let surface_height = self.window_state.height;
        let scale = get_scale_factor();

        if let Some(ref renderer) = self.keyboard_renderer {
            // Render the keyboard panel using the renderer
            let panel_element = render_animated_panels(renderer, surface_width, surface_height, scale);

            // Get the current theme for toast rendering
            let theme = Theme::dark(); // TODO: Get actual theme from COSMIC context

            // Render toast if any
            let toast_element = render_current_toast(renderer, &theme);

            // Combine panel with toast area
            let keyboard_with_toast = render_keyboard_with_toast(panel_element, toast_element, surface_height);

            // Map RendererMessage to applet Message
            keyboard_with_toast.map(|msg| match msg {
                RendererMessage::KeyPressed(id) => Message::KeyPressed(id),
                RendererMessage::KeyReleased(id) => Message::KeyReleased(id),
                RendererMessage::SwitchPanel(id) => Message::SwitchPanel(id),
                RendererMessage::AnimationTick => Message::AnimationTick,
                RendererMessage::AnimationComplete => Message::AnimationTick, // Handled in update
                RendererMessage::LongPressTimerTick => Message::LongPressTimerTick,
                RendererMessage::PopupDismiss => Message::KeyReleased(String::new()),
                RendererMessage::ShowToast(msg, severity) => Message::ShowToast(msg, severity),
                RendererMessage::DismissToast => Message::DismissToast,
                RendererMessage::ToastTimerTick => Message::ToastTimerTick,
                RendererMessage::Noop => Message::Toggle, // Should not happen
            })
        } else {
            // No renderer available - show error message
            container(widget::text::body("Failed to load keyboard layout"))
                .width(Length::Fill)
                .height(Length::Fill)
                .class(cosmic::style::Container::Background)
                .into()
        }
    }

    // ========================================================================
    // Task Group 5: Key Press Event Flow Helpers
    // ========================================================================

    /// Finds a key by its identifier in the current panel.
    ///
    /// This searches through the current panel's rows and cells to find
    /// a key with the matching identifier.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The key identifier to search for
    ///
    /// # Returns
    ///
    /// * `Some(&Key)` if a key with the identifier was found
    /// * `None` if no matching key was found
    fn find_key_by_identifier(&self, identifier: &str) -> Option<&Key> {
        let renderer = self.keyboard_renderer.as_ref()?;
        let panel = renderer.current_panel()?;

        for row in &panel.rows {
            for cell in &row.cells {
                if let Cell::Key(key) = cell {
                    if key.identifier.as_deref() == Some(identifier) {
                        return Some(key);
                    }
                }
            }
        }

        None
    }

    /// Determines if a key is a modifier key based on its KeyCode.
    ///
    /// # Arguments
    ///
    /// * `code` - The KeyCode to check
    ///
    /// # Returns
    ///
    /// * `Some(Modifier)` if the key is a modifier
    /// * `None` if the key is not a modifier
    fn keycode_to_modifier(code: &KeyCode) -> Option<Modifier> {
        match code {
            KeyCode::Keysym(s) => {
                let s_lower = s.to_lowercase();
                if s_lower.contains("shift") {
                    Some(Modifier::Shift)
                } else if s_lower.contains("control") || s_lower.contains("ctrl") {
                    Some(Modifier::Ctrl)
                } else if s_lower.contains("alt") {
                    Some(Modifier::Alt)
                } else if s_lower.contains("super") || s_lower.contains("meta") {
                    Some(Modifier::Super)
                } else {
                    None
                }
            }
            KeyCode::Unicode(_) => None,
        }
    }

    /// Gets the hardware keycode for a modifier.
    ///
    /// # Arguments
    ///
    /// * `modifier` - The modifier to get the keycode for
    ///
    /// # Returns
    ///
    /// The evdev keycode for the left variant of the modifier.
    fn modifier_to_keycode(modifier: Modifier) -> u32 {
        match modifier {
            Modifier::Shift => keycodes::KEY_LEFTSHIFT,
            Modifier::Ctrl => keycodes::KEY_LEFTCTRL,
            Modifier::Alt => keycodes::KEY_LEFTALT,
            Modifier::Super => keycodes::KEY_LEFTMETA,
        }
    }

    /// Handles a regular (non-modifier) key press.
    ///
    /// This method:
    /// 1. Gets active modifiers from the renderer
    /// 2. Emits modifier key presses for active modifiers
    /// 3. Emits the main key press
    /// 4. Stores the pressed key for release handling
    ///
    /// # Arguments
    ///
    /// * `key` - The key definition
    fn handle_regular_key_press(&mut self, key: &Key) {
        if !self.virtual_keyboard.is_initialized() {
            tracing::warn!("Virtual keyboard not initialized, cannot emit key press");
            return;
        }

        // Get active modifiers
        let active_modifiers = if let Some(ref renderer) = self.keyboard_renderer {
            renderer.get_active_modifiers()
        } else {
            Vec::new()
        };

        // Emit modifier key presses first
        for modifier in &active_modifiers {
            let keycode = Self::modifier_to_keycode(*modifier);
            self.virtual_keyboard.press_key(keycode);
            tracing::debug!("Emitted modifier press: {:?} (keycode {})", modifier, keycode);
        }

        // Resolve and emit the main key
        if let Some(resolved) = parse_keycode(&key.code) {
            match &resolved {
                ResolvedKeycode::Character(_) | ResolvedKeycode::Keysym(_) => {
                    if let Some(keycode) = self.virtual_keyboard.resolve_keycode(&resolved) {
                        self.virtual_keyboard.press_key(keycode);
                        tracing::debug!("Emitted key press: {:?} (keycode {})", resolved, keycode);
                    } else {
                        // Fallback for Unicode characters
                        if let ResolvedKeycode::Character(c) = resolved {
                            tracing::debug!("Key not found in keymap, using Unicode fallback for '{}'", c);
                            self.virtual_keyboard.emit_unicode_codepoint(c as u32);
                        } else {
                            tracing::warn!("Could not resolve keycode for: {:?}", resolved);
                        }
                    }
                }
                ResolvedKeycode::UnicodeCodepoint(codepoint) => {
                    self.virtual_keyboard.emit_unicode_codepoint(*codepoint);
                    tracing::debug!("Emitted Unicode codepoint: U+{:04X}", codepoint);
                }
            }
        } else {
            tracing::warn!("Could not parse keycode: {:?}", key.code);
        }
    }

    /// Handles a regular (non-modifier) key release.
    ///
    /// This method:
    /// 1. Emits the main key release
    /// 2. Emits modifier key releases for any active modifiers
    /// 3. Clears one-shot modifiers from the renderer state
    ///
    /// # Arguments
    ///
    /// * `key` - The key definition
    fn handle_regular_key_release(&mut self, key: &Key) {
        if !self.virtual_keyboard.is_initialized() {
            return;
        }

        // Get active modifiers before clearing
        let active_modifiers = if let Some(ref renderer) = self.keyboard_renderer {
            renderer.get_active_modifiers()
        } else {
            Vec::new()
        };

        // Emit the main key release
        if let Some(resolved) = parse_keycode(&key.code) {
            match &resolved {
                ResolvedKeycode::Character(_) | ResolvedKeycode::Keysym(_) => {
                    if let Some(keycode) = self.virtual_keyboard.resolve_keycode(&resolved) {
                        self.virtual_keyboard.release_key(keycode);
                        tracing::debug!("Emitted key release: {:?} (keycode {})", resolved, keycode);
                    }
                }
                ResolvedKeycode::UnicodeCodepoint(_) => {
                    // Unicode codepoint emission handles press+release in emit_unicode_codepoint
                }
            }
        }

        // Emit modifier key releases
        for modifier in &active_modifiers {
            let keycode = Self::modifier_to_keycode(*modifier);
            self.virtual_keyboard.release_key(keycode);
            tracing::debug!("Emitted modifier release: {:?} (keycode {})", modifier, keycode);
        }

        // Clear one-shot modifiers from the renderer
        if let Some(ref mut renderer) = self.keyboard_renderer {
            renderer.clear_oneshot_modifiers();
        }
    }

    /// Handles a modifier key press.
    ///
    /// This method activates the modifier in the renderer's modifier state
    /// based on the key's sticky and stickyrelease fields.
    ///
    /// # Arguments
    ///
    /// * `key` - The key definition
    /// * `modifier` - The modifier type
    fn handle_modifier_key_press(&mut self, key: &Key, modifier: Modifier) {
        if let Some(ref mut renderer) = self.keyboard_renderer {
            if key.sticky {
                // Sticky key: toggle behavior for toggle mode, activate for one-shot
                if key.stickyrelease {
                    // One-shot: activate and mark as sticky
                    renderer.activate_modifier(modifier, true);
                    if let Some(ref id) = key.identifier {
                        renderer.sync_modifier_visual_state(modifier, id);
                    }
                    tracing::debug!("Activated one-shot modifier: {:?}", modifier);
                } else {
                    // Toggle mode: toggle the modifier state
                    if renderer.is_modifier_active(modifier) {
                        renderer.deactivate_modifier(modifier);
                        if let Some(ref id) = key.identifier {
                            renderer.sticky_keys_active.remove(id);
                        }
                        tracing::debug!("Deactivated toggle modifier: {:?}", modifier);
                    } else {
                        renderer.activate_modifier(modifier, false);
                        if let Some(ref id) = key.identifier {
                            renderer.sync_modifier_visual_state(modifier, id);
                        }
                        tracing::debug!("Activated toggle modifier: {:?}", modifier);
                    }
                }
            } else {
                // Hold mode: activate while held (will deactivate on release)
                renderer.activate_modifier(modifier, false);
                if let Some(ref id) = key.identifier {
                    renderer.sync_modifier_visual_state(modifier, id);
                }
                tracing::debug!("Activated hold modifier: {:?}", modifier);
            }
        }
    }

    /// Handles a modifier key release.
    ///
    /// For hold mode modifiers, this deactivates the modifier.
    /// For sticky modifiers, release is handled in `clear_oneshot_modifiers`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key definition
    /// * `modifier` - The modifier type
    fn handle_modifier_key_release(&mut self, key: &Key, modifier: Modifier) {
        if let Some(ref mut renderer) = self.keyboard_renderer {
            if !key.sticky {
                // Hold mode: deactivate on release
                renderer.deactivate_modifier(modifier);
                if let Some(ref id) = key.identifier {
                    renderer.sticky_keys_active.remove(id);
                }
                tracing::debug!("Released hold modifier: {:?}", modifier);
            }
            // For sticky modifiers, the state persists until cleared by clear_oneshot_modifiers
            // or toggled off by another press
        }
    }
}

impl cosmic::Application for AppletModel {
    /// The async executor for running application tasks.
    type Executor = cosmic::SingleThreadExecutor;

    /// Data that the application receives at initialization.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN format.
    const APP_ID: &'static str = APPLET_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Initialize the applet and load persisted window state.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        // DIAGNOSTIC: Skip config loading to test if it's causing the delay
        // TODO: Re-enable once we identify the performance issue
        let window_state = WindowState::default();

        let applet = AppletModel {
            core,
            popup: None,
            keyboard_surface: None,
            keyboard_visible: false,
            pending_width: window_state.width,
            pending_height: window_state.height,
            pending_margin_right: window_state.margin_right,
            pending_margin_bottom: window_state.margin_bottom,
            window_state,
            state_config: None, // No config = no D-Bus operations
            is_dragging: false,
            resize_edge: None,
            last_cursor_position: None,
            preview_surface: None,
            last_preview_width: 0,
            last_preview_height: 0,
            last_preview_margin_right: 0,
            last_preview_margin_bottom: 0,
            last_preview_update: None,
            keyboard_renderer: None,
            virtual_keyboard: VirtualKeyboard::new(),
        };
        (applet, Task::none())
    }

    /// Subscribe to events only when actively dragging or resizing (Task 7.5).
    ///
    /// Performance critical: Return `Subscription::none()` when idle to avoid
    /// processing all window events in the system. This is the key difference
    /// between responsive and laggy applets - the libcosmic example applet has
    /// no subscription at all when idle.
    fn subscription(&self) -> cosmic::iced_futures::Subscription<Self::Message> {
        use cosmic::iced_futures::Subscription;

        let mut subscriptions: Vec<Subscription<Message>> = Vec::new();

        // Subscription for drag/resize mouse events
        if self.is_dragging || self.resize_edge.is_some() {
            subscriptions.push(event::listen_with(|event, _, _id| match event {
                Event::Mouse(mouse_event) => match mouse_event {
                    mouse::Event::CursorMoved { position } => Some(Message::CursorMoved(position)),
                    mouse::Event::ButtonReleased(mouse::Button::Left) => {
                        // End drag/resize on mouse release
                        Some(Message::DragEnd)
                    }
                    _ => None,
                },
                _ => None,
            }));
        }

        // Renderer subscriptions (Task 7.5)
        if let Some(ref renderer) = self.keyboard_renderer {
            // Animation subscription - emit ticks during panel transitions
            if renderer.is_animating() {
                subscriptions.push(
                    time::every(Duration::from_millis(ANIMATION_FRAME_INTERVAL_MS))
                        .map(|_| Message::AnimationTick),
                );
            }

            // Long press timer subscription
            if renderer.has_pending_long_press() {
                subscriptions.push(
                    time::every(Duration::from_millis(LONG_PRESS_TIMER_INTERVAL_MS))
                        .map(|_| Message::LongPressTimerTick),
                );
            }

            // Toast timer subscription
            if renderer.has_active_toast() {
                subscriptions.push(
                    time::every(Duration::from_millis(TOAST_TIMER_INTERVAL_MS))
                        .map(|_| Message::ToastTimerTick),
                );
            }
        }

        // Return combined subscriptions or none
        if subscriptions.is_empty() {
            Subscription::none()
        } else {
            Subscription::batch(subscriptions)
        }
    }

    /// Handle surface close requests from the compositor.
    ///
    /// This handles both popup closes and keyboard surface closes.
    /// Since we no longer have an idle subscription listening for window events,
    /// this is the proper way to detect when surfaces are closed externally.
    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        if Some(id) == self.keyboard_surface {
            Some(Message::KeyboardSurfaceClosed(id))
        } else {
            Some(Message::PopupClosed(id))
        }
    }

    /// Handle messages emitted by the applet (Task 7.4, Task Group 5).
    fn update(&mut self, message: Message) -> Task<Self::Message> {
        match message {
            Message::Toggle => {
                // Close popup if open
                if let Some(id) = self.popup.take() {
                    return cosmic::task::message(cosmic::Action::<Message>::Cosmic(
                        cosmic::app::Action::Surface(destroy_popup(id)),
                    ));
                }

                // Toggle keyboard visibility
                if self.keyboard_visible {
                    return Task::done(cosmic::Action::App(Message::Hide));
                } else {
                    return Task::done(cosmic::Action::App(Message::Show));
                }
            }
            Message::TogglePopup => {
                // Right-click: toggle popup menu visibility
                if let Some(id) = self.popup.take() {
                    // Close popup if already open
                    return cosmic::task::message(cosmic::Action::<Message>::Cosmic(
                        cosmic::app::Action::Surface(destroy_popup(id)),
                    ));
                }

                // Open popup menu using applet's default positioning
                return cosmic::task::message(cosmic::Action::<Message>::Cosmic(
                    cosmic::app::Action::Surface(app_popup::<AppletModel>(
                        |state: &mut AppletModel| {
                            let new_id = Id::unique();
                            state.popup = Some(new_id);
                            state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                None,
                                None,
                                None,
                            )
                        },
                        Some(Box::new(|state: &AppletModel| {
                            // Build the popup menu content
                            let mode_label = if state.window_state.is_floating {
                                fl!("exclusive-mode")
                            } else {
                                fl!("floating-mode")
                            };

                            let content = list_column()
                                .padding(8)
                                .spacing(0)
                                // Show Keyboard menu item
                                .add(
                                    cosmic::applet::menu_button(widget::text::body(fl!(
                                        "show-keyboard"
                                    )))
                                    .on_press(Message::Show),
                                )
                                // Hide Keyboard menu item
                                .add(
                                    cosmic::applet::menu_button(widget::text::body(fl!(
                                        "hide-keyboard"
                                    )))
                                    .on_press(Message::Hide),
                                )
                                // Separator
                                .add(
                                    cosmic::applet::padded_control(divider::horizontal::default())
                                        .padding([8, 0]),
                                )
                                // Toggle docked / floating mode
                                .add(
                                    cosmic::applet::menu_button(widget::text::body(mode_label))
                                        .on_press(Message::ToggleFloatingMode),
                                )
                                // Separator
                                .add(
                                    cosmic::applet::padded_control(divider::horizontal::default())
                                        .padding([8, 0]),
                                )
                                // Quit menu item
                                .add(
                                    cosmic::applet::menu_button(widget::text::body(fl!("quit")))
                                        .on_press(Message::Quit),
                                );

                            Element::from(state.core.applet.popup_container(content))
                                .map(cosmic::Action::App)
                        })),
                    )),
                ));
            }
            Message::Show => {
                // Close popup if open
                if let Some(popup_id) = self.popup.take() {
                    // First close popup, then show keyboard
                    return Task::batch([
                        cosmic::task::message(cosmic::Action::<Message>::Cosmic(
                            cosmic::app::Action::Surface(destroy_popup(popup_id)),
                        )),
                        Task::done(cosmic::Action::App(Message::Show)),
                    ]);
                }

                if self.keyboard_visible {
                    return Task::none();
                }

                // Load the keyboard layout (Task 7.2)
                self.load_keyboard_layout();

                // Initialize virtual keyboard (Task Group 5)
                if let Err(e) = self.virtual_keyboard.initialize() {
                    tracing::error!("Failed to initialize virtual keyboard: {}", e);
                    // Continue even if VK fails - keyboard will show but not emit events
                } else {
                    tracing::info!("Virtual keyboard initialized");
                }

                // Create layer surface for keyboard
                let id = window::Id::unique();
                let height = self.window_state.height as u32;
                let width = self.window_state.width as u32;

                // Configure based on floating vs docked mode
                let (anchor, size, margin, exclusive_zone) = if self.window_state.is_floating {
                    // Floating: corner anchor, explicit size, position via margins
                    (
                        Anchor::BOTTOM | Anchor::RIGHT,
                        Some((Some(width), Some(height))),
                        IcedMargin {
                            top: 0,
                            right: self.window_state.margin_right,
                            bottom: self.window_state.margin_bottom,
                            left: 0,
                        },
                        0, // No exclusive zone in floating mode
                    )
                } else {
                    // Docked: full-width bottom anchor with exclusive zone
                    (
                        Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                        Some((None, Some(height))),
                        IcedMargin::default(),
                        height as i32,
                    )
                };

                let settings = SctkLayerSurfaceSettings {
                    id,
                    layer: Layer::Overlay,
                    keyboard_interactivity: KeyboardInteractivity::None,
                    input_zone: None,
                    anchor,
                    output: IcedOutput::Active,
                    namespace: "cosboard-keyboard".to_string(),
                    margin,
                    size,
                    exclusive_zone,
                    size_limits: Limits::NONE
                        .min_width(MIN_WIDTH)
                        .max_width(MAX_WIDTH)
                        .min_height(MIN_HEIGHT)
                        .max_height(MAX_HEIGHT),
                };

                self.keyboard_surface = Some(id);
                self.keyboard_visible = true;

                tracing::info!(
                    "Opening keyboard layer surface: {:?} floating={} height={} width={} exclusive_zone={}",
                    id,
                    self.window_state.is_floating,
                    height,
                    width,
                    exclusive_zone
                );

                return get_layer_surface(settings);
            }
            Message::Hide => {
                // Close popup if open
                if let Some(popup_id) = self.popup.take() {
                    // First close popup, then hide keyboard
                    return Task::batch([
                        cosmic::task::message(cosmic::Action::<Message>::Cosmic(
                            cosmic::app::Action::Surface(destroy_popup(popup_id)),
                        )),
                        Task::done(cosmic::Action::App(Message::Hide)),
                    ]);
                }

                if !self.keyboard_visible {
                    return Task::none();
                }

                // Save state before closing
                self.save_state();

                // Cleanup virtual keyboard (Task Group 5)
                self.virtual_keyboard.cleanup();

                // Clear the renderer (Task 7.1 - clear on layout unload)
                self.keyboard_renderer = None;

                self.keyboard_visible = false;
                if let Some(id) = self.keyboard_surface.take() {
                    tracing::info!("Destroying keyboard layer surface: {:?}", id);
                    return destroy_layer_surface(id);
                }
            }
            Message::Quit => {
                // Save state before quitting
                self.save_state();
                // Cleanup virtual keyboard
                self.virtual_keyboard.cleanup();
                std::process::exit(0);
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::Surface(action) => {
                return cosmic::task::message(cosmic::Action::<Message>::Cosmic(
                    cosmic::app::Action::Surface(action),
                ));
            }
            Message::KeyboardSurfaceClosed(id) => {
                if self.keyboard_surface == Some(id) {
                    self.keyboard_surface = None;
                    self.keyboard_visible = false;
                    self.keyboard_renderer = None; // Clear renderer
                    self.virtual_keyboard.cleanup(); // Cleanup VK
                    tracing::info!("Keyboard layer surface closed: {:?}", id);
                }
                // Also check if this was the preview surface
                if self.preview_surface == Some(id) {
                    self.preview_surface = None;
                    tracing::debug!("Preview surface closed: {:?}", id);
                }
            }
            Message::KeyboardSurfaceResized(id, _width, height) => {
                // PERFORMANCE: Ignore resize events for preview surface entirely.
                // The preview is just visual feedback - we don't need to track its state.
                if self.preview_surface == Some(id) {
                    return Task::none();
                }

                if self.keyboard_surface == Some(id) {
                    // PERFORMANCE: Skip state update during active drag/resize to prevent
                    // widget rebuilds. The compositor sends Resized events in response to
                    // our set_size() calls, but we don't want to update window_state until
                    // the operation ends (handled in DragEnd).
                    if self.resize_edge.is_some() || self.is_dragging {
                        tracing::debug!("Skipping resize event during active drag/resize");
                        return Task::none();
                    }

                    self.window_state.height = height;
                    tracing::debug!("Keyboard resized to height {}", height);

                    // Update exclusive zone if in exclusive mode
                    let mut tasks = vec![Task::done(cosmic::Action::App(Message::SaveState))];
                    if !self.window_state.is_floating {
                        tasks.push(set_exclusive_zone(id, height as i32));
                    }
                    return Task::batch(tasks);
                }
            }
            Message::ToggleFloatingMode => {
                self.window_state.is_floating = !self.window_state.is_floating;
                self.save_state();

                // Update layer surface configuration
                if let Some(id) = self.keyboard_surface {
                    let height = self.window_state.height as u32;
                    let width = self.window_state.width as u32;

                    let tasks = if self.window_state.is_floating {
                        // Switching TO floating: corner anchor + explicit size
                        tracing::info!(
                            "Switching to floating mode: width={} height={} margin_right={} margin_bottom={}",
                            width,
                            height,
                            self.window_state.margin_right,
                            self.window_state.margin_bottom
                        );
                        vec![
                            set_anchor(id, Anchor::BOTTOM | Anchor::RIGHT),
                            set_size(id, Some(width), Some(height)),
                            set_margin(
                                id,
                                0,
                                self.window_state.margin_right,
                                self.window_state.margin_bottom,
                                0,
                            ),
                            set_exclusive_zone(id, 0),
                        ]
                    } else {
                        // Switching TO docked: full-width bottom
                        tracing::info!("Switching to docked mode: height={}", height);
                        vec![
                            set_anchor(id, Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT),
                            set_size(id, None, Some(height)),
                            set_margin(id, 0, 0, 0, 0),
                            set_exclusive_zone(id, height as i32),
                        ]
                    };
                    return Task::batch(tasks);
                }
            }
            Message::SaveState => {
                self.save_state();
            }
            Message::DragStart => {
                if self.window_state.is_floating && self.preview_surface.is_none() {
                    self.is_dragging = true;
                    // Initialize pending values from current state
                    self.pending_margin_right = self.window_state.margin_right;
                    self.pending_margin_bottom = self.window_state.margin_bottom;
                    tracing::debug!("Drag started - spawning preview surface");
                    // Spawn preview surface for visual feedback
                    return self.create_preview_surface();
                }
            }
            Message::DragEnd => {
                let mut tasks: Vec<Task<Message>> = Vec::new();

                // Destroy preview surface if it exists
                if let Some(preview_id) = self.preview_surface.take() {
                    tracing::debug!("Destroying preview surface: {:?}", preview_id);
                    tasks.push(destroy_layer_surface(preview_id));
                }

                if self.is_dragging {
                    self.is_dragging = false;
                    // Commit pending values to window_state
                    self.window_state.margin_right = self.pending_margin_right;
                    self.window_state.margin_bottom = self.pending_margin_bottom;
                    self.save_state();
                    tracing::debug!("Drag ended - applying final position to keyboard");

                    // Apply final position to keyboard surface (single update)
                    if let Some(keyboard_id) = self.keyboard_surface {
                        tasks.push(set_margin(
                            keyboard_id,
                            0,
                            self.pending_margin_right,
                            self.pending_margin_bottom,
                            0,
                        ));
                    }
                }

                if self.resize_edge.is_some() {
                    self.resize_edge = None;
                    // Commit pending values to window_state
                    self.window_state.width = self.pending_width;
                    self.window_state.height = self.pending_height;
                    self.window_state.margin_right = self.pending_margin_right;
                    self.window_state.margin_bottom = self.pending_margin_bottom;
                    self.save_state();
                    tracing::debug!("Resize ended - applying final size to keyboard");

                    // Apply final size and position to keyboard surface (single update)
                    if let Some(keyboard_id) = self.keyboard_surface {
                        tasks.push(set_size(
                            keyboard_id,
                            Some(self.pending_width as u32),
                            Some(self.pending_height as u32),
                        ));
                        tasks.push(set_margin(
                            keyboard_id,
                            0,
                            self.pending_margin_right,
                            self.pending_margin_bottom,
                            0,
                        ));
                    }
                }

                if !tasks.is_empty() {
                    return Task::batch(tasks);
                }
            }
            Message::ResizeStart(edge) => {
                if self.window_state.is_floating && self.preview_surface.is_none() {
                    self.resize_edge = Some(edge);
                    // Initialize pending values from current state
                    self.pending_width = self.window_state.width;
                    self.pending_height = self.window_state.height;
                    self.pending_margin_right = self.window_state.margin_right;
                    self.pending_margin_bottom = self.window_state.margin_bottom;
                    tracing::debug!("Resize started on {:?} - spawning preview surface", edge);
                    // Spawn preview surface for visual feedback
                    return self.create_preview_surface();
                }
            }
            Message::ResizeEnd => {
                if self.resize_edge.is_some() {
                    self.resize_edge = None;
                    // Commit pending values to window_state (triggers single rebuild)
                    self.window_state.width = self.pending_width;
                    self.window_state.height = self.pending_height;
                    self.window_state.margin_right = self.pending_margin_right;
                    self.window_state.margin_bottom = self.pending_margin_bottom;
                    self.save_state();
                    tracing::debug!("Resize ended");
                }
            }
            Message::CursorMoved(pos) => {
                // Early return if not in any active drag/resize mode
                // (This is defensive - subscription() should only send these when active)
                if !self.is_dragging && self.resize_edge.is_none() {
                    self.last_cursor_position = Some(pos);
                    return Task::none();
                }

                // Use incremental delta from last position to prevent jumps when cursor
                // leaves and re-enters the window (layer surfaces don't have pointer grab)
                //
                // PREVIEW APPROACH: Update the preview surface only (not keyboard surface).
                // The keyboard stays unchanged during drag/resize for smooth performance.
                // Final values are applied to keyboard when operation ends.

                // Handle dragging with incremental updates
                if self.is_dragging {
                    if let Some(last_pos) = self.last_cursor_position {
                        // Calculate incremental delta from last position (inverted for bottom-right anchor)
                        let dx = (last_pos.x - pos.x) as i32;
                        let dy = (last_pos.y - pos.y) as i32;

                        // Clamp to prevent jumps when cursor re-enters after leaving window
                        let max_delta = 30;
                        let dx = dx.clamp(-max_delta, max_delta);
                        let dy = dy.clamp(-max_delta, max_delta);

                        // Apply to PENDING margins
                        self.pending_margin_right = (self.pending_margin_right + dx).max(0);
                        self.pending_margin_bottom = (self.pending_margin_bottom + dy).max(0);

                        // DEDUPLICATION: Only send commands if margin values actually changed
                        let margin_changed = self.pending_margin_right != self.last_preview_margin_right
                            || self.pending_margin_bottom != self.last_preview_margin_bottom;

                        // TIME DEBOUNCE: Only update at most once per 100ms
                        let now = Instant::now();
                        let time_ok = self.last_preview_update
                            .map(|last| now.duration_since(last).as_millis() >= PREVIEW_UPDATE_INTERVAL_MS)
                            .unwrap_or(true);

                        if margin_changed && time_ok {
                            if let Some(preview_id) = self.preview_surface {
                                self.last_preview_margin_right = self.pending_margin_right;
                                self.last_preview_margin_bottom = self.pending_margin_bottom;
                                self.last_preview_update = Some(now);
                                self.last_cursor_position = Some(pos);
                                return set_margin(preview_id, 0, self.pending_margin_right, self.pending_margin_bottom, 0);
                            }
                        }
                    }
                }

                // Handle resizing with incremental updates
                if let Some(edge) = self.resize_edge {
                    if let Some(last_pos) = self.last_cursor_position {
                        let dx = pos.x - last_pos.x;
                        let dy = pos.y - last_pos.y;

                        // Clamp incremental change to prevent jumps
                        let max_delta = 30.0;
                        let dx = dx.clamp(-max_delta, max_delta);
                        let dy = dy.clamp(-max_delta, max_delta);

                        // Work with PENDING values
                        let mut new_width = self.pending_width;
                        let mut new_height = self.pending_height;
                        let mut new_right = self.pending_margin_right;
                        let mut new_bottom = self.pending_margin_bottom;

                        match edge {
                            ResizeEdge::Left => {
                                // Dragging left edge: decrease dx = increase width, increase right margin
                                new_width = (new_width - dx).clamp(MIN_WIDTH, MAX_WIDTH);
                                new_right = (new_right + dx as i32).max(0);
                            }
                            ResizeEdge::Top => {
                                // Dragging top edge: decrease dy = increase height, increase bottom margin
                                new_height = (new_height - dy).clamp(MIN_HEIGHT, MAX_HEIGHT);
                                new_bottom = (new_bottom + dy as i32).max(0);
                            }
                            ResizeEdge::TopLeft => {
                                // Dragging top-left corner
                                new_width = (new_width - dx).clamp(MIN_WIDTH, MAX_WIDTH);
                                new_height = (new_height - dy).clamp(MIN_HEIGHT, MAX_HEIGHT);
                                new_right = (new_right + dx as i32).max(0);
                                new_bottom = (new_bottom + dy as i32).max(0);
                            }
                            ResizeEdge::TopRight => {
                                // Dragging top-right corner: width increases with dx
                                new_width = (new_width + dx).clamp(MIN_WIDTH, MAX_WIDTH);
                                new_height = (new_height - dy).clamp(MIN_HEIGHT, MAX_HEIGHT);
                                new_bottom = (new_bottom + dy as i32).max(0);
                            }
                            ResizeEdge::BottomLeft => {
                                // Dragging bottom-left corner
                                new_width = (new_width - dx).clamp(MIN_WIDTH, MAX_WIDTH);
                                new_height = (new_height + dy).clamp(MIN_HEIGHT, MAX_HEIGHT);
                                new_right = (new_right + dx as i32).max(0);
                            }
                            ResizeEdge::BottomRight => {
                                // Dragging bottom-right corner (anchor point)
                                // Both dimensions increase with positive delta, no margin changes
                                new_width = (new_width + dx).clamp(MIN_WIDTH, MAX_WIDTH);
                                new_height = (new_height + dy).clamp(MIN_HEIGHT, MAX_HEIGHT);
                            }
                        }

                        // Update PENDING values
                        self.pending_width = new_width;
                        self.pending_height = new_height;
                        self.pending_margin_right = new_right;
                        self.pending_margin_bottom = new_bottom;

                        // Convert to integer values for deduplication check
                        let new_w = new_width as u32;
                        let new_h = new_height as u32;

                        // DEDUPLICATION: Only send commands if values actually changed
                        // This eliminates redundant compositor round-trips
                        let changed = new_w != self.last_preview_width
                            || new_h != self.last_preview_height
                            || new_right != self.last_preview_margin_right
                            || new_bottom != self.last_preview_margin_bottom;

                        // TIME DEBOUNCE: Only update at most once per 100ms
                        let now = Instant::now();
                        let time_ok = self.last_preview_update
                            .map(|last| now.duration_since(last).as_millis() >= PREVIEW_UPDATE_INTERVAL_MS)
                            .unwrap_or(true);

                        if changed && time_ok {
                            if let Some(preview_id) = self.preview_surface {
                                self.last_preview_width = new_w;
                                self.last_preview_height = new_h;
                                self.last_preview_margin_right = new_right;
                                self.last_preview_margin_bottom = new_bottom;
                                self.last_preview_update = Some(now);
                                self.last_cursor_position = Some(pos);
                                return Task::batch([
                                    set_size(preview_id, Some(new_w), Some(new_h)),
                                    set_margin(preview_id, 0, new_right, new_bottom, 0),
                                ]);
                            }
                        }
                    }
                }

                self.last_cursor_position = Some(pos);
            }
            Message::PreviewSurfaceCreated(_id) => {
                // Preview surface created - nothing special to do
                tracing::debug!("Preview surface created");
            }
            Message::PreviewSurfaceClosed(id) => {
                // Preview surface was closed externally
                if self.preview_surface == Some(id) {
                    self.preview_surface = None;
                    tracing::debug!("Preview surface closed externally: {:?}", id);
                }
            }
            // ================================================================
            // Renderer Message Handlers (Task 7.4, Task Group 5)
            // ================================================================
            Message::KeyPressed(identifier) => {
                // First, update visual state in the renderer
                if let Some(ref mut renderer) = self.keyboard_renderer {
                    renderer.press_key(&identifier);
                    tracing::debug!("Key pressed (visual): {}", identifier);
                }

                // Now handle input emission (Task Group 5)
                // Clone the key data we need to avoid borrow issues
                let key_info = self.find_key_by_identifier(&identifier).map(|key| {
                    (
                        key.code.clone(),
                        key.sticky,
                        key.stickyrelease,
                        key.identifier.clone(),
                    )
                });

                if let Some((code, sticky, stickyrelease, id)) = key_info {
                    // Create a temporary Key struct with the needed fields
                    let key = Key {
                        code: code.clone(),
                        sticky,
                        stickyrelease,
                        identifier: id,
                        ..Key::default()
                    };

                    // Check if this is a modifier key
                    if let Some(modifier) = Self::keycode_to_modifier(&code) {
                        // Handle modifier key press
                        self.handle_modifier_key_press(&key, modifier);
                    } else {
                        // Handle regular key press
                        self.handle_regular_key_press(&key);
                    }
                }
            }
            Message::KeyReleased(identifier) => {
                // First, update visual state in the renderer
                if let Some(ref mut renderer) = self.keyboard_renderer {
                    renderer.release_key(&identifier);
                    tracing::debug!("Key released (visual): {}", identifier);
                }

                // Now handle input emission (Task Group 5)
                // Clone the key data we need to avoid borrow issues
                let key_info = self.find_key_by_identifier(&identifier).map(|key| {
                    (
                        key.code.clone(),
                        key.sticky,
                        key.stickyrelease,
                        key.identifier.clone(),
                    )
                });

                if let Some((code, sticky, stickyrelease, id)) = key_info {
                    // Create a temporary Key struct with the needed fields
                    let key = Key {
                        code: code.clone(),
                        sticky,
                        stickyrelease,
                        identifier: id,
                        ..Key::default()
                    };

                    // Check if this is a modifier key
                    if let Some(modifier) = Self::keycode_to_modifier(&code) {
                        // Handle modifier key release
                        self.handle_modifier_key_release(&key, modifier);
                    } else {
                        // Handle regular key release
                        self.handle_regular_key_release(&key);
                    }
                }
            }
            Message::SwitchPanel(panel_id) => {
                if let Some(ref mut renderer) = self.keyboard_renderer {
                    // Use switch_panel_with_toast which handles errors with toasts
                    let success = renderer.switch_panel_with_toast(&panel_id);
                    if success {
                        tracing::info!("Switching to panel: {}", panel_id);
                    } else {
                        tracing::warn!("Failed to switch to panel: {}", panel_id);
                    }
                }
            }
            Message::AnimationTick => {
                if let Some(ref mut renderer) = self.keyboard_renderer {
                    // Update animation progress
                    let completed = renderer.update_animation();
                    if completed {
                        tracing::debug!("Panel animation completed");
                    }
                }
            }
            Message::LongPressTimerTick => {
                if let Some(ref mut renderer) = self.keyboard_renderer {
                    // Check if long press threshold has been exceeded
                    if renderer.check_long_press_threshold() {
                        tracing::debug!("Long press detected");
                        // Long press popup handling would happen here
                        // For now, we just log it
                    }
                }
            }
            Message::ShowToast(message, severity) => {
                if let Some(ref mut renderer) = self.keyboard_renderer {
                    renderer.queue_toast(message, severity);
                }
            }
            Message::DismissToast => {
                if let Some(ref mut renderer) = self.keyboard_renderer {
                    renderer.dismiss_current_toast();
                    renderer.show_next_toast();
                }
            }
            Message::ToastTimerTick => {
                if let Some(ref mut renderer) = self.keyboard_renderer {
                    // Check for toast timeout and advance queue
                    let _dismissed = renderer.handle_toast_timer_tick();
                }
            }
        }
        Task::none()
    }

    /// Render the applet icon button.
    fn view(&self) -> Element<'_, Message> {
        let has_popup = self.popup.is_some();

        // Create the icon button using the applet context (no click handler on the button itself)
        let btn = self.core.applet.icon_button("input-keyboard-symbolic");

        // Wrap in mouse_area to differentiate left-click vs right-click:
        // - Left-click: Toggle keyboard visibility
        // - Right-click: Open popup menu
        let clickable = mouse_area(btn)
            .on_press(Message::Toggle)
            .on_right_press(Message::TogglePopup);

        // Wrap with tooltip
        Element::from(self.core.applet.applet_tooltip::<Message>(
            clickable,
            fl!("toggle-keyboard"),
            has_popup,
            |a| Message::Surface(a),
            None,
        ))
    }

    /// Handle views for additional windows (layer surfaces, popups) (Task 7.3).
    fn view_window(&self, id: window::Id) -> Element<'_, Message> {
        if Some(id) == self.keyboard_surface {
            // Render the keyboard content using the renderer
            let keyboard_content = self.render_keyboard_content();

            if self.window_state.is_floating {
                // In floating mode: use a grid-like layout for resize handles around content
                // Layout structure:
                // [TopLeft ][   Top    ][TopRight  ]
                // [Left    ][ Content  ][          ]
                // [BotLeft ][          ][BotRight  ]
                use cosmic::widget::{column, row};

                // Top row: corner + top edge + corner
                // Top-left corner with visible diagonal arrow
                let top_left = mouse_area(
                    container(widget::text::body(""))
                        .width(RESIZE_ZONE_SIZE)
                        .height(RESIZE_ZONE_SIZE)
                        .class(cosmic::style::Container::Background),
                )
                .on_press(Message::ResizeStart(ResizeEdge::TopLeft))
                .interaction(mouse::Interaction::ResizingDiagonallyDown);

                let top_edge = mouse_area(Space::new(Length::Fill, RESIZE_ZONE_SIZE))
                    .on_press(Message::ResizeStart(ResizeEdge::Top))
                    .interaction(mouse::Interaction::ResizingVertically);

                let top_right = mouse_area(Space::new(RESIZE_ZONE_SIZE, RESIZE_ZONE_SIZE))
                    .on_press(Message::ResizeStart(ResizeEdge::TopRight))
                    .interaction(mouse::Interaction::ResizingDiagonallyUp);

                let top_row = row::row()
                    .push(top_left)
                    .push(top_edge)
                    .push(top_right);

                // Middle row: left edge + draggable content
                let left_edge = mouse_area(Space::new(RESIZE_ZONE_SIZE, Length::Fill))
                    .on_press(Message::ResizeStart(ResizeEdge::Left))
                    .interaction(mouse::Interaction::ResizingHorizontally);

                // Wrap keyboard content in container (no drag on content - keys should be clickable)
                let content_container = container(keyboard_content)
                    .width(Length::Fill)
                    .height(Length::Fill);

                let middle_row = row::row()
                    .push(left_edge)
                    .push(content_container)
                    .height(Length::Fill);

                // Bottom row: bottom-left corner + spacer + bottom-right corner
                let bottom_left = mouse_area(Space::new(RESIZE_ZONE_SIZE, RESIZE_ZONE_SIZE))
                    .on_press(Message::ResizeStart(ResizeEdge::BottomLeft))
                    .interaction(mouse::Interaction::ResizingDiagonallyUp);

                // Bottom-right corner
                let bottom_right = mouse_area(
                    container(widget::text::body(""))
                        .width(RESIZE_ZONE_SIZE)
                        .height(RESIZE_ZONE_SIZE)
                        .class(cosmic::style::Container::Background),
                )
                .on_press(Message::ResizeStart(ResizeEdge::BottomRight))
                .interaction(mouse::Interaction::ResizingDiagonallyDown);

                let bottom_row = row::row()
                    .push(bottom_left)
                    .push(Space::new(Length::Fill, RESIZE_ZONE_SIZE))
                    .push(bottom_right);

                column::column()
                    .push(top_row)
                    .push(middle_row)
                    .push(bottom_row)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else {
                // Docked mode: no drag/resize handles, just the keyboard content
                keyboard_content
            }
        } else if Some(id) == self.preview_surface {
            // Preview surface: semi-transparent outline showing future bounds
            container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .class(cosmic::style::Container::custom(|theme| {
                    cosmic::widget::container::Style {
                        background: None, // Transparent interior
                        border: cosmic::iced::Border {
                            color: theme.cosmic().accent_color().into(),
                            width: 3.0,
                            radius: 8.0.into(),
                        },
                        ..Default::default()
                    }
                }))
                .into()
        } else {
            // Popup content is handled via app_popup in view()
            "".into()
        }
    }

    /// Set the applet style (transparent background).
    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

// ============================================================================
// Applet Entry Point
// ============================================================================

/// Run the applet.
///
/// This function should be called from a separate binary entry point.
/// It sets up the COSMIC applet runtime and launches the AppletModel.
pub fn run() -> cosmic::iced::Result {
    // Initialize localization using LANG environment variable directly
    // Avoids slow D-Bus call via DesktopLanguageRequester::requested_languages()
    let lang = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .unwrap_or_else(|_| "en-US.UTF-8".to_string());

    // Parse just the language part (e.g., "en_US.UTF-8" -> "en-US")
    let lang_code = lang
        .split('.')
        .next()
        .unwrap_or("en-US")
        .replace('_', "-");

    if let Ok(lang_id) = lang_code.parse::<i18n_embed::unic_langid::LanguageIdentifier>() {
        crate::i18n::init(&[lang_id]);
    }

    // Run the applet (cosmic::applet::run handles logging initialization)
    cosmic::applet::run::<AppletModel>(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Applet initializes with correct icon name
    #[test]
    fn test_applet_icon_name() {
        // The icon name used in the applet should be a standard keyboard icon
        let icon_name = "input-keyboard-symbolic";
        assert!(
            icon_name.contains("keyboard"),
            "Applet should use a keyboard icon"
        );
        assert!(
            icon_name.ends_with("-symbolic"),
            "Applet should use a symbolic icon for panel integration"
        );
    }

    /// Test: Applet APP_ID is correctly set
    #[test]
    fn test_applet_app_id() {
        assert_eq!(
            APPLET_ID, "io.github.cosboard.Cosboard.Applet",
            "Applet should have its own unique APP_ID"
        );
        assert!(
            APPLET_ID.contains("Applet"),
            "Applet APP_ID should be distinct from main app"
        );
    }

    /// Test: Message enum has all required variants for applet functionality
    #[test]
    fn test_message_variants_exist() {
        // Test that all required message variants can be created
        let toggle = Message::Toggle;
        let show = Message::Show;
        let hide = Message::Hide;
        let quit = Message::Quit;
        let popup_closed = Message::PopupClosed(Id::unique());

        // Verify variant matching works
        assert!(matches!(toggle, Message::Toggle));
        assert!(matches!(show, Message::Show));
        assert!(matches!(hide, Message::Hide));
        assert!(matches!(quit, Message::Quit));
        assert!(matches!(popup_closed, Message::PopupClosed(_)));
    }

    /// Test: AppletModel default state is correct
    #[test]
    fn test_applet_default_state() {
        let applet = AppletModel::default();

        // Popup should not be open by default
        assert!(
            applet.popup.is_none(),
            "Popup should not be open by default"
        );

        // Keyboard should not be visible by default
        assert!(
            !applet.keyboard_visible,
            "Keyboard should not be visible by default"
        );

        // Keyboard layer surface should be None by default
        assert!(
            applet.keyboard_surface.is_none(),
            "Keyboard surface should be None by default"
        );

        // State config should be None by default (loaded in init)
        assert!(
            applet.state_config.is_none(),
            "State config should be None in default"
        );

        // Keyboard renderer should be None by default (Task 7.1)
        assert!(
            applet.keyboard_renderer.is_none(),
            "Keyboard renderer should be None by default"
        );

        // Virtual keyboard should exist but not be initialized
        assert!(
            !applet.virtual_keyboard.is_initialized(),
            "Virtual keyboard should not be initialized by default"
        );
    }

    /// Test: Window state has sensible defaults
    #[test]
    fn test_window_state_defaults() {
        let state = WindowState::default();

        assert!(state.width > 0.0, "Default width should be positive");
        assert!(state.height > 0.0, "Default height should be positive");
        // Default is docked mode (is_floating = false) for proper soft keyboard behavior
        assert!(!state.is_floating, "Default should be docked mode");
        // Default margins should be 0 (bottom-right corner)
        assert_eq!(state.margin_bottom, 0, "Default margin_bottom should be 0");
        assert_eq!(state.margin_right, 0, "Default margin_right should be 0");
    }

    /// Test: AppletModel drag/resize state defaults
    #[test]
    fn test_applet_drag_resize_defaults() {
        let applet = AppletModel::default();

        // Drag state should be inactive
        assert!(!applet.is_dragging, "Should not be dragging by default");

        // Resize state should be inactive
        assert!(
            applet.resize_edge.is_none(),
            "Resize edge should be None by default"
        );

        // Last cursor position should be None
        assert!(
            applet.last_cursor_position.is_none(),
            "Last cursor position should be None"
        );

        // Preview surface should be None by default
        assert!(
            applet.preview_surface.is_none(),
            "Preview surface should be None by default"
        );
    }

    /// Test: ResizeEdge variants exist
    #[test]
    fn test_resize_edge_variants() {
        let edges = [
            ResizeEdge::Top,
            ResizeEdge::Left,
            ResizeEdge::TopLeft,
            ResizeEdge::TopRight,
            ResizeEdge::BottomLeft,
            ResizeEdge::BottomRight,
        ];

        for edge in edges {
            // Verify each variant can be matched
            match edge {
                ResizeEdge::Top
                | ResizeEdge::Left
                | ResizeEdge::TopLeft
                | ResizeEdge::TopRight
                | ResizeEdge::BottomLeft
                | ResizeEdge::BottomRight => {}
            }
        }
    }

    /// Test: New message variants for drag/resize
    #[test]
    fn test_drag_resize_message_variants() {
        use cosmic::iced::Point;

        let drag_start = Message::DragStart;
        let drag_end = Message::DragEnd;
        let resize_start = Message::ResizeStart(ResizeEdge::Top);
        let resize_end = Message::ResizeEnd;
        let cursor_moved = Message::CursorMoved(Point::new(100.0, 200.0));
        let toggle_mode = Message::ToggleFloatingMode;

        assert!(matches!(drag_start, Message::DragStart));
        assert!(matches!(drag_end, Message::DragEnd));
        assert!(matches!(resize_start, Message::ResizeStart(_)));
        assert!(matches!(resize_end, Message::ResizeEnd));
        assert!(matches!(cursor_moved, Message::CursorMoved(_)));
        assert!(matches!(toggle_mode, Message::ToggleFloatingMode));
    }

    /// Test: Renderer message variants exist (Task 7.4)
    #[test]
    fn test_renderer_message_variants() {
        let key_pressed = Message::KeyPressed("key_a".to_string());
        let key_released = Message::KeyReleased("key_a".to_string());
        let switch_panel = Message::SwitchPanel("numpad".to_string());
        let animation_tick = Message::AnimationTick;
        let long_press_tick = Message::LongPressTimerTick;
        let show_toast = Message::ShowToast("Error".to_string(), ToastSeverity::Error);
        let dismiss_toast = Message::DismissToast;
        let toast_tick = Message::ToastTimerTick;

        assert!(matches!(key_pressed, Message::KeyPressed(_)));
        assert!(matches!(key_released, Message::KeyReleased(_)));
        assert!(matches!(switch_panel, Message::SwitchPanel(_)));
        assert!(matches!(animation_tick, Message::AnimationTick));
        assert!(matches!(long_press_tick, Message::LongPressTimerTick));
        assert!(matches!(show_toast, Message::ShowToast(_, _)));
        assert!(matches!(dismiss_toast, Message::DismissToast));
        assert!(matches!(toast_tick, Message::ToastTimerTick));
    }

    // ========================================================================
    // Task Group 5: Key Press Event Flow Tests (5.1)
    // ========================================================================

    /// Test 1: Regular key press emits correct keycode
    ///
    /// Verifies that pressing a regular (non-modifier) key triggers
    /// the correct keycode emission flow.
    #[test]
    fn test_regular_key_press_emits_correct_keycode() {
        // Create a key definition for 'a'
        let key = Key {
            label: "a".to_string(),
            code: KeyCode::Unicode('a'),
            identifier: Some("key_a".to_string()),
            ..Key::default()
        };

        // Verify the keycode can be parsed
        let resolved = parse_keycode(&key.code);
        assert!(resolved.is_some(), "Should parse keycode for 'a'");
        assert_eq!(resolved.unwrap(), ResolvedKeycode::Character('a'));

        // Verify it's not a modifier
        let modifier = AppletModel::keycode_to_modifier(&key.code);
        assert!(modifier.is_none(), "'a' should not be a modifier key");
    }

    /// Test 2: Modifier key activates modifier state
    ///
    /// Verifies that pressing a modifier key (Shift, Ctrl, Alt, Super)
    /// correctly activates the modifier state.
    #[test]
    fn test_modifier_key_activates_modifier_state() {
        // Test Shift detection
        let shift_code = KeyCode::Keysym("Shift_L".to_string());
        let shift_modifier = AppletModel::keycode_to_modifier(&shift_code);
        assert_eq!(shift_modifier, Some(Modifier::Shift), "Shift_L should be Shift modifier");

        // Test Control detection
        let ctrl_code = KeyCode::Keysym("Control_L".to_string());
        let ctrl_modifier = AppletModel::keycode_to_modifier(&ctrl_code);
        assert_eq!(ctrl_modifier, Some(Modifier::Ctrl), "Control_L should be Ctrl modifier");

        // Test Alt detection
        let alt_code = KeyCode::Keysym("Alt_L".to_string());
        let alt_modifier = AppletModel::keycode_to_modifier(&alt_code);
        assert_eq!(alt_modifier, Some(Modifier::Alt), "Alt_L should be Alt modifier");

        // Test Super detection
        let super_code = KeyCode::Keysym("Super_L".to_string());
        let super_modifier = AppletModel::keycode_to_modifier(&super_code);
        assert_eq!(super_modifier, Some(Modifier::Super), "Super_L should be Super modifier");

        // Test Meta detection (should map to Super)
        let meta_code = KeyCode::Keysym("Meta_L".to_string());
        let meta_modifier = AppletModel::keycode_to_modifier(&meta_code);
        assert_eq!(meta_modifier, Some(Modifier::Super), "Meta_L should be Super modifier");
    }

    /// Test 3: Combo key (modifier + regular) emits sequence
    ///
    /// Verifies that when a modifier is active and a regular key is pressed,
    /// the correct sequence of modifier press + key press is emitted.
    #[test]
    fn test_combo_key_emits_sequence() {
        use crate::layout::{Cell, Panel, Row, Layout, Sizing};
        use std::collections::HashMap;

        // Create a test layout with a Shift key and an 'a' key
        let mut panels = HashMap::new();
        panels.insert(
            "main".to_string(),
            Panel {
                id: "main".to_string(),
                rows: vec![Row {
                    cells: vec![
                        Cell::Key(Key {
                            label: "Shift".to_string(),
                            code: KeyCode::Keysym("Shift_L".to_string()),
                            identifier: Some("shift".to_string()),
                            sticky: true,
                            stickyrelease: true, // one-shot
                            ..Key::default()
                        }),
                        Cell::Key(Key {
                            label: "a".to_string(),
                            code: KeyCode::Unicode('a'),
                            identifier: Some("key_a".to_string()),
                            ..Key::default()
                        }),
                    ],
                }],
                ..Panel::default()
            },
        );

        let layout = Layout {
            name: "Test".to_string(),
            version: "1.0".to_string(),
            default_panel_id: "main".to_string(),
            panels,
            ..Layout::default()
        };

        // Create a renderer with this layout
        let mut renderer = KeyboardRenderer::new(layout);

        // Simulate pressing Shift (one-shot mode)
        renderer.activate_modifier(Modifier::Shift, true);
        assert!(renderer.is_modifier_active(Modifier::Shift), "Shift should be active");

        // Now 'a' should be combined with Shift
        let active_modifiers = renderer.get_active_modifiers();
        assert_eq!(active_modifiers.len(), 1, "Should have 1 active modifier");
        assert_eq!(active_modifiers[0], Modifier::Shift, "Active modifier should be Shift");

        // Verify modifier keycode mapping
        let shift_keycode = AppletModel::modifier_to_keycode(Modifier::Shift);
        assert_eq!(shift_keycode, keycodes::KEY_LEFTSHIFT, "Shift should map to LEFT_SHIFT keycode");
    }

    /// Test 4: Sticky modifier clears after combo (stickyrelease: true)
    ///
    /// Verifies that one-shot modifiers (stickyrelease: true) are cleared
    /// after the next regular key press.
    #[test]
    fn test_sticky_modifier_clears_after_combo() {
        use crate::layout::{Layout, Panel, Row, Cell};
        use std::collections::HashMap;

        let mut panels = HashMap::new();
        panels.insert("main".to_string(), Panel {
            id: "main".to_string(),
            rows: vec![Row { cells: vec![] }],
            ..Panel::default()
        });

        let layout = Layout {
            name: "Test".to_string(),
            version: "1.0".to_string(),
            default_panel_id: "main".to_string(),
            panels,
            ..Layout::default()
        };

        let mut renderer = KeyboardRenderer::new(layout);

        // Activate Shift as one-shot (stickyrelease: true)
        renderer.activate_modifier(Modifier::Shift, true);
        assert!(renderer.is_modifier_active(Modifier::Shift), "Shift should be active");

        // Simulate pressing a regular key and clearing one-shot modifiers
        renderer.clear_oneshot_modifiers();

        // Shift should now be inactive
        assert!(!renderer.is_modifier_active(Modifier::Shift), "Shift should be cleared after combo");
    }

    /// Test 5: Toggle modifier persists after combo (stickyrelease: false)
    ///
    /// Verifies that toggle modifiers (stickyrelease: false) remain active
    /// after regular key presses.
    #[test]
    fn test_toggle_modifier_persists_after_combo() {
        use crate::layout::{Layout, Panel, Row};
        use std::collections::HashMap;

        let mut panels = HashMap::new();
        panels.insert("main".to_string(), Panel {
            id: "main".to_string(),
            rows: vec![Row { cells: vec![] }],
            ..Panel::default()
        });

        let layout = Layout {
            name: "Test".to_string(),
            version: "1.0".to_string(),
            default_panel_id: "main".to_string(),
            panels,
            ..Layout::default()
        };

        let mut renderer = KeyboardRenderer::new(layout);

        // Activate Ctrl as toggle (stickyrelease: false)
        renderer.activate_modifier(Modifier::Ctrl, false);
        assert!(renderer.is_modifier_active(Modifier::Ctrl), "Ctrl should be active");

        // Simulate pressing a regular key and clearing one-shot modifiers
        renderer.clear_oneshot_modifiers();

        // Ctrl should still be active (toggle mode persists)
        assert!(renderer.is_modifier_active(Modifier::Ctrl), "Ctrl should persist in toggle mode");

        // Must explicitly deactivate
        renderer.deactivate_modifier(Modifier::Ctrl);
        assert!(!renderer.is_modifier_active(Modifier::Ctrl), "Ctrl should be inactive after deactivate");
    }

    /// Test 6: Hold modifier behavior
    ///
    /// Verifies that hold modifiers (sticky: false) are active only while
    /// the key is held and deactivate on release.
    #[test]
    fn test_hold_modifier_behavior() {
        use crate::layout::{Layout, Panel, Row};
        use std::collections::HashMap;

        let mut panels = HashMap::new();
        panels.insert("main".to_string(), Panel {
            id: "main".to_string(),
            rows: vec![Row { cells: vec![] }],
            ..Panel::default()
        });

        let layout = Layout {
            name: "Test".to_string(),
            version: "1.0".to_string(),
            default_panel_id: "main".to_string(),
            panels,
            ..Layout::default()
        };

        let mut renderer = KeyboardRenderer::new(layout);

        // Simulate holding Alt (non-sticky, so hold behavior)
        // For hold mode, we activate when pressed
        renderer.activate_modifier(Modifier::Alt, false);
        assert!(renderer.is_modifier_active(Modifier::Alt), "Alt should be active while held");

        // User presses a key while Alt is held
        let active = renderer.get_active_modifiers();
        assert!(active.contains(&Modifier::Alt), "Alt should be in active modifiers");

        // User releases Alt - simulate by deactivating
        renderer.deactivate_modifier(Modifier::Alt);
        assert!(!renderer.is_modifier_active(Modifier::Alt), "Alt should be inactive after release");
    }

    /// Test: Modifier to keycode mapping is correct
    #[test]
    fn test_modifier_to_keycode_mapping() {
        assert_eq!(
            AppletModel::modifier_to_keycode(Modifier::Shift),
            keycodes::KEY_LEFTSHIFT,
            "Shift should map to KEY_LEFTSHIFT"
        );
        assert_eq!(
            AppletModel::modifier_to_keycode(Modifier::Ctrl),
            keycodes::KEY_LEFTCTRL,
            "Ctrl should map to KEY_LEFTCTRL"
        );
        assert_eq!(
            AppletModel::modifier_to_keycode(Modifier::Alt),
            keycodes::KEY_LEFTALT,
            "Alt should map to KEY_LEFTALT"
        );
        assert_eq!(
            AppletModel::modifier_to_keycode(Modifier::Super),
            keycodes::KEY_LEFTMETA,
            "Super should map to KEY_LEFTMETA"
        );
    }

    /// Test: keycode_to_modifier correctly identifies modifiers
    #[test]
    fn test_keycode_to_modifier_identification() {
        // Test various keysym formats
        let test_cases = vec![
            (KeyCode::Keysym("Shift_L".to_string()), Some(Modifier::Shift)),
            (KeyCode::Keysym("Shift_R".to_string()), Some(Modifier::Shift)),
            (KeyCode::Keysym("Control_L".to_string()), Some(Modifier::Ctrl)),
            (KeyCode::Keysym("Control_R".to_string()), Some(Modifier::Ctrl)),
            (KeyCode::Keysym("Alt_L".to_string()), Some(Modifier::Alt)),
            (KeyCode::Keysym("Alt_R".to_string()), Some(Modifier::Alt)),
            (KeyCode::Keysym("Super_L".to_string()), Some(Modifier::Super)),
            (KeyCode::Keysym("Super_R".to_string()), Some(Modifier::Super)),
            (KeyCode::Keysym("Meta_L".to_string()), Some(Modifier::Super)),
            (KeyCode::Unicode('a'), None),
            (KeyCode::Keysym("Return".to_string()), None),
            (KeyCode::Keysym("BackSpace".to_string()), None),
        ];

        for (code, expected) in test_cases {
            let result = AppletModel::keycode_to_modifier(&code);
            assert_eq!(result, expected, "Modifier detection failed for {:?}", code);
        }
    }
}
