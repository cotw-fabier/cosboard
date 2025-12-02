# Spec Requirements: Basic Key Input

## Initial Description

Basic Key Input - Connect rendered keys to system input events. The scope includes:
- Regular key presses
- Modifier keys (Shift, Ctrl, Alt, Super/Meta)
- Combo keys (key combinations like Ctrl+C, Shift+A, etc.)

## Requirements Discussion

### First Round Questions

**Q1:** What keycode format should we use in the JSON layouts?
**Answer:** Keep the flexibility - maintain the current format that supports multiple formats:
- Single characters: `"q"`, `"a"`, `" "`
- XKB keysym names: `"Shift_L"`, `"BackSpace"`, `"Return"`
- Unicode codepoints: `"U+2022"`, `"U+03C0"`

**Q2:** Should modifier keys have a `stickyrelease` field in JSON to control whether they auto-release after the next non-modifier key?
**Answer:** Yes, add `stickyrelease` as a new field with default `true`. This allows:
- `"stickyrelease": true` (default) - modifier releases after next key press
- `"stickyrelease": false` - modifier stays active until explicitly tapped again

**Q3:** What should be explicitly out of scope for this spec?
**Answer:** The following are explicitly out of scope:
- Key repeat (explicitly out because they have hold for secondary key functions)
- Haptic/audio feedback
- Dead keys / compose sequences
- Macro recording

### Existing Code to Reference

**Similar Features Identified:**
- Feature: Layout Renderer - Path: `/home/fabier/Documents/code/cosboard/src/renderer/`
  - Contains KeyboardRenderer state management
  - Has key press detection and event handling structure
  - Provides the foundation this spec builds upon
- Feature: Layout Parser - Path: `/home/fabier/Documents/code/cosboard/src/layout/`
  - Defines layout types including key definitions
  - Contains validation logic for layout structures

### Follow-up Questions

No follow-up questions were needed. The user's answers were comprehensive and clear.

## Visual Assets

### Files Provided:
No visual assets provided.

### Visual Insights:
N/A

## Requirements Summary

### Functional Requirements

**Regular Key Input:**
- Parse keycode from key definition using flexible format (single char, XKB keysym, Unicode codepoint)
- Convert keycode to appropriate system input event
- Emit key press event when key is touched
- Emit key release event when key touch ends

**Modifier Key Handling:**
- Support standard modifiers: Shift, Ctrl, Alt, Super/Meta (left and right variants)
- Add `stickyrelease` JSON field to modifier key definitions (default: `true`)
- When `stickyrelease: true`: modifier activates on tap, releases after next non-modifier key
- When `stickyrelease: false`: modifier toggles on/off with each tap
- Visual indication of active modifier state

**Combo Key Support:**
- Allow multiple modifiers to be active simultaneously
- Combine active modifiers with next key press to emit combo (e.g., Ctrl+C)
- Clear sticky modifiers after combo is emitted

**Keycode Format Support:**
- Single characters: `"q"`, `"a"`, `" "` (space)
- XKB keysym names: `"Shift_L"`, `"BackSpace"`, `"Return"`, `"Tab"`, etc.
- Unicode codepoints: `"U+2022"` (bullet), `"U+03C0"` (pi), etc.

### Reusability Opportunities

- KeyboardRenderer's existing press/release detection can be extended
- Layout types already define key structure; add `stickyrelease` field
- May be able to reference existing Wayland virtual keyboard protocol implementations

### Scope Boundaries

**In Scope:**
- Regular key press/release events
- Modifier key handling with sticky behavior
- Combo key combinations (Ctrl+C, Shift+A, etc.)
- Multiple keycode format parsing (char, XKB keysym, Unicode)
- `stickyrelease` JSON field for modifier keys
- Visual feedback for active modifier state
- Integration with Wayland virtual keyboard protocol

**Out of Scope:**
- Key repeat functionality (reserved for hold/secondary key feature)
- Haptic or audio feedback on key press
- Dead keys and compose sequences (e.g., ` + e = e with grave)
- Macro recording and playback
- Long-press actions (already handled by renderer, separate from input)
- Swipe gestures (Phase 2 roadmap item)

### Technical Considerations

- Must use Wayland virtual keyboard protocol for input injection
- XKB keysym lookup required for keysym name support
- Unicode codepoint input may require special handling (compose or direct unicode input)
- Should integrate cleanly with existing KeyboardRenderer state management
- Modifier state should be tracked in renderer for visual feedback
- Consider using `xkbcommon` crate for keysym parsing if available
