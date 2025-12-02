// SPDX-License-Identifier: GPL-3.0-only

//! Sizing calculations for the keyboard layout renderer.
//!
//! This module provides functions for calculating pixel dimensions from layout
//! sizing specifications, supporting both relative sizing (multiples of a base unit)
//! and pixel sizing with HDPI scaling.
//!
//! # Base Unit Calculation
//!
//! The base unit is calculated from the keyboard surface dimensions divided by
//! the maximum number of cells in any row. This ensures keys scale proportionally
//! with the keyboard size.
//!
//! # Sizing Modes
//!
//! - **Relative sizing**: `Sizing::Relative(1.0)` equals one base unit.
//!   `Sizing::Relative(1.5)` equals 1.5 base units (e.g., for wider keys like Shift).
//!
//! - **Pixel sizing**: `Sizing::Pixels("20px")` specifies an exact pixel value.
//!   This value is multiplied by the HDPI scale factor for proper display on
//!   high-resolution screens.

use crate::layout::Sizing;

// ============================================================================
// Public API
// ============================================================================

/// Calculates the base unit size from surface dimensions.
///
/// The base unit is determined by taking the minimum of:
/// - Surface width divided by the widest row's cell count
/// - Surface height divided by the total height units of all rows
///
/// This ensures all rows fit within the available space while maintaining
/// proper aspect ratios.
///
/// # Arguments
///
/// * `surface_width` - Width of the keyboard surface in pixels
/// * `surface_height` - Height of the keyboard surface in pixels
/// * `max_row_width` - Maximum width units in any row (sum of relative widths in widest row)
/// * `total_height_units` - Sum of all row height multipliers
///
/// # Returns
///
/// The base unit size in pixels. Returns a minimum of 1.0 to avoid division by zero.
///
/// # Example
///
/// ```rust,ignore
/// // 4 rows of height 1.0 each = 4.0 total height units
/// let base_unit = calculate_base_unit(800.0, 300.0, 10, 4.0);
/// // width_unit = 800/10 = 80, height_unit = 300/4 = 75
/// // Returns min(80, 75) = 75
/// ```
pub fn calculate_base_unit(
    surface_width: f32,
    surface_height: f32,
    max_row_width: usize,
    total_height_units: f32,
) -> f32 {
    // Avoid division by zero
    if max_row_width == 0 || surface_width <= 0.0 || total_height_units <= 0.0 || surface_height <= 0.0 {
        return 1.0;
    }

    // Calculate base unit from width (horizontal constraint)
    let width_based_unit = surface_width / max_row_width as f32;

    // Calculate base unit from height (vertical constraint)
    let height_based_unit = surface_height / total_height_units;

    // Use the smaller of the two to ensure everything fits
    width_based_unit.min(height_based_unit).max(1.0)
}

/// Calculates the total height units from a slice of rows.
///
/// For each row, finds the maximum height among all cells (keys, widgets, panel refs),
/// then sums these row heights to get the total. This accounts for variable-height
/// rows (e.g., a row with a double-height key).
///
/// # Arguments
///
/// * `rows` - Slice of rows to calculate height for
///
/// # Returns
///
/// Total height units as a sum of max heights per row. Returns 1.0 minimum for empty rows.
///
/// # Example
///
/// ```rust,ignore
/// // 4 standard rows with height 1.0 each = 4.0
/// // 3 rows plus one double-height row = 5.0
/// ```
pub fn calculate_total_height_units(rows: &[crate::layout::Row]) -> f32 {
    use crate::layout::Cell;

    if rows.is_empty() {
        return 1.0;
    }

    rows.iter()
        .map(|row| {
            // Find the maximum height in this row
            row.cells
                .iter()
                .map(|cell| match cell {
                    Cell::Key(key) => key.height.as_relative(),
                    Cell::Widget(widget) => widget.height.as_relative(),
                    Cell::PanelRef(panel_ref) => panel_ref.height.as_relative(),
                })
                .fold(1.0_f32, |max, h| max.max(h))
        })
        .sum()
}

/// Resolves a sizing specification to a pixel value.
///
/// Handles both relative sizing (multiples of base unit) and pixel sizing
/// (with HDPI scaling applied).
///
/// # Arguments
///
/// * `sizing` - The sizing specification from the layout
/// * `base_unit` - The calculated base unit size in pixels
/// * `scale_factor` - HDPI scaling factor (1.0 for standard DPI, 2.0 for Retina, etc.)
///
/// # Returns
///
/// The resolved size in logical pixels. The result is always at least 1.0.
///
/// # Example
///
/// ```rust,ignore
/// use cosboard::layout::Sizing;
///
/// // Relative sizing: 1.5x base unit of 80px = 120px
/// let size = resolve_sizing(&Sizing::Relative(1.5), 80.0, 1.0);
/// assert_eq!(size, 120.0);
///
/// // Pixel sizing: 20px at 2x scale = 40px
/// let size = resolve_sizing(&Sizing::Pixels("20px".to_string()), 80.0, 2.0);
/// assert_eq!(size, 40.0);
/// ```
pub fn resolve_sizing(sizing: &Sizing, base_unit: f32, scale_factor: f32) -> f32 {
    let result = match sizing {
        Sizing::Relative(multiplier) => {
            // Relative sizing: multiply base unit by the multiplier
            base_unit * multiplier
        }
        Sizing::Pixels(pixel_str) => {
            // Pixel sizing: parse the pixel value and apply scale factor
            if let Some(pixels) = parse_pixels(pixel_str) {
                pixels * scale_factor
            } else {
                // Fallback to base unit if parsing fails
                base_unit
            }
        }
    };

    // Ensure minimum size of 1.0
    result.max(1.0)
}

/// Parses a pixel string (e.g., "20px") to extract the numeric value.
///
/// The function expects strings in the format "Npx" where N is a positive
/// number (integer or decimal). The "px" suffix is optional but recommended
/// for clarity. Whitespace is tolerated around the value.
///
/// # Arguments
///
/// * `pixel_str` - The pixel string to parse (e.g., "20px", "15.5px", "30")
///
/// # Returns
///
/// `Some(f32)` containing the parsed pixel value, or `None` if parsing fails.
///
/// # Example
///
/// ```rust,ignore
/// assert_eq!(parse_pixels("20px"), Some(20.0));
/// assert_eq!(parse_pixels("15.5px"), Some(15.5));
/// assert_eq!(parse_pixels("30"), Some(30.0));
/// assert_eq!(parse_pixels("invalid"), None);
/// ```
pub fn parse_pixels(pixel_str: &str) -> Option<f32> {
    // Trim whitespace
    let trimmed = pixel_str.trim();

    // Remove "px" suffix if present (case-insensitive)
    let number_part = if trimmed.to_lowercase().ends_with("px") {
        &trimmed[..trimmed.len() - 2]
    } else {
        trimmed
    };

    // Trim any remaining whitespace after removing suffix
    let number_part = number_part.trim();

    // Parse as f32
    number_part.parse::<f32>().ok().filter(|&v| v >= 0.0)
}

/// Retrieves the current HDPI scale factor.
///
/// This function attempts to get the scale factor from the COSMIC/Iced
/// environment. If unavailable, it returns a fallback of 1.0.
///
/// # Returns
///
/// The scale factor as a positive f32 (e.g., 1.0, 1.5, 2.0).
///
/// # Note
///
/// This is a utility function. In actual rendering, you may want to
/// obtain the scale factor from the window or surface context directly.
pub fn get_scale_factor() -> f32 {
    // In a real COSMIC application, we would use:
    // cosmic::app::cosmic::scale_factor() or similar API
    //
    // However, this function may not be available in all contexts,
    // so we provide a fallback mechanism.
    //
    // The actual scale factor should typically be obtained from:
    // 1. The window's scale_factor() method
    // 2. The COSMIC/Iced runtime
    // 3. Environment variables (GDK_SCALE, QT_SCALE_FACTOR)
    //
    // For now, we return 1.0 as the fallback.
    // When integrated into the applet, this can be replaced with
    // the actual scale factor from the rendering context.

    1.0
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Task 2.1: Focused tests for sizing calculations (2-6 tests)
    // ========================================================================

    /// Test 1: Relative sizing calculation (1.0 = base unit)
    ///
    /// Verifies that a relative sizing of 1.0 returns exactly the base unit.
    #[test]
    fn test_relative_sizing_base_unit() {
        let base_unit = 80.0;
        let scale_factor = 1.0;

        // Sizing::Relative(1.0) should equal exactly one base unit
        let sizing = Sizing::Relative(1.0);
        let result = resolve_sizing(&sizing, base_unit, scale_factor);

        assert!(
            (result - base_unit).abs() < f32::EPSILON,
            "Relative(1.0) should equal base unit: expected {}, got {}",
            base_unit,
            result
        );

        // Test with different base units
        let base_unit_small = 50.0;
        let result_small = resolve_sizing(&sizing, base_unit_small, scale_factor);
        assert!(
            (result_small - base_unit_small).abs() < f32::EPSILON,
            "Relative(1.0) should equal base unit: expected {}, got {}",
            base_unit_small,
            result_small
        );
    }

    /// Test 2: Relative sizing with multipliers (1.5x, 2.0x)
    ///
    /// Verifies that relative sizing correctly multiplies the base unit.
    #[test]
    fn test_relative_sizing_with_multipliers() {
        let base_unit = 80.0;
        let scale_factor = 1.0;

        // Test 1.5x multiplier (e.g., Shift key)
        let sizing_1_5x = Sizing::Relative(1.5);
        let result_1_5x = resolve_sizing(&sizing_1_5x, base_unit, scale_factor);
        let expected_1_5x = 80.0 * 1.5; // 120.0
        assert!(
            (result_1_5x - expected_1_5x).abs() < f32::EPSILON,
            "Relative(1.5) should equal 1.5x base unit: expected {}, got {}",
            expected_1_5x,
            result_1_5x
        );

        // Test 2.0x multiplier (e.g., Space bar)
        let sizing_2x = Sizing::Relative(2.0);
        let result_2x = resolve_sizing(&sizing_2x, base_unit, scale_factor);
        let expected_2x = 80.0 * 2.0; // 160.0
        assert!(
            (result_2x - expected_2x).abs() < f32::EPSILON,
            "Relative(2.0) should equal 2.0x base unit: expected {}, got {}",
            expected_2x,
            result_2x
        );

        // Test 0.5x multiplier (smaller key)
        let sizing_0_5x = Sizing::Relative(0.5);
        let result_0_5x = resolve_sizing(&sizing_0_5x, base_unit, scale_factor);
        let expected_0_5x = 80.0 * 0.5; // 40.0
        assert!(
            (result_0_5x - expected_0_5x).abs() < f32::EPSILON,
            "Relative(0.5) should equal 0.5x base unit: expected {}, got {}",
            expected_0_5x,
            result_0_5x
        );
    }

    /// Test 3: Pixel sizing parsing ("20px" format)
    ///
    /// Verifies that pixel strings are correctly parsed.
    #[test]
    fn test_pixel_sizing_parsing() {
        // Standard "Npx" format
        assert_eq!(parse_pixels("20px"), Some(20.0));
        assert_eq!(parse_pixels("100px"), Some(100.0));
        assert_eq!(parse_pixels("0px"), Some(0.0));

        // Decimal values
        assert_eq!(parse_pixels("15.5px"), Some(15.5));
        assert_eq!(parse_pixels("33.33px"), Some(33.33));

        // Without "px" suffix (should still work)
        assert_eq!(parse_pixels("25"), Some(25.0));
        assert_eq!(parse_pixels("10.5"), Some(10.5));

        // With whitespace (permissive parsing)
        assert_eq!(parse_pixels("  20px  "), Some(20.0));
        assert_eq!(parse_pixels("20 px"), Some(20.0)); // Whitespace before suffix is tolerated

        // Case insensitive suffix
        assert_eq!(parse_pixels("20PX"), Some(20.0));
        assert_eq!(parse_pixels("20Px"), Some(20.0));

        // Invalid inputs
        assert_eq!(parse_pixels("invalid"), None);
        assert_eq!(parse_pixels("px20"), None);
        assert_eq!(parse_pixels(""), None);
        assert_eq!(parse_pixels("-5px"), None); // Negative values not allowed
    }

    /// Test 4: Pixel sizing with HDPI scaling (1x, 1.5x, 2x factors)
    ///
    /// Verifies that pixel values are correctly scaled for HDPI displays.
    #[test]
    fn test_pixel_sizing_with_hdpi_scaling() {
        let base_unit = 80.0; // Base unit is not used for pixel sizing

        // 20px at 1x scale = 20px
        let sizing = Sizing::Pixels("20px".to_string());
        let result_1x = resolve_sizing(&sizing, base_unit, 1.0);
        assert!(
            (result_1x - 20.0).abs() < f32::EPSILON,
            "20px at 1x scale should be 20.0: got {}",
            result_1x
        );

        // 20px at 1.5x scale = 30px
        let result_1_5x = resolve_sizing(&sizing, base_unit, 1.5);
        assert!(
            (result_1_5x - 30.0).abs() < f32::EPSILON,
            "20px at 1.5x scale should be 30.0: got {}",
            result_1_5x
        );

        // 20px at 2x scale = 40px (Retina)
        let result_2x = resolve_sizing(&sizing, base_unit, 2.0);
        assert!(
            (result_2x - 40.0).abs() < f32::EPSILON,
            "20px at 2x scale should be 40.0: got {}",
            result_2x
        );

        // 50px at 2.5x scale = 125px
        let sizing_50 = Sizing::Pixels("50px".to_string());
        let result_2_5x = resolve_sizing(&sizing_50, base_unit, 2.5);
        assert!(
            (result_2_5x - 125.0).abs() < f32::EPSILON,
            "50px at 2.5x scale should be 125.0: got {}",
            result_2_5x
        );
    }

    /// Test 5: Base unit calculation from surface dimensions
    ///
    /// Verifies that the base unit is correctly calculated from surface
    /// dimensions, row width, and total height units.
    #[test]
    fn test_base_unit_calculation_from_surface_dimensions() {
        // Width-constrained: 800px/10 = 80, 400px/4 = 100 -> returns 80 (width limited)
        let base_unit_1 = calculate_base_unit(800.0, 400.0, 10, 4.0);
        assert!(
            (base_unit_1 - 80.0).abs() < f32::EPSILON,
            "Width-limited case should return 80.0: got {}",
            base_unit_1
        );

        // Height-constrained: 1200px/10 = 120, 300px/4 = 75 -> returns 75 (height limited)
        let base_unit_2 = calculate_base_unit(1200.0, 300.0, 10, 4.0);
        assert!(
            (base_unit_2 - 75.0).abs() < f32::EPSILON,
            "Height-limited case should return 75.0: got {}",
            base_unit_2
        );

        // Equal constraints: 800px/10 = 80, 320px/4 = 80 -> returns 80
        let base_unit_3 = calculate_base_unit(800.0, 320.0, 10, 4.0);
        assert!(
            (base_unit_3 - 80.0).abs() < f32::EPSILON,
            "Equal constraints should return 80.0: got {}",
            base_unit_3
        );

        // Variable height rows: 800px/10 = 80, 300px/5 = 60 (5 height units) -> returns 60
        let base_unit_4 = calculate_base_unit(800.0, 300.0, 10, 5.0);
        assert!(
            (base_unit_4 - 60.0).abs() < f32::EPSILON,
            "5 height units case should return 60.0: got {}",
            base_unit_4
        );
    }

    /// Test 6: Edge cases and minimum values
    ///
    /// Verifies proper handling of edge cases like zero cells, zero width,
    /// and ensures minimum values are enforced.
    #[test]
    fn test_edge_cases_and_minimum_values() {
        // Zero cell count should return minimum (1.0)
        let base_unit_zero_cells = calculate_base_unit(800.0, 300.0, 0, 4.0);
        assert!(
            (base_unit_zero_cells - 1.0).abs() < f32::EPSILON,
            "Zero cells should return minimum 1.0: got {}",
            base_unit_zero_cells
        );

        // Zero width should return minimum (1.0)
        let base_unit_zero_width = calculate_base_unit(0.0, 300.0, 10, 4.0);
        assert!(
            (base_unit_zero_width - 1.0).abs() < f32::EPSILON,
            "Zero width should return minimum 1.0: got {}",
            base_unit_zero_width
        );

        // Zero height units should return minimum (1.0)
        let base_unit_zero_height = calculate_base_unit(800.0, 300.0, 10, 0.0);
        assert!(
            (base_unit_zero_height - 1.0).abs() < f32::EPSILON,
            "Zero height units should return minimum 1.0: got {}",
            base_unit_zero_height
        );

        // Negative width should return minimum (1.0)
        let base_unit_negative = calculate_base_unit(-100.0, 300.0, 10, 4.0);
        assert!(
            (base_unit_negative - 1.0).abs() < f32::EPSILON,
            "Negative width should return minimum 1.0: got {}",
            base_unit_negative
        );

        // Very small relative sizing should return minimum (1.0)
        let sizing_tiny = Sizing::Relative(0.001);
        let result = resolve_sizing(&sizing_tiny, 100.0, 1.0);
        assert!(
            result >= 1.0,
            "Minimum size should be at least 1.0: got {}",
            result
        );

        // Invalid pixel string should fallback to base unit
        let sizing_invalid = Sizing::Pixels("invalid".to_string());
        let result_invalid = resolve_sizing(&sizing_invalid, 80.0, 1.0);
        assert!(
            (result_invalid - 80.0).abs() < f32::EPSILON,
            "Invalid pixel string should fallback to base unit: got {}",
            result_invalid
        );
    }
}
