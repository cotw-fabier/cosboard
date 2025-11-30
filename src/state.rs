// SPDX-License-Identifier: GPL-3.0-only

use crate::app_settings;
use cosmic::cosmic_config;
use cosmic::cosmic_config::{cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};

/// Window state that persists between application runs.
#[derive(Debug, Clone, CosmicConfigEntry, PartialEq)]
#[version = 1]
pub struct WindowState {
    /// Window X position.
    pub x: i32,
    /// Window Y position.
    pub y: i32,
    /// Window width.
    pub width: f32,
    /// Window height.
    pub height: f32,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: app_settings::DEFAULT_WIDTH,
            height: app_settings::DEFAULT_HEIGHT,
        }
    }
}
