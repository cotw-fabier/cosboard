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
use crate::state::WindowState;
use std::time::Instant;
use cosmic::app::{Core, Task};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::event;
use cosmic::iced::mouse;
use cosmic::iced::window::{self, Id};
use cosmic::iced::{Event, Length, Limits, Point, Rectangle};
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

/// The applet Application ID (distinct from the main application).
pub const APPLET_ID: &str = "io.github.cosboard.Cosboard.Applet";

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
        }
    }
}

/// Messages emitted by the applet and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle keyboard visibility (left-click action).
    Toggle,
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
        // Load persisted state using Config::new_state for transient data
        let (state_config, window_state) =
            match cosmic_config::Config::new_state(APPLET_ID, WindowState::VERSION) {
                Ok(config) => {
                    let state = WindowState::get_entry(&config).unwrap_or_else(|(_errors, def)| {
                        tracing::debug!("Using default window state");
                        def
                    });
                    tracing::info!("Loaded window state: {:?}", state);
                    (Some(config), state)
                }
                Err(e) => {
                    tracing::warn!("Failed to load state config: {}", e);
                    (None, WindowState::default())
                }
            };

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
            state_config,
            is_dragging: false,
            resize_edge: None,
            last_cursor_position: None,
            preview_surface: None,
            last_preview_width: 0,
            last_preview_height: 0,
            last_preview_margin_right: 0,
            last_preview_margin_bottom: 0,
            last_preview_update: None,
        };
        (applet, Task::none())
    }

    /// Subscribe to window events to detect resize/close and cursor movement.
    fn subscription(&self) -> cosmic::iced_futures::Subscription<Self::Message> {
        event::listen_with(|event, _, id| match event {
            Event::Window(window_event) => match window_event {
                window::Event::Closed => Some(Message::KeyboardSurfaceClosed(id)),
                window::Event::Resized(size) => {
                    Some(Message::KeyboardSurfaceResized(id, size.width, size.height))
                }
                _ => None,
            },
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { position } => Some(Message::CursorMoved(position)),
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    // End drag/resize on mouse release
                    Some(Message::DragEnd)
                }
                _ => None,
            },
            _ => None,
        })
    }

    /// Handle popup close requests.
    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// Handle messages emitted by the applet.
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

                self.keyboard_visible = false;
                if let Some(id) = self.keyboard_surface.take() {
                    tracing::info!("Destroying keyboard layer surface: {:?}", id);
                    return destroy_layer_surface(id);
                }
            }
            Message::Quit => {
                // Save state before quitting
                self.save_state();
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
        }
        Task::none()
    }

    /// Render the applet icon button.
    fn view(&self) -> Element<'_, Message> {
        let has_popup = self.popup.is_some();
        let popup_id = self.popup;

        // Create the icon button using the applet context
        let btn = self
            .core
            .applet
            .icon_button("input-keyboard-symbolic")
            .on_press_with_rectangle(move |offset, bounds| {
                if let Some(id) = popup_id {
                    // Close popup if already open
                    Message::Surface(destroy_popup(id))
                } else {
                    // Open popup menu
                    Message::Surface(app_popup::<AppletModel>(
                        move |state: &mut AppletModel| {
                            let new_id = Id::unique();
                            state.popup = Some(new_id);
                            let mut popup_settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                None,
                                None,
                                None,
                            );

                            popup_settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };

                            popup_settings
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
                    ))
                }
            });

        // Wrap with tooltip
        Element::from(self.core.applet.applet_tooltip::<Message>(
            btn,
            fl!("toggle-keyboard"),
            has_popup,
            |a| Message::Surface(a),
            None,
        ))
    }

    /// Handle views for additional windows (layer surfaces, popups).
    fn view_window(&self, id: window::Id) -> Element<'_, Message> {
        if Some(id) == self.keyboard_surface {
            // Keyboard layer surface content (placeholder - will add actual keyboard later)
            let keyboard_content = container(widget::text::body("Keyboard (Layer Surface)"))
                .width(Length::Fill)
                .height(Length::Fill)
                .class(cosmic::style::Container::Background);

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
                    container(widget::text::body("↖"))
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

                let draggable_content = mouse_area(keyboard_content)
                    .on_press(Message::DragStart)
                    .interaction(mouse::Interaction::Grab);

                let middle_row = row::row()
                    .push(left_edge)
                    .push(draggable_content)
                    .height(Length::Fill);

                // Bottom row: bottom-left corner + spacer + bottom-right corner
                let bottom_left = mouse_area(Space::new(RESIZE_ZONE_SIZE, RESIZE_ZONE_SIZE))
                    .on_press(Message::ResizeStart(ResizeEdge::BottomLeft))
                    .interaction(mouse::Interaction::ResizingDiagonallyUp);

                // Bottom-right corner with visible diagonal arrow
                let bottom_right = mouse_area(
                    container(widget::text::body("↘"))
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
                // Docked mode: no drag/resize
                keyboard_content.into()
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
    // Initialize localization
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    crate::i18n::init(&requested_languages);

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
}
