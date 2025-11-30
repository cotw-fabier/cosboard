// SPDX-License-Identifier: GPL-3.0-only

//! D-Bus interface for controlling the Cosboard window.
//!
//! This module provides the D-Bus service that allows external applications
//! (like the system tray applet) to show, hide, or toggle the keyboard window.
//!
//! # Architecture
//!
//! The D-Bus interface consists of two parts:
//! - **Server**: The main application registers as a D-Bus service and exposes methods
//!   to control window visibility.
//! - **Client**: The applet (and other applications) can connect to the service and
//!   invoke methods to control the keyboard window.
//!
//! # Interface
//!
//! - Object path: `/io/github/cosboard/Cosboard`
//! - Interface name: `io.github.cosboard.Cosboard`
//! - Methods: `Show()`, `Hide()`, `Toggle()`, `Quit()`
//! - Signals: `VisibilityChanged(visible: bool)`

use crate::app_settings::{DBUS_INTERFACE, DBUS_PATH};
use futures::channel::mpsc;
use futures::SinkExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use zbus::interface;
use zbus::object_server::SignalEmitter;

/// Commands that can be sent from D-Bus to the main application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbusCommand {
    /// Show the keyboard window.
    Show,
    /// Hide the keyboard window.
    Hide,
    /// Toggle the keyboard window visibility.
    Toggle,
    /// Quit the application.
    Quit,
}

/// The D-Bus interface implementation for Cosboard window control.
///
/// This struct is registered as a D-Bus object and handles incoming method calls.
/// It uses a channel to communicate with the main application.
pub struct CosboardInterface {
    /// Channel sender to send commands to the main application.
    command_tx: mpsc::Sender<DbusCommand>,
    /// Current visibility state for signal emission.
    #[allow(dead_code)]
    visible: Arc<AtomicBool>,
}

impl CosboardInterface {
    /// Create a new interface instance with a command sender.
    #[allow(dead_code)]
    pub fn new(command_tx: mpsc::Sender<DbusCommand>) -> Self {
        Self {
            command_tx,
            visible: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Create a new interface instance with a command sender and shared visibility state.
    pub fn with_visibility(
        command_tx: mpsc::Sender<DbusCommand>,
        visible: Arc<AtomicBool>,
    ) -> Self {
        Self { command_tx, visible }
    }

    /// Get the current visibility state.
    #[allow(dead_code)]
    pub fn is_visible(&self) -> bool {
        self.visible.load(Ordering::SeqCst)
    }

    /// Set the visibility state (called from the main application).
    #[allow(dead_code)]
    pub fn set_visible(&self, visible: bool) {
        self.visible.store(visible, Ordering::SeqCst);
    }
}

#[interface(name = "io.github.cosboard.Cosboard")]
impl CosboardInterface {
    /// Show the keyboard window.
    async fn show(&mut self) {
        tracing::debug!("D-Bus: Show() called");
        if let Err(e) = self.command_tx.send(DbusCommand::Show).await {
            tracing::error!("Failed to send Show command: {}", e);
        }
    }

    /// Hide the keyboard window.
    async fn hide(&mut self) {
        tracing::debug!("D-Bus: Hide() called");
        if let Err(e) = self.command_tx.send(DbusCommand::Hide).await {
            tracing::error!("Failed to send Hide command: {}", e);
        }
    }

    /// Toggle the keyboard window visibility.
    async fn toggle(&mut self) {
        tracing::debug!("D-Bus: Toggle() called");
        if let Err(e) = self.command_tx.send(DbusCommand::Toggle).await {
            tracing::error!("Failed to send Toggle command: {}", e);
        }
    }

    /// Quit the application.
    async fn quit(&mut self) {
        tracing::debug!("D-Bus: Quit() called");
        if let Err(e) = self.command_tx.send(DbusCommand::Quit).await {
            tracing::error!("Failed to send Quit command: {}", e);
        }
    }

    /// Signal emitted when visibility changes.
    #[zbus(signal)]
    async fn visibility_changed(emitter: &SignalEmitter<'_>, visible: bool) -> zbus::Result<()>;
}

/// Result type for D-Bus operations.
pub type DbusResult<T> = Result<T, DbusError>;

/// Errors that can occur during D-Bus operations.
#[derive(Debug, Clone)]
pub enum DbusError {
    /// Failed to connect to the session bus.
    ConnectionFailed(String),
    /// Failed to register the service.
    RegistrationFailed(String),
    /// Failed to call a method.
    MethodCallFailed(String),
    /// Service not available.
    #[allow(dead_code)]
    ServiceUnavailable,
}

impl std::fmt::Display for DbusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbusError::ConnectionFailed(msg) => write!(f, "D-Bus connection failed: {}", msg),
            DbusError::RegistrationFailed(msg) => {
                write!(f, "D-Bus service registration failed: {}", msg)
            }
            DbusError::MethodCallFailed(msg) => write!(f, "D-Bus method call failed: {}", msg),
            DbusError::ServiceUnavailable => write!(f, "D-Bus service is not available"),
        }
    }
}

impl std::error::Error for DbusError {}

/// D-Bus server handle for the main application.
///
/// This struct manages the D-Bus connection and provides methods to interact
/// with the service.
pub struct DbusServer {
    /// The D-Bus connection.
    connection: zbus::Connection,
    /// Shared visibility state.
    visible: Arc<AtomicBool>,
}

impl std::fmt::Debug for DbusServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DbusServer")
            .field("visible", &self.visible.load(Ordering::SeqCst))
            .finish_non_exhaustive()
    }
}

impl DbusServer {
    /// Start the D-Bus server and register the Cosboard interface.
    ///
    /// # Arguments
    /// * `command_tx` - Channel sender to forward commands to the main application.
    ///
    /// # Returns
    /// A `DbusServer` instance if successful, or an error if registration fails.
    pub async fn start(command_tx: mpsc::Sender<DbusCommand>) -> DbusResult<Self> {
        let visible = Arc::new(AtomicBool::new(true));
        let interface = CosboardInterface::with_visibility(command_tx, Arc::clone(&visible));

        // Connect to the session bus
        let connection = zbus::connection::Builder::session()
            .map_err(|e| DbusError::ConnectionFailed(e.to_string()))?
            .name(DBUS_INTERFACE)
            .map_err(|e| DbusError::RegistrationFailed(e.to_string()))?
            .serve_at(DBUS_PATH, interface)
            .map_err(|e| DbusError::RegistrationFailed(e.to_string()))?
            .build()
            .await
            .map_err(|e| DbusError::ConnectionFailed(e.to_string()))?;

        tracing::info!(
            "D-Bus service registered: {} at {}",
            DBUS_INTERFACE,
            DBUS_PATH
        );

        Ok(Self { connection, visible })
    }

    /// Get the D-Bus connection.
    #[allow(dead_code)]
    pub fn connection(&self) -> &zbus::Connection {
        &self.connection
    }

    /// Get the current visibility state.
    #[allow(dead_code)]
    pub fn is_visible(&self) -> bool {
        self.visible.load(Ordering::SeqCst)
    }

    /// Update the visibility state and emit a signal.
    pub async fn set_visible(&self, visible: bool) -> DbusResult<()> {
        let old_visible = self.visible.swap(visible, Ordering::SeqCst);
        if old_visible != visible {
            self.emit_visibility_changed(visible).await?;
        }
        Ok(())
    }

    /// Emit a visibility changed signal.
    async fn emit_visibility_changed(&self, visible: bool) -> DbusResult<()> {
        let iface_ref = self
            .connection
            .object_server()
            .interface::<_, CosboardInterface>(DBUS_PATH)
            .await
            .map_err(|e| DbusError::MethodCallFailed(e.to_string()))?;

        CosboardInterface::visibility_changed(iface_ref.signal_emitter(), visible)
            .await
            .map_err(|e| DbusError::MethodCallFailed(e.to_string()))?;

        tracing::debug!("D-Bus: VisibilityChanged({}) signal emitted", visible);
        Ok(())
    }
}

// ============================================================================
// D-Bus Client for Applet
// ============================================================================

/// D-Bus proxy for connecting to the Cosboard service.
///
/// This is generated from the interface definition and provides type-safe
/// method calls to the D-Bus service.
#[zbus::proxy(
    interface = "io.github.cosboard.Cosboard",
    default_service = "io.github.cosboard.Cosboard",
    default_path = "/io/github/cosboard/Cosboard"
)]
trait CosboardProxy {
    /// Show the keyboard window.
    async fn show(&self) -> zbus::Result<()>;

    /// Hide the keyboard window.
    async fn hide(&self) -> zbus::Result<()>;

    /// Toggle the keyboard window visibility.
    async fn toggle(&self) -> zbus::Result<()>;

    /// Quit the application.
    async fn quit(&self) -> zbus::Result<()>;

    /// Signal for visibility changes.
    #[zbus(signal)]
    async fn visibility_changed(&self, visible: bool) -> zbus::Result<()>;
}

/// D-Bus client for connecting to the Cosboard service from the applet.
///
/// This provides a convenient API for the applet to control the main application.
/// This struct is intended for use by the applet module (Task Group 6).
#[allow(dead_code)]
pub struct DbusClient {
    /// The proxy to the Cosboard service.
    proxy: CosboardProxyProxy<'static>,
}

#[allow(dead_code)]
impl DbusClient {
    /// Connect to the Cosboard D-Bus service.
    ///
    /// This will attempt to connect to the session bus and create a proxy
    /// to the Cosboard service.
    pub async fn connect() -> DbusResult<Self> {
        let connection = zbus::Connection::session()
            .await
            .map_err(|e| DbusError::ConnectionFailed(e.to_string()))?;

        let proxy = CosboardProxyProxy::new(&connection)
            .await
            .map_err(|e| DbusError::ConnectionFailed(e.to_string()))?;

        Ok(Self { proxy })
    }

    /// Connect to the Cosboard D-Bus service with retries.
    ///
    /// This will attempt to connect multiple times with exponential backoff.
    ///
    /// # Arguments
    /// * `max_retries` - Maximum number of connection attempts.
    /// * `initial_delay_ms` - Initial delay between retries in milliseconds.
    pub async fn connect_with_retries(
        max_retries: u32,
        initial_delay_ms: u64,
    ) -> DbusResult<Self> {
        let mut attempts = 0;
        let mut delay = initial_delay_ms;

        loop {
            match Self::connect().await {
                Ok(client) => return Ok(client),
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_retries {
                        return Err(e);
                    }
                    tracing::warn!(
                        "D-Bus connection attempt {} failed, retrying in {}ms: {}",
                        attempts,
                        delay,
                        e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    delay *= 2; // Exponential backoff
                }
            }
        }
    }

    /// Show the keyboard window.
    pub async fn show(&self) -> DbusResult<()> {
        self.proxy
            .show()
            .await
            .map_err(|e| DbusError::MethodCallFailed(e.to_string()))
    }

    /// Hide the keyboard window.
    pub async fn hide(&self) -> DbusResult<()> {
        self.proxy
            .hide()
            .await
            .map_err(|e| DbusError::MethodCallFailed(e.to_string()))
    }

    /// Toggle the keyboard window visibility.
    pub async fn toggle(&self) -> DbusResult<()> {
        self.proxy
            .toggle()
            .await
            .map_err(|e| DbusError::MethodCallFailed(e.to_string()))
    }

    /// Quit the application.
    pub async fn quit(&self) -> DbusResult<()> {
        self.proxy
            .quit()
            .await
            .map_err(|e| DbusError::MethodCallFailed(e.to_string()))
    }

    /// Get the underlying proxy for advanced usage.
    pub fn proxy(&self) -> &CosboardProxyProxy<'static> {
        &self.proxy
    }
}

// ============================================================================
// Blocking Client API
// ============================================================================

/// Blocking D-Bus client for simple use cases.
///
/// This provides blocking variants of the async client methods for use in
/// contexts where async is not convenient (e.g., simple scripts or CLI tools).
/// This struct is intended for use by external scripts or the applet module.
#[allow(dead_code)]
pub struct DbusClientBlocking {
    /// Runtime for executing async operations.
    runtime: tokio::runtime::Runtime,
}

#[allow(dead_code)]
impl DbusClientBlocking {
    /// Create a new blocking client.
    pub fn new() -> DbusResult<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| DbusError::ConnectionFailed(e.to_string()))?;
        Ok(Self { runtime })
    }

    /// Show the keyboard window (blocking).
    pub fn show(&self) -> DbusResult<()> {
        self.runtime.block_on(async {
            let client = DbusClient::connect().await?;
            client.show().await
        })
    }

    /// Hide the keyboard window (blocking).
    pub fn hide(&self) -> DbusResult<()> {
        self.runtime.block_on(async {
            let client = DbusClient::connect().await?;
            client.hide().await
        })
    }

    /// Toggle the keyboard window visibility (blocking).
    pub fn toggle(&self) -> DbusResult<()> {
        self.runtime.block_on(async {
            let client = DbusClient::connect().await?;
            client.toggle().await
        })
    }

    /// Quit the application (blocking).
    pub fn quit(&self) -> DbusResult<()> {
        self.runtime.block_on(async {
            let client = DbusClient::connect().await?;
            client.quit().await
        })
    }
}

impl Default for DbusClientBlocking {
    fn default() -> Self {
        Self::new().expect("Failed to create blocking D-Bus client runtime")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    /// Test: D-Bus command enum variants exist and can be matched.
    #[test]
    fn test_dbus_command_variants() {
        let show = DbusCommand::Show;
        let hide = DbusCommand::Hide;
        let toggle = DbusCommand::Toggle;
        let quit = DbusCommand::Quit;

        assert!(matches!(show, DbusCommand::Show));
        assert!(matches!(hide, DbusCommand::Hide));
        assert!(matches!(toggle, DbusCommand::Toggle));
        assert!(matches!(quit, DbusCommand::Quit));
    }

    /// Test: D-Bus error types can be created and displayed.
    #[test]
    fn test_dbus_error_display() {
        let conn_err = DbusError::ConnectionFailed("test".to_string());
        let reg_err = DbusError::RegistrationFailed("test".to_string());
        let method_err = DbusError::MethodCallFailed("test".to_string());
        let unavail_err = DbusError::ServiceUnavailable;

        assert!(conn_err.to_string().contains("connection failed"));
        assert!(reg_err.to_string().contains("registration failed"));
        assert!(method_err.to_string().contains("method call failed"));
        assert!(unavail_err.to_string().contains("not available"));
    }

    /// Test: CosboardInterface can be created and visibility tracked.
    #[test]
    fn test_interface_visibility_tracking() {
        let (tx, _rx) = mpsc::channel::<DbusCommand>(10);
        let visible = Arc::new(AtomicBool::new(true));
        let interface = CosboardInterface::with_visibility(tx, Arc::clone(&visible));

        // Initial state should be visible
        assert!(interface.is_visible());

        // Can update visibility
        interface.set_visible(false);
        assert!(!interface.is_visible());

        // Shared state is updated
        assert!(!visible.load(Ordering::SeqCst));
    }

    /// Test: D-Bus service registers successfully (requires D-Bus session).
    /// This test is async and requires a D-Bus session bus to be available.
    #[tokio::test]
    async fn test_dbus_service_registration() {
        let (tx, _rx) = mpsc::channel::<DbusCommand>(10);

        // Try to start the server - this tests the registration path
        match DbusServer::start(tx).await {
            Ok(server) => {
                // Server started successfully
                assert!(server.is_visible(), "Default visibility should be true");
                tracing::info!("D-Bus server registered successfully");
            }
            Err(DbusError::ConnectionFailed(msg)) => {
                // D-Bus session not available (common in CI environments)
                tracing::warn!("D-Bus session not available: {}", msg);
                // This is not a test failure - D-Bus might not be available
            }
            Err(DbusError::RegistrationFailed(msg)) => {
                // Service name might already be taken
                tracing::warn!("D-Bus registration issue: {}", msg);
            }
            Err(e) => {
                panic!("Unexpected error during D-Bus registration: {}", e);
            }
        }
    }

    /// Test: Show/Hide/Toggle/Quit methods send correct commands through channel.
    #[tokio::test]
    async fn test_dbus_methods_send_commands() {
        let (tx, mut rx) = mpsc::channel::<DbusCommand>(10);
        let mut interface = CosboardInterface::new(tx);

        // Test Show
        interface.show().await;
        let cmd = rx.next().await;
        assert_eq!(cmd, Some(DbusCommand::Show), "Show should send Show command");

        // Test Hide
        interface.hide().await;
        let cmd = rx.next().await;
        assert_eq!(cmd, Some(DbusCommand::Hide), "Hide should send Hide command");

        // Test Toggle
        interface.toggle().await;
        let cmd = rx.next().await;
        assert_eq!(
            cmd,
            Some(DbusCommand::Toggle),
            "Toggle should send Toggle command"
        );

        // Test Quit
        interface.quit().await;
        let cmd = rx.next().await;
        assert_eq!(cmd, Some(DbusCommand::Quit), "Quit should send Quit command");
    }

    /// Test: Interface constants match app_settings.
    #[test]
    fn test_dbus_constants() {
        use crate::app_settings;

        assert_eq!(
            app_settings::DBUS_PATH, "/io/github/cosboard/Cosboard",
            "DBUS_PATH should match expected value"
        );
        assert_eq!(
            app_settings::DBUS_INTERFACE, "io.github.cosboard.Cosboard",
            "DBUS_INTERFACE should match expected value"
        );
    }
}
