// SPDX-License-Identifier: GPL-3.0-only

//! Widget placeholder rendering for the keyboard layout renderer.
//!
//! This module provides rendering for widget placeholders such as trackpads
//! and autocomplete bars. These are shown as placeholder containers until
//! actual widget functionality is implemented.

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, container};
use cosmic::Element;

use crate::layout::Widget;
use crate::renderer::message::RendererMessage;
use crate::renderer::sizing::resolve_sizing;

/// Renders a widget placeholder container.
///
/// Placeholder widgets are rendered with:
/// - Themed background color matching key background
/// - Centered label showing the widget type (e.g., "Trackpad", "Autocomplete")
/// - Width and height from the layout specification
///
/// # Arguments
///
/// * `widget` - The widget definition from the layout
/// * `base_unit` - The calculated base unit for relative sizing
/// * `scale` - HDPI scale factor for pixel sizing
///
/// # Returns
///
/// An Element containing the rendered widget placeholder.
pub fn render_widget_placeholder<'a>(
    widget: &Widget,
    base_unit: f32,
    scale: f32,
) -> Element<'a, RendererMessage> {
    let width = resolve_sizing(&widget.width, base_unit, scale);
    let height = resolve_sizing(&widget.height, base_unit, scale);

    // Format the widget type label (capitalize first letter)
    let label = format_widget_label(&widget.widget_type);

    // Create centered label
    let content = container(widget::text::body(label))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

    // Wrap in styled container
    container(content)
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .class(cosmic::style::Container::Card)
        .into()
}

/// Formats a widget type string for display.
///
/// Capitalizes the first letter and returns common display names:
/// - "trackpad" -> "Trackpad"
/// - "autocomplete" -> "Autocomplete"
/// - other -> Capitalized first letter
///
/// # Arguments
///
/// * `widget_type` - The widget type string from the layout
///
/// # Returns
///
/// A formatted display string.
fn format_widget_label(widget_type: &str) -> String {
    match widget_type.to_lowercase().as_str() {
        "trackpad" => "Trackpad".to_string(),
        "autocomplete" => "Autocomplete".to_string(),
        "prediction" | "prediction_bar" => "Prediction".to_string(),
        other => {
            // Capitalize first letter
            let mut chars = other.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Sizing;

    /// Test: Widget placeholder rendering
    #[test]
    fn test_widget_placeholder_rendering() {
        let widget = Widget {
            widget_type: "trackpad".to_string(),
            width: Sizing::Relative(2.0),
            height: Sizing::Relative(2.0),
        };

        let base_unit = 80.0;
        let scale = 1.0;

        // This should not panic
        let _element = render_widget_placeholder(&widget, base_unit, scale);
    }

    /// Test: Widget label formatting
    #[test]
    fn test_format_widget_label() {
        assert_eq!(format_widget_label("trackpad"), "Trackpad");
        assert_eq!(format_widget_label("TRACKPAD"), "Trackpad");
        assert_eq!(format_widget_label("autocomplete"), "Autocomplete");
        assert_eq!(format_widget_label("prediction"), "Prediction");
        assert_eq!(format_widget_label("prediction_bar"), "Prediction");
        assert_eq!(format_widget_label("custom"), "Custom");
        assert_eq!(format_widget_label(""), "");
    }

    /// Test: Autocomplete widget rendering
    #[test]
    fn test_autocomplete_widget_rendering() {
        let widget = Widget {
            widget_type: "autocomplete".to_string(),
            width: Sizing::Relative(10.0),
            height: Sizing::Relative(1.0),
        };

        let base_unit = 60.0;
        let scale = 1.5;

        // This should not panic
        let _element = render_widget_placeholder(&widget, base_unit, scale);
    }

    /// Test: Widget with pixel sizing
    #[test]
    fn test_widget_with_pixel_sizing() {
        let widget = Widget {
            widget_type: "trackpad".to_string(),
            width: Sizing::Pixels("200px".to_string()),
            height: Sizing::Pixels("100px".to_string()),
        };

        let base_unit = 80.0;
        let scale = 2.0;

        // This should not panic
        let _element = render_widget_placeholder(&widget, base_unit, scale);
    }
}
