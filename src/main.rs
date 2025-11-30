// SPDX-License-Identifier: GPL-3.0-only

//! Cosboard Main Application
//!
//! This is the main entry point for the Cosboard soft keyboard application.
//! The application displays a floating keyboard window that can be controlled
//! via the system tray applet.

use cosboard::{app, app_settings, i18n, layer_shell};

fn main() -> cosmic::iced::Result {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("cosboard=info".parse().unwrap()),
        )
        .init();

    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Get the window level for always-on-top behavior.
    // On X11: Uses EWMH _NET_WM_STATE_ABOVE hint.
    // On Wayland: The hint is not effective; true overlay requires layer-shell.
    // See layer_shell module documentation for details.
    let window_level = layer_shell::get_window_level();

    // Settings for configuring the application window and iced runtime.
    let settings = cosmic::app::Settings::default()
        // Set initial window size from app_settings
        .size(cosmic::iced::Size::new(
            app_settings::DEFAULT_WIDTH,
            app_settings::DEFAULT_HEIGHT,
        ))
        // Set minimum window size constraints
        .size_limits(
            cosmic::iced::Limits::NONE
                .min_width(app_settings::MIN_WIDTH)
                .min_height(app_settings::MIN_HEIGHT),
        )
        // Enable resize border for borderless window resizing
        .resizable(Some(app_settings::RESIZE_BORDER))
        // Use client-side decorations (no window manager decorations)
        .client_decorations(true)
        // Enable transparency for the window
        .transparent(true)
        // Keep app running when window closes (for D-Bus service to remain active)
        .exit_on_close(false)
        // Start without a main window - we'll create secondary windows on demand
        // This prevents the app from exiting when windows are closed
        .no_main_window(true);

    // Log the window level configuration
    tracing::info!(
        "Configuring soft keyboard window with level: {:?}",
        window_level
    );

    // Note: The window level cannot be set through cosmic::app::Settings directly.
    // It would need to be applied after window creation via the Application trait.
    // For now, we document the limitation and handle it in the app module.

    // Starts the application's event loop with `()` as the application's flags.
    cosmic::app::run::<app::AppModel>(settings, ())
}
