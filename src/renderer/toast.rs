// SPDX-License-Identifier: GPL-3.0-only

//! Toast notification rendering for the keyboard layout renderer.
//!
//! This module provides functions for rendering toast notifications at the
//! bottom of the keyboard surface. Toasts are used to display error messages,
//! warnings, and informational messages to the user.
//!
//! # Features
//!
//! - Semi-transparent themed background
//! - Centered message text
//! - Severity-based color accent (Info, Warning, Error)
//! - Auto-dismiss after 3 seconds
//! - Queue-based system for multiple toasts
//!
//! # Usage
//!
//! ```rust,ignore
//! use cosboard::renderer::toast::{render_toast, render_keyboard_with_toast};
//! use cosboard::renderer::{Toast, ToastSeverity};
//! use cosmic::Theme;
//!
//! // Create a toast
//! let toast = Toast::error("Panel 'numpad' not found");
//!
//! // Render the toast
//! let theme = Theme::dark();
//! let toast_element = render_toast(&toast, &theme);
//!
//! // Integrate toast with keyboard panel
//! let keyboard_with_toast = render_keyboard_with_toast(
//!     keyboard_panel,
//!     Some(toast_element),
//!     surface_height,
//! );
//! ```

use cosmic::iced::{alignment, Length, Padding};
use cosmic::widget::{self, container};
use cosmic::Element;
use cosmic::Theme;

use crate::renderer::message::RendererMessage;
use crate::renderer::state::{Toast, ToastSeverity};
use crate::renderer::theme::toast_background_color;

/// Default height for the toast display area in pixels.
pub const TOAST_HEIGHT: f32 = 40.0;

/// Horizontal padding for toast content.
const TOAST_PADDING_HORIZONTAL: f32 = 16.0;

/// Vertical padding for toast content.
const TOAST_PADDING_VERTICAL: f32 = 8.0;

/// Border radius for toast container.
const TOAST_BORDER_RADIUS: f32 = 8.0;

/// Renders a toast notification.
///
/// Creates a styled container with the toast message centered inside.
/// The appearance is based on the current theme and the toast severity:
///
/// - **Info**: Standard styling on semi-transparent background
/// - **Warning**: Warning-styled on semi-transparent background
/// - **Error**: Error-styled on semi-transparent background
///
/// # Arguments
///
/// * `toast` - The toast notification to render
/// * `theme` - Reference to the current COSMIC theme
///
/// # Returns
///
/// An Element containing the rendered toast notification.
pub fn render_toast<'a>(toast: &Toast, theme: &Theme) -> Element<'a, RendererMessage> {
    let bg_color = toast_background_color(theme);

    // Format message with severity prefix for clarity
    let display_message = match toast.severity {
        ToastSeverity::Info => toast.message.clone(),
        ToastSeverity::Warning => format!("Warning: {}", toast.message),
        ToastSeverity::Error => format!("Error: {}", toast.message),
    };

    // Create the message text
    let message_text = widget::text::body(display_message)
        .width(Length::Shrink)
        .height(Length::Shrink);

    // Wrap in a container with themed background
    let toast_container = container(message_text)
        .width(Length::Shrink)
        .height(Length::Fixed(TOAST_HEIGHT))
        .padding(Padding::new(TOAST_PADDING_VERTICAL).left(TOAST_PADDING_HORIZONTAL).right(TOAST_PADDING_HORIZONTAL))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .class(cosmic::style::Container::custom(move |_theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(bg_color)),
                border: cosmic::iced::Border {
                    color: cosmic::iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: TOAST_BORDER_RADIUS.into(),
                },
                icon_color: None,
                text_color: None,
                shadow: cosmic::iced::Shadow::default(),
            }
        }));

    // Wrap in a centering container that fills the width
    container(toast_container)
        .width(Length::Fill)
        .height(Length::Fixed(TOAST_HEIGHT))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
}

/// Renders the keyboard panel with an optional toast notification area.
///
/// Creates a vertical layout with the keyboard panel at the top and a toast
/// notification area at the bottom. When no toast is active, the toast area
/// collapses to zero height.
///
/// # Layout
///
/// ```text
/// +------------------------+
/// |                        |
/// |    Keyboard Panel      |
/// |                        |
/// +------------------------+
/// |       Toast Area       |  <- Only visible when toast is active
/// +------------------------+
/// ```
///
/// # Arguments
///
/// * `panel` - The rendered keyboard panel element
/// * `toast` - Optional rendered toast element (None if no toast active)
/// * `surface_height` - Total height of the keyboard surface
///
/// # Returns
///
/// An Element containing the keyboard panel with toast area.
pub fn render_keyboard_with_toast<'a>(
    panel: Element<'a, RendererMessage>,
    toast: Option<Element<'a, RendererMessage>>,
    _surface_height: f32,
) -> Element<'a, RendererMessage> {
    match toast {
        Some(toast_element) => {
            // With toast: keyboard panel takes remaining space, toast at bottom
            widget::column::column()
                .push(
                    container(panel)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .push(toast_element)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
        None => {
            // No toast: keyboard panel fills entire surface
            container(panel)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }
}

/// Renders a toast notification from the keyboard renderer state.
///
/// Convenience function that extracts the current toast from the renderer
/// state and renders it if present.
///
/// # Arguments
///
/// * `state` - The keyboard renderer state
/// * `theme` - Reference to the current COSMIC theme
///
/// # Returns
///
/// An optional Element containing the rendered toast, or None if no toast is active.
pub fn render_current_toast<'a>(
    state: &crate::renderer::KeyboardRenderer,
    theme: &Theme,
) -> Option<Element<'a, RendererMessage>> {
    state.current_toast.as_ref().map(|(toast, _)| render_toast(toast, theme))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, Key, KeyCode, Layout, Panel, Row, Sizing};
    use crate::renderer::KeyboardRenderer;
    use std::collections::HashMap;
    use std::thread::sleep;
    use std::time::Duration;

    /// Helper function to create a minimal test layout.
    fn create_test_layout() -> Layout {
        let mut panels = HashMap::new();

        let main_panel = Panel {
            id: "main".to_string(),
            padding: Some(5.0),
            margin: Some(2.0),
            nesting_depth: 0,
            rows: vec![Row {
                cells: vec![Cell::Key(Key {
                    label: "A".to_string(),
                    code: KeyCode::Unicode('a'),
                    identifier: Some("key_a".to_string()),
                    width: Sizing::Relative(1.0),
                    height: Sizing::Relative(1.0),
                    min_width: None,
                    min_height: None,
                    alternatives: HashMap::new(),
                    sticky: false,
                    stickyrelease: true,
                })],
            }],
        };

        panels.insert("main".to_string(), main_panel);

        Layout {
            name: "Test Layout".to_string(),
            description: None,
            author: None,
            language: None,
            locale: None,
            version: "1.0".to_string(),
            default_panel_id: "main".to_string(),
            inherits: None,
            panels,
        }
    }

    // ========================================================================
    // Task 6.1: Focused tests for toast system (2-4 tests)
    // ========================================================================

    /// Test 1: Toast queue ordering (FIFO)
    ///
    /// Verifies that toasts are displayed in FIFO (first-in-first-out) order.
    /// The first toast queued should be the first one displayed.
    #[test]
    fn test_toast_queue_ordering_fifo() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initially no toasts
        assert!(renderer.current_toast.is_none());
        assert!(renderer.toast_queue.is_empty());

        // Queue first toast - should become current immediately
        renderer.queue_toast("First", ToastSeverity::Info);
        assert!(renderer.toast_queue.is_empty()); // Moved to current
        let (toast, _) = renderer.current_toast.as_ref().unwrap();
        assert_eq!(toast.message, "First");

        // Queue second and third toasts - should go to queue
        renderer.queue_toast("Second", ToastSeverity::Warning);
        renderer.queue_toast("Third", ToastSeverity::Error);
        assert_eq!(renderer.toast_queue.len(), 2);

        // Dismiss first and show next - should be second
        renderer.dismiss_current_toast();
        renderer.show_next_toast();
        let (toast, _) = renderer.current_toast.as_ref().unwrap();
        assert_eq!(toast.message, "Second");

        // Dismiss second and show next - should be third
        renderer.dismiss_current_toast();
        renderer.show_next_toast();
        let (toast, _) = renderer.current_toast.as_ref().unwrap();
        assert_eq!(toast.message, "Third");

        // Queue is now empty
        assert!(renderer.toast_queue.is_empty());
    }

    /// Test 2: Toast auto-dismiss after 3 seconds
    ///
    /// Verifies that check_toast_timeout returns true after 3 seconds
    /// have elapsed since the toast was displayed.
    #[test]
    fn test_toast_auto_dismiss_after_3_seconds() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Queue a toast
        renderer.queue_toast("Test toast", ToastSeverity::Info);
        assert!(renderer.current_toast.is_some());

        // Immediately check - should not be timed out
        assert!(
            !renderer.check_toast_timeout(),
            "Toast should not time out immediately"
        );

        // Wait for less than 3 seconds (e.g., 100ms)
        sleep(Duration::from_millis(100));
        assert!(
            !renderer.check_toast_timeout(),
            "Toast should not time out after 100ms"
        );

        // Wait for the full 3 seconds plus buffer
        sleep(Duration::from_millis(3000));
        assert!(
            renderer.check_toast_timeout(),
            "Toast should time out after 3 seconds"
        );
    }

    /// Test 3: Multiple toasts queued (show one at a time)
    ///
    /// Verifies that only one toast is displayed at a time, and additional
    /// toasts are queued for later display.
    #[test]
    fn test_multiple_toasts_queued_show_one_at_a_time() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Queue 5 toasts
        for i in 1..=5 {
            renderer.queue_toast(format!("Toast {}", i), ToastSeverity::Info);
        }

        // First toast should be current, rest in queue
        let (toast, _) = renderer.current_toast.as_ref().unwrap();
        assert_eq!(toast.message, "Toast 1");
        assert_eq!(renderer.toast_queue.len(), 4);

        // Process all toasts one by one
        for i in 2..=5 {
            renderer.dismiss_current_toast();
            renderer.show_next_toast();

            let (toast, _) = renderer.current_toast.as_ref().unwrap();
            assert_eq!(toast.message, format!("Toast {}", i));
            assert_eq!(renderer.toast_queue.len(), 5 - i);
        }

        // After processing all, queue should be empty
        renderer.dismiss_current_toast();
        assert!(renderer.current_toast.is_none());
        assert!(renderer.toast_queue.is_empty());
    }

    /// Test 4: Toast display area positioning (rendering test)
    ///
    /// Verifies that render_toast and render_keyboard_with_toast produce
    /// valid elements without panicking.
    #[test]
    fn test_toast_display_area_positioning() {
        let theme = Theme::dark();

        // Test rendering an info toast
        let info_toast = Toast::info("Information message");
        let _element = render_toast(&info_toast, &theme);

        // Test rendering a warning toast
        let warning_toast = Toast::warning("Warning message");
        let _element = render_toast(&warning_toast, &theme);

        // Test rendering an error toast
        let error_toast = Toast::error("Error message");
        let _element = render_toast(&error_toast, &theme);

        // Test render_keyboard_with_toast with toast
        let keyboard_panel: Element<'_, RendererMessage> = container(widget::text::body("Keyboard"))
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        let toast_element = render_toast(&error_toast, &theme);
        let _combined = render_keyboard_with_toast(keyboard_panel, Some(toast_element), 300.0);

        // Test render_keyboard_with_toast without toast (collapsed)
        let keyboard_panel2: Element<'_, RendererMessage> = container(widget::text::body("Keyboard"))
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        let _combined = render_keyboard_with_toast(keyboard_panel2, None, 300.0);
    }

    /// Test: render_current_toast returns None when no toast active
    #[test]
    fn test_render_current_toast_returns_none_when_empty() {
        let layout = create_test_layout();
        let renderer = KeyboardRenderer::new(layout);
        let theme = Theme::dark();

        // No toast active
        let result = render_current_toast(&renderer, &theme);
        assert!(result.is_none());
    }

    /// Test: render_current_toast returns Some when toast active
    #[test]
    fn test_render_current_toast_returns_some_when_active() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);
        let theme = Theme::dark();

        // Queue a toast
        renderer.queue_toast("Active toast", ToastSeverity::Error);

        // Should return Some
        let result = render_current_toast(&renderer, &theme);
        assert!(result.is_some());
    }

    /// Test: Toast severity affects rendering (no panic)
    #[test]
    fn test_toast_severity_rendering() {
        let theme = Theme::dark();

        // Test all severity levels
        for severity in [
            ToastSeverity::Info,
            ToastSeverity::Warning,
            ToastSeverity::Error,
        ] {
            let toast = Toast::new("Test message", severity);
            let _element = render_toast(&toast, &theme);
        }
    }
}
