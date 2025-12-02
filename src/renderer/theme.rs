// SPDX-License-Identifier: GPL-3.0-only

//! Theme integration for the keyboard layout renderer.
//!
//! This module provides functions for retrieving COSMIC theme colors for
//! keyboard rendering. It integrates with the COSMIC theming system to ensure
//! the keyboard matches the user's desktop theme.
//!
//! # Color Functions
//!
//! Each function takes a reference to the COSMIC theme and returns the
//! appropriate color for a specific keyboard element:
//!
//! - `key_background_color`: Default key background
//! - `key_pressed_color`: Key background when pressed
//! - `key_text_color`: Text color for key labels
//! - `sticky_active_color`: Background for active sticky keys (Shift, Ctrl, etc.)
//! - `toast_background_color`: Background for toast notifications

use cosmic::iced::Color;
use cosmic::Theme;

// ============================================================================
// Public API
// ============================================================================

/// Returns the background color for a normal (unpressed) key.
///
/// Uses the theme's component background color, which provides good contrast
/// with the overall keyboard background while maintaining visual consistency.
///
/// # Arguments
///
/// * `theme` - Reference to the current COSMIC theme
///
/// # Returns
///
/// The background color for unpressed keys.
pub fn key_background_color(theme: &Theme) -> Color {
    let cosmic = theme.cosmic();

    // Use the component background (button-like) for keys
    // This provides a raised appearance that distinguishes keys from the background
    let bg = cosmic.bg_component_color();

    Color::from(bg)
}

/// Returns the background color for a pressed key.
///
/// Uses the theme's accent color to provide clear visual feedback when a key
/// is being pressed. This creates an instant, noticeable change.
///
/// # Arguments
///
/// * `theme` - Reference to the current COSMIC theme
///
/// # Returns
///
/// The accent color for pressed key state.
pub fn key_pressed_color(theme: &Theme) -> Color {
    let cosmic = theme.cosmic();

    // Use accent color for pressed state
    let accent = cosmic.accent_color();

    Color::from(accent)
}

/// Returns the text color for key labels.
///
/// Uses the theme's text color appropriate for component backgrounds,
/// ensuring good contrast and readability.
///
/// # Arguments
///
/// * `theme` - Reference to the current COSMIC theme
///
/// # Returns
///
/// The text color for key labels.
pub fn key_text_color(theme: &Theme) -> Color {
    let cosmic = theme.cosmic();

    // Use the on_bg_component color for text on component backgrounds
    let text = cosmic.on_bg_component_color();

    Color::from(text)
}

/// Returns the background color for an active sticky key.
///
/// Uses a variant of the accent color to distinguish sticky keys (like Shift
/// or Ctrl) that are currently active. This helps users track modifier state.
///
/// # Arguments
///
/// * `theme` - Reference to the current COSMIC theme
///
/// # Returns
///
/// The background color for active sticky keys.
pub fn sticky_active_color(theme: &Theme) -> Color {
    let cosmic = theme.cosmic();

    // Use a modified accent color for sticky active state
    // We use the success color for a distinct but harmonious appearance
    // Alternatively, we could use accent.with_alpha(0.7) for a subtler effect
    let success = cosmic.success_color();

    Color::from(success)
}

/// Returns the background color for toast notifications.
///
/// Uses a semi-transparent version of the background color to create a
/// floating notification appearance without obscuring too much content.
///
/// # Arguments
///
/// * `theme` - Reference to the current COSMIC theme
///
/// # Returns
///
/// The semi-transparent background color for toasts.
pub fn toast_background_color(theme: &Theme) -> Color {
    let cosmic = theme.cosmic();

    // Use the background component color with some transparency
    let bg = cosmic.bg_component_color();

    // Apply slight transparency for the toast overlay effect
    Color::from(bg).scale_alpha(0.95)
}

/// Returns the text color for toast notifications based on severity.
///
/// Provides appropriate text colors for different toast severity levels:
/// - Info: Standard text color
/// - Warning: Warning/caution color
/// - Error: Error/destructive color
///
/// # Arguments
///
/// * `theme` - Reference to the current COSMIC theme
/// * `severity` - The toast severity level
///
/// # Returns
///
/// The appropriate text color for the toast severity.
pub fn toast_text_color(theme: &Theme, severity: crate::renderer::ToastSeverity) -> Color {
    let cosmic = theme.cosmic();

    match severity {
        crate::renderer::ToastSeverity::Info => {
            // Standard text color for informational toasts
            Color::from(cosmic.on_bg_component_color())
        }
        crate::renderer::ToastSeverity::Warning => {
            // Warning color (typically yellow/orange)
            Color::from(cosmic.warning_color())
        }
        crate::renderer::ToastSeverity::Error => {
            // Error color (typically red)
            Color::from(cosmic.destructive_color())
        }
    }
}

/// Returns the border color for pressed keys.
///
/// Provides a subtle border effect for pressed keys using the accent color.
///
/// # Arguments
///
/// * `theme` - Reference to the current COSMIC theme
///
/// # Returns
///
/// The border color for pressed keys.
pub fn key_pressed_border_color(theme: &Theme) -> Color {
    let cosmic = theme.cosmic();

    // Use a slightly darker accent for the border
    let accent = cosmic.accent_color();
    Color::from(accent).scale_alpha(0.8)
}

/// Returns the keyboard surface background color.
///
/// Uses the theme's primary background color for the overall keyboard
/// surface background.
///
/// # Arguments
///
/// * `theme` - Reference to the current COSMIC theme
///
/// # Returns
///
/// The background color for the keyboard surface.
pub fn keyboard_background_color(theme: &Theme) -> Color {
    let cosmic = theme.cosmic();

    // Use the primary background color
    Color::from(cosmic.bg_color())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Theme color functions return valid colors
    ///
    /// Verifies that all theme color functions return colors with valid
    /// RGBA components (0.0 to 1.0 range).
    #[test]
    fn test_theme_colors_are_valid() {
        // Get the default dark theme
        let theme = Theme::dark();

        // Helper to validate color components are in valid range
        fn validate_color(color: Color, name: &str) {
            assert!(
                color.r >= 0.0 && color.r <= 1.0,
                "{} red component out of range: {}",
                name,
                color.r
            );
            assert!(
                color.g >= 0.0 && color.g <= 1.0,
                "{} green component out of range: {}",
                name,
                color.g
            );
            assert!(
                color.b >= 0.0 && color.b <= 1.0,
                "{} blue component out of range: {}",
                name,
                color.b
            );
            assert!(
                color.a >= 0.0 && color.a <= 1.0,
                "{} alpha component out of range: {}",
                name,
                color.a
            );
        }

        // Test all color functions
        validate_color(key_background_color(&theme), "key_background");
        validate_color(key_pressed_color(&theme), "key_pressed");
        validate_color(key_text_color(&theme), "key_text");
        validate_color(sticky_active_color(&theme), "sticky_active");
        validate_color(toast_background_color(&theme), "toast_background");
        validate_color(keyboard_background_color(&theme), "keyboard_background");
        validate_color(key_pressed_border_color(&theme), "key_pressed_border");
    }

    /// Test: Key background and pressed colors are different
    ///
    /// Verifies that the pressed color is visually distinct from the normal
    /// key background color.
    #[test]
    fn test_key_states_have_different_colors() {
        let theme = Theme::dark();

        let normal = key_background_color(&theme);
        let pressed = key_pressed_color(&theme);

        // Colors should be different (at least one component must differ)
        let different = (normal.r - pressed.r).abs() > 0.01
            || (normal.g - pressed.g).abs() > 0.01
            || (normal.b - pressed.b).abs() > 0.01;

        assert!(
            different,
            "Pressed color should be different from normal background"
        );
    }

    /// Test: Toast severity colors are distinct
    ///
    /// Verifies that different toast severity levels have distinct colors.
    #[test]
    fn test_toast_severity_colors_are_distinct() {
        let theme = Theme::dark();

        let info_color = toast_text_color(&theme, crate::renderer::ToastSeverity::Info);
        let warning_color = toast_text_color(&theme, crate::renderer::ToastSeverity::Warning);
        let error_color = toast_text_color(&theme, crate::renderer::ToastSeverity::Error);

        // Warning and error should have distinct colors from info
        let warning_diff = (info_color.r - warning_color.r).abs()
            + (info_color.g - warning_color.g).abs()
            + (info_color.b - warning_color.b).abs();

        let error_diff = (info_color.r - error_color.r).abs()
            + (info_color.g - error_color.g).abs()
            + (info_color.b - error_color.b).abs();

        // At least warning or error should be different from info
        // (some themes might have similar info and warning colors)
        assert!(
            warning_diff > 0.01 || error_diff > 0.01,
            "Severity colors should be distinguishable"
        );
    }

    /// Test: Toast background has transparency
    ///
    /// Verifies that the toast background color has some transparency
    /// for the overlay effect.
    #[test]
    fn test_toast_background_has_transparency() {
        let theme = Theme::dark();

        let toast_bg = toast_background_color(&theme);

        // Alpha should be less than 1.0 (fully opaque) but greater than 0.0 (fully transparent)
        assert!(
            toast_bg.a < 1.0,
            "Toast background should have some transparency: alpha = {}",
            toast_bg.a
        );
        assert!(
            toast_bg.a > 0.5,
            "Toast background should not be too transparent: alpha = {}",
            toast_bg.a
        );
    }

    /// Test: Theme works with both light and dark themes
    ///
    /// Verifies that color functions work with both light and dark themes.
    #[test]
    fn test_theme_works_with_light_and_dark() {
        let dark_theme = Theme::dark();
        let light_theme = Theme::light();

        // All functions should work without panicking
        let _ = key_background_color(&dark_theme);
        let _ = key_background_color(&light_theme);

        let _ = key_pressed_color(&dark_theme);
        let _ = key_pressed_color(&light_theme);

        let _ = key_text_color(&dark_theme);
        let _ = key_text_color(&light_theme);

        let _ = sticky_active_color(&dark_theme);
        let _ = sticky_active_color(&light_theme);

        let _ = toast_background_color(&dark_theme);
        let _ = toast_background_color(&light_theme);

        // Light and dark themes should produce different background colors
        let dark_bg = key_background_color(&dark_theme);
        let light_bg = key_background_color(&light_theme);

        // At least the brightness should differ significantly
        let dark_brightness = (dark_bg.r + dark_bg.g + dark_bg.b) / 3.0;
        let light_brightness = (light_bg.r + light_bg.g + light_bg.b) / 3.0;

        assert!(
            (dark_brightness - light_brightness).abs() > 0.1,
            "Light and dark themes should have different brightness levels"
        );
    }
}
