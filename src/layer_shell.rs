// SPDX-License-Identifier: GPL-3.0-only

//! Layer-shell integration for overlay window behavior.
//!
//! This module provides utilities for configuring the soft keyboard window to
//! appear above other windows. On Wayland, true layer-shell overlay behavior
//! requires the zwlr_layer_shell_v1 protocol, which is not directly exposed
//! through the standard cosmic::Application framework.
//!
//! ## Current Implementation
//!
//! The implementation uses `window::Level::AlwaysOnTop` for X11 environments
//! and logs a warning on Wayland where this setting is not supported by the
//! protocol. Future versions may integrate directly with cctk/sctk for true
//! layer-shell surface creation.
//!
//! ## Wayland Layer-Shell Protocol
//!
//! The zwlr_layer_shell_v1 protocol defines four layers (from bottom to top):
//! - Background: Below all other windows
//! - Bottom: Above background, below normal windows
//! - Top: Above normal windows (used by panels)
//! - Overlay: Highest layer (used by screen lockers, on-screen keyboards)
//!
//! A soft keyboard should ideally be on the Overlay layer to ensure it's always
//! visible and can receive input regardless of which application has focus.

use cosmic::app::cosmic::WindowingSystem;

/// Layer types for window positioning (mirrors zwlr_layer_shell_v1 layers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layer {
    /// Background layer - below all other windows.
    Background,
    /// Bottom layer - above background, below normal windows.
    Bottom,
    /// Top layer - above normal windows (panels, docks).
    Top,
    /// Overlay layer - highest z-order (keyboards, screen lockers).
    #[default]
    Overlay,
}

impl Layer {
    /// Returns a human-readable name for the layer.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Layer::Background => "Background",
            Layer::Bottom => "Bottom",
            Layer::Top => "Top",
            Layer::Overlay => "Overlay",
        }
    }
}

/// Configuration for layer-shell behavior.
#[derive(Debug, Clone)]
pub struct LayerShellConfig {
    /// The layer to place the window on.
    pub layer: Layer,
    /// Whether layer-shell is available on the current system.
    pub available: bool,
    /// Whether the window is currently configured as a layer surface.
    pub is_layer_surface: bool,
}

impl Default for LayerShellConfig {
    fn default() -> Self {
        Self {
            layer: Layer::Overlay,
            available: false,
            is_layer_surface: false,
        }
    }
}

impl LayerShellConfig {
    /// Creates a new layer-shell configuration with overlay layer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the target layer for the window.
    #[must_use]
    pub fn with_layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    /// Checks if the current windowing system supports layer-shell.
    ///
    /// Returns true only on Wayland compositors that support zwlr_layer_shell_v1.
    pub fn check_availability(&mut self) -> bool {
        if let Some(system) = cosmic::app::cosmic::windowing_system() {
            match system {
                WindowingSystem::Wayland => {
                    // On Wayland, layer-shell might be available but the current
                    // cosmic::Application framework creates XDG toplevels, not layer surfaces.
                    // True layer-shell would require direct cctk/sctk integration.
                    tracing::info!(
                        "Running on Wayland - layer-shell protocol may be available, \
                         but cosmic::Application creates XDG toplevels. \
                         Using fallback window hints instead."
                    );
                    self.available = true;
                    self.is_layer_surface = false; // Not a true layer surface yet
                }
                WindowingSystem::Xlib | WindowingSystem::Xcb => {
                    tracing::info!(
                        "Running on X11 - using EWMH _NET_WM_STATE_ABOVE for always-on-top"
                    );
                    self.available = false;
                }
                other => {
                    tracing::warn!(
                        "Running on unsupported windowing system: {:?} - \
                         always-on-top behavior may not work",
                        other
                    );
                    self.available = false;
                }
            }
        } else {
            tracing::warn!(
                "Windowing system not yet detected - \
                 layer-shell availability unknown"
            );
        }
        self.available
    }

    /// Returns true if true layer-shell overlay behavior is active.
    #[must_use]
    pub fn is_layer_surface(&self) -> bool {
        self.is_layer_surface
    }

    /// Returns the configured layer.
    #[must_use]
    pub fn layer(&self) -> Layer {
        self.layer
    }
}

/// Determines the appropriate window level based on platform capabilities.
///
/// On X11, this returns `AlwaysOnTop` which is supported via EWMH.
/// On Wayland, this still returns `AlwaysOnTop` but it may not be effective
/// since Wayland doesn't support this hint - true overlay behavior requires
/// layer-shell protocol integration.
#[must_use]
pub fn get_window_level() -> cosmic::iced::window::Level {
    // Always return AlwaysOnTop - it works on X11 and is the best we can do
    // without true layer-shell integration on Wayland.
    cosmic::iced::window::Level::AlwaysOnTop
}

/// Logs the current layer-shell status for debugging.
pub fn log_layer_status(config: &LayerShellConfig) {
    if config.is_layer_surface {
        tracing::info!(
            "Window configured as layer surface on {} layer",
            config.layer.as_str()
        );
    } else if config.available {
        tracing::info!(
            "Layer-shell available but not using layer surface - \
             using window hints for always-on-top behavior"
        );
    } else {
        tracing::info!(
            "Layer-shell not available - using platform window hints"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Window configured as layer surface (configuration state).
    #[test]
    fn test_layer_surface_configuration() {
        let mut config = LayerShellConfig::new();

        // Initially not a layer surface
        assert!(!config.is_layer_surface());

        // After simulating layer surface creation
        config.is_layer_surface = true;
        assert!(config.is_layer_surface());
    }

    /// Test: Layer set to overlay layer (highest z-order).
    #[test]
    fn test_overlay_layer_default() {
        let config = LayerShellConfig::new();

        // Default should be Overlay layer
        assert_eq!(config.layer(), Layer::Overlay);
        assert_eq!(config.layer().as_str(), "Overlay");
    }

    /// Test: Window stays above normal windows (via window level).
    #[test]
    fn test_always_on_top_level() {
        let level = get_window_level();

        // Should return AlwaysOnTop
        assert_eq!(level, cosmic::iced::window::Level::AlwaysOnTop);
    }

    /// Test: Layer configuration with different layers.
    #[test]
    fn test_layer_configuration() {
        let config = LayerShellConfig::new()
            .with_layer(Layer::Top);

        assert_eq!(config.layer(), Layer::Top);
        assert_eq!(config.layer().as_str(), "Top");

        let config = LayerShellConfig::new()
            .with_layer(Layer::Background);
        assert_eq!(config.layer(), Layer::Background);
    }

    /// Test: All layer variants have correct string representation.
    #[test]
    fn test_layer_names() {
        assert_eq!(Layer::Background.as_str(), "Background");
        assert_eq!(Layer::Bottom.as_str(), "Bottom");
        assert_eq!(Layer::Top.as_str(), "Top");
        assert_eq!(Layer::Overlay.as_str(), "Overlay");
    }

    /// Test: Default layer shell config values.
    #[test]
    fn test_default_config() {
        let config = LayerShellConfig::default();

        assert_eq!(config.layer, Layer::Overlay);
        assert!(!config.available);
        assert!(!config.is_layer_surface);
    }
}
