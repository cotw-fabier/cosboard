// SPDX-License-Identifier: GPL-3.0-only

//! Input handling module for Cosboard keyboard.
//!
//! This module provides functionality for parsing keycodes from layout definitions,
//! managing modifier state, and emitting virtual keyboard events via Wayland.
//!
//! # Features
//!
//! - **Keycode parsing**: Parse keycodes from layout `code` field in multiple formats
//! - **Modifier state management**: Track active modifiers with one-shot, toggle, and hold modes
//! - **Virtual keyboard**: Emit key events via Wayland's `zwp_virtual_keyboard_v1` protocol
//!
//! # Keycode Formats
//!
//! The keycode parser supports three formats:
//!
//! 1. **Single characters**: `"a"`, `"1"`, `" "` (space)
//! 2. **XKB keysym names**: `"Shift_L"`, `"BackSpace"`, `"Return"`
//! 3. **Unicode codepoints**: `"U+2022"` (bullet), `"U+03C0"` (pi)
//!
//! # Example Usage
//!
//! ## Parsing Keycodes
//!
//! ```rust,ignore
//! use cosboard::input::{parse_keycode, ResolvedKeycode};
//! use cosboard::layout::KeyCode;
//!
//! // Parse a Unicode character
//! let code = KeyCode::Unicode('a');
//! if let Some(resolved) = parse_keycode(&code) {
//!     match resolved {
//!         ResolvedKeycode::Character(c) => println!("Character: {}", c),
//!         ResolvedKeycode::Keysym(name) => println!("Keysym: {}", name),
//!         ResolvedKeycode::UnicodeCodepoint(cp) => println!("Unicode: U+{:04X}", cp),
//!     }
//! }
//! ```
//!
//! ## Managing Modifier State
//!
//! ```rust,ignore
//! use cosboard::input::ModifierState;
//! use cosboard::layout::Modifier;
//!
//! let mut state = ModifierState::new();
//!
//! // Activate a modifier (one-shot by default)
//! state.activate(Modifier::Shift, true);
//!
//! // Check if modifier is active
//! if state.is_active(Modifier::Shift) {
//!     println!("Shift is active");
//! }
//!
//! // Clear one-shot modifiers after key press
//! state.clear_sticky();
//! ```
//!
//! ## Emitting Virtual Keyboard Events
//!
//! ```rust,ignore
//! use cosboard::input::VirtualKeyboard;
//!
//! let mut vk = VirtualKeyboard::new();
//! vk.initialize().expect("Failed to initialize");
//!
//! // Look up keycode and emit key press/release
//! if let Some(keycode) = vk.keysym_to_keycode("Return") {
//!     vk.press_key(keycode);
//!     vk.release_key(keycode);
//! }
//!
//! // For Unicode characters not in keymap, use fallback
//! vk.emit_unicode_codepoint(0x03C0); // pi symbol
//! ```

// Sub-modules
pub mod keycode;
pub mod modifier;
pub mod virtual_keyboard;

// Re-export public API
pub use keycode::{parse_keycode, ResolvedKeycode};
pub use modifier::ModifierState;
pub use virtual_keyboard::{keycodes, KeyEvent, KeyState, VirtualKeyboard};

// ============================================================================
// Module Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{KeyCode, Modifier};

    // ========================================================================
    // Task 1.1: Focused tests for input module public API (4-6 tests)
    // ========================================================================

    /// Test 1: Keycode parsing for single characters
    ///
    /// Tests that single character keycodes like "a", "1", " " are parsed
    /// correctly as Character variants.
    #[test]
    fn test_keycode_parsing_single_characters() {
        // Test lowercase letter
        let code = KeyCode::Unicode('a');
        let resolved = parse_keycode(&code);
        assert!(resolved.is_some(), "Should parse single character 'a'");
        assert_eq!(resolved.unwrap(), ResolvedKeycode::Character('a'));

        // Test uppercase letter
        let code = KeyCode::Unicode('Q');
        let resolved = parse_keycode(&code);
        assert!(resolved.is_some(), "Should parse single character 'Q'");
        assert_eq!(resolved.unwrap(), ResolvedKeycode::Character('Q'));

        // Test digit
        let code = KeyCode::Unicode('1');
        let resolved = parse_keycode(&code);
        assert!(resolved.is_some(), "Should parse single character '1'");
        assert_eq!(resolved.unwrap(), ResolvedKeycode::Character('1'));

        // Test space
        let code = KeyCode::Unicode(' ');
        let resolved = parse_keycode(&code);
        assert!(resolved.is_some(), "Should parse space character");
        assert_eq!(resolved.unwrap(), ResolvedKeycode::Character(' '));

        // Test special characters
        let code = KeyCode::Unicode('@');
        let resolved = parse_keycode(&code);
        assert!(resolved.is_some(), "Should parse special character '@'");
        assert_eq!(resolved.unwrap(), ResolvedKeycode::Character('@'));
    }

    /// Test 2: Keycode parsing for XKB keysym names
    ///
    /// Tests that keysym strings like "Shift_L", "BackSpace" are parsed
    /// correctly as Keysym variants.
    #[test]
    fn test_keycode_parsing_xkb_keysym_names() {
        // Test Shift_L
        let code = KeyCode::Keysym("Shift_L".to_string());
        let resolved = parse_keycode(&code);
        assert!(resolved.is_some(), "Should parse keysym 'Shift_L'");
        assert_eq!(
            resolved.unwrap(),
            ResolvedKeycode::Keysym("Shift_L".to_string())
        );

        // Test BackSpace
        let code = KeyCode::Keysym("BackSpace".to_string());
        let resolved = parse_keycode(&code);
        assert!(resolved.is_some(), "Should parse keysym 'BackSpace'");
        assert_eq!(
            resolved.unwrap(),
            ResolvedKeycode::Keysym("BackSpace".to_string())
        );

        // Test Return
        let code = KeyCode::Keysym("Return".to_string());
        let resolved = parse_keycode(&code);
        assert!(resolved.is_some(), "Should parse keysym 'Return'");
        assert_eq!(
            resolved.unwrap(),
            ResolvedKeycode::Keysym("Return".to_string())
        );

        // Test Tab
        let code = KeyCode::Keysym("Tab".to_string());
        let resolved = parse_keycode(&code);
        assert!(resolved.is_some(), "Should parse keysym 'Tab'");
        assert_eq!(
            resolved.unwrap(),
            ResolvedKeycode::Keysym("Tab".to_string())
        );

        // Test Control_L
        let code = KeyCode::Keysym("Control_L".to_string());
        let resolved = parse_keycode(&code);
        assert!(resolved.is_some(), "Should parse keysym 'Control_L'");
        assert_eq!(
            resolved.unwrap(),
            ResolvedKeycode::Keysym("Control_L".to_string())
        );
    }

    /// Test 3: Keycode parsing for Unicode codepoints
    ///
    /// Tests that Unicode codepoint strings like "U+2022", "U+03C0" are parsed
    /// correctly as UnicodeCodepoint variants.
    #[test]
    fn test_keycode_parsing_unicode_codepoints() {
        // Test bullet point U+2022
        let code = KeyCode::Keysym("U+2022".to_string());
        let resolved = parse_keycode(&code);
        assert!(resolved.is_some(), "Should parse Unicode codepoint 'U+2022'");
        assert_eq!(resolved.unwrap(), ResolvedKeycode::UnicodeCodepoint(0x2022));

        // Test pi U+03C0
        let code = KeyCode::Keysym("U+03C0".to_string());
        let resolved = parse_keycode(&code);
        assert!(resolved.is_some(), "Should parse Unicode codepoint 'U+03C0'");
        assert_eq!(resolved.unwrap(), ResolvedKeycode::UnicodeCodepoint(0x03C0));

        // Test lowercase hex U+00e9 (e with acute accent)
        let code = KeyCode::Keysym("U+00e9".to_string());
        let resolved = parse_keycode(&code);
        assert!(
            resolved.is_some(),
            "Should parse lowercase Unicode codepoint 'U+00e9'"
        );
        assert_eq!(resolved.unwrap(), ResolvedKeycode::UnicodeCodepoint(0x00E9));

        // Test mixed case U+1F600 (emoji)
        let code = KeyCode::Keysym("U+1F600".to_string());
        let resolved = parse_keycode(&code);
        assert!(resolved.is_some(), "Should parse Unicode codepoint 'U+1F600'");
        assert_eq!(
            resolved.unwrap(),
            ResolvedKeycode::UnicodeCodepoint(0x1F600)
        );
    }

    /// Test 4: Graceful failure for unknown formats
    ///
    /// Tests that unknown or malformed keycodes return None instead of panicking.
    #[test]
    fn test_keycode_parsing_graceful_failure() {
        // Test empty keysym string
        let code = KeyCode::Keysym(String::new());
        let resolved = parse_keycode(&code);
        assert!(resolved.is_none(), "Empty keysym should return None");

        // Test invalid Unicode codepoint format (missing hex digits)
        let code = KeyCode::Keysym("U+".to_string());
        let resolved = parse_keycode(&code);
        assert!(
            resolved.is_none(),
            "Invalid Unicode format 'U+' should return None"
        );

        // Test invalid Unicode codepoint format (invalid hex)
        let code = KeyCode::Keysym("U+ZZZZ".to_string());
        let resolved = parse_keycode(&code);
        assert!(
            resolved.is_none(),
            "Invalid Unicode format 'U+ZZZZ' should return None"
        );

        // Test out-of-range Unicode codepoint
        let code = KeyCode::Keysym("U+FFFFFF".to_string());
        let resolved = parse_keycode(&code);
        assert!(
            resolved.is_none(),
            "Out-of-range Unicode codepoint should return None"
        );
    }

    /// Test 5: Modifier state management - activate, deactivate, toggle, is_active
    ///
    /// Tests that modifier state can be properly managed with all basic operations.
    #[test]
    fn test_modifier_state_management() {
        let mut state = ModifierState::new();

        // Initially all modifiers should be inactive
        assert!(!state.is_active(Modifier::Shift), "Shift should be inactive initially");
        assert!(!state.is_active(Modifier::Ctrl), "Ctrl should be inactive initially");
        assert!(!state.is_active(Modifier::Alt), "Alt should be inactive initially");
        assert!(!state.is_active(Modifier::Super), "Super should be inactive initially");

        // Activate Shift (one-shot)
        state.activate(Modifier::Shift, true);
        assert!(state.is_active(Modifier::Shift), "Shift should be active after activate");

        // Activate Ctrl (toggle mode - not sticky)
        state.activate(Modifier::Ctrl, false);
        assert!(state.is_active(Modifier::Ctrl), "Ctrl should be active after activate");

        // Deactivate Shift
        state.deactivate(Modifier::Shift);
        assert!(!state.is_active(Modifier::Shift), "Shift should be inactive after deactivate");
        assert!(state.is_active(Modifier::Ctrl), "Ctrl should still be active");

        // Toggle Alt (should activate)
        state.toggle(Modifier::Alt, true);
        assert!(state.is_active(Modifier::Alt), "Alt should be active after toggle");

        // Toggle Alt again (should deactivate)
        state.toggle(Modifier::Alt, true);
        assert!(!state.is_active(Modifier::Alt), "Alt should be inactive after second toggle");

        // Get active modifiers
        state.activate(Modifier::Shift, true);
        let active = state.get_active_modifiers();
        assert!(active.contains(&Modifier::Shift), "Active list should contain Shift");
        assert!(active.contains(&Modifier::Ctrl), "Active list should contain Ctrl");
        assert_eq!(active.len(), 2, "Should have exactly 2 active modifiers");
    }

    /// Test 6: Modifier state clear_sticky - clearing one-shot modifiers
    ///
    /// Tests that clear_sticky only clears modifiers that were activated with stickyrelease=true.
    #[test]
    fn test_modifier_state_clear_sticky() {
        let mut state = ModifierState::new();

        // Activate Shift as one-shot (stickyrelease=true)
        state.activate(Modifier::Shift, true);

        // Activate Ctrl as toggle (stickyrelease=false)
        state.activate(Modifier::Ctrl, false);

        // Activate Alt as one-shot
        state.activate(Modifier::Alt, true);

        // All should be active
        assert!(state.is_active(Modifier::Shift), "Shift should be active");
        assert!(state.is_active(Modifier::Ctrl), "Ctrl should be active");
        assert!(state.is_active(Modifier::Alt), "Alt should be active");

        // Clear sticky modifiers
        state.clear_sticky();

        // One-shot modifiers should be cleared, toggle modifiers should remain
        assert!(
            !state.is_active(Modifier::Shift),
            "Shift should be inactive after clear_sticky"
        );
        assert!(
            state.is_active(Modifier::Ctrl),
            "Ctrl should still be active (toggle mode)"
        );
        assert!(
            !state.is_active(Modifier::Alt),
            "Alt should be inactive after clear_sticky"
        );
    }
}
