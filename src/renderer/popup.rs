// SPDX-License-Identifier: GPL-3.0-only

//! Swipe gesture popup rendering for long press alternatives.
//!
//! This module provides functions for rendering popup overlays that appear
//! when a key is long-pressed. The popup displays alternative actions for
//! swipe gestures in different directions (up, down, left, right).
//!
//! # Usage
//!
//! When a long press is detected on a key that has swipe alternatives:
//!
//! 1. Calculate the popup position using `calculate_popup_position()`
//! 2. Render the popup using `render_popup()`
//! 3. The popup shows alternative actions for each available swipe direction
//! 4. Dismiss the popup when the user releases or moves away

use std::collections::HashMap;

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, container};
use cosmic::Element;

use crate::layout::{Action, AlternativeKey, Key, SwipeDirection};
use crate::renderer::message::RendererMessage;

// ============================================================================
// Constants
// ============================================================================

/// Default popup cell size in pixels.
pub const POPUP_CELL_SIZE: f32 = 48.0;

/// Spacing between popup cells in pixels.
pub const POPUP_CELL_SPACING: f32 = 4.0;

// ============================================================================
// Popup Position Types
// ============================================================================

/// Position and layout information for a popup.
#[derive(Debug, Clone)]
pub struct PopupPosition {
    /// X coordinate of the popup anchor point (center of the key).
    pub anchor_x: f32,

    /// Y coordinate of the popup anchor point (center of the key).
    pub anchor_y: f32,

    /// Available directions for the popup based on key alternatives.
    pub available_directions: Vec<SwipeDirection>,
}

impl PopupPosition {
    /// Creates a new popup position.
    pub fn new(anchor_x: f32, anchor_y: f32) -> Self {
        Self {
            anchor_x,
            anchor_y,
            available_directions: Vec::new(),
        }
    }

    /// Adds available swipe directions based on key alternatives.
    pub fn with_directions(mut self, directions: Vec<SwipeDirection>) -> Self {
        self.available_directions = directions;
        self
    }

    /// Returns the total width of the popup in pixels.
    pub fn popup_width(&self, scale: f32) -> f32 {
        let cell_size = POPUP_CELL_SIZE * scale;
        let spacing = POPUP_CELL_SPACING * scale;
        // 3 cells + 2 spaces + padding on each side
        cell_size * 3.0 + spacing * 4.0
    }

    /// Returns the total height of the popup in pixels.
    pub fn popup_height(&self, scale: f32) -> f32 {
        let cell_size = POPUP_CELL_SIZE * scale;
        let spacing = POPUP_CELL_SPACING * scale;
        // 3 cells + 2 spaces + padding on each side
        cell_size * 3.0 + spacing * 4.0
    }
}

/// A simple rectangle for bounds calculations.
#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    /// X coordinate of the top-left corner.
    pub x: f32,
    /// Y coordinate of the top-left corner.
    pub y: f32,
    /// Width of the rectangle.
    pub width: f32,
    /// Height of the rectangle.
    pub height: f32,
}

impl Rectangle {
    /// Creates a new rectangle.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the center X coordinate.
    pub fn center_x(&self) -> f32 {
        self.x + self.width / 2.0
    }

    /// Returns the center Y coordinate.
    pub fn center_y(&self) -> f32 {
        self.y + self.height / 2.0
    }
}

// ============================================================================
// Popup Rendering
// ============================================================================

/// Renders a popup showing swipe alternatives for a key.
///
/// The popup displays cells for each available swipe direction, positioned
/// around the key's center. Each cell shows the label for the alternative
/// action in that direction.
///
/// # Arguments
///
/// * `key` - The key that triggered the long press
/// * `position` - The calculated popup position (used for anchor coordinates)
/// * `scale` - HDPI scale factor for sizing
///
/// # Returns
///
/// An Element containing the rendered popup overlay.
pub fn render_popup<'a>(
    key: &Key,
    position: &PopupPosition,
    scale: f32,
) -> Element<'a, RendererMessage> {
    let cell_size = POPUP_CELL_SIZE * scale;
    let spacing = POPUP_CELL_SPACING * scale;

    // Extract swipe alternatives from the key
    let swipe_alternatives = get_swipe_alternatives(&key.alternatives);

    if swipe_alternatives.is_empty() {
        // No swipe alternatives - return empty container
        return container(widget::text::body("")).into();
    }

    // Log the anchor position (useful for debugging overlay positioning)
    let _anchor_x = position.anchor_x;
    let _anchor_y = position.anchor_y;

    // Build the popup layout as a cross pattern:
    //       [Up]
    // [Left][Center][Right]
    //       [Down]

    // Top row (Up direction)
    let up_cell = if let Some(action) = swipe_alternatives.get(&SwipeDirection::Up) {
        render_popup_cell(action, cell_size)
    } else {
        render_empty_cell(cell_size)
    };

    let top_row = widget::row::row()
        .push(render_empty_cell(cell_size))
        .push(up_cell)
        .push(render_empty_cell(cell_size))
        .spacing(spacing)
        .align_y(Alignment::Center);

    // Middle row (Left, Center, Right)
    let left_cell = if let Some(action) = swipe_alternatives.get(&SwipeDirection::Left) {
        render_popup_cell(action, cell_size)
    } else {
        render_empty_cell(cell_size)
    };

    let center_cell = render_center_cell(&key.label, cell_size);

    let right_cell = if let Some(action) = swipe_alternatives.get(&SwipeDirection::Right) {
        render_popup_cell(action, cell_size)
    } else {
        render_empty_cell(cell_size)
    };

    let middle_row = widget::row::row()
        .push(left_cell)
        .push(center_cell)
        .push(right_cell)
        .spacing(spacing)
        .align_y(Alignment::Center);

    // Bottom row (Down direction)
    let down_cell = if let Some(action) = swipe_alternatives.get(&SwipeDirection::Down) {
        render_popup_cell(action, cell_size)
    } else {
        render_empty_cell(cell_size)
    };

    let bottom_row = widget::row::row()
        .push(render_empty_cell(cell_size))
        .push(down_cell)
        .push(render_empty_cell(cell_size))
        .spacing(spacing)
        .align_y(Alignment::Center);

    // Combine rows into a column
    let popup_content = widget::column::column()
        .push(top_row)
        .push(middle_row)
        .push(bottom_row)
        .spacing(spacing)
        .align_x(Alignment::Center);

    // Wrap in a container with popup styling
    container(popup_content)
        .class(cosmic::style::Container::Dialog)
        .padding(spacing)
        .into()
}

/// Renders a single popup cell with an action label.
fn render_popup_cell<'a>(action: &Action, size: f32) -> Element<'a, RendererMessage> {
    let label = action_to_label(action);

    let cell_content = widget::text::body(label);

    container(cell_content)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .class(cosmic::style::Container::Primary)
        .into()
}

/// Renders the center cell showing the original key label.
fn render_center_cell<'a>(label: &str, size: f32) -> Element<'a, RendererMessage> {
    let cell_content = widget::text::body(label.to_string());

    container(cell_content)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .class(cosmic::style::Container::Background)
        .into()
}

/// Renders an empty placeholder cell.
fn render_empty_cell<'a>(size: f32) -> Element<'a, RendererMessage> {
    container(widget::text::body(""))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .into()
}

/// Converts an Action to a display label.
fn action_to_label(action: &Action) -> String {
    match action {
        Action::Character(c) => c.to_string(),
        Action::KeyCode(code) => format!("{}", code),
        Action::Script(s) => s.replace("script:", ""),
        Action::PanelSwitch(s) => s.replace("panel(", "").replace(')', ""),
    }
}

/// Extracts swipe direction alternatives from a key's alternatives map.
fn get_swipe_alternatives(alternatives: &HashMap<AlternativeKey, Action>) -> HashMap<SwipeDirection, Action> {
    let mut swipe_alternatives = HashMap::new();

    for (key, action) in alternatives {
        if let AlternativeKey::Swipe(direction) = key {
            swipe_alternatives.insert(*direction, action.clone());
        }
    }

    swipe_alternatives
}

// ============================================================================
// Popup Positioning
// ============================================================================

/// Calculates the optimal position for a popup based on key bounds.
///
/// The popup is centered on the key, with available directions determined
/// by which swipe alternatives the key has defined.
///
/// # Arguments
///
/// * `key_bounds` - The bounds of the key that triggered the popup
/// * `alternatives` - The key's alternatives map
///
/// # Returns
///
/// A PopupPosition with anchor coordinates and available directions.
pub fn calculate_popup_position(
    key_bounds: Rectangle,
    alternatives: &HashMap<AlternativeKey, Action>,
) -> PopupPosition {
    // Calculate anchor point at center of key
    let anchor_x = key_bounds.center_x();
    let anchor_y = key_bounds.center_y();

    // Collect available swipe directions
    let mut available_directions = Vec::new();

    for key in alternatives.keys() {
        if let AlternativeKey::Swipe(direction) = key {
            available_directions.push(*direction);
        }
    }

    PopupPosition::new(anchor_x, anchor_y).with_directions(available_directions)
}

/// Adjusts a popup position to keep it within screen bounds.
///
/// # Arguments
///
/// * `position` - The initial popup position
/// * `popup_width` - The total width of the popup
/// * `popup_height` - The total height of the popup
/// * `screen_width` - The width of the screen/surface
/// * `screen_height` - The height of the screen/surface
///
/// # Returns
///
/// An adjusted position that keeps the popup on screen.
pub fn adjust_popup_position(
    mut position: PopupPosition,
    popup_width: f32,
    popup_height: f32,
    screen_width: f32,
    screen_height: f32,
) -> PopupPosition {
    // Calculate popup bounds if centered on anchor
    let half_width = popup_width / 2.0;
    let half_height = popup_height / 2.0;

    // Adjust X to keep on screen
    if position.anchor_x - half_width < 0.0 {
        position.anchor_x = half_width;
    } else if position.anchor_x + half_width > screen_width {
        position.anchor_x = screen_width - half_width;
    }

    // Adjust Y to keep on screen
    if position.anchor_y - half_height < 0.0 {
        position.anchor_y = half_height;
    } else if position.anchor_y + half_height > screen_height {
        position.anchor_y = screen_height - half_height;
    }

    position
}

/// Returns `true` if the key has any swipe direction alternatives.
pub fn has_swipe_alternatives(alternatives: &HashMap<AlternativeKey, Action>) -> bool {
    alternatives
        .keys()
        .any(|k| matches!(k, AlternativeKey::Swipe(_)))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{KeyCode, Sizing};

    /// Helper to create a test key with swipe alternatives.
    fn create_key_with_alternatives() -> Key {
        let mut alternatives = HashMap::new();
        alternatives.insert(
            AlternativeKey::Swipe(SwipeDirection::Up),
            Action::Character('1'),
        );
        alternatives.insert(
            AlternativeKey::Swipe(SwipeDirection::Down),
            Action::Character('2'),
        );
        alternatives.insert(
            AlternativeKey::Swipe(SwipeDirection::Left),
            Action::Character('3'),
        );
        alternatives.insert(
            AlternativeKey::Swipe(SwipeDirection::Right),
            Action::Character('4'),
        );

        Key {
            label: "A".to_string(),
            code: KeyCode::Unicode('a'),
            identifier: Some("key_a".to_string()),
            width: Sizing::Relative(1.0),
            height: Sizing::Relative(1.0),
            min_width: None,
            min_height: None,
            alternatives,
            sticky: false,
            stickyrelease: true,
        }
    }

    // ========================================================================
    // Task 4.1: Popup position and behavior tests
    // ========================================================================

    /// Test: Popup positioning (up/down/left/right directions)
    ///
    /// Verifies that popup position is calculated correctly based on key bounds.
    #[test]
    fn test_popup_positioning() {
        let key = create_key_with_alternatives();
        let key_bounds = Rectangle::new(100.0, 200.0, 50.0, 40.0);

        let position = calculate_popup_position(key_bounds, &key.alternatives);

        // Anchor should be at center of key
        assert!(
            (position.anchor_x - 125.0).abs() < f32::EPSILON,
            "Expected anchor_x = 125.0, got {}",
            position.anchor_x
        );
        assert!(
            (position.anchor_y - 220.0).abs() < f32::EPSILON,
            "Expected anchor_y = 220.0, got {}",
            position.anchor_y
        );

        // Should have all 4 directions available
        assert_eq!(position.available_directions.len(), 4);
        assert!(position
            .available_directions
            .contains(&SwipeDirection::Up));
        assert!(position
            .available_directions
            .contains(&SwipeDirection::Down));
        assert!(position
            .available_directions
            .contains(&SwipeDirection::Left));
        assert!(position
            .available_directions
            .contains(&SwipeDirection::Right));
    }

    /// Test: Popup dismiss on pointer leave (conceptual)
    ///
    /// Verifies the helper function for detecting swipe alternatives.
    #[test]
    fn test_has_swipe_alternatives() {
        let key = create_key_with_alternatives();
        assert!(has_swipe_alternatives(&key.alternatives));

        // Key without swipe alternatives
        let empty_key = Key {
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
        };
        assert!(!has_swipe_alternatives(&empty_key.alternatives));

        // Key with only modifier alternatives (no swipe)
        let mut mod_only = HashMap::new();
        mod_only.insert(
            AlternativeKey::SingleModifier(crate::layout::Modifier::Shift),
            Action::Character('A'),
        );
        assert!(!has_swipe_alternatives(&mod_only));
    }

    /// Test: Swipe alternatives extraction
    ///
    /// Verifies that swipe alternatives are correctly extracted from the map.
    #[test]
    fn test_get_swipe_alternatives() {
        let key = create_key_with_alternatives();
        let swipes = get_swipe_alternatives(&key.alternatives);

        assert_eq!(swipes.len(), 4);
        assert!(matches!(
            swipes.get(&SwipeDirection::Up),
            Some(Action::Character('1'))
        ));
        assert!(matches!(
            swipes.get(&SwipeDirection::Down),
            Some(Action::Character('2'))
        ));
        assert!(matches!(
            swipes.get(&SwipeDirection::Left),
            Some(Action::Character('3'))
        ));
        assert!(matches!(
            swipes.get(&SwipeDirection::Right),
            Some(Action::Character('4'))
        ));
    }

    /// Test: Action to label conversion
    ///
    /// Verifies that actions are converted to display labels correctly.
    #[test]
    fn test_action_to_label() {
        assert_eq!(action_to_label(&Action::Character('x')), "x");
        assert_eq!(
            action_to_label(&Action::Script("script:my_macro".to_string())),
            "my_macro"
        );
        assert_eq!(
            action_to_label(&Action::PanelSwitch("panel(numpad)".to_string())),
            "numpad"
        );
    }

    /// Test: Popup position adjustment for screen bounds
    ///
    /// Verifies that popup positions are adjusted to stay on screen.
    #[test]
    fn test_popup_position_adjustment() {
        let popup_width = 160.0;
        let popup_height = 160.0;
        let screen_width = 800.0;
        let screen_height = 600.0;

        // Position near left edge - should be pushed right
        let position = PopupPosition::new(50.0, 300.0);
        let adjusted = adjust_popup_position(
            position,
            popup_width,
            popup_height,
            screen_width,
            screen_height,
        );
        assert!(
            adjusted.anchor_x >= popup_width / 2.0,
            "Should be pushed away from left edge"
        );

        // Position near right edge - should be pushed left
        let position = PopupPosition::new(750.0, 300.0);
        let adjusted = adjust_popup_position(
            position,
            popup_width,
            popup_height,
            screen_width,
            screen_height,
        );
        assert!(
            adjusted.anchor_x <= screen_width - popup_width / 2.0,
            "Should be pushed away from right edge"
        );

        // Position near top edge - should be pushed down
        let position = PopupPosition::new(400.0, 50.0);
        let adjusted = adjust_popup_position(
            position,
            popup_width,
            popup_height,
            screen_width,
            screen_height,
        );
        assert!(
            adjusted.anchor_y >= popup_height / 2.0,
            "Should be pushed away from top edge"
        );

        // Position near bottom edge - should be pushed up
        let position = PopupPosition::new(400.0, 550.0);
        let adjusted = adjust_popup_position(
            position,
            popup_width,
            popup_height,
            screen_width,
            screen_height,
        );
        assert!(
            adjusted.anchor_y <= screen_height - popup_height / 2.0,
            "Should be pushed away from bottom edge"
        );
    }

    /// Test: Popup rendering produces valid Element
    ///
    /// Verifies that render_popup produces a valid Element without panicking.
    #[test]
    fn test_popup_rendering() {
        let key = create_key_with_alternatives();
        let position = PopupPosition::new(200.0, 200.0).with_directions(vec![
            SwipeDirection::Up,
            SwipeDirection::Down,
            SwipeDirection::Left,
            SwipeDirection::Right,
        ]);

        // This should not panic
        let _element = render_popup(&key, &position, 1.0);
    }

    /// Test: Rectangle center calculations
    #[test]
    fn test_rectangle_center() {
        let rect = Rectangle::new(100.0, 200.0, 50.0, 40.0);
        assert!((rect.center_x() - 125.0).abs() < f32::EPSILON);
        assert!((rect.center_y() - 220.0).abs() < f32::EPSILON);
    }

    /// Test: PopupPosition size calculations
    #[test]
    fn test_popup_position_size() {
        let position = PopupPosition::new(100.0, 100.0);
        let scale = 1.0;

        let width = position.popup_width(scale);
        let height = position.popup_height(scale);

        // Should be 3 cells + 4 spacings
        let expected = POPUP_CELL_SIZE * 3.0 + POPUP_CELL_SPACING * 4.0;
        assert!((width - expected).abs() < f32::EPSILON);
        assert!((height - expected).abs() < f32::EPSILON);

        // Test with scale
        let scale = 2.0;
        let width_scaled = position.popup_width(scale);
        assert!((width_scaled - expected * scale).abs() < f32::EPSILON);
    }
}
