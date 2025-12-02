# Specification: Basic Key Input

## Goal

Connect rendered keyboard keys to system input events, enabling users to type characters and use modifier keys through the Cosboard soft keyboard by emitting virtual keyboard events via Wayland's `zwp_virtual_keyboard_v1` protocol.

## User Stories

- As a touchscreen user, I want to tap keys on the soft keyboard and have characters appear in the focused text field, so that I can type without a physical keyboard.
- As a user needing keyboard shortcuts, I want to tap modifier keys (Shift, Ctrl, Alt) and then tap another key to perform combinations like Ctrl+C, so that I can use keyboard shortcuts.

## Specific Requirements

**Wayland Virtual Keyboard Integration**
- Use `zwp_virtual_keyboard_v1` protocol for input injection
- Initialize virtual keyboard interface on keyboard surface creation
- Use default system XKB keymap (avoid custom keymap complexity)
- Emit `key` events with keycodes and press/release states
- Handle protocol object lifecycle with keyboard surface

**Keycode Parsing Module**
- Create `src/input/keycode.rs` for keycode parsing and resolution
- Support single character codes: `"q"`, `"a"`, `" "` (space)
- Support XKB keysym names: `"Shift_L"`, `"BackSpace"`, `"Return"`, `"Tab"`
- Support Unicode codepoints: `"U+2022"` (bullet), `"U+03C0"` (pi)
- Parse layout's `code` field using a prioritized format detection strategy
- Return `Option<Keycode>` to handle graceful failure for unknown formats

**Key Press Event Flow**
- Intercept `RendererMessage::KeyPressed(identifier)` in applet update handler
- Look up key definition from current panel by identifier
- Resolve keycode from key's `code` field using keycode parser
- Emit virtual key press event with resolved keycode and active modifiers
- Intercept `RendererMessage::KeyReleased(identifier)` for key release events
- Emit virtual key release event to complete the keystroke

**Modifier Key State Management**
- Track active modifier state in `KeyboardRenderer` using `HashSet<Modifier>`
- Support modifiers: Shift, Ctrl, Alt, Super/Meta (left and right variants)
- Add `stickyrelease` JSON field to Key struct (default: `true`)
- Implement one-shot behavior: `sticky: true` + `stickyrelease: true` releases after next key
- Implement toggle behavior: `sticky: true` + `stickyrelease: false` stays until re-tapped
- Implement hold behavior: `sticky: false` requires holding key to keep modifier active

**Combo Key Support**
- Collect all active modifiers before emitting regular key event
- Emit modifier key presses before the main key press
- Emit modifier key releases after the main key release (for combos)
- Clear sticky modifiers with `stickyrelease: true` after emitting combo
- Preserve toggle modifiers until explicitly deactivated

**Unicode Fallback Mechanism**
- Detect when keycode cannot be mapped to XKB keycode
- Use Ctrl+Shift+U hex input method as fallback for Unicode codepoints
- Emit sequence: Ctrl down, Shift down, U key, hex digits, space, modifiers up
- Log warning when falling back to unicode input method

**Visual Modifier State Indication**
- Use existing `sticky_active_color` theme function for active modifiers
- Apply visual styling to modifier keys when their state is active
- Ensure sticky vs toggle vs held modifiers all show active state

**Input Module Architecture**
- Create new `src/input/mod.rs` as module root
- Create `src/input/keycode.rs` for keycode parsing logic
- Create `src/input/virtual_keyboard.rs` for Wayland protocol handling
- Create `src/input/modifier.rs` for modifier state management
- Re-export public API from module root

## Visual Design

No visual mockups were provided for this feature.

## Existing Code to Leverage

**KeyboardRenderer State (src/renderer/state.rs)**
- Already tracks `pressed_keys` HashSet for visual feedback
- Already tracks `sticky_keys_active` HashSet for sticky key state
- Already has `toggle_sticky()` method for toggling sticky keys
- Extend to track modifier state with new `active_modifiers: HashSet<Modifier>` field

**RendererMessage Types (src/renderer/message.rs)**
- Already defines `KeyPressed(String)` and `KeyReleased(String)` messages
- These messages are already routed from renderer to applet update handler
- Add new input-related message variant like `EmitKeyEvent(KeyEvent)` if needed

**Layout Types (src/layout/types.rs)**
- `Key` struct already has `code: KeyCode` field for keycode storage
- `KeyCode` enum has `Unicode(char)` and `Keysym(String)` variants
- `Modifier` enum already defines `Shift`, `Ctrl`, `Alt`, `Super`
- Extend `Key` struct with `stickyrelease: Option<bool>` field

**Applet Update Handler (src/applet/mod.rs)**
- Already handles `Message::KeyPressed` and `Message::KeyReleased`
- Currently only updates renderer state for visual feedback
- Add input emission logic after renderer state update

**Existing Sticky Key Styling (src/renderer/theme.rs)**
- Already exports `sticky_active_color()` for active sticky key highlighting
- Key rendering already checks `renderer.is_sticky_active(identifier)`
- Visual indication should work automatically when modifier state is tracked

## Out of Scope

- Key repeat functionality (conflicts with hold-for-secondary feature already implemented)
- Haptic or vibration feedback on key press
- Audio feedback or click sounds on key press
- Dead keys and compose sequences (e.g., ` + e = accented e)
- Macro recording and playback functionality
- Long-press actions and swipe alternative popups (already implemented in renderer)
- Custom XKB keymap loading or switching
- Input method editor (IME) integration
- Multiple keyboard layout switching at runtime
- Key event timing or velocity sensitivity
