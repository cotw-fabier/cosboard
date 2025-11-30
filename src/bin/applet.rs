// SPDX-License-Identifier: GPL-3.0-only

//! Cosboard System Tray Applet
//!
//! This binary provides a system tray applet for the COSMIC panel that allows
//! users to control the Cosboard soft keyboard.
//!
//! # Usage
//!
//! The applet is launched by the COSMIC panel when configured. It can also be
//! run standalone for testing:
//!
//! ```bash
//! cargo run --bin cosboard-applet
//! ```
//!
//! # Features
//!
//! - Shows a keyboard icon in the system tray
//! - Left-click: Toggle keyboard visibility
//! - Right-click/Click: Open popup menu with options
//! - Communicates with main Cosboard application via D-Bus

// Re-export the main cosboard crate's modules
use cosboard::applet;

fn main() -> cosmic::iced::Result {
    // Initialize logging for the applet
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("cosboard=info".parse().unwrap()),
        )
        .init();

    tracing::info!("Starting Cosboard applet");

    // Run the applet
    applet::run()
}
