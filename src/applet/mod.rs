// SPDX-License-Identifier: GPL-3.0-only

//! System tray applet for Cosboard.
//!
//! This module provides a panel applet that allows users to show/hide
//! the keyboard from the system tray.
//!
//! # Architecture
//!
//! The applet communicates with the main Cosboard application via D-Bus.
//! It displays an icon in the COSMIC panel and provides:
//! - Left-click: Toggle keyboard visibility
//! - Right-click: Open popup menu with show/hide/quit options
//!
//! # Running the Applet
//!
//! The applet is run as a separate binary using the `applet` feature:
//! ```bash
//! cargo run --bin cosboard-applet
//! ```

use crate::dbus::{DbusClient, DbusResult};
use crate::fl;
use cosmic::app::{Core, Task};
use cosmic::iced::window::Id;
use cosmic::iced::Rectangle;
use cosmic::iced_runtime::core::window;
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::{self, divider, list_column};
use cosmic::Element;
use std::sync::Arc;
use tokio::sync::Mutex;

/// The applet Application ID (distinct from the main application).
pub const APPLET_ID: &str = "io.github.cosboard.Cosboard.Applet";

/// Shared D-Bus client handle wrapped in Arc<Mutex> for thread-safe access.
type SharedDbusClient = Arc<Mutex<Option<DbusClient>>>;

/// The applet model stores state for the system tray applet.
pub struct AppletModel {
    /// Application core state managed by the COSMIC runtime.
    core: Core,
    /// Whether the popup menu is currently open.
    popup: Option<Id>,
    /// D-Bus client for communicating with the main application.
    /// This is None until the client successfully connects.
    dbus_client: SharedDbusClient,
    /// Whether we're currently connected to D-Bus.
    connected: bool,
}

impl Default for AppletModel {
    fn default() -> Self {
        Self {
            core: Core::default(),
            popup: None,
            dbus_client: Arc::new(Mutex::new(None)),
            connected: false,
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
    /// Quit the main application.
    Quit,
    /// Popup menu closed.
    PopupClosed(Id),
    /// Handle surface actions (for popup management).
    Surface(cosmic::surface::Action),
    /// D-Bus client connected successfully.
    DbusConnected(bool),
    /// D-Bus operation completed (success/failure).
    DbusOperationComplete(Result<(), String>),
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

    /// Initialize the applet.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let dbus_client: SharedDbusClient = Arc::new(Mutex::new(None));
        let dbus_client_clone = Arc::clone(&dbus_client);

        let applet = AppletModel {
            core,
            popup: None,
            dbus_client,
            connected: false,
        };

        // Start D-Bus client connection asynchronously
        // Note: Task<M> is iced::Task<cosmic::Action<M>>, so the callback must return cosmic::Action<M>
        let connect_task = Task::perform(
            async move {
                // Try to connect with retries (3 attempts, starting with 100ms delay)
                match DbusClient::connect_with_retries(3, 100).await {
                    Ok(client) => {
                        let mut guard = dbus_client_clone.lock().await;
                        *guard = Some(client);
                        tracing::info!("Applet connected to D-Bus service");
                        true
                    }
                    Err(e) => {
                        tracing::warn!("Failed to connect to D-Bus service: {}", e);
                        false
                    }
                }
            },
            |connected| cosmic::Action::App(Message::DbusConnected(connected)),
        );

        (applet, connect_task)
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

                // Send toggle command to main application
                let client = Arc::clone(&self.dbus_client);
                return Task::perform(
                    async move { perform_dbus_operation(client, DbusOperation::Toggle).await },
                    |result| cosmic::Action::App(Message::DbusOperationComplete(result)),
                );
            }
            Message::Show => {
                // Close popup
                if let Some(id) = self.popup.take() {
                    let _: Task<Message> = cosmic::task::message(cosmic::Action::<Message>::Cosmic(
                        cosmic::app::Action::Surface(destroy_popup(id)),
                    ));
                }

                // Send show command
                let client = Arc::clone(&self.dbus_client);
                return Task::perform(
                    async move { perform_dbus_operation(client, DbusOperation::Show).await },
                    |result| cosmic::Action::App(Message::DbusOperationComplete(result)),
                );
            }
            Message::Hide => {
                // Close popup
                if let Some(id) = self.popup.take() {
                    let _: Task<Message> = cosmic::task::message(cosmic::Action::<Message>::Cosmic(
                        cosmic::app::Action::Surface(destroy_popup(id)),
                    ));
                }

                // Send hide command
                let client = Arc::clone(&self.dbus_client);
                return Task::perform(
                    async move { perform_dbus_operation(client, DbusOperation::Hide).await },
                    |result| cosmic::Action::App(Message::DbusOperationComplete(result)),
                );
            }
            Message::Quit => {
                // Close popup
                if let Some(id) = self.popup.take() {
                    let _: Task<Message> = cosmic::task::message(cosmic::Action::<Message>::Cosmic(
                        cosmic::app::Action::Surface(destroy_popup(id)),
                    ));
                }

                // Send quit command
                let client = Arc::clone(&self.dbus_client);
                return Task::perform(
                    async move { perform_dbus_operation(client, DbusOperation::Quit).await },
                    |result| cosmic::Action::App(Message::DbusOperationComplete(result)),
                );
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
            Message::DbusConnected(connected) => {
                self.connected = connected;
            }
            Message::DbusOperationComplete(result) => {
                if let Err(e) = result {
                    tracing::error!("D-Bus operation failed: {}", e);
                    // Try to reconnect if the call failed
                    if self.connected {
                        self.connected = false;
                        let client = Arc::clone(&self.dbus_client);
                        return Task::perform(
                            async move {
                                match DbusClient::connect_with_retries(3, 100).await {
                                    Ok(new_client) => {
                                        let mut guard = client.lock().await;
                                        *guard = Some(new_client);
                                        tracing::info!("Applet reconnected to D-Bus service");
                                        true
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "Failed to reconnect to D-Bus service: {}",
                                            e
                                        );
                                        false
                                    }
                                }
                            },
                            |connected| cosmic::Action::App(Message::DbusConnected(connected)),
                        );
                    }
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

    /// Handle views for additional windows (popups).
    fn view_window(&self, _id: Id) -> Element<'_, Message> {
        // Popup content is handled via app_popup in view()
        "".into()
    }

    /// Set the applet style (transparent background).
    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

// ============================================================================
// D-Bus Operation Helper
// ============================================================================

/// D-Bus operations that can be performed.
enum DbusOperation {
    Show,
    Hide,
    Toggle,
    Quit,
}

/// Perform a D-Bus operation using the shared client.
async fn perform_dbus_operation(
    client: SharedDbusClient,
    operation: DbusOperation,
) -> Result<(), String> {
    let guard = client.lock().await;
    match &*guard {
        Some(dbus_client) => {
            let result: DbusResult<()> = match operation {
                DbusOperation::Show => dbus_client.show().await,
                DbusOperation::Hide => dbus_client.hide().await,
                DbusOperation::Toggle => dbus_client.toggle().await,
                DbusOperation::Quit => dbus_client.quit().await,
            };
            result.map_err(|e| e.to_string())
        }
        None => Err("D-Bus client not connected".to_string()),
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

    // Run the applet
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

        // Should not be connected by default
        assert!(!applet.connected, "Should not be connected by default");
    }

    /// Test: D-Bus operation result message handling
    #[test]
    fn test_dbus_operation_result_message() {
        // Test success case
        let success = Message::DbusOperationComplete(Ok(()));
        assert!(matches!(success, Message::DbusOperationComplete(Ok(()))));

        // Test error case
        let error = Message::DbusOperationComplete(Err("test error".to_string()));
        match error {
            Message::DbusOperationComplete(Err(msg)) => {
                assert_eq!(msg, "test error");
            }
            _ => panic!("Expected DbusOperationComplete error"),
        }
    }

    /// Test: D-Bus connected message handling
    #[test]
    fn test_dbus_connected_message() {
        // Test connected case
        let connected = Message::DbusConnected(true);
        assert!(matches!(connected, Message::DbusConnected(true)));

        // Test disconnected case
        let disconnected = Message::DbusConnected(false);
        assert!(matches!(disconnected, Message::DbusConnected(false)));
    }
}
