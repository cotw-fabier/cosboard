# Task Breakdown: Basic Key Input

## Overview
Total Tasks: 42

This feature connects rendered keyboard keys to system input events via Wayland's `zwp_virtual_keyboard_v1` protocol, enabling users to type characters and use modifier keys through the Cosboard soft keyboard.

## Task List

### Foundation Layer

#### Task Group 1: Input Module Architecture
**Dependencies:** None

- [x] 1.0 Complete input module structure
  - [x] 1.1 Write 4-6 focused tests for input module public API
    - Test keycode parsing for single characters (`"q"`, `"a"`, `" "`)
    - Test keycode parsing for XKB keysym names (`"Shift_L"`, `"BackSpace"`)
    - Test keycode parsing for Unicode codepoints (`"U+2022"`, `"U+03C0"`)
    - Test graceful failure for unknown formats (returns `None`)
    - Test modifier enum serialization/deserialization
  - [x] 1.2 Create `src/input/mod.rs` as module root
    - Define public API exports
    - Add module documentation
    - Follow existing module patterns (see `src/layout/mod.rs`)
  - [x] 1.3 Create `src/input/keycode.rs` for keycode parsing
    - Define `ResolvedKeycode` type for parsed keycodes
    - Implement format detection (single char, XKB keysym, Unicode codepoint)
    - Implement `parse_keycode(code: &KeyCode) -> Option<ResolvedKeycode>`
    - Handle graceful failure for unknown formats
  - [x] 1.4 Create `src/input/modifier.rs` for modifier state management
    - Define `ModifierState` struct with `HashSet<Modifier>`
    - Implement `activate()`, `deactivate()`, `toggle()`, `is_active()` methods
    - Implement `clear_sticky()` for clearing one-shot modifiers
    - Support left/right modifier variants if needed
  - [x] 1.5 Update `src/lib.rs` to expose input module
    - Add `pub mod input;` declaration
    - Re-export key public types
  - [x] 1.6 Ensure input module tests pass
    - Run ONLY the 4-6 tests written in 1.1
    - Verify module compiles and exports correctly

**Acceptance Criteria:**
- The 4-6 tests written in 1.1 pass
- Input module structure follows existing codebase patterns
- Keycode parser handles all three formats (char, keysym, unicode)
- Modifier state management works correctly

---

#### Task Group 2: Layout Schema Updates
**Dependencies:** None (can run parallel with Task Group 1)

- [x] 2.0 Complete layout schema updates for stickyrelease
  - [x] 2.1 Write 3-4 focused tests for stickyrelease field
    - Test default value is `true` when field is omitted
    - Test explicit `false` value is preserved
    - Test JSON deserialization with and without field
    - Test sticky + stickyrelease behavior combinations
  - [x] 2.2 Extend `Key` struct in `src/layout/types.rs`
    - Add `stickyrelease: bool` field with serde defaults
    - Add `#[serde(default = "default_stickyrelease")]` attribute
    - Document the field behavior in struct documentation
  - [x] 2.3 Update layout validation in `src/layout/validation.rs`
    - Add validation for stickyrelease field (if needed)
    - Ensure backwards compatibility with layouts without the field
  - [x] 2.4 Update example layout JSON file
    - Add stickyrelease examples to `resources/layouts/example_qwerty.json`
    - Show both `true` and `false` usage on modifier keys
  - [x] 2.5 Ensure layout schema tests pass
    - Run ONLY the 3-4 tests written in 2.1
    - Verify existing layout tests still pass

**Acceptance Criteria:**
- The 3-4 tests written in 2.1 pass
- Existing layouts continue to parse correctly
- New stickyrelease field defaults to true
- Example layout demonstrates the feature

---

### Wayland Protocol Layer

#### Task Group 3: Virtual Keyboard Protocol Integration
**Dependencies:** Task Group 1 (needs keycode types)

- [x] 3.0 Complete Wayland virtual keyboard integration
  - [x] 3.1 Write 4-6 focused tests for virtual keyboard wrapper
    - Test key press event emission
    - Test key release event emission
    - Test modifier key handling
    - Test XKB keycode conversion
    - Test initialization and cleanup
  - [x] 3.2 Add wayland protocol dependencies to `Cargo.toml`
    - Add `wayland-client` crate (if not already present)
    - Add `zwp-virtual-keyboard` protocol support
    - Add `xkbcommon` crate for keysym handling
  - [x] 3.3 Create `src/input/virtual_keyboard.rs`
    - Define `VirtualKeyboard` struct for protocol handling
    - Implement initialization with default system XKB keymap
    - Implement `press_key(keycode: u32)` method
    - Implement `release_key(keycode: u32)` method
    - Handle protocol object lifecycle
  - [x] 3.4 Implement XKB keysym to keycode conversion
    - Use xkbcommon crate for keysym lookup
    - Handle conversion from keysym name to keycode
    - Return `Option<u32>` for graceful failure
  - [x] 3.5 Implement Unicode codepoint handling
    - Detect when keycode cannot be mapped to XKB
    - Implement Ctrl+Shift+U hex input fallback sequence
    - Log warning when using fallback method
  - [x] 3.6 Ensure virtual keyboard tests pass
    - Run ONLY the 4-6 tests written in 3.1
    - Verify protocol integration compiles

**Acceptance Criteria:**
- The 4-6 tests written in 3.1 pass
- Virtual keyboard protocol initializes correctly
- Key events are emitted properly
- Unicode fallback works for unmapped characters

---

### State Management Layer

#### Task Group 4: Modifier State Tracking in Renderer
**Dependencies:** Task Groups 1, 2

- [x] 4.0 Complete modifier state tracking in renderer
  - [x] 4.1 Write 4-5 focused tests for modifier state in renderer
    - Test one-shot behavior (sticky: true, stickyrelease: true)
    - Test toggle behavior (sticky: true, stickyrelease: false)
    - Test hold behavior (sticky: false)
    - Test multiple simultaneous modifiers
    - Test modifier clearing after combo key
  - [x] 4.2 Extend `KeyboardRenderer` in `src/renderer/state.rs`
    - Add `modifier_state: ModifierState` field (delegates to input module)
    - Import `Modifier` from `src/layout/types.rs`
    - Import `ModifierState` from `src/input/modifier.rs`
    - Initialize empty in `KeyboardRenderer::new()`
  - [x] 4.3 Implement modifier activation methods
    - Add `activate_modifier(modifier: Modifier, stickyrelease: bool)`
    - Add `deactivate_modifier(modifier: Modifier)`
    - Add `is_modifier_active(modifier: Modifier) -> bool`
    - Add `get_active_modifiers() -> Vec<Modifier>`
  - [x] 4.4 Implement one-shot modifier behavior
    - Track which modifiers should clear after next key
    - Add `clear_oneshot_modifiers()` method
    - Call after combo key emission
  - [x] 4.5 Integrate with existing sticky key tracking
    - Coordinate with `sticky_keys_active` HashSet
    - Ensure visual state matches logical modifier state
    - Add `sync_modifier_visual_state()` helper method
  - [x] 4.6 Ensure modifier state tests pass
    - Run ONLY the 4-5 tests written in 4.1
    - Verify integration with existing state

**Acceptance Criteria:**
- The 4-5 tests written in 4.1 pass
- Modifier state tracks correctly for all three modes
- One-shot modifiers clear after combo
- Visual state matches logical state

---

### Event Handling Layer

#### Task Group 5: Key Press Event Flow
**Dependencies:** Task Groups 3, 4

- [x] 5.0 Complete key press event handling
  - [x] 5.1 Write 5-6 focused tests for key press flow
    - Test regular key press emits correct keycode
    - Test modifier key activates modifier state
    - Test combo key (modifier + regular) emits sequence
    - Test sticky modifier clears after combo (stickyrelease: true)
    - Test toggle modifier persists after combo (stickyrelease: false)
    - Test hold modifier behavior
  - [x] 5.2 Extend applet update handler in `src/applet/mod.rs`
    - Handle `RendererMessage::KeyPressed` for input emission
    - Look up key definition from current panel by identifier
    - Resolve keycode using keycode parser
  - [x] 5.3 Implement regular key emission
    - Get active modifiers before key press
    - Emit modifier key presses (if any active)
    - Emit main key press
    - Coordinate with virtual keyboard module
  - [x] 5.4 Implement modifier key handling
    - Detect if pressed key is a modifier (Shift, Ctrl, Alt, Super)
    - Check `sticky` and `stickyrelease` fields from key definition
    - Apply correct behavior (one-shot, toggle, or hold)
    - Update renderer state
  - [x] 5.5 Handle `RendererMessage::KeyReleased`
    - Emit main key release event
    - Emit modifier key releases (for combos)
    - Clear one-shot modifiers if applicable
    - Handle hold modifier release
  - [x] 5.6 Add new message types if needed
    - Consider `EmitKeyEvent(KeyEvent)` variant
    - Update `src/renderer/message.rs` if required
  - [x] 5.7 Ensure key press flow tests pass
    - Run ONLY the 5-6 tests written in 5.1
    - Verify complete key press cycle works

**Acceptance Criteria:**
- The 5-6 tests written in 5.1 pass
- Regular keys emit correct keycodes
- Modifier keys activate/deactivate correctly
- Combo keys emit proper sequences
- Key release completes the cycle

---

### Visual Feedback Layer

#### Task Group 6: Visual Modifier State Indication
**Dependencies:** Task Group 4

- [x] 6.0 Complete visual modifier state indication
  - [x] 6.1 Write 2-3 focused tests for visual modifier indication
    - Test active modifier key shows `sticky_active_color`
    - Test inactive modifier key shows normal styling
    - Test visual state updates on modifier toggle
  - [x] 6.2 Verify existing theme integration
    - Confirm `sticky_active_color()` in `src/renderer/theme.rs` works
    - Confirm key rendering checks `renderer.is_sticky_active()`
    - Ensure modifier state triggers visual update
  - [x] 6.3 Update key rendering for modifier visualization
    - Coordinate modifier state with sticky key visuals
    - Ensure all three modes (one-shot, toggle, hold) show active state
    - Apply `sticky_active_color` when modifier is active
  - [x] 6.4 Ensure visual feedback tests pass
    - Run ONLY the 2-3 tests written in 6.1
    - Verify visual updates correctly

**Acceptance Criteria:**
- The 2-3 tests written in 6.1 pass
- Active modifiers display with correct styling
- Visual state updates immediately on interaction
- All modifier modes show consistent visual feedback

---

### Integration Layer

#### Task Group 7: End-to-End Integration
**Dependencies:** Task Groups 1-6

- [x] 7.0 Complete end-to-end integration
  - [x] 7.1 Write 4-5 focused integration tests
    - Test full key press -> input emission -> key release cycle
    - Test modifier + key combo flow
    - Test panel switch does not affect modifier state unexpectedly
    - Test multiple rapid key presses
    - Test error handling for unmapped keycodes
  - [x] 7.2 Initialize virtual keyboard in applet
    - Create virtual keyboard instance on keyboard surface creation
    - Handle initialization errors gracefully
    - Clean up on keyboard surface destruction
  - [x] 7.3 Wire up all components
    - Connect keycode parser to key definitions
    - Connect modifier state to virtual keyboard
    - Connect renderer state to visual updates
    - Ensure message routing is complete
  - [x] 7.4 Add logging and error handling
    - Log warnings for Unicode fallback usage
    - Log errors for protocol failures
    - Use `tracing` crate for consistent logging
  - [x] 7.5 Ensure integration tests pass
    - Run ONLY the 4-5 tests written in 7.1
    - Verify complete flow works end-to-end

**Acceptance Criteria:**
- The 4-5 tests written in 7.1 pass
- Virtual keyboard initializes with applet
- All components work together seamlessly
- Errors are handled gracefully with logging

---

### Testing

#### Task Group 8: Test Review and Gap Analysis
**Dependencies:** Task Groups 1-7

- [x] 8.0 Review existing tests and fill critical gaps only
  - [x] 8.1 Review tests from Task Groups 1-7
    - Review the 4-6 tests from Task 1.1 (input module)
    - Review the 3-4 tests from Task 2.1 (layout schema)
    - Review the 4-6 tests from Task 3.1 (virtual keyboard)
    - Review the 4-5 tests from Task 4.1 (modifier state)
    - Review the 5-6 tests from Task 5.1 (key press flow)
    - Review the 2-3 tests from Task 6.1 (visual feedback)
    - Review the 4-5 tests from Task 7.1 (integration)
    - Total existing tests: approximately 26-35 tests
  - [x] 8.2 Analyze test coverage gaps for THIS feature only
    - Identify critical user workflows that lack test coverage
    - Focus ONLY on gaps related to basic key input requirements
    - Do NOT assess entire application test coverage
    - Prioritize edge cases: unicode fallback, protocol errors
  - [x] 8.3 Write up to 10 additional strategic tests maximum
    - Add tests for identified critical gaps
    - Focus on integration points between modules
    - Skip exhaustive edge case testing
    - Prioritize: protocol error recovery, keymap edge cases
  - [x] 8.4 Run feature-specific tests only
    - Run ONLY tests related to basic key input feature
    - Expected total: approximately 36-45 tests maximum
    - Do NOT run the entire application test suite
    - Verify critical workflows pass

**Acceptance Criteria:**
- All feature-specific tests pass (approximately 36-45 tests total)
- Critical user workflows for key input are covered
- No more than 10 additional tests added when filling gaps
- Testing focused exclusively on this spec's requirements

---

## Execution Order

Recommended implementation sequence:

1. **Foundation (Parallel)**
   - Task Group 1: Input Module Architecture
   - Task Group 2: Layout Schema Updates

2. **Protocol Layer**
   - Task Group 3: Virtual Keyboard Protocol Integration

3. **State Layer**
   - Task Group 4: Modifier State Tracking in Renderer

4. **Event Layer**
   - Task Group 5: Key Press Event Flow

5. **Visual Layer**
   - Task Group 6: Visual Modifier State Indication

6. **Integration**
   - Task Group 7: End-to-End Integration

7. **Testing**
   - Task Group 8: Test Review and Gap Analysis

---

## Key Files to Modify/Create

### New Files
- `src/input/mod.rs` - Module root with public API
- `src/input/keycode.rs` - Keycode parsing logic
- `src/input/modifier.rs` - Modifier state management
- `src/input/virtual_keyboard.rs` - Wayland protocol handling

### Modified Files
- `src/lib.rs` - Add input module export
- `src/layout/types.rs` - Add stickyrelease field to Key struct
- `src/layout/validation.rs` - Update validation if needed
- `src/renderer/state.rs` - Add active_modifiers to KeyboardRenderer
- `src/applet/mod.rs` - Handle key press/release for input emission
- `Cargo.toml` - Add wayland/xkbcommon dependencies
- `resources/layouts/example_qwerty.json` - Add stickyrelease examples

---

## Dependencies (External Crates)

- `wayland-client` - Wayland protocol communication
- `xkbcommon` - XKB keymap and keysym handling
- Protocol support for `zwp_virtual_keyboard_v1`

---

## Notes

- The existing `sticky_keys_active` HashSet in `KeyboardRenderer` should be coordinated with the new `active_modifiers` tracking
- The `sticky_active_color()` theme function already exists and should work for visual feedback
- Key repeat is explicitly OUT OF SCOPE (conflicts with existing hold-for-secondary feature)
- Unicode fallback uses Ctrl+Shift+U hex input method (standard Linux/GTK method)
