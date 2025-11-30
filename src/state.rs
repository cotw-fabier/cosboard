// SPDX-License-Identifier: GPL-3.0-only

use crate::app_settings;
use cosmic::cosmic_config;
use cosmic::cosmic_config::{cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};

/// Window state that persists between application runs.
///
/// Note: Layer surfaces on Wayland don't support programmatic positioning,
/// so x/y coordinates are not stored. The keyboard is anchored to the bottom
/// of the screen by the compositor.
#[derive(Debug, Clone, CosmicConfigEntry, PartialEq)]
#[version = 3]
pub struct WindowState {
    /// Window width (may be ignored for full-width layer surfaces).
    pub width: f32,
    /// Window height.
    pub height: f32,
    /// Whether the keyboard floats (overlay) or reserves exclusive screen space.
    /// - `true`: Floating mode - keyboard overlays content without reserving space
    /// - `false`: Exclusive zone mode - other windows resize to avoid keyboard
    pub is_floating: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: app_settings::DEFAULT_WIDTH,
            height: app_settings::DEFAULT_HEIGHT,
            is_floating: false, // Default to exclusive zone for proper soft keyboard behavior
        }
    }
}
