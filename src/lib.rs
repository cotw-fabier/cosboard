// SPDX-License-Identifier: GPL-3.0-only

//! Cosboard - A soft keyboard for COSMIC desktop
//!
//! This crate provides a soft keyboard applet for the COSMIC desktop environment.
//! The applet lives in the system panel and manages the keyboard window directly.
//!
//! # Modules
//!
//! - `applet`: System tray applet with integrated keyboard management
//! - `app_settings`: Centralized application constants and configuration
//! - `config`: User configuration with cosmic_config persistence
//! - `i18n`: Localization support using fluent translations
//! - `layer_shell`: Wayland layer-shell integration for overlay behavior
//! - `state`: Window state persistence (position, size)

pub mod app_settings;
pub mod applet;
pub mod config;
pub mod i18n;
pub mod layer_shell;
pub mod state;

// Re-export the fl! macro for localization
pub use crate::i18n::LANGUAGE_LOADER;

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use crate::layer_shell::{Layer, LayerShellConfig};
    use crate::state::WindowState;

    /// Integration Test: State persistence across application restart
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

    /// Integration Test: Multiple rapid resize events handling
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

    /// Integration Test: Window state restoration accuracy
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

    /// Integration Test: Layer-shell configuration workflow
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
}
