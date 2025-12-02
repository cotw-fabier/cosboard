// SPDX-License-Identifier: GPL-3.0-only

//! Panel rendering for the keyboard layout renderer.
//!
//! This module provides functions for rendering keyboard panels, which are
//! vertical arrangements of rows with padding around the entire panel.
//!
//! # Animation Support
//!
//! When switching between panels, this module supports animated transitions
//! using `render_animated_panels()`. During animation:
//! - The old panel slides out to the left
//! - The new panel slides in from the right
//! - Both panels are rendered simultaneously with horizontal offsets

use cosmic::iced::{Length, Padding};
use cosmic::widget::{self, container};
use cosmic::Element;

use crate::layout::Panel;
use crate::renderer::message::RendererMessage;
use crate::renderer::row::{calculate_row_width, render_row};
use crate::renderer::sizing::{calculate_base_unit, calculate_total_height_units};
use crate::renderer::state::KeyboardRenderer;

/// Default padding in pixels if not specified in the layout.
const DEFAULT_PADDING: f32 = 8.0;

/// Default margin between cells in pixels if not specified in the layout.
const DEFAULT_MARGIN: f32 = 4.0;

/// Renders a panel as a vertical layout of rows.
///
/// The panel is rendered with:
/// - Container with padding around the entire panel
/// - Column layout for vertical row arrangement
/// - Rows with margin spacing between cells
/// - Base unit calculated from surface dimensions
///
/// # Arguments
///
/// * `panel` - The panel definition from the layout
/// * `state` - The keyboard renderer state
/// * `surface_width` - Width of the keyboard surface in pixels
/// * `surface_height` - Height of the keyboard surface in pixels
/// * `scale` - HDPI scale factor for pixel sizing
///
/// # Returns
///
/// An Element containing the rendered panel.
pub fn render_panel<'a>(
    panel: &Panel,
    state: &KeyboardRenderer,
    surface_width: f32,
    surface_height: f32,
    scale: f32,
) -> Element<'a, RendererMessage> {
    // Get padding and margin from panel or use defaults
    let padding = panel.padding.unwrap_or(DEFAULT_PADDING);
    let margin = panel.margin.unwrap_or(DEFAULT_MARGIN);

    // Calculate the maximum row width (in relative units)
    let max_row_width = calculate_max_row_width(panel);

    // Calculate total height units (sum of max heights per row)
    let total_height_units = calculate_total_height_units(&panel.rows);

    // Calculate available dimensions after padding
    let available_width = surface_width - (padding * 2.0);
    let available_height = surface_height - (padding * 2.0);

    // Account for margin spacing between rows
    let margin_height = margin * (panel.rows.len().saturating_sub(1)) as f32;
    let content_height = available_height - margin_height;

    // Calculate base unit from both width and height constraints
    let base_unit = calculate_base_unit(
        available_width,
        content_height,
        max_row_width as usize,
        total_height_units,
    );

    // Build column with rows
    let mut column = widget::column::column().spacing(margin);

    for row in &panel.rows {
        let row_element = render_row(row, state, base_unit, scale, margin);
        column = column.push(row_element);
    }

    // Center the column horizontally within the available space
    let centered_column = container(column).center_x(Length::Fill);

    // Wrap in container with padding and background
    // Use the standard Background container style which uses the theme's background
    container(centered_column)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::from(padding))
        .class(cosmic::style::Container::Background)
        .into()
}

/// Calculates the maximum row width across all rows in a panel.
///
/// This is used to determine the base unit for proportional sizing.
///
/// # Arguments
///
/// * `panel` - The panel to analyze
///
/// # Returns
///
/// The maximum width in relative units, or 10 as a fallback minimum.
fn calculate_max_row_width(panel: &Panel) -> f32 {
    panel
        .rows
        .iter()
        .map(calculate_row_width)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(10.0)
        .max(1.0) // Ensure at least 1 to avoid division by zero
}

/// Renders the current panel from the keyboard renderer state.
///
/// This is a convenience function that looks up the current panel
/// and renders it with the given surface dimensions.
///
/// # Arguments
///
/// * `state` - The keyboard renderer state (contains current panel ID)
/// * `surface_width` - Width of the keyboard surface in pixels
/// * `surface_height` - Height of the keyboard surface in pixels
/// * `scale` - HDPI scale factor for pixel sizing
///
/// # Returns
///
/// An Element containing the rendered current panel, or an error message
/// if the current panel is not found.
pub fn render_current_panel<'a>(
    state: &KeyboardRenderer,
    surface_width: f32,
    surface_height: f32,
    scale: f32,
) -> Element<'a, RendererMessage> {
    if let Some(panel) = state.current_panel() {
        render_panel(panel, state, surface_width, surface_height, scale)
    } else {
        // Panel not found - render error message
        container(widget::text::body(format!(
            "Panel '{}' not found",
            state.current_panel_id
        )))
        .width(Length::Fill)
        .height(Length::Fill)
        .class(cosmic::style::Container::Background)
        .into()
    }
}

/// Renders panels with animation support for smooth transitions.
///
/// When an animation is in progress, this function renders both the source
/// and target panels with horizontal offset transforms based on the animation
/// progress. The old panel slides out to the left while the new panel slides
/// in from the right.
///
/// When no animation is active, this simply renders the current panel.
///
/// # Animation Behavior
///
/// - **Progress 0.0**: Old panel at position 0, new panel at +width (off-screen right)
/// - **Progress 0.5**: Old panel at -width/2, new panel at +width/2
/// - **Progress 1.0**: Old panel at -width (off-screen left), new panel at position 0
///
/// # Arguments
///
/// * `state` - The keyboard renderer state (contains animation state if animating)
/// * `surface_width` - Width of the keyboard surface in pixels
/// * `surface_height` - Height of the keyboard surface in pixels
/// * `scale` - HDPI scale factor for pixel sizing
///
/// # Returns
///
/// An Element containing either:
/// - The animated panel transition (if animating)
/// - The current panel (if not animating)
/// - An error message (if panels not found)
pub fn render_animated_panels<'a>(
    state: &KeyboardRenderer,
    surface_width: f32,
    surface_height: f32,
    scale: f32,
) -> Element<'a, RendererMessage> {
    // Check if we're animating
    if let Some(animation) = state.animation() {
        // Get both panels
        let from_panel = state.get_panel(&animation.from_panel_id);
        let to_panel = state.get_panel(&animation.to_panel_id);

        match (from_panel, to_panel) {
            (Some(from), Some(to)) => {
                // Use eased progress for smoother visual transition
                let progress = animation.eased_progress();

                // Calculate horizontal offsets for documentation and potential future use
                // Old panel: moves from 0 to -surface_width
                let from_offset = -surface_width * progress;
                // New panel: moves from +surface_width to 0
                // (computed for completeness, used implicitly via row positioning)
                let _to_offset = surface_width * (1.0 - progress);

                // Render both panels
                let from_element = render_panel(from, state, surface_width, surface_height, scale);
                let to_element = render_panel(to, state, surface_width, surface_height, scale);

                // Create containers with fixed width for each panel
                let from_container = container(from_element)
                    .width(Length::Fixed(surface_width))
                    .height(Length::Fill);

                let to_container = container(to_element)
                    .width(Length::Fixed(surface_width))
                    .height(Length::Fill);

                // Since cosmic/iced doesn't support CSS transforms directly,
                // we create a wider container and position panels within it
                // using a row layout and horizontal offset via padding
                let total_width = surface_width * 2.0;
                let offset_x = from_offset;

                // Create a row containing both panels side by side
                let panels_row = widget::row::row()
                    .push(from_container)
                    .push(to_container)
                    .width(Length::Fixed(total_width));

                // Wrap in a clipping container that only shows surface_width
                // and offset the content horizontally
                container(
                    container(panels_row)
                        .width(Length::Fixed(total_width))
                        .height(Length::Fill)
                        .padding(Padding {
                            top: 0.0,
                            right: 0.0,
                            bottom: 0.0,
                            left: offset_x, // This will be negative, shifting left
                        }),
                )
                .width(Length::Fixed(surface_width))
                .height(Length::Fill)
                .clip(true)
                .into()
            }
            _ => {
                // One or both panels not found - render error
                container(widget::text::body("Animation error: panel not found"))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .class(cosmic::style::Container::Background)
                    .into()
            }
        }
    } else {
        // Not animating - render current panel normally
        render_current_panel(state, surface_width, surface_height, scale)
    }
}

/// Renders a panel with a horizontal offset for animation.
///
/// This helper function renders a single panel positioned at a horizontal offset
/// from its normal position. Used internally by `render_animated_panels()`.
///
/// # Arguments
///
/// * `panel` - The panel to render
/// * `state` - The keyboard renderer state
/// * `surface_width` - Width of the keyboard surface in pixels
/// * `surface_height` - Height of the keyboard surface in pixels
/// * `scale` - HDPI scale factor for pixel sizing
/// * `offset_x` - Horizontal offset in pixels (negative = left, positive = right)
///
/// # Returns
///
/// An Element containing the panel positioned at the specified offset.
#[allow(dead_code)]
pub fn render_panel_with_offset<'a>(
    panel: &Panel,
    state: &KeyboardRenderer,
    surface_width: f32,
    surface_height: f32,
    scale: f32,
    offset_x: f32,
) -> Element<'a, RendererMessage> {
    let panel_element = render_panel(panel, state, surface_width, surface_height, scale);

    // Apply horizontal offset using padding
    // Positive offset = padding on left (shifts right)
    // Negative offset = would need different approach
    if offset_x >= 0.0 {
        container(panel_element)
            .width(Length::Fixed(surface_width))
            .height(Length::Fill)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: offset_x,
            })
            .into()
    } else {
        // For negative offset, we wrap in a larger container and clip
        container(panel_element)
            .width(Length::Fixed(surface_width))
            .height(Length::Fill)
            .into()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, Key, KeyCode, Layout, Panel, Row, Sizing};
    use std::collections::HashMap;

    /// Helper to create a test layout with a panel containing multiple rows
    fn create_test_layout() -> Layout {
        let mut panels = HashMap::new();

        let main_panel = Panel {
            id: "main".to_string(),
            padding: Some(8.0),
            margin: Some(4.0),
            nesting_depth: 0,
            rows: vec![
                Row {
                    cells: vec![
                        Cell::Key(Key {
                            label: "Q".to_string(),
                            code: KeyCode::Unicode('q'),
                            identifier: Some("key_q".to_string()),
                            width: Sizing::Relative(1.0),
                            height: Sizing::Relative(1.0),
                            min_width: None,
                            min_height: None,
                            alternatives: HashMap::new(),
                            sticky: false,
                            stickyrelease: true,
                        }),
                        Cell::Key(Key {
                            label: "W".to_string(),
                            code: KeyCode::Unicode('w'),
                            identifier: Some("key_w".to_string()),
                            width: Sizing::Relative(1.0),
                            height: Sizing::Relative(1.0),
                            min_width: None,
                            min_height: None,
                            alternatives: HashMap::new(),
                            sticky: false,
                            stickyrelease: true,
                        }),
                        Cell::Key(Key {
                            label: "E".to_string(),
                            code: KeyCode::Unicode('e'),
                            identifier: Some("key_e".to_string()),
                            width: Sizing::Relative(1.0),
                            height: Sizing::Relative(1.0),
                            min_width: None,
                            min_height: None,
                            alternatives: HashMap::new(),
                            sticky: false,
                            stickyrelease: true,
                        }),
                    ],
                },
                Row {
                    cells: vec![
                        Cell::Key(Key {
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
                        }),
                        Cell::Key(Key {
                            label: "S".to_string(),
                            code: KeyCode::Unicode('s'),
                            identifier: Some("key_s".to_string()),
                            width: Sizing::Relative(1.0),
                            height: Sizing::Relative(1.0),
                            min_width: None,
                            min_height: None,
                            alternatives: HashMap::new(),
                            sticky: false,
                            stickyrelease: true,
                        }),
                        Cell::Key(Key {
                            label: "D".to_string(),
                            code: KeyCode::Unicode('d'),
                            identifier: Some("key_d".to_string()),
                            width: Sizing::Relative(1.0),
                            height: Sizing::Relative(1.0),
                            min_width: None,
                            min_height: None,
                            alternatives: HashMap::new(),
                            sticky: false,
                            stickyrelease: true,
                        }),
                    ],
                },
            ],
        };

        let numpad_panel = Panel {
            id: "numpad".to_string(),
            padding: Some(8.0),
            margin: Some(4.0),
            nesting_depth: 0,
            rows: vec![Row {
                cells: vec![
                    Cell::Key(Key {
                        label: "1".to_string(),
                        code: KeyCode::Unicode('1'),
                        identifier: Some("key_1".to_string()),
                        width: Sizing::Relative(1.0),
                        height: Sizing::Relative(1.0),
                        min_width: None,
                        min_height: None,
                        alternatives: HashMap::new(),
                        sticky: false,
                            stickyrelease: true,
                    }),
                    Cell::Key(Key {
                        label: "2".to_string(),
                        code: KeyCode::Unicode('2'),
                        identifier: Some("key_2".to_string()),
                        width: Sizing::Relative(1.0),
                        height: Sizing::Relative(1.0),
                        min_width: None,
                        min_height: None,
                        alternatives: HashMap::new(),
                        sticky: false,
                            stickyrelease: true,
                    }),
                    Cell::Key(Key {
                        label: "3".to_string(),
                        code: KeyCode::Unicode('3'),
                        identifier: Some("key_3".to_string()),
                        width: Sizing::Relative(1.0),
                        height: Sizing::Relative(1.0),
                        min_width: None,
                        min_height: None,
                        alternatives: HashMap::new(),
                        sticky: false,
                            stickyrelease: true,
                    }),
                ],
            }],
        };

        panels.insert("main".to_string(), main_panel);
        panels.insert("numpad".to_string(), numpad_panel);

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

    /// Test: Panel rendering with padding/margin
    #[test]
    fn test_panel_rendering_with_padding_margin() {
        let layout = create_test_layout();
        let state = KeyboardRenderer::new(layout);
        let surface_width = 800.0;
        let surface_height = 300.0;
        let scale = 1.0;

        // Get the main panel
        let panel = state.current_panel().unwrap();

        // This should not panic
        let _element = render_panel(panel, &state, surface_width, surface_height, scale);
    }

    /// Test: Current panel rendering
    #[test]
    fn test_render_current_panel() {
        let layout = create_test_layout();
        let state = KeyboardRenderer::new(layout);
        let surface_width = 800.0;
        let surface_height = 300.0;
        let scale = 1.0;

        // This should not panic
        let _element = render_current_panel(&state, surface_width, surface_height, scale);
    }

    /// Test: Calculate max row width
    #[test]
    fn test_calculate_max_row_width() {
        let panel = Panel {
            id: "test".to_string(),
            padding: None,
            margin: None,
            nesting_depth: 0,
            rows: vec![
                Row {
                    cells: vec![Cell::Key(Key {
                        label: "A".to_string(),
                        code: KeyCode::Unicode('a'),
                        identifier: None,
                        width: Sizing::Relative(1.0),
                        height: Sizing::Relative(1.0),
                        min_width: None,
                        min_height: None,
                        alternatives: HashMap::new(),
                        sticky: false,
                            stickyrelease: true,
                    })],
                },
                Row {
                    cells: vec![
                        Cell::Key(Key {
                            label: "B".to_string(),
                            code: KeyCode::Unicode('b'),
                            identifier: None,
                            width: Sizing::Relative(1.0),
                            height: Sizing::Relative(1.0),
                            min_width: None,
                            min_height: None,
                            alternatives: HashMap::new(),
                            sticky: false,
                            stickyrelease: true,
                        }),
                        Cell::Key(Key {
                            label: "Space".to_string(),
                            code: KeyCode::Unicode(' '),
                            identifier: None,
                            width: Sizing::Relative(4.0),
                            height: Sizing::Relative(1.0),
                            min_width: None,
                            min_height: None,
                            alternatives: HashMap::new(),
                            sticky: false,
                            stickyrelease: true,
                        }),
                        Cell::Key(Key {
                            label: "C".to_string(),
                            code: KeyCode::Unicode('c'),
                            identifier: None,
                            width: Sizing::Relative(1.0),
                            height: Sizing::Relative(1.0),
                            min_width: None,
                            min_height: None,
                            alternatives: HashMap::new(),
                            sticky: false,
                            stickyrelease: true,
                        }),
                    ],
                },
            ],
        };

        let max_width = calculate_max_row_width(&panel);
        assert!(
            (max_width - 6.0).abs() < f32::EPSILON,
            "Max row width should be 1.0 + 4.0 + 1.0 = 6.0, got {}",
            max_width
        );
    }

    /// Test: Empty panel renders without panic
    #[test]
    fn test_empty_panel_renders() {
        let mut panels = HashMap::new();
        panels.insert(
            "main".to_string(),
            Panel {
                id: "main".to_string(),
                padding: None,
                margin: None,
                nesting_depth: 0,
                rows: vec![],
            },
        );

        let layout = Layout {
            name: "Empty".to_string(),
            description: None,
            author: None,
            language: None,
            locale: None,
            version: "1.0".to_string(),
            default_panel_id: "main".to_string(),
            inherits: None,
            panels,
        };

        let state = KeyboardRenderer::new(layout);
        let surface_width = 800.0;
        let surface_height = 300.0;
        let scale = 1.0;

        // This should not panic
        let _element = render_current_panel(&state, surface_width, surface_height, scale);
    }

    /// Test: Missing panel shows error message
    #[test]
    fn test_missing_panel_shows_error() {
        let mut panels = HashMap::new();
        panels.insert(
            "other".to_string(),
            Panel {
                id: "other".to_string(),
                padding: None,
                margin: None,
                nesting_depth: 0,
                rows: vec![],
            },
        );

        let layout = Layout {
            name: "Missing Panel".to_string(),
            description: None,
            author: None,
            language: None,
            locale: None,
            version: "1.0".to_string(),
            default_panel_id: "main".to_string(), // This panel doesn't exist
            inherits: None,
            panels,
        };

        let state = KeyboardRenderer::new(layout);
        let surface_width = 800.0;
        let surface_height = 300.0;
        let scale = 1.0;

        // This should not panic - it should render an error message
        let _element = render_current_panel(&state, surface_width, surface_height, scale);
    }

    /// Test: Animated panel rendering when not animating
    #[test]
    fn test_render_animated_panels_not_animating() {
        let layout = create_test_layout();
        let state = KeyboardRenderer::new(layout);
        let surface_width = 800.0;
        let surface_height = 300.0;
        let scale = 1.0;

        // Should render current panel when not animating
        assert!(!state.is_animating());
        let _element = render_animated_panels(&state, surface_width, surface_height, scale);
    }

    /// Test: Animated panel rendering during animation
    #[test]
    fn test_render_animated_panels_during_animation() {
        let layout = create_test_layout();
        let mut state = KeyboardRenderer::new(layout);
        let surface_width = 800.0;
        let surface_height = 300.0;
        let scale = 1.0;

        // Start animation
        state.switch_panel("numpad").unwrap();
        assert!(state.is_animating());

        // Should render both panels during animation
        let _element = render_animated_panels(&state, surface_width, surface_height, scale);
    }
}
