// SPDX-License-Identifier: GPL-3.0-only

//! Centralized application settings and constants.

/// Application ID in RDNN (reverse domain name notation) format.
pub const APP_ID: &str = "io.github.cosboard.Cosboard";

/// Application version for config/state versioning.
pub const APP_VERSION: u64 = 1;

/// Default window width in pixels.
pub const DEFAULT_WIDTH: f32 = 800.0;

/// Default window height in pixels.
pub const DEFAULT_HEIGHT: f32 = 300.0;

/// Minimum window width in pixels.
pub const MIN_WIDTH: f32 = 400.0;

/// Minimum window height in pixels.
pub const MIN_HEIGHT: f32 = 150.0;

/// Resize border width in pixels.
pub const RESIZE_BORDER: f64 = 8.0;
