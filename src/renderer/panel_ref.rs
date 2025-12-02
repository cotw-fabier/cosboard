// SPDX-License-Identifier: GPL-3.0-only

//! Panel reference button rendering for the keyboard layout renderer.
//!
//! This module provides rendering for panel reference buttons, which allow
//! users to switch between different keyboard panels (e.g., switching from
//! the main QWERTY panel to a symbols panel or numpad).

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, container};
use cosmic::Element;

use crate::layout::PanelRef;
use crate::renderer::message::RendererMessage;
use crate::renderer::sizing::resolve_sizing;

/// Renders a panel reference as a button.
///
/// Panel reference buttons are rendered with:
/// - Button appearance matching key styling
/// - Panel ID as the label text
/// - Width and height from the layout specification
/// - Emits `RendererMessage::SwitchPanel` on click
///
/// # Arguments
///
/// * `panel_ref` - The panel reference definition from the layout
/// * `base_unit` - The calculated base unit for relative sizing
/// * `scale` - HDPI scale factor for pixel sizing
///
/// # Returns
///
/// An Element containing the rendered panel reference button.
pub fn render_panel_ref_button<'a>(
    panel_ref: &PanelRef,
    base_unit: f32,
    scale: f32,
) -> Element<'a, RendererMessage> {
    let width = resolve_sizing(&panel_ref.width, base_unit, scale);
    let height = resolve_sizing(&panel_ref.height, base_unit, scale);

    // Use the panel_id as the button label
    let label = format_panel_label(&panel_ref.panel_id);

    // Create centered label content
    let content = container(widget::text::body(label))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

    // Create button that emits SwitchPanel message on click
    let panel_id = panel_ref.panel_id.clone();

    button::custom(content)
        .on_press(RendererMessage::SwitchPanel(panel_id))
        .class(cosmic::style::Button::Standard)
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .into()
}

/// Formats a panel ID for display as a button label.
///
/// Common panel names are given friendly display names:
/// - "numpad" -> "123"
/// - "symbols" -> "#+='"
/// - "emoji" -> emoji symbol
/// - other -> Capitalized first letter
///
/// # Arguments
///
/// * `panel_id` - The panel ID string from the layout
///
/// # Returns
///
/// A formatted display string.
fn format_panel_label(panel_id: &str) -> String {
    match panel_id.to_lowercase().as_str() {
        "numpad" | "numbers" | "num" => "123".to_string(),
        "symbols" | "sym" | "symbol" => "#+=".to_string(),
        "emoji" | "emojis" => "\u{1F600}".to_string(), // Grinning face emoji
        "main" | "qwerty" | "default" => "ABC".to_string(),
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

    /// Test: Panel reference button rendering
    #[test]
    fn test_panel_ref_button_rendering() {
        let panel_ref = PanelRef {
            panel_id: "numpad".to_string(),
            width: Sizing::Relative(1.5),
            height: Sizing::Relative(1.0),
        };

        let base_unit = 80.0;
        let scale = 1.0;

        // This should not panic
        let _element = render_panel_ref_button(&panel_ref, base_unit, scale);
    }

    /// Test: Panel label formatting
    #[test]
    fn test_format_panel_label() {
        assert_eq!(format_panel_label("numpad"), "123");
        assert_eq!(format_panel_label("NUMPAD"), "123");
        assert_eq!(format_panel_label("numbers"), "123");
        assert_eq!(format_panel_label("symbols"), "#+=");
        assert_eq!(format_panel_label("main"), "ABC");
        assert_eq!(format_panel_label("qwerty"), "ABC");
        assert_eq!(format_panel_label("emoji"), "\u{1F600}");
        assert_eq!(format_panel_label("custom"), "Custom");
        assert_eq!(format_panel_label(""), "");
    }

    /// Test: Panel reference with pixel sizing
    #[test]
    fn test_panel_ref_with_pixel_sizing() {
        let panel_ref = PanelRef {
            panel_id: "symbols".to_string(),
            width: Sizing::Pixels("100px".to_string()),
            height: Sizing::Pixels("50px".to_string()),
        };

        let base_unit = 80.0;
        let scale = 2.0;

        // This should not panic
        let _element = render_panel_ref_button(&panel_ref, base_unit, scale);
    }

    /// Test: Panel reference emits correct message type
    #[test]
    fn test_panel_ref_message_type() {
        // The message emitted should contain the panel_id
        let msg = RendererMessage::SwitchPanel("numpad".to_string());
        match msg {
            RendererMessage::SwitchPanel(id) => {
                assert_eq!(id, "numpad");
            }
            _ => panic!("Expected SwitchPanel message"),
        }
    }
}
