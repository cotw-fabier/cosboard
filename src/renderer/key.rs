// SPDX-License-Identifier: GPL-3.0-only

//! Key rendering for the keyboard layout renderer.
//!
//! This module provides functions for rendering individual keyboard keys
//! using libcosmic/Iced widgets. Keys are rendered as buttons with appropriate
//! sizing, styling, and label content.

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, container, icon};
use cosmic::Element;

use crate::layout::Key;
use crate::renderer::message::RendererMessage;
use crate::renderer::sizing::resolve_sizing;
use crate::renderer::state::KeyboardRenderer;

/// Icon names that should be rendered with system icons.
const ICON_NAMES: &[&str] = &[
    "backspace",
    "edit-delete-symbolic",
    "return",
    "enter",
    "go-next-symbolic",
    "shift",
    "shift-symbolic",
    "tab",
    "tab-symbolic",
    "caps",
    "capslock",
    "space",
    "spacebar",
    "arrow-up",
    "arrow-down",
    "arrow-left",
    "arrow-right",
    "go-up-symbolic",
    "go-down-symbolic",
    "go-previous-symbolic",
    "keyboard-symbolic",
    "input-keyboard-symbolic",
];

/// Renders a single key as an Element.
///
/// The key is rendered as a button with:
/// - Width and height calculated from sizing specifications
/// - Background color based on pressed/sticky state
/// - Centered label (text or icon)
///
/// # Arguments
///
/// * `key` - The key definition from the layout
/// * `state` - The keyboard renderer state (for pressed/sticky checks)
/// * `base_unit` - The calculated base unit for relative sizing
/// * `scale` - HDPI scale factor for pixel sizing
///
/// # Returns
///
/// An Element containing the rendered key button.
pub fn render_key<'a>(
    key: &Key,
    state: &KeyboardRenderer,
    base_unit: f32,
    scale: f32,
) -> Element<'a, RendererMessage> {
    let width = resolve_sizing(&key.width, base_unit, scale);
    let height = resolve_sizing(&key.height, base_unit, scale);

    // Determine the key identifier for state lookups
    let identifier = key
        .identifier
        .clone()
        .unwrap_or_else(|| key.label.clone());

    // Check if this key should show active modifier styling.
    // Uses the helper function to determine visual state based on:
    // - For sticky keys (sticky: true): Checks sticky_keys_active HashSet
    // - For hold keys (sticky: false): Uses native button pressed state (not tracked here)
    let is_sticky_active = should_show_modifier_active(key, state, &identifier);

    // Create the label content
    let label: Element<'a, RendererMessage> = render_label(&key.label);

    // Create styled button
    let id_for_message = identifier.clone();

    // Choose button style based on state
    // - Sticky keys that are active use accent/suggested color
    // - All other keys use standard styling (native pressed state handled by Iced button)
    let button_class = if is_sticky_active {
        cosmic::style::Button::Suggested // Use accent color for active sticky keys
    } else {
        cosmic::style::Button::Standard // Use standard button color for all other states
    };

    let btn = button::custom(
        container(label)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .on_press(RendererMessage::KeyPressed(id_for_message))
    .class(button_class)
    .width(Length::Fixed(width))
    .height(Length::Fixed(height));

    btn.into()
}

/// Determines if a key should display the active modifier visual state.
///
/// This function checks whether a modifier key should be visually highlighted
/// based on its current state. It handles all three modifier behaviors:
///
/// - **One-shot** (`sticky: true`, `stickyrelease: true`): Shows active styling
///   when the modifier is in the `sticky_keys_active` set.
/// - **Toggle** (`sticky: true`, `stickyrelease: false`): Shows active styling
///   when the modifier is in the `sticky_keys_active` set.
/// - **Hold** (`sticky: false`): Returns `false` here; the native button widget
///   provides visual feedback while the key is physically pressed/held.
///
/// # Arguments
///
/// * `key` - The key definition from the layout
/// * `state` - The keyboard renderer state
/// * `identifier` - The key identifier for state lookups
///
/// # Returns
///
/// `true` if the key should display active modifier styling, `false` otherwise.
///
/// # Note
///
/// For hold-mode modifiers, this function returns `false` because the button
/// widget's native pressed state provides visual feedback. The `sticky_active_color`
/// from the theme is only applied to sticky keys (one-shot and toggle modes).
#[must_use]
pub fn should_show_modifier_active(key: &Key, state: &KeyboardRenderer, identifier: &str) -> bool {
    // For sticky keys (one-shot or toggle mode), check the sticky_keys_active set.
    // This set is kept in sync with modifier_state via sync_modifier_visual_state().
    if key.sticky {
        return state.is_sticky_active(identifier);
    }

    // For non-sticky (hold) keys, we don't apply custom active styling here.
    // The button widget provides native pressed-state visual feedback while held.
    false
}

/// Renders a key label as either text or an icon.
///
/// The function detects icon names and renders them using `widget::icon::from_name()`.
/// Regular text labels are rendered using `widget::text::body()`.
/// Unicode symbols are rendered directly as text.
///
/// # Arguments
///
/// * `label` - The label string from the key definition
///
/// # Returns
///
/// An Element containing the rendered label.
pub fn render_label<'a>(label: &str) -> Element<'a, RendererMessage> {
    let label_lower = label.to_lowercase();

    // Check if this is a known icon name
    if is_icon_name(&label_lower) {
        // Map common key names to system icon names
        let icon_name = match label_lower.as_str() {
            "backspace" | "delete" => "edit-delete-symbolic",
            "return" | "enter" => "go-next-symbolic",
            "shift" => "keyboard-shift-symbolic",
            "tab" => "format-indent-more-symbolic",
            "caps" | "capslock" => "keyboard-caps-symbolic",
            "space" | "spacebar" => "keyboard-spacebar-symbolic",
            "arrow-up" | "up" => "go-up-symbolic",
            "arrow-down" | "down" => "go-down-symbolic",
            "arrow-left" | "left" => "go-previous-symbolic",
            "arrow-right" | "right" => "go-next-symbolic",
            other => other, // Use as-is if it looks like an icon name already
        };

        // Render as icon
        icon::from_name(icon_name)
            .size(16)
            .symbolic(true)
            .into()
    } else {
        // Render as text (includes Unicode symbols)
        // Use to_string() to take ownership of the label
        widget::text::body(label.to_string()).into()
    }
}

/// Checks if a label should be rendered as an icon.
///
/// # Arguments
///
/// * `label` - The label string (should be lowercase for comparison)
///
/// # Returns
///
/// `true` if the label should be rendered as an icon.
pub fn is_icon_name(label: &str) -> bool {
    // Check against known icon names
    for icon in ICON_NAMES {
        if label == *icon {
            return true;
        }
    }

    // Also check for common key names that map to icons
    matches!(
        label,
        "backspace"
            | "delete"
            | "return"
            | "enter"
            | "shift"
            | "tab"
            | "caps"
            | "capslock"
            | "space"
            | "spacebar"
            | "up"
            | "down"
            | "left"
            | "right"
    )
}

/// Calculates the effective identifier for a key.
///
/// Returns the explicit identifier if set, otherwise uses the label.
///
/// # Arguments
///
/// * `key` - The key definition
///
/// # Returns
///
/// The identifier string to use for state lookups.
pub fn key_identifier(key: &Key) -> String {
    key.identifier
        .clone()
        .unwrap_or_else(|| key.label.clone())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, KeyCode, Layout, Modifier, Panel, Row, Sizing};
    use std::collections::HashMap;

    /// Helper function to create a test layout with a simple key.
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
    // Task 3.1: Tests for key rendering (part of 2-6 focused tests)
    // ========================================================================

    /// Test: Single key rendering produces valid Element
    #[test]
    fn test_single_key_rendering_produces_element() {
        let layout = create_test_layout();
        let state = KeyboardRenderer::new(layout);
        let base_unit = 80.0;
        let scale = 1.0;

        let key = Key {
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
        };

        // This should not panic and should produce a valid Element
        let _element = render_key(&key, &state, base_unit, scale);

        // If we get here without panic, the test passes
        // Element type is opaque, so we can't inspect it directly
    }

    /// Test: Key label rendering (text vs icon detection)
    #[test]
    fn test_key_label_rendering_text_vs_icon() {
        // Test text labels
        assert!(!is_icon_name("a"), "Lowercase 'a' should be text");
        assert!(!is_icon_name("A"), "Uppercase 'A' should be text");
        assert!(!is_icon_name("123"), "Numbers should be text");
        assert!(!is_icon_name("@"), "Special chars should be text");

        // Test icon labels
        assert!(is_icon_name("backspace"), "backspace should be icon");
        assert!(is_icon_name("return"), "return should be icon");
        assert!(is_icon_name("enter"), "enter should be icon");
        assert!(is_icon_name("shift"), "shift should be icon");
        assert!(is_icon_name("tab"), "tab should be icon");
        assert!(is_icon_name("space"), "space should be icon");

        // Test Unicode symbols should be text (rendered directly)
        assert!(
            !is_icon_name("\u{2190}"),
            "Unicode arrow should be text, not icon"
        );
    }

    /// Test: Key identifier extraction
    #[test]
    fn test_key_identifier_extraction() {
        // Key with explicit identifier
        let key_with_id = Key {
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
        };
        assert_eq!(key_identifier(&key_with_id), "key_a");

        // Key without explicit identifier (should use label)
        let key_without_id = Key {
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
        assert_eq!(key_identifier(&key_without_id), "B");
    }

    /// Test: render_label produces Elements for both text and icon names
    #[test]
    fn test_render_label_produces_elements() {
        // These should not panic
        let _text_element: Element<'_, RendererMessage> = render_label("A");
        let _icon_element: Element<'_, RendererMessage> = render_label("backspace");
        let _unicode_element: Element<'_, RendererMessage> = render_label("\u{2190}"); // Left arrow
        let _number_element: Element<'_, RendererMessage> = render_label("1");

        // If we get here without panic, the test passes
    }

    // ========================================================================
    // Task Group 6: Visual Modifier State Indication Tests (2-3 focused tests)
    // ========================================================================

    /// Test 1: Active modifier key shows sticky_active styling
    ///
    /// Verifies that when a sticky modifier key is activated, the visual state
    /// function returns `true`, indicating the key should use `sticky_active_color`.
    #[test]
    fn test_active_modifier_shows_sticky_active_styling() {
        let layout = create_test_layout();
        let mut state = KeyboardRenderer::new(layout);

        // Create a sticky Shift key (one-shot mode: sticky=true, stickyrelease=true)
        let shift_key = Key {
            label: "Shift".to_string(),
            code: KeyCode::Keysym("Shift_L".to_string()),
            identifier: Some("shift".to_string()),
            width: Sizing::Relative(1.5),
            height: Sizing::Relative(1.0),
            min_width: None,
            min_height: None,
            alternatives: HashMap::new(),
            sticky: true, // Sticky mode enabled
            stickyrelease: true, // One-shot behavior
        };

        // Initially, the modifier should NOT show active styling
        assert!(
            !should_show_modifier_active(&shift_key, &state, "shift"),
            "Inactive modifier should not show active styling"
        );

        // Activate the Shift modifier and sync visual state
        state.activate_modifier(Modifier::Shift, true);
        state.sync_modifier_visual_state(Modifier::Shift, "shift");

        // Now the modifier SHOULD show active styling
        assert!(
            should_show_modifier_active(&shift_key, &state, "shift"),
            "Active modifier should show sticky_active styling"
        );

        // Verify is_sticky_active also returns true
        assert!(
            state.is_sticky_active("shift"),
            "sticky_keys_active should contain 'shift'"
        );
    }

    /// Test 2: Inactive modifier key shows normal styling
    ///
    /// Verifies that when a sticky modifier key is not active, it displays
    /// with normal (non-highlighted) styling.
    #[test]
    fn test_inactive_modifier_shows_normal_styling() {
        let layout = create_test_layout();
        let mut state = KeyboardRenderer::new(layout);

        // Create a sticky Ctrl key (toggle mode: sticky=true, stickyrelease=false)
        let ctrl_key = Key {
            label: "Ctrl".to_string(),
            code: KeyCode::Keysym("Control_L".to_string()),
            identifier: Some("ctrl".to_string()),
            width: Sizing::Relative(1.5),
            height: Sizing::Relative(1.0),
            min_width: None,
            min_height: None,
            alternatives: HashMap::new(),
            sticky: true, // Sticky mode enabled
            stickyrelease: false, // Toggle behavior
        };

        // Inactive modifier should show normal styling
        assert!(
            !should_show_modifier_active(&ctrl_key, &state, "ctrl"),
            "Inactive Ctrl should show normal styling"
        );

        // Non-sticky keys should always show normal styling (even if in sticky_keys_active)
        let regular_key = Key {
            label: "A".to_string(),
            code: KeyCode::Unicode('a'),
            identifier: Some("key_a".to_string()),
            width: Sizing::Relative(1.0),
            height: Sizing::Relative(1.0),
            min_width: None,
            min_height: None,
            alternatives: HashMap::new(),
            sticky: false, // Not a sticky key
            stickyrelease: true,
        };

        // Even if we somehow add "key_a" to sticky_keys_active, it should not show active
        // because the key itself is not marked as sticky
        state.sticky_keys_active.insert("key_a".to_string());
        assert!(
            !should_show_modifier_active(&regular_key, &state, "key_a"),
            "Non-sticky key should not show sticky active styling"
        );
    }

    /// Test 3: Visual state updates on modifier toggle
    ///
    /// Verifies that the visual state correctly updates when a modifier is
    /// toggled on and off. Tests the integration between modifier_state
    /// and sticky_keys_active for visual synchronization.
    #[test]
    fn test_visual_state_updates_on_modifier_toggle() {
        let layout = create_test_layout();
        let mut state = KeyboardRenderer::new(layout);

        // Create a sticky Alt key (toggle mode)
        let alt_key = Key {
            label: "Alt".to_string(),
            code: KeyCode::Keysym("Alt_L".to_string()),
            identifier: Some("alt".to_string()),
            width: Sizing::Relative(1.5),
            height: Sizing::Relative(1.0),
            min_width: None,
            min_height: None,
            alternatives: HashMap::new(),
            sticky: true,
            stickyrelease: false, // Toggle mode
        };

        // Step 1: Initially inactive
        assert!(
            !should_show_modifier_active(&alt_key, &state, "alt"),
            "Initial state: Alt should not show active styling"
        );
        assert!(!state.is_modifier_active(Modifier::Alt));

        // Step 2: Activate Alt (simulating user tap)
        state.activate_modifier(Modifier::Alt, false); // Toggle mode
        state.sync_modifier_visual_state(Modifier::Alt, "alt");

        assert!(
            should_show_modifier_active(&alt_key, &state, "alt"),
            "After activation: Alt should show active styling"
        );
        assert!(state.is_modifier_active(Modifier::Alt));
        assert!(state.is_sticky_active("alt"));

        // Step 3: Deactivate Alt (simulating second tap to toggle off)
        state.deactivate_modifier(Modifier::Alt);
        state.sync_modifier_visual_state(Modifier::Alt, "alt");

        assert!(
            !should_show_modifier_active(&alt_key, &state, "alt"),
            "After deactivation: Alt should not show active styling"
        );
        assert!(!state.is_modifier_active(Modifier::Alt));
        assert!(!state.is_sticky_active("alt"));

        // Step 4: Test one-shot modifier clears visual state
        // Activate Shift as one-shot
        state.activate_modifier(Modifier::Shift, true);
        state.sync_modifier_visual_state(Modifier::Shift, "shift");

        let shift_key = Key {
            label: "Shift".to_string(),
            code: KeyCode::Keysym("Shift_L".to_string()),
            identifier: Some("shift".to_string()),
            sticky: true,
            stickyrelease: true, // One-shot
            ..Key::default()
        };

        assert!(
            should_show_modifier_active(&shift_key, &state, "shift"),
            "One-shot Shift should show active styling"
        );

        // Clear one-shot modifiers (simulating combo key press)
        state.clear_oneshot_modifiers();

        assert!(
            !should_show_modifier_active(&shift_key, &state, "shift"),
            "After clear_oneshot: Shift should not show active styling"
        );
        assert!(!state.is_sticky_active("shift"));
    }
}
