// SPDX-License-Identifier: GPL-3.0-only

//! Keycode parsing for keyboard input emission.
//!
//! This module provides functionality for parsing keycodes from layout definitions
//! into resolved keycodes that can be used for input emission via the virtual
//! keyboard protocol.
//!
//! # Supported Formats
//!
//! The parser supports three keycode formats:
//!
//! 1. **Single characters** (from `KeyCode::Unicode`): `'a'`, `'1'`, `' '`
//! 2. **XKB keysym names** (from `KeyCode::Keysym`): `"Shift_L"`, `"BackSpace"`
//! 3. **Unicode codepoints** (from `KeyCode::Keysym`): `"U+2022"`, `"U+03C0"`
//!
//! # Format Detection
//!
//! The parser uses the following priority for `KeyCode::Keysym` values:
//!
//! 1. Check if string starts with `"U+"` for Unicode codepoint format
//! 2. Otherwise, treat as XKB keysym name

use crate::layout::KeyCode;

/// A resolved keycode ready for input emission.
///
/// This enum represents the different types of keycodes that can be
/// emitted through the virtual keyboard interface.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolvedKeycode {
    /// A single character that can be directly emitted.
    /// Examples: 'a', '1', ' ', '@'
    Character(char),

    /// An XKB keysym name for special keys and modifiers.
    /// Examples: "Shift_L", "BackSpace", "Return", "Tab"
    Keysym(String),

    /// A Unicode codepoint for characters not on standard keyboards.
    /// The codepoint is stored as a u32 value.
    /// Examples: 0x2022 (bullet), 0x03C0 (pi), 0x1F600 (emoji)
    UnicodeCodepoint(u32),
}

/// Parses a `KeyCode` from a layout definition into a `ResolvedKeycode`.
///
/// This function handles the three supported keycode formats and returns
/// `None` for unknown or malformed formats.
///
/// # Arguments
///
/// * `code` - The keycode from the layout definition
///
/// # Returns
///
/// * `Some(ResolvedKeycode)` if the keycode could be parsed successfully
/// * `None` if the format is unknown or malformed
///
/// # Examples
///
/// ```rust,ignore
/// use cosboard::input::{parse_keycode, ResolvedKeycode};
/// use cosboard::layout::KeyCode;
///
/// // Parse a character
/// let code = KeyCode::Unicode('a');
/// assert_eq!(parse_keycode(&code), Some(ResolvedKeycode::Character('a')));
///
/// // Parse a keysym
/// let code = KeyCode::Keysym("Shift_L".to_string());
/// assert_eq!(parse_keycode(&code), Some(ResolvedKeycode::Keysym("Shift_L".to_string())));
///
/// // Parse a Unicode codepoint
/// let code = KeyCode::Keysym("U+2022".to_string());
/// assert_eq!(parse_keycode(&code), Some(ResolvedKeycode::UnicodeCodepoint(0x2022)));
/// ```
pub fn parse_keycode(code: &KeyCode) -> Option<ResolvedKeycode> {
    match code {
        KeyCode::Unicode(c) => Some(ResolvedKeycode::Character(*c)),
        KeyCode::Keysym(s) => parse_keysym_string(s),
    }
}

/// Parses a keysym string, detecting the format automatically.
///
/// Format detection priority:
/// 1. Unicode codepoint: starts with "U+" followed by hex digits
/// 2. XKB keysym name: any other non-empty string
fn parse_keysym_string(s: &str) -> Option<ResolvedKeycode> {
    // Handle empty strings
    if s.is_empty() {
        return None;
    }

    // Check for Unicode codepoint format: U+XXXX
    if let Some(stripped) = s.strip_prefix("U+") {
        return parse_unicode_codepoint(stripped);
    }

    // Check for lowercase variant: u+XXXX
    if let Some(stripped) = s.strip_prefix("u+") {
        return parse_unicode_codepoint(stripped);
    }

    // Treat as XKB keysym name
    Some(ResolvedKeycode::Keysym(s.to_string()))
}

/// Parses a Unicode codepoint from a hex string.
///
/// Returns `None` if:
/// - The hex string is empty
/// - The hex string contains invalid characters
/// - The codepoint is out of the valid Unicode range (> 0x10FFFF)
/// - The codepoint is a surrogate (0xD800-0xDFFF)
fn parse_unicode_codepoint(hex_str: &str) -> Option<ResolvedKeycode> {
    // Handle empty hex string
    if hex_str.is_empty() {
        return None;
    }

    // Parse hex string
    let codepoint = u32::from_str_radix(hex_str, 16).ok()?;

    // Validate Unicode range (0x0000-0x10FFFF, excluding surrogates)
    if codepoint > 0x10FFFF {
        return None;
    }

    // Exclude surrogate range (0xD800-0xDFFF)
    if (0xD800..=0xDFFF).contains(&codepoint) {
        return None;
    }

    Some(ResolvedKeycode::UnicodeCodepoint(codepoint))
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Unicode character parsing
    #[test]
    fn test_parse_unicode_character() {
        let code = KeyCode::Unicode('x');
        let result = parse_keycode(&code);
        assert_eq!(result, Some(ResolvedKeycode::Character('x')));
    }

    /// Test keysym parsing
    #[test]
    fn test_parse_keysym() {
        let code = KeyCode::Keysym("Escape".to_string());
        let result = parse_keycode(&code);
        assert_eq!(
            result,
            Some(ResolvedKeycode::Keysym("Escape".to_string()))
        );
    }

    /// Test Unicode codepoint parsing with uppercase U+
    #[test]
    fn test_parse_unicode_codepoint_uppercase() {
        let code = KeyCode::Keysym("U+00A9".to_string()); // Copyright symbol
        let result = parse_keycode(&code);
        assert_eq!(result, Some(ResolvedKeycode::UnicodeCodepoint(0x00A9)));
    }

    /// Test Unicode codepoint parsing with lowercase u+
    #[test]
    fn test_parse_unicode_codepoint_lowercase() {
        let code = KeyCode::Keysym("u+00A9".to_string());
        let result = parse_keycode(&code);
        assert_eq!(result, Some(ResolvedKeycode::UnicodeCodepoint(0x00A9)));
    }

    /// Test surrogate range rejection
    #[test]
    fn test_reject_surrogate_range() {
        let code = KeyCode::Keysym("U+D800".to_string());
        let result = parse_keycode(&code);
        assert_eq!(result, None, "Should reject surrogate codepoints");
    }

    /// Test maximum valid codepoint
    #[test]
    fn test_max_valid_codepoint() {
        let code = KeyCode::Keysym("U+10FFFF".to_string());
        let result = parse_keycode(&code);
        assert_eq!(result, Some(ResolvedKeycode::UnicodeCodepoint(0x10FFFF)));
    }

    /// Test ResolvedKeycode Debug implementation
    #[test]
    fn test_resolved_keycode_debug() {
        let char_code = ResolvedKeycode::Character('a');
        let keysym_code = ResolvedKeycode::Keysym("Return".to_string());
        let unicode_code = ResolvedKeycode::UnicodeCodepoint(0x2022);

        // Just verify Debug works without panicking
        let _ = format!("{:?}", char_code);
        let _ = format!("{:?}", keysym_code);
        let _ = format!("{:?}", unicode_code);
    }
}
