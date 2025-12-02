// SPDX-License-Identifier: GPL-3.0-only

//! Row rendering for the keyboard layout renderer.
//!
//! This module provides functions for rendering keyboard rows, which are
//! horizontal arrangements of cells (keys, widgets, panel references).

use cosmic::widget;
use cosmic::Element;

use crate::layout::{Cell, Row};
use crate::renderer::key::render_key;
use crate::renderer::message::RendererMessage;
use crate::renderer::panel_ref::render_panel_ref_button;
use crate::renderer::state::KeyboardRenderer;
use crate::renderer::widget_placeholder::render_widget_placeholder;

/// Renders a row of cells as a horizontal layout.
///
/// Uses `cosmic::widget::row()` to arrange cells horizontally with
/// margin spacing between them.
///
/// # Arguments
///
/// * `row` - The row definition from the layout
/// * `state` - The keyboard renderer state
/// * `base_unit` - The calculated base unit for relative sizing
/// * `scale` - HDPI scale factor for pixel sizing
/// * `margin` - Spacing between cells in pixels
///
/// # Returns
///
/// An Element containing the rendered row.
pub fn render_row<'a>(
    row: &Row,
    state: &KeyboardRenderer,
    base_unit: f32,
    scale: f32,
    margin: f32,
) -> Element<'a, RendererMessage> {
    let mut row_widget = widget::row::row().spacing(margin);

    for cell in &row.cells {
        let cell_element = render_cell(cell, state, base_unit, scale);
        row_widget = row_widget.push(cell_element);
    }

    row_widget.into()
}

/// Renders a single cell based on its type.
///
/// Dispatches to the appropriate rendering function based on the cell type:
/// - `Cell::Key` -> `render_key()`
/// - `Cell::Widget` -> `render_widget_placeholder()`
/// - `Cell::PanelRef` -> `render_panel_ref_button()`
///
/// # Arguments
///
/// * `cell` - The cell to render
/// * `state` - The keyboard renderer state
/// * `base_unit` - The calculated base unit for relative sizing
/// * `scale` - HDPI scale factor for pixel sizing
///
/// # Returns
///
/// An Element containing the rendered cell.
pub fn render_cell<'a>(
    cell: &Cell,
    state: &KeyboardRenderer,
    base_unit: f32,
    scale: f32,
) -> Element<'a, RendererMessage> {
    match cell {
        Cell::Key(key) => render_key(key, state, base_unit, scale),
        Cell::Widget(widget) => render_widget_placeholder(widget, base_unit, scale),
        Cell::PanelRef(panel_ref) => render_panel_ref_button(panel_ref, base_unit, scale),
    }
}

/// Calculates the total width of a row in base units.
///
/// This is used to determine the maximum row width for base unit calculations.
///
/// # Arguments
///
/// * `row` - The row to measure
///
/// # Returns
///
/// The total width in relative units (sum of all cell widths).
pub fn calculate_row_width(row: &Row) -> f32 {
    row.cells.iter().map(|cell| cell_width(cell)).sum()
}

/// Gets the width of a cell in base units.
///
/// Returns the relative width multiplier for the cell.
/// For pixel-based sizing, returns 1.0 as the unit contribution.
///
/// # Arguments
///
/// * `cell` - The cell to measure
///
/// # Returns
///
/// The width multiplier for the cell.
fn cell_width(cell: &Cell) -> f32 {
    match cell {
        Cell::Key(key) => match &key.width {
            crate::layout::Sizing::Relative(w) => *w,
            crate::layout::Sizing::Pixels(_) => 1.0, // Treat as 1 unit for max calculation
        },
        Cell::Widget(widget) => match &widget.width {
            crate::layout::Sizing::Relative(w) => *w,
            crate::layout::Sizing::Pixels(_) => 1.0,
        },
        Cell::PanelRef(panel_ref) => match &panel_ref.width {
            crate::layout::Sizing::Relative(w) => *w,
            crate::layout::Sizing::Pixels(_) => 1.0,
        },
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, Key, KeyCode, Layout, Panel, PanelRef, Row, Sizing, Widget};
    use std::collections::HashMap;

    /// Helper to create a test layout
    fn create_test_layout() -> Layout {
        let mut panels = HashMap::new();

        let main_panel = Panel {
            id: "main".to_string(),
            padding: Some(5.0),
            margin: Some(2.0),
            nesting_depth: 0,
            rows: vec![],
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

    /// Test: Row rendering with multiple keys
    #[test]
    fn test_row_rendering_with_multiple_keys() {
        let layout = create_test_layout();
        let state = KeyboardRenderer::new(layout);
        let base_unit = 80.0;
        let scale = 1.0;
        let margin = 4.0;

        let row = Row {
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
                    label: "B".to_string(),
                    code: KeyCode::Unicode('b'),
                    identifier: Some("key_b".to_string()),
                    width: Sizing::Relative(1.0),
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
                    identifier: Some("key_c".to_string()),
                    width: Sizing::Relative(1.0),
                    height: Sizing::Relative(1.0),
                    min_width: None,
                    min_height: None,
                    alternatives: HashMap::new(),
                    sticky: false,
                    stickyrelease: true,
                }),
            ],
        };

        // This should not panic
        let _element = render_row(&row, &state, base_unit, scale, margin);
    }

    /// Test: Row with mixed cell types
    #[test]
    fn test_row_with_mixed_cell_types() {
        let layout = create_test_layout();
        let state = KeyboardRenderer::new(layout);
        let base_unit = 80.0;
        let scale = 1.0;
        let margin = 4.0;

        let row = Row {
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
                Cell::Widget(Widget {
                    widget_type: "trackpad".to_string(),
                    width: Sizing::Relative(2.0),
                    height: Sizing::Relative(2.0),
                }),
                Cell::PanelRef(PanelRef {
                    panel_id: "numpad".to_string(),
                    width: Sizing::Relative(1.0),
                    height: Sizing::Relative(1.0),
                }),
            ],
        };

        // This should not panic
        let _element = render_row(&row, &state, base_unit, scale, margin);
    }

    /// Test: Calculate row width
    #[test]
    fn test_calculate_row_width() {
        let row = Row {
            cells: vec![
                Cell::Key(Key {
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
                }),
                Cell::Key(Key {
                    label: "Shift".to_string(),
                    code: KeyCode::Keysym("Shift_L".to_string()),
                    identifier: None,
                    width: Sizing::Relative(1.5),
                    height: Sizing::Relative(1.0),
                    min_width: None,
                    min_height: None,
                    alternatives: HashMap::new(),
                    sticky: true,
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
            ],
        };

        let width = calculate_row_width(&row);
        assert!(
            (width - 6.5).abs() < f32::EPSILON,
            "Row width should be 1.0 + 1.5 + 4.0 = 6.5"
        );
    }

    /// Test: Empty row renders without panic
    #[test]
    fn test_empty_row_renders() {
        let layout = create_test_layout();
        let state = KeyboardRenderer::new(layout);
        let base_unit = 80.0;
        let scale = 1.0;
        let margin = 4.0;

        let row = Row { cells: vec![] };

        // This should not panic
        let _element = render_row(&row, &state, base_unit, scale, margin);
    }
}
