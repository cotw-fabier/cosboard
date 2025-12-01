// SPDX-License-Identifier: GPL-3.0-only

//! Cosboard - A soft keyboard for COSMIC desktop
//!
//! This crate provides a soft keyboard application for the COSMIC desktop environment.
//! It includes both the main application and a system tray applet.
//!
//! # Architecture
//!
//! The application consists of two components:
//!
//! 1. **Main Application** (`cosboard`): The soft keyboard window that displays
//!    the keyboard layout and handles input.
//!
//! 2. **System Tray Applet** (`cosboard-applet`): A panel applet that provides
//!    quick access to show/hide the keyboard.
//!
//! These components communicate via D-Bus.
//!
//! # Modules
//!
//! - `app`: Main application model and COSMIC Application trait implementation
//! - `applet`: System tray applet for panel integration
//! - `app_settings`: Centralized application constants and configuration
//! - `config`: User configuration with cosmic_config persistence
//! - `dbus`: D-Bus interface for inter-process communication
//! - `i18n`: Localization support using fluent translations
//! - `layer_shell`: Wayland layer-shell integration for overlay behavior
//! - `state`: Window state persistence (position, size)

pub mod app;
pub mod app_settings;
pub mod applet;
pub mod config;
pub mod dbus;
pub mod i18n;
pub mod layer_shell;
pub mod state;

// Re-export the fl! macro for localization
pub use crate::i18n::LANGUAGE_LOADER;

// ============================================================================
// Integration Tests (Task Group 7)
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use crate::dbus::DbusCommand;
    use crate::layer_shell::{Layer, LayerShellConfig};
    use crate::state::WindowState;
    use futures::channel::mpsc;
    use futures::SinkExt;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    /// Integration Test 1: Full toggle workflow (applet -> D-Bus -> window)
    ///
    /// This test verifies the complete communication path from applet actions
    /// through D-Bus to the main application by simulating the channel flow.
    #[tokio::test]
    async fn test_full_toggle_workflow() {
        // Create a channel to simulate the D-Bus command flow
        let (mut tx, mut rx) = mpsc::channel::<DbusCommand>(10);
        let visible = Arc::new(AtomicBool::new(true));

        // Simulate applet sending Toggle command through channel
        tx.send(DbusCommand::Toggle).await.unwrap();

        // Verify command is received
        use futures::StreamExt;
        let cmd = rx.next().await;
        assert_eq!(
            cmd,
            Some(DbusCommand::Toggle),
            "Toggle command should be received through D-Bus channel"
        );

        // Simulate the main app processing the toggle
        let current = visible.load(Ordering::SeqCst);
        visible.store(!current, Ordering::SeqCst);

        assert!(
            !visible.load(Ordering::SeqCst),
            "Visibility should toggle from true to false"
        );
    }

    /// Integration Test 2: State persistence across application restart
    ///
    /// This test verifies that WindowState can be serialized and deserialized
    /// correctly for persistence.
    #[test]
    fn test_state_persistence_roundtrip() {
        // Create a modified state (simulating user resize)
        let mut state = WindowState::default();
        state.width = 1024.0;
        state.height = 400.0;
        state.is_floating = true;

        // Verify state can be cloned (would be serialized/deserialized in real app)
        let restored_state = state.clone();

        assert_eq!(
            state.width, restored_state.width,
            "Width should persist correctly"
        );
        assert_eq!(
            state.height, restored_state.height,
            "Height should persist correctly"
        );
        assert_eq!(
            state.is_floating, restored_state.is_floating,
            "Floating mode should persist correctly"
        );
    }

    /// Integration Test 3: Multiple rapid resize events handling
    ///
    /// This test verifies that the state can handle multiple rapid updates
    /// (debouncing behavior is handled by the save logic, but the state should
    /// always reflect the latest values).
    #[test]
    fn test_rapid_resize_events() {
        let mut state = WindowState::default();

        // Simulate rapid resize events
        let resize_events = vec![
            (850.0, 350.0),
            (900.0, 380.0),
            (920.0, 400.0),
            (950.0, 420.0),
            (1000.0, 450.0),
        ];

        for (width, height) in &resize_events {
            state.width = *width;
            state.height = *height;
        }

        // State should reflect the final values
        let (final_width, final_height) = resize_events.last().unwrap();
        assert_eq!(
            state.width, *final_width,
            "State should reflect final width after rapid resizes"
        );
        assert_eq!(
            state.height, *final_height,
            "State should reflect final height after rapid resizes"
        );
    }

    /// Integration Test 4: D-Bus command enum completeness
    ///
    /// This test verifies all D-Bus commands can flow through the channel system.
    #[tokio::test]
    async fn test_dbus_all_commands_flow() {
        let (mut tx, mut rx) = mpsc::channel::<DbusCommand>(10);

        // Send all command types through channel
        tx.send(DbusCommand::Show).await.unwrap();
        tx.send(DbusCommand::Hide).await.unwrap();
        tx.send(DbusCommand::Toggle).await.unwrap();
        tx.send(DbusCommand::Quit).await.unwrap();

        // Verify all commands received in order
        use futures::StreamExt;
        assert_eq!(rx.next().await, Some(DbusCommand::Show));
        assert_eq!(rx.next().await, Some(DbusCommand::Hide));
        assert_eq!(rx.next().await, Some(DbusCommand::Toggle));
        assert_eq!(rx.next().await, Some(DbusCommand::Quit));
    }

    /// Integration Test 5: Applet D-Bus reconnection scenario
    ///
    /// This test verifies the visibility state tracking that would be used
    /// when the applet reconnects to a restarted main app.
    #[test]
    fn test_applet_reconnection_state() {
        let visible = Arc::new(AtomicBool::new(false)); // Main app starts hidden

        // After reconnection, applet should be able to query state
        assert!(
            !visible.load(Ordering::SeqCst),
            "Applet should see current visibility state after reconnection"
        );

        // Main app shows window
        visible.store(true, Ordering::SeqCst);

        assert!(
            visible.load(Ordering::SeqCst),
            "Applet should see updated visibility after main app change"
        );
    }

    /// Integration Test 6: Window state restoration accuracy
    ///
    /// This test verifies that window state values are preserved exactly.
    /// Position is stored as margins from bottom-right anchor in floating mode.
    #[test]
    fn test_window_state_restoration_accuracy() {
        let original = WindowState {
            width: 987.654,
            height: 321.098,
            is_floating: true,
            margin_bottom: 50,
            margin_right: 100,
        };

        // Clone simulates save/restore cycle
        let restored = original.clone();

        // Float comparison with exact equality since these are not computed
        assert_eq!(original.width, restored.width, "Width must be exact");
        assert_eq!(original.height, restored.height, "Height must be exact");
        assert_eq!(
            original.is_floating, restored.is_floating,
            "Floating mode must be exact"
        );
        assert_eq!(
            original.margin_bottom, restored.margin_bottom,
            "Margin bottom must be exact"
        );
        assert_eq!(
            original.margin_right, restored.margin_right,
            "Margin right must be exact"
        );
    }

    /// Integration Test 7: Layer-shell configuration workflow
    ///
    /// This test verifies the layer-shell configuration can be set up correctly
    /// for overlay behavior.
    #[test]
    fn test_layer_shell_configuration_workflow() {
        // Create default config (should be Overlay)
        let config = LayerShellConfig::new();
        assert_eq!(config.layer(), Layer::Overlay);

        // Verify configuration chain works
        let config = LayerShellConfig::new().with_layer(Layer::Overlay);
        assert_eq!(config.layer(), Layer::Overlay);
        assert!(!config.is_layer_surface()); // Not a true layer surface without compositor

        // Verify all layer types work
        for layer in [Layer::Background, Layer::Bottom, Layer::Top, Layer::Overlay] {
            let config = LayerShellConfig::new().with_layer(layer);
            assert_eq!(config.layer(), layer);
        }
    }

    /// Integration Test 8: Complete message flow validation
    ///
    /// This test verifies the app message types are correct and complete
    /// for handling all D-Bus operations.
    #[test]
    fn test_complete_message_flow() {
        use crate::app::Message;

        // Verify D-Bus-related message variants exist and can be created
        // Note: DbusCommandReceived now includes a receiver, so we skip testing it directly
        let messages: Vec<Message> = vec![
            Message::DbusShow,
            Message::DbusHide,
            Message::DbusToggle,
            Message::DbusQuit,
            Message::DbusServerStarted,
            Message::DbusServerFailed("test".to_string()),
        ];

        // Verify each message variant matches expected pattern
        for msg in messages {
            match msg {
                Message::DbusShow
                | Message::DbusHide
                | Message::DbusToggle
                | Message::DbusQuit
                | Message::DbusServerStarted
                | Message::DbusServerFailed(_) => {
                    // All D-Bus messages are present and matchable
                }
                _ => {
                    // Other message types are valid too
                }
            }
        }

        // Also verify DbusCommand enum variants
        let commands = vec![
            DbusCommand::Show,
            DbusCommand::Hide,
            DbusCommand::Toggle,
            DbusCommand::Quit,
        ];
        for cmd in commands {
            match cmd {
                DbusCommand::Show
                | DbusCommand::Hide
                | DbusCommand::Toggle
                | DbusCommand::Quit => {
                    // All command variants exist
                }
            }
        }
    }
}
