// SPDX-License-Identifier: GPL-3.0-only

use crate::app_settings;
use crate::config::Config;
use crate::dbus::{DbusCommand, DbusServer};
use crate::layer_shell::{LayerShellConfig, log_layer_status, Layer};
use crate::state::WindowState;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{window, Length, Subscription};
use cosmic::prelude::*;
use futures::channel::mpsc;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared D-Bus server handle wrapped in Arc<Mutex> for thread-safe access.
type SharedDbusServer = Arc<Mutex<Option<DbusServer>>>;

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Configuration data that persists between application runs.
    config: Config,
    /// Window state (position and size) that persists between runs.
    window_state: WindowState,
    /// Config handle for state persistence.
    state_config: Option<cosmic_config::Config>,
    /// Whether the keyboard window is currently visible.
    visible: bool,
    /// D-Bus server handle (kept alive to maintain the service).
    #[allow(dead_code)]
    dbus_server: SharedDbusServer,
    /// Receiver for D-Bus commands.
    dbus_rx: Option<mpsc::Receiver<DbusCommand>>,
    /// Layer-shell configuration for overlay behavior.
    layer_shell_config: LayerShellConfig,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    /// Window has been resized.
    WindowResized(f32, f32),
    /// Save the current window state.
    SaveState,
    /// Configuration has been updated.
    UpdateConfig(Config),
    /// D-Bus command received: Show the window.
    DbusShow,
    /// D-Bus command received: Hide the window.
    DbusHide,
    /// D-Bus command received: Toggle window visibility.
    DbusToggle,
    /// D-Bus command received: Quit the application.
    DbusQuit,
    /// D-Bus server started successfully.
    DbusServerStarted,
    /// D-Bus server failed to start.
    DbusServerFailed(String),
    /// Poll for D-Bus commands.
    DbusCommandReceived(Option<DbusCommand>),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = app_settings::APP_ID;

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        mut core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Load configuration
        let config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                Ok(config) => config,
                Err((_errors, config)) => config,
            })
            .unwrap_or_default();

        // Load window state
        let state_config =
            cosmic_config::Config::new_state(Self::APP_ID, WindowState::VERSION).ok();
        let window_state = state_config
            .as_ref()
            .map(|context| match WindowState::get_entry(context) {
                Ok(state) => state,
                Err((_errors, state)) => state,
            })
            .unwrap_or_default();

        // Set chromeless appearance - hide the header bar
        core.window.show_headerbar = false;

        // Create a channel for D-Bus commands
        let (dbus_tx, dbus_rx) = mpsc::channel::<DbusCommand>(10);

        // Create shared D-Bus server handle
        let dbus_server: SharedDbusServer = Arc::new(Mutex::new(None));
        let dbus_server_clone = Arc::clone(&dbus_server);

        // Initialize layer-shell configuration with Overlay layer (highest z-order)
        let layer_shell_config = LayerShellConfig::new().with_layer(Layer::Overlay);

        let app = AppModel {
            core,
            config,
            window_state,
            state_config,
            visible: true,
            dbus_server,
            dbus_rx: Some(dbus_rx),
            layer_shell_config,
        };

        // Start D-Bus server asynchronously
        let start_dbus_task = Task::perform(
            async move {
                match DbusServer::start(dbus_tx).await {
                    Ok(server) => {
                        // Store the server in the shared handle
                        let mut guard = dbus_server_clone.lock().await;
                        *guard = Some(server);
                        Message::DbusServerStarted
                    }
                    Err(e) => Message::DbusServerFailed(e.to_string()),
                }
            },
            |msg| cosmic::Action::App(msg),
        );

        (app, start_dbus_task)
    }

    /// Describes the interface based on the current state of the application model.
    fn view(&self) -> Element<'_, Self::Message> {
        // Skeleton: Empty transparent container for now
        cosmic::widget::container(cosmic::widget::Space::new(Length::Fill, Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Register subscriptions for this application.
    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subscriptions = vec![];

        // Watch for configuration changes
        let config_subscription = self
            .core()
            .watch_config::<Config>(Self::APP_ID)
            .map(|update| Message::UpdateConfig(update.config));
        subscriptions.push(config_subscription);

        // D-Bus command subscription is handled via the receiver in update()

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::WindowResized(width, height) => {
                // Update internal window state
                self.window_state.width = width;
                self.window_state.height = height;

                // Log resize event for debugging
                tracing::debug!("Window resized to {}x{}", width, height);

                // Trigger state save
                self.save_state();
            }
            Message::SaveState => {
                self.save_state();
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::DbusShow => {
                tracing::info!("D-Bus: Showing window");
                self.visible = true;
                // Emit visibility changed signal
                self.emit_visibility_signal(true);
                // TODO: Actually show the window when layer-shell is implemented
                return self.poll_dbus_commands();
            }
            Message::DbusHide => {
                tracing::info!("D-Bus: Hiding window");
                self.visible = false;
                // Emit visibility changed signal
                self.emit_visibility_signal(false);
                // TODO: Actually hide the window when layer-shell is implemented
                return self.poll_dbus_commands();
            }
            Message::DbusToggle => {
                tracing::info!("D-Bus: Toggling window visibility");
                self.visible = !self.visible;
                // Emit visibility changed signal
                self.emit_visibility_signal(self.visible);
                // TODO: Actually toggle the window when layer-shell is implemented
                return self.poll_dbus_commands();
            }
            Message::DbusQuit => {
                tracing::info!("D-Bus: Quitting application");
                return cosmic::iced::exit();
            }
            Message::DbusServerStarted => {
                tracing::info!("D-Bus server started successfully");

                // Check layer-shell availability now that the window is up
                self.layer_shell_config.check_availability();
                log_layer_status(&self.layer_shell_config);

                // Start polling D-Bus commands
                return self.poll_dbus_commands();
            }
            Message::DbusServerFailed(error) => {
                tracing::error!("Failed to start D-Bus server: {}", error);
                // Continue without D-Bus - the application can still function

                // Still check layer-shell availability
                self.layer_shell_config.check_availability();
                log_layer_status(&self.layer_shell_config);
            }
            Message::DbusCommandReceived(cmd) => {
                match cmd {
                    Some(DbusCommand::Show) => {
                        return Task::done(cosmic::Action::App(Message::DbusShow));
                    }
                    Some(DbusCommand::Hide) => {
                        return Task::done(cosmic::Action::App(Message::DbusHide));
                    }
                    Some(DbusCommand::Toggle) => {
                        return Task::done(cosmic::Action::App(Message::DbusToggle));
                    }
                    Some(DbusCommand::Quit) => {
                        return Task::done(cosmic::Action::App(Message::DbusQuit));
                    }
                    None => {
                        tracing::warn!("D-Bus command channel closed");
                    }
                }
            }
        }

        Task::none()
    }

    /// Called when a window is resized.
    fn on_window_resize(&mut self, _id: window::Id, width: f32, height: f32) {
        // Update window_state with new dimensions
        self.window_state.width = width;
        self.window_state.height = height;

        // Log resize event for debugging
        tracing::debug!("on_window_resize: {}x{}", width, height);

        // Save state using write_entry for atomic writes
        self.save_state();
    }
}

impl AppModel {
    /// Save the current window state to cosmic_config for persistence.
    fn save_state(&self) {
        if let Some(ref state_config) = self.state_config {
            if let Err(err) = self.window_state.write_entry(state_config) {
                tracing::error!("Failed to save window state: {:?}", err);
            } else {
                tracing::debug!(
                    "Window state saved: {}x{}",
                    self.window_state.width,
                    self.window_state.height
                );
            }
        }
    }

    /// Emit a D-Bus visibility changed signal.
    fn emit_visibility_signal(&self, visible: bool) {
        let dbus_server = Arc::clone(&self.dbus_server);
        // Spawn a task to emit the signal asynchronously
        tokio::spawn(async move {
            let guard = dbus_server.lock().await;
            if let Some(ref server) = *guard {
                if let Err(e) = server.set_visible(visible).await {
                    tracing::error!("Failed to emit visibility signal: {}", e);
                }
            }
        });
    }

    /// Poll for D-Bus commands from the receiver.
    fn poll_dbus_commands(&mut self) -> Task<cosmic::Action<Message>> {
        if let Some(mut rx) = self.dbus_rx.take() {
            return Task::perform(
                async move {
                    let cmd = rx.next().await;
                    (cmd, rx)
                },
                |(cmd, rx)| {
                    // We'll need to restore the receiver somehow
                    // For now, we just process the command
                    cosmic::Action::App(Message::DbusCommandReceived(cmd))
                },
            );
        }
        Task::none()
    }

    /// Get the layer-shell configuration.
    #[cfg(test)]
    pub fn layer_shell_config(&self) -> &LayerShellConfig {
        &self.layer_shell_config
    }

    /// Get the current window state (for testing).
    #[cfg(test)]
    pub fn window_state(&self) -> &WindowState {
        &self.window_state
    }

    /// Get the current config (for testing).
    #[cfg(test)]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Check if headerbar is hidden (for testing).
    #[cfg(test)]
    pub fn is_headerbar_hidden(&self) -> bool {
        !self.core.window.show_headerbar
    }

    /// Check if the window is visible (for testing).
    #[cfg(test)]
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_settings;
    use crate::layer_shell::Layer;

    /// Test: Application initializes with correct default dimensions
    #[test]
    fn test_default_dimensions() {
        // WindowState should use defaults from app_settings
        let state = WindowState::default();

        assert_eq!(
            state.width, app_settings::DEFAULT_WIDTH,
            "Default width should match app_settings::DEFAULT_WIDTH"
        );
        assert_eq!(
            state.height, app_settings::DEFAULT_HEIGHT,
            "Default height should match app_settings::DEFAULT_HEIGHT"
        );
    }

    /// Test: Window settings match app_settings values
    #[test]
    fn test_window_settings_match_app_settings() {
        // Verify app_settings constants are set correctly
        assert_eq!(
            app_settings::DEFAULT_WIDTH, 800.0,
            "DEFAULT_WIDTH should be 800.0"
        );
        assert_eq!(
            app_settings::DEFAULT_HEIGHT, 300.0,
            "DEFAULT_HEIGHT should be 300.0"
        );
        assert_eq!(
            app_settings::MIN_WIDTH, 400.0,
            "MIN_WIDTH should be 400.0"
        );
        assert_eq!(
            app_settings::MIN_HEIGHT, 150.0,
            "MIN_HEIGHT should be 150.0"
        );
        assert_eq!(
            app_settings::RESIZE_BORDER, 8.0,
            "RESIZE_BORDER should be 8.0"
        );
        assert_eq!(
            app_settings::APP_ID,
            "io.github.cosboard.Cosboard",
            "APP_ID should match RDNN format"
        );
    }

    /// Test: State struct has correct version for cosmic_config
    #[test]
    fn test_state_version() {
        // WindowState should have VERSION = 1
        assert_eq!(
            WindowState::VERSION, 1,
            "WindowState::VERSION should be 1"
        );
    }

    /// Test: Config struct has correct version for cosmic_config
    #[test]
    fn test_config_version() {
        // Config should have VERSION = 1
        assert_eq!(Config::VERSION, 1, "Config::VERSION should be 1");
    }

    /// Test: WindowState can be created and modified
    #[test]
    fn test_window_state_modification() {
        let mut state = WindowState::default();

        // Modify state values
        state.width = 1024.0;
        state.height = 400.0;
        state.x = 100;
        state.y = 200;

        assert_eq!(state.width, 1024.0, "Width should be modifiable");
        assert_eq!(state.height, 400.0, "Height should be modifiable");
        assert_eq!(state.x, 100, "X position should be modifiable");
        assert_eq!(state.y, 200, "Y position should be modifiable");
    }

    /// Test: Message enum variants exist and can be created
    #[test]
    fn test_message_variants() {
        // Test WindowResized variant
        let resize_msg = Message::WindowResized(800.0, 600.0);
        match resize_msg {
            Message::WindowResized(w, h) => {
                assert_eq!(w, 800.0);
                assert_eq!(h, 600.0);
            }
            _ => panic!("Expected WindowResized message"),
        }

        // Test SaveState variant
        let save_msg = Message::SaveState;
        assert!(matches!(save_msg, Message::SaveState));

        // Test UpdateConfig variant
        let config = Config::default();
        let config_msg = Message::UpdateConfig(config);
        assert!(matches!(config_msg, Message::UpdateConfig(_)));

        // Test D-Bus message variants
        let show_msg = Message::DbusShow;
        assert!(matches!(show_msg, Message::DbusShow));

        let hide_msg = Message::DbusHide;
        assert!(matches!(hide_msg, Message::DbusHide));

        let toggle_msg = Message::DbusToggle;
        assert!(matches!(toggle_msg, Message::DbusToggle));

        let quit_msg = Message::DbusQuit;
        assert!(matches!(quit_msg, Message::DbusQuit));
    }

    /// Test: D-Bus message variants for server lifecycle
    #[test]
    fn test_dbus_server_message_variants() {
        // Test DbusServerFailed variant
        let fail_msg = Message::DbusServerFailed("test error".to_string());
        match fail_msg {
            Message::DbusServerFailed(err) => {
                assert_eq!(err, "test error");
            }
            _ => panic!("Expected DbusServerFailed message"),
        }

        // Test DbusServerStarted variant
        let start_msg = Message::DbusServerStarted;
        assert!(matches!(start_msg, Message::DbusServerStarted));
    }

    /// Test: D-Bus command received message variant
    #[test]
    fn test_dbus_command_received_variants() {
        // Test with Some commands
        let show_cmd = Message::DbusCommandReceived(Some(DbusCommand::Show));
        assert!(matches!(
            show_cmd,
            Message::DbusCommandReceived(Some(DbusCommand::Show))
        ));

        let hide_cmd = Message::DbusCommandReceived(Some(DbusCommand::Hide));
        assert!(matches!(
            hide_cmd,
            Message::DbusCommandReceived(Some(DbusCommand::Hide))
        ));

        let toggle_cmd = Message::DbusCommandReceived(Some(DbusCommand::Toggle));
        assert!(matches!(
            toggle_cmd,
            Message::DbusCommandReceived(Some(DbusCommand::Toggle))
        ));

        let quit_cmd = Message::DbusCommandReceived(Some(DbusCommand::Quit));
        assert!(matches!(
            quit_cmd,
            Message::DbusCommandReceived(Some(DbusCommand::Quit))
        ));

        // Test with None
        let none_cmd = Message::DbusCommandReceived(None);
        assert!(matches!(none_cmd, Message::DbusCommandReceived(None)));
    }

    /// Test: Layer-shell config defaults to Overlay layer
    #[test]
    fn test_layer_shell_defaults_to_overlay() {
        let config = LayerShellConfig::new();
        assert_eq!(
            config.layer(),
            Layer::Overlay,
            "Layer-shell should default to Overlay layer"
        );
    }

    /// Test: Layer-shell config can be configured with different layers
    #[test]
    fn test_layer_shell_with_layer() {
        let config = LayerShellConfig::new().with_layer(Layer::Top);
        assert_eq!(
            config.layer(),
            Layer::Top,
            "Layer-shell should be configurable to Top layer"
        );
    }
}
