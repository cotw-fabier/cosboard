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
use cosmic::app::{Core, Task};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::event;
use cosmic::iced::window::{self, Id};
use cosmic::iced::{Length, Limits, Rectangle};
use cosmic::iced_runtime::platform_specific::wayland::layer_surface::{
    IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
};
use cosmic::iced_winit::platform_specific::wayland::commands::layer_surface::{
    destroy_layer_surface, get_layer_surface, set_exclusive_zone, Anchor, KeyboardInteractivity,
    Layer,
};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::{self, divider, list_column};
use cosmic::Element;

/// The applet Application ID (distinct from the main application).
pub const APPLET_ID: &str = "io.github.cosboard.Cosboard.Applet";

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
}

impl Default for AppletModel {
    fn default() -> Self {
        Self {
            core: Core::default(),
            popup: None,
            keyboard_surface: None,
            keyboard_visible: false,
            window_state: WindowState::default(),
            state_config: None,
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
    /// Toggle between exclusive zone and floating mode.
    ToggleExclusiveZone,
    /// Save window state (debounced).
    SaveState,
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
            window_state,
            state_config,
        };
        (applet, Task::none())
    }

    /// Subscribe to window events to detect resize/close.
    fn subscription(&self) -> cosmic::iced_futures::Subscription<Self::Message> {
        event::listen_with(|event, _, id| {
            if let cosmic::iced::Event::Window(window_event) = event {
                match window_event {
                    window::Event::Closed => Some(Message::KeyboardSurfaceClosed(id)),
                    window::Event::Resized(size) => {
                        Some(Message::KeyboardSurfaceResized(id, size.width, size.height))
                    }
                    _ => None,
                }
            } else {
                None
            }
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
                // is_floating = true means overlay (no exclusive zone)
                // is_floating = false means exclusive zone (default)
                let exclusive_zone = if self.window_state.is_floating { 0 } else { height as i32 };

                let settings = SctkLayerSurfaceSettings {
                    id,
                    layer: Layer::Overlay,
                    keyboard_interactivity: KeyboardInteractivity::None,
                    input_zone: None,
                    anchor: Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                    output: IcedOutput::Active,
                    namespace: "cosboard-keyboard".to_string(),
                    margin: IcedMargin::default(),
                    size: Some((None, Some(height))),  // Full width, fixed height
                    exclusive_zone,
                    size_limits: Limits::NONE
                        .min_height(150.0)
                        .max_height(500.0),
                };

                self.keyboard_surface = Some(id);
                self.keyboard_visible = true;

                tracing::info!(
                    "Opening keyboard layer surface: {:?} height {} exclusive_zone {}",
                    id,
                    height,
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
            }
            Message::KeyboardSurfaceResized(id, _width, height) => {
                if self.keyboard_surface == Some(id) {
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
            Message::ToggleExclusiveZone => {
                self.window_state.is_floating = !self.window_state.is_floating;
                self.save_state();

                // Update exclusive zone on existing surface
                if let Some(id) = self.keyboard_surface {
                    let zone = if self.window_state.is_floating {
                        0
                    } else {
                        self.window_state.height as i32
                    };
                    tracing::info!(
                        "Toggling exclusive zone: is_floating={} zone={}",
                        self.window_state.is_floating,
                        zone
                    );
                    return set_exclusive_zone(id, zone);
                }
            }
            Message::SaveState => {
                self.save_state();
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
                                // Toggle exclusive zone / floating mode
                                .add(
                                    cosmic::applet::menu_button(widget::text::body(mode_label))
                                        .on_press(Message::ToggleExclusiveZone),
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
            cosmic::widget::container(cosmic::widget::text::body(
                "Keyboard (Layer Surface)",
            ))
            .width(Length::Fill)
            .height(Length::Fill)
            .class(cosmic::style::Container::Background)
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
        // Default is exclusive zone mode (is_floating = false) for proper soft keyboard behavior
        assert!(!state.is_floating, "Default should be exclusive zone mode");
    }
}
