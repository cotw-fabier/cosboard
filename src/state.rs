// SPDX-License-Identifier: GPL-3.0-only

use crate::app_settings;
use cosmic::cosmic_config;
use cosmic::cosmic_config::{cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};

/// Window state that persists between application runs.
///
/// In docked mode, the keyboard is anchored full-width to the bottom of the screen.
/// In floating mode, the keyboard is anchored to the bottom-right corner and can
/// be repositioned via margins and resized.
#[derive(Debug, Clone, CosmicConfigEntry, PartialEq)]
#[version = 4]
pub struct WindowState {
    /// Window width (used in floating mode, ignored in docked mode).
    pub width: f32,
    /// Window height.
    pub height: f32,
    /// Whether the keyboard floats (overlay) or reserves exclusive screen space.
    /// - `true`: Floating mode - keyboard overlays content, can be dragged/resized
    /// - `false`: Docked mode - full-width bottom, other windows resize to avoid
    pub is_floating: bool,
    /// Margin from bottom edge (floating mode position).
    pub margin_bottom: i32,
    /// Margin from right edge (floating mode position).
    pub margin_right: i32,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: app_settings::DEFAULT_WIDTH,
            height: app_settings::DEFAULT_HEIGHT,
            is_floating: false, // Default to docked mode for proper soft keyboard behavior
            margin_bottom: 0,
            margin_right: 0,
        }
    }
}
