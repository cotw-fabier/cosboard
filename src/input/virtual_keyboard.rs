// SPDX-License-Identifier: GPL-3.0-only

//! Virtual keyboard protocol handling for Wayland input injection.
//!
//! This module provides functionality for emitting virtual keyboard events
//! through Wayland's `zwp_virtual_keyboard_v1` protocol. It handles:
//!
//! - Initialization with the system XKB keymap
//! - Key press and release event emission
//! - XKB keysym to hardware keycode conversion
//! - Unicode codepoint fallback via Ctrl+Shift+U hex input
//!
//! # Architecture
//!
//! The `VirtualKeyboard` struct wraps the Wayland virtual keyboard protocol
//! and provides a high-level API for emitting key events. Since libcosmic
//! manages the Wayland connection internally, this module uses a deferred
//! initialization pattern where the actual protocol binding happens when
//! the keyboard surface is created.
//!
//! # Unicode Fallback
//!
//! For characters that cannot be mapped to XKB keycodes (e.g., special
//! Unicode symbols), the module falls back to the Ctrl+Shift+U hex input
//! method standard in GTK/Linux applications:
//!
//! 1. Press Ctrl+Shift+U
//! 2. Type the hex codepoint (e.g., "03c0" for pi)
//! 3. Press Space or Enter to commit
//!
//! # Example
//!
//! ```rust,ignore
//! use cosboard::input::VirtualKeyboard;
//!
//! // Create virtual keyboard instance
//! let mut vk = VirtualKeyboard::new();
//!
//! // Emit a regular key press/release
//! if let Some(keycode) = vk.keysym_to_keycode("a") {
//!     vk.press_key(keycode);
//!     vk.release_key(keycode);
//! }
//!
//! // Emit a Unicode character via fallback
//! vk.emit_unicode_codepoint(0x03C0); // pi symbol
//! ```

use crate::input::ResolvedKeycode;
use xkbcommon::xkb::keysyms::KEY_NoSymbol;
use xkbcommon::xkb::Keysym;

/// Key event state for virtual keyboard protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    /// Key was pressed down.
    Pressed,
    /// Key was released.
    Released,
}

/// A key event to be emitted through the virtual keyboard protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    /// The hardware keycode (evdev keycode).
    pub keycode: u32,
    /// Whether the key was pressed or released.
    pub state: KeyState,
    /// Timestamp in milliseconds (usually from compositor).
    pub time: u32,
}

impl KeyEvent {
    /// Creates a new key press event.
    #[must_use]
    pub fn press(keycode: u32, time: u32) -> Self {
        Self {
            keycode,
            state: KeyState::Pressed,
            time,
        }
    }

    /// Creates a new key release event.
    #[must_use]
    pub fn release(keycode: u32, time: u32) -> Self {
        Self {
            keycode,
            state: KeyState::Released,
            time,
        }
    }
}

/// Virtual keyboard for emitting key events via Wayland protocol.
///
/// This struct provides the interface for emitting virtual keyboard events
/// through the `zwp_virtual_keyboard_v1` protocol. It maintains state for
/// the XKB context and keymap used for keysym-to-keycode conversion.
///
/// # Lifecycle
///
/// The virtual keyboard follows a specific lifecycle:
///
/// 1. **Creation**: `VirtualKeyboard::new()` creates an uninitialized instance
/// 2. **Initialization**: `initialize()` sets up the XKB context and keymap
/// 3. **Usage**: `press_key()`, `release_key()` emit key events
/// 4. **Cleanup**: Drop trait handles protocol object cleanup
///
/// # Thread Safety
///
/// This struct is NOT thread-safe. All operations should occur on the
/// main thread where the Wayland event loop runs.
pub struct VirtualKeyboard {
    /// Whether the virtual keyboard has been initialized.
    initialized: bool,

    /// Pending key events waiting to be flushed (for batching).
    pending_events: Vec<KeyEvent>,

    /// XKB context for keymap operations.
    /// This is only Some after successful initialization.
    xkb_context: Option<xkbcommon::xkb::Context>,

    /// XKB keymap loaded from the system.
    /// This is only Some after successful initialization.
    xkb_keymap: Option<xkbcommon::xkb::Keymap>,

    /// XKB state for key state tracking.
    /// This is only Some after successful initialization.
    xkb_state: Option<xkbcommon::xkb::State>,
}

impl std::fmt::Debug for VirtualKeyboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualKeyboard")
            .field("initialized", &self.initialized)
            .field("pending_events", &self.pending_events)
            .field("xkb_context", &self.xkb_context.is_some())
            .field("xkb_keymap", &self.xkb_keymap.is_some())
            .field("xkb_state", &self.xkb_state.is_some())
            .finish()
    }
}

impl Default for VirtualKeyboard {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualKeyboard {
    /// Creates a new virtual keyboard instance.
    ///
    /// The instance is created in an uninitialized state. Call `initialize()`
    /// to set up the XKB context and keymap before emitting key events.
    #[must_use]
    pub fn new() -> Self {
        Self {
            initialized: false,
            pending_events: Vec::new(),
            xkb_context: None,
            xkb_keymap: None,
            xkb_state: None,
        }
    }

    /// Initializes the virtual keyboard with the default system XKB keymap.
    ///
    /// This method sets up the XKB context and loads the default keymap from
    /// environment variables (XKBLAYOUT, XKBVARIANT, etc.) or system defaults.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if initialization succeeded
    /// * `Err(String)` with error description if initialization failed
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut vk = VirtualKeyboard::new();
    /// if let Err(e) = vk.initialize() {
    ///     tracing::error!("Failed to initialize virtual keyboard: {}", e);
    /// }
    /// ```
    pub fn initialize(&mut self) -> Result<(), String> {
        if self.initialized {
            return Ok(());
        }

        // Create XKB context
        let context = xkbcommon::xkb::Context::new(xkbcommon::xkb::CONTEXT_NO_FLAGS);

        // Load default keymap from environment/system
        let keymap = xkbcommon::xkb::Keymap::new_from_names(
            &context,
            &"", // rules (empty = default)
            &"", // model (empty = default)
            &"", // layout (empty = default from env)
            &"", // variant (empty = default)
            None, // options
            xkbcommon::xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .ok_or_else(|| "Failed to create XKB keymap from system defaults".to_string())?;

        // Create XKB state for tracking modifier state
        let state = xkbcommon::xkb::State::new(&keymap);

        self.xkb_context = Some(context);
        self.xkb_keymap = Some(keymap);
        self.xkb_state = Some(state);
        self.initialized = true;

        tracing::info!("Virtual keyboard initialized with system XKB keymap");
        Ok(())
    }

    /// Returns whether the virtual keyboard has been initialized.
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Queues a key press event.
    ///
    /// The event is added to the pending events queue and will be emitted
    /// when `flush()` is called or automatically during the next frame.
    ///
    /// # Arguments
    ///
    /// * `keycode` - The hardware keycode (evdev keycode) to press
    ///
    /// # Note
    ///
    /// Keycodes are evdev keycodes (e.g., KEY_A = 30). Use `keysym_to_keycode()`
    /// to convert XKB keysyms to keycodes.
    pub fn press_key(&mut self, keycode: u32) {
        if !self.initialized {
            tracing::warn!("Virtual keyboard not initialized, ignoring key press");
            return;
        }

        let event = KeyEvent::press(keycode, self.get_timestamp());
        self.pending_events.push(event);

        // Update XKB state
        if let Some(ref mut state) = self.xkb_state {
            state.update_key(
                xkbcommon::xkb::Keycode::new(keycode + 8),
                xkbcommon::xkb::KeyDirection::Down,
            );
        }

        tracing::debug!("Queued key press: keycode={}", keycode);
    }

    /// Queues a key release event.
    ///
    /// # Arguments
    ///
    /// * `keycode` - The hardware keycode (evdev keycode) to release
    pub fn release_key(&mut self, keycode: u32) {
        if !self.initialized {
            tracing::warn!("Virtual keyboard not initialized, ignoring key release");
            return;
        }

        let event = KeyEvent::release(keycode, self.get_timestamp());
        self.pending_events.push(event);

        // Update XKB state
        if let Some(ref mut state) = self.xkb_state {
            state.update_key(
                xkbcommon::xkb::Keycode::new(keycode + 8),
                xkbcommon::xkb::KeyDirection::Up,
            );
        }

        tracing::debug!("Queued key release: keycode={}", keycode);
    }

    /// Returns the pending key events and clears the queue.
    ///
    /// This method is used by the applet to retrieve queued events and
    /// emit them through the actual Wayland virtual keyboard protocol.
    #[must_use]
    pub fn take_pending_events(&mut self) -> Vec<KeyEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Returns a reference to the pending key events without clearing.
    #[must_use]
    pub fn pending_events(&self) -> &[KeyEvent] {
        &self.pending_events
    }

    /// Clears all pending key events.
    pub fn clear_pending_events(&mut self) {
        self.pending_events.clear();
    }

    /// Converts an XKB keysym name to a hardware keycode.
    ///
    /// This method looks up the keysym by name in the current keymap and
    /// returns the corresponding evdev keycode if found.
    ///
    /// # Arguments
    ///
    /// * `keysym_name` - The XKB keysym name (e.g., "a", "Shift_L", "BackSpace")
    ///
    /// # Returns
    ///
    /// * `Some(keycode)` if the keysym was found in the keymap
    /// * `None` if the keysym is unknown or not in the current keymap
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut vk = VirtualKeyboard::new();
    /// vk.initialize().unwrap();
    ///
    /// // Look up the keycode for 'a'
    /// if let Some(keycode) = vk.keysym_to_keycode("a") {
    ///     println!("Keycode for 'a': {}", keycode);
    /// }
    /// ```
    #[must_use]
    pub fn keysym_to_keycode(&self, keysym_name: &str) -> Option<u32> {
        if !self.initialized {
            return None;
        }

        let keymap = self.xkb_keymap.as_ref()?;

        // Get keysym from name
        let keysym = xkbcommon::xkb::keysym_from_name(keysym_name, xkbcommon::xkb::KEYSYM_NO_FLAGS);

        // Convert KEY_NoSymbol to Keysym for comparison
        let no_symbol: Keysym = KEY_NoSymbol.into();

        if keysym == no_symbol {
            // Try case-insensitive search
            let keysym =
                xkbcommon::xkb::keysym_from_name(keysym_name, xkbcommon::xkb::KEYSYM_CASE_INSENSITIVE);

            if keysym == no_symbol {
                return None;
            }

            return self.find_keycode_for_keysym(keymap, keysym);
        }

        self.find_keycode_for_keysym(keymap, keysym)
    }

    /// Converts a character to a hardware keycode.
    ///
    /// This method converts a Unicode character to its corresponding keysym
    /// and then looks up the keycode in the current keymap.
    ///
    /// # Arguments
    ///
    /// * `c` - The Unicode character to convert
    ///
    /// # Returns
    ///
    /// * `Some(keycode)` if the character can be typed with a single key
    /// * `None` if the character requires modifiers or is not in the keymap
    #[must_use]
    pub fn char_to_keycode(&self, c: char) -> Option<u32> {
        if !self.initialized {
            return None;
        }

        let keymap = self.xkb_keymap.as_ref()?;

        // Convert character to keysym
        // For ASCII, the keysym is typically the character code
        // For Unicode, use xkb_utf32_to_keysym
        let keysym_raw = if c.is_ascii() {
            // ASCII characters map directly to keysyms for lowercase
            // For uppercase and special chars, we need the actual keysym
            let code = c as u32;
            if code >= 0x20 && code <= 0x7E {
                // Printable ASCII - keysym matches codepoint for lowercase
                // but uppercase letters need special handling
                if c.is_ascii_uppercase() {
                    // Uppercase letters: keysym = character code
                    code
                } else if c.is_ascii_lowercase() {
                    // Lowercase letters: keysym = character code
                    code
                } else {
                    // Other printable ASCII
                    code
                }
            } else {
                return None;
            }
        } else {
            // Non-ASCII Unicode: convert to keysym
            // Unicode keysyms are 0x01000000 + codepoint
            0x0100_0000 | (c as u32)
        };

        let keysym: Keysym = keysym_raw.into();
        self.find_keycode_for_keysym(keymap, keysym)
    }

    /// Finds the keycode that produces the given keysym at any level.
    fn find_keycode_for_keysym(
        &self,
        keymap: &xkbcommon::xkb::Keymap,
        target_keysym: Keysym,
    ) -> Option<u32> {
        // Iterate through all keycodes to find one that produces this keysym
        let min_keycode = keymap.min_keycode();
        let max_keycode = keymap.max_keycode();

        for keycode_raw in min_keycode.raw()..=max_keycode.raw() {
            let keycode = xkbcommon::xkb::Keycode::new(keycode_raw);
            // Check all groups and levels for this keycode
            let num_layouts = keymap.num_layouts_for_key(keycode);
            for layout in 0..num_layouts {
                let num_levels = keymap.num_levels_for_key(keycode, layout);
                for level in 0..num_levels {
                    let keysyms = keymap.key_get_syms_by_level(keycode, layout, level);
                    for &keysym in keysyms {
                        if keysym == target_keysym {
                            // Convert XKB keycode (8-offset) to evdev keycode
                            return Some(keycode_raw - 8);
                        }
                    }
                }
            }
        }

        None
    }

    /// Resolves a `ResolvedKeycode` to a hardware keycode.
    ///
    /// This is a convenience method that handles all three keycode types:
    /// - `Character`: Uses `char_to_keycode()`
    /// - `Keysym`: Uses `keysym_to_keycode()`
    /// - `UnicodeCodepoint`: Returns None (requires Unicode fallback)
    ///
    /// # Returns
    ///
    /// * `Some(keycode)` if the keycode can be resolved
    /// * `None` if the keycode cannot be mapped (use Unicode fallback)
    #[must_use]
    pub fn resolve_keycode(&self, resolved: &ResolvedKeycode) -> Option<u32> {
        match resolved {
            ResolvedKeycode::Character(c) => self.char_to_keycode(*c),
            ResolvedKeycode::Keysym(name) => self.keysym_to_keycode(name),
            ResolvedKeycode::UnicodeCodepoint(_) => {
                // Unicode codepoints require the fallback mechanism
                None
            }
        }
    }

    /// Emits a Unicode codepoint using the Ctrl+Shift+U hex input fallback.
    ///
    /// This method implements the standard GTK/Linux Unicode input method:
    /// 1. Press Ctrl+Shift+U (enters Unicode hex input mode)
    /// 2. Type the hex digits of the codepoint
    /// 3. Press Space to commit
    /// 4. Release Ctrl+Shift
    ///
    /// # Arguments
    ///
    /// * `codepoint` - The Unicode codepoint (e.g., 0x03C0 for pi)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut vk = VirtualKeyboard::new();
    /// vk.initialize().unwrap();
    ///
    /// // Type the pi symbol
    /// vk.emit_unicode_codepoint(0x03C0);
    /// ```
    pub fn emit_unicode_codepoint(&mut self, codepoint: u32) {
        if !self.initialized {
            tracing::warn!(
                "Virtual keyboard not initialized, ignoring Unicode codepoint U+{:04X}",
                codepoint
            );
            return;
        }

        tracing::warn!(
            "Using Ctrl+Shift+U fallback for Unicode codepoint U+{:04X}",
            codepoint
        );

        // Keycodes for the fallback sequence (evdev keycodes)
        const KEY_LEFTCTRL: u32 = 29;
        const KEY_LEFTSHIFT: u32 = 42;
        const KEY_U: u32 = 22;
        const KEY_SPACE: u32 = 57;

        // Hex digit keycodes
        const KEY_0: u32 = 11;
        const KEY_1: u32 = 2;
        const KEY_2: u32 = 3;
        const KEY_3: u32 = 4;
        const KEY_4: u32 = 5;
        const KEY_5: u32 = 6;
        const KEY_6: u32 = 7;
        const KEY_7: u32 = 8;
        const KEY_8: u32 = 9;
        const KEY_9: u32 = 10;
        const KEY_A: u32 = 30;
        const KEY_B: u32 = 48;
        const KEY_C: u32 = 46;
        const KEY_D: u32 = 32;
        const KEY_E: u32 = 18;
        const KEY_F: u32 = 33;

        // Step 1: Press Ctrl+Shift+U
        self.press_key(KEY_LEFTCTRL);
        self.press_key(KEY_LEFTSHIFT);
        self.press_key(KEY_U);
        self.release_key(KEY_U);

        // Step 2: Type hex digits (skip leading zeros but ensure at least one digit)
        let hex_string = format!("{:X}", codepoint);
        for hex_char in hex_string.chars() {
            let keycode = match hex_char {
                '0' => KEY_0,
                '1' => KEY_1,
                '2' => KEY_2,
                '3' => KEY_3,
                '4' => KEY_4,
                '5' => KEY_5,
                '6' => KEY_6,
                '7' => KEY_7,
                '8' => KEY_8,
                '9' => KEY_9,
                'A' | 'a' => KEY_A,
                'B' | 'b' => KEY_B,
                'C' | 'c' => KEY_C,
                'D' | 'd' => KEY_D,
                'E' | 'e' => KEY_E,
                'F' | 'f' => KEY_F,
                _ => continue,
            };
            self.press_key(keycode);
            self.release_key(keycode);
        }

        // Step 3: Press Space to commit
        self.press_key(KEY_SPACE);
        self.release_key(KEY_SPACE);

        // Step 4: Release Ctrl+Shift
        self.release_key(KEY_LEFTSHIFT);
        self.release_key(KEY_LEFTCTRL);
    }

    /// Returns the current timestamp in milliseconds.
    ///
    /// Uses the system monotonic clock for consistent timing.
    fn get_timestamp(&self) -> u32 {
        use std::time::{SystemTime, UNIX_EPOCH};

        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| (d.as_millis() % u128::from(u32::MAX)) as u32)
            .unwrap_or(0)
    }

    /// Cleans up virtual keyboard resources.
    ///
    /// This method should be called before the keyboard surface is destroyed.
    /// It clears pending events and releases XKB resources.
    pub fn cleanup(&mut self) {
        self.pending_events.clear();
        self.xkb_state = None;
        self.xkb_keymap = None;
        self.xkb_context = None;
        self.initialized = false;

        tracing::info!("Virtual keyboard cleaned up");
    }
}

impl Drop for VirtualKeyboard {
    fn drop(&mut self) {
        if self.initialized {
            self.cleanup();
        }
    }
}

// ============================================================================
// Common Keycodes (evdev)
// ============================================================================

/// Common evdev keycodes for convenience.
///
/// These are the most frequently used keycodes for modifier keys and
/// special keys. They correspond to Linux evdev key codes.
pub mod keycodes {
    /// Escape key
    pub const KEY_ESC: u32 = 1;
    /// Backspace key
    pub const KEY_BACKSPACE: u32 = 14;
    /// Tab key
    pub const KEY_TAB: u32 = 15;
    /// Enter/Return key
    pub const KEY_ENTER: u32 = 28;
    /// Left Control key
    pub const KEY_LEFTCTRL: u32 = 29;
    /// Left Shift key
    pub const KEY_LEFTSHIFT: u32 = 42;
    /// Right Shift key
    pub const KEY_RIGHTSHIFT: u32 = 54;
    /// Left Alt key
    pub const KEY_LEFTALT: u32 = 56;
    /// Space key
    pub const KEY_SPACE: u32 = 57;
    /// Caps Lock key
    pub const KEY_CAPSLOCK: u32 = 58;
    /// Right Control key
    pub const KEY_RIGHTCTRL: u32 = 97;
    /// Right Alt key
    pub const KEY_RIGHTALT: u32 = 100;
    /// Left Super/Meta/Windows key
    pub const KEY_LEFTMETA: u32 = 125;
    /// Right Super/Meta/Windows key
    pub const KEY_RIGHTMETA: u32 = 126;
}

// ============================================================================
// Tests (Task 3.1)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Task 3.1: Focused tests for virtual keyboard wrapper (4-6 tests)
    // ========================================================================

    /// Test 1: Key press event emission
    ///
    /// Tests that key press events are properly queued with correct state
    /// and keycode values.
    #[test]
    fn test_key_press_event_emission() {
        let mut vk = VirtualKeyboard::new();

        // Initialize the virtual keyboard
        // Note: This may fail in CI environments without XKB support
        if vk.initialize().is_err() {
            // Skip test if XKB initialization fails (e.g., in headless CI)
            eprintln!("Skipping test: XKB initialization failed (likely headless environment)");
            return;
        }

        // Press a key
        vk.press_key(keycodes::KEY_SPACE);

        // Verify event was queued
        let events = vk.pending_events();
        assert_eq!(events.len(), 1, "Should have one pending event");

        let event = &events[0];
        assert_eq!(event.keycode, keycodes::KEY_SPACE, "Keycode should be Space");
        assert_eq!(event.state, KeyState::Pressed, "State should be Pressed");
        assert!(event.time > 0, "Timestamp should be positive");
    }

    /// Test 2: Key release event emission
    ///
    /// Tests that key release events are properly queued with correct state
    /// and keycode values.
    #[test]
    fn test_key_release_event_emission() {
        let mut vk = VirtualKeyboard::new();

        if vk.initialize().is_err() {
            eprintln!("Skipping test: XKB initialization failed");
            return;
        }

        // Press and release a key
        vk.press_key(keycodes::KEY_ENTER);
        vk.release_key(keycodes::KEY_ENTER);

        // Verify both events were queued
        let events = vk.pending_events();
        assert_eq!(events.len(), 2, "Should have two pending events");

        // Check release event
        let release_event = &events[1];
        assert_eq!(release_event.keycode, keycodes::KEY_ENTER, "Keycode should be Enter");
        assert_eq!(release_event.state, KeyState::Released, "State should be Released");
    }

    /// Test 3: Modifier key handling
    ///
    /// Tests that modifier keys (Shift, Ctrl, Alt) are handled correctly
    /// and can be combined with regular keys.
    #[test]
    fn test_modifier_key_handling() {
        let mut vk = VirtualKeyboard::new();

        if vk.initialize().is_err() {
            eprintln!("Skipping test: XKB initialization failed");
            return;
        }

        // Simulate Ctrl+C: press Ctrl, press C, release C, release Ctrl
        const KEY_C: u32 = 46; // evdev keycode for 'c'

        vk.press_key(keycodes::KEY_LEFTCTRL);
        vk.press_key(KEY_C);
        vk.release_key(KEY_C);
        vk.release_key(keycodes::KEY_LEFTCTRL);

        // Verify all events were queued in correct order
        let events = vk.pending_events();
        assert_eq!(events.len(), 4, "Should have four pending events");

        // Check sequence
        assert_eq!(events[0].keycode, keycodes::KEY_LEFTCTRL);
        assert_eq!(events[0].state, KeyState::Pressed);

        assert_eq!(events[1].keycode, KEY_C);
        assert_eq!(events[1].state, KeyState::Pressed);

        assert_eq!(events[2].keycode, KEY_C);
        assert_eq!(events[2].state, KeyState::Released);

        assert_eq!(events[3].keycode, keycodes::KEY_LEFTCTRL);
        assert_eq!(events[3].state, KeyState::Released);
    }

    /// Test 4: XKB keysym to keycode conversion
    ///
    /// Tests that common keysyms can be converted to their corresponding
    /// hardware keycodes using the XKB keymap.
    #[test]
    fn test_xkb_keycode_conversion() {
        let mut vk = VirtualKeyboard::new();

        if vk.initialize().is_err() {
            eprintln!("Skipping test: XKB initialization failed");
            return;
        }

        // Test common keysym conversions
        // Note: Results depend on the system keymap

        // Return/Enter should be found in any keymap
        let return_keycode = vk.keysym_to_keycode("Return");
        assert!(
            return_keycode.is_some(),
            "Return keysym should have a keycode"
        );

        // Shift_L should also be universal
        let shift_keycode = vk.keysym_to_keycode("Shift_L");
        assert!(
            shift_keycode.is_some(),
            "Shift_L keysym should have a keycode"
        );

        // BackSpace should be present
        let backspace_keycode = vk.keysym_to_keycode("BackSpace");
        assert!(
            backspace_keycode.is_some(),
            "BackSpace keysym should have a keycode"
        );

        // Invalid keysym should return None
        let invalid_keycode = vk.keysym_to_keycode("InvalidKeysymThatDoesNotExist123");
        assert!(
            invalid_keycode.is_none(),
            "Invalid keysym should return None"
        );
    }

    /// Test 5: Initialization and cleanup
    ///
    /// Tests the lifecycle of the virtual keyboard: creation, initialization,
    /// and cleanup.
    #[test]
    fn test_initialization_and_cleanup() {
        let mut vk = VirtualKeyboard::new();

        // Should not be initialized initially
        assert!(
            !vk.is_initialized(),
            "Should not be initialized after creation"
        );

        // Initialize
        let init_result = vk.initialize();
        // May fail in headless environments, which is acceptable
        if init_result.is_ok() {
            assert!(vk.is_initialized(), "Should be initialized after initialize()");

            // Queue some events
            vk.press_key(keycodes::KEY_SPACE);
            assert_eq!(vk.pending_events().len(), 1);

            // Cleanup
            vk.cleanup();
            assert!(!vk.is_initialized(), "Should not be initialized after cleanup");
            assert_eq!(vk.pending_events().len(), 0, "Events should be cleared");
        }

        // Double initialization should be safe
        if vk.initialize().is_ok() {
            assert!(vk.initialize().is_ok(), "Double initialization should succeed");
        }
    }

    /// Test 6: Unicode codepoint fallback sequence
    ///
    /// Tests that the Ctrl+Shift+U hex input sequence is generated correctly
    /// for Unicode codepoints.
    #[test]
    fn test_unicode_codepoint_fallback() {
        let mut vk = VirtualKeyboard::new();

        if vk.initialize().is_err() {
            eprintln!("Skipping test: XKB initialization failed");
            return;
        }

        // Emit a Unicode codepoint (pi symbol: U+03C0)
        vk.emit_unicode_codepoint(0x03C0);

        let events = vk.pending_events();

        // The sequence should be:
        // 1. Press Ctrl
        // 2. Press Shift
        // 3. Press U, Release U
        // 4. Press 3, Release 3
        // 5. Press C, Release C
        // 6. Press 0, Release 0 (03C0 has leading 0)
        // 7. Press Space, Release Space
        // 8. Release Shift
        // 9. Release Ctrl

        // Verify the sequence starts with Ctrl+Shift+U
        assert!(events.len() >= 10, "Should have at least 10 events for Ctrl+Shift+U sequence");

        // Check Ctrl press
        assert_eq!(events[0].keycode, keycodes::KEY_LEFTCTRL);
        assert_eq!(events[0].state, KeyState::Pressed);

        // Check Shift press
        assert_eq!(events[1].keycode, keycodes::KEY_LEFTSHIFT);
        assert_eq!(events[1].state, KeyState::Pressed);

        // Verify sequence ends with Shift and Ctrl release
        let last_idx = events.len() - 1;
        assert_eq!(events[last_idx].keycode, keycodes::KEY_LEFTCTRL);
        assert_eq!(events[last_idx].state, KeyState::Released);

        assert_eq!(events[last_idx - 1].keycode, keycodes::KEY_LEFTSHIFT);
        assert_eq!(events[last_idx - 1].state, KeyState::Released);
    }

    // ========================================================================
    // Additional Unit Tests
    // ========================================================================

    /// Test KeyEvent construction
    #[test]
    fn test_key_event_construction() {
        let press = KeyEvent::press(42, 1000);
        assert_eq!(press.keycode, 42);
        assert_eq!(press.state, KeyState::Pressed);
        assert_eq!(press.time, 1000);

        let release = KeyEvent::release(42, 2000);
        assert_eq!(release.keycode, 42);
        assert_eq!(release.state, KeyState::Released);
        assert_eq!(release.time, 2000);
    }

    /// Test pending events management
    #[test]
    fn test_pending_events_management() {
        let mut vk = VirtualKeyboard::new();

        if vk.initialize().is_err() {
            return;
        }

        // Queue some events
        vk.press_key(10);
        vk.press_key(20);
        assert_eq!(vk.pending_events().len(), 2);

        // Take events
        let events = vk.take_pending_events();
        assert_eq!(events.len(), 2);
        assert_eq!(vk.pending_events().len(), 0, "Queue should be empty after take");

        // Queue more and clear
        vk.press_key(30);
        vk.clear_pending_events();
        assert_eq!(vk.pending_events().len(), 0);
    }

    /// Test uninitialized virtual keyboard behavior
    #[test]
    fn test_uninitialized_behavior() {
        let mut vk = VirtualKeyboard::new();

        // Operations on uninitialized VK should not panic
        vk.press_key(10);
        vk.release_key(10);
        vk.emit_unicode_codepoint(0x03C0);

        // Events should not be queued
        assert_eq!(vk.pending_events().len(), 0);

        // Lookups should return None
        assert!(vk.keysym_to_keycode("Return").is_none());
        assert!(vk.char_to_keycode('a').is_none());
    }

    /// Test ResolvedKeycode resolution
    #[test]
    fn test_resolved_keycode_resolution() {
        let mut vk = VirtualKeyboard::new();

        if vk.initialize().is_err() {
            return;
        }

        // Character should resolve via char_to_keycode
        let char_resolved = ResolvedKeycode::Character('a');
        // May or may not find 'a' depending on keymap
        let _ = vk.resolve_keycode(&char_resolved);

        // Keysym should resolve via keysym_to_keycode
        let keysym_resolved = ResolvedKeycode::Keysym("Return".to_string());
        assert!(
            vk.resolve_keycode(&keysym_resolved).is_some(),
            "Return keysym should resolve"
        );

        // Unicode codepoint should return None (requires fallback)
        let unicode_resolved = ResolvedKeycode::UnicodeCodepoint(0x03C0);
        assert!(
            vk.resolve_keycode(&unicode_resolved).is_none(),
            "Unicode codepoint should not resolve directly"
        );
    }

    /// Test Default trait
    #[test]
    fn test_default_trait() {
        let vk = VirtualKeyboard::default();
        assert!(!vk.is_initialized());
        assert_eq!(vk.pending_events().len(), 0);
    }
}
