# Task Breakdown: Layout Renderer

## Overview
Total Tasks: 46 (across 7 task groups)

This spec transforms parsed JSON keyboard layouts into visual UI using libcosmic/Iced widgets. The renderer consumes `Layout` data structures from `src/layout/` and renders them on the keyboard layer-shell surface.

## Task List

### Core Infrastructure

#### Task Group 1: Renderer State and Core Types
**Dependencies:** None (builds on existing `src/layout/` types)

- [x] 1.0 Complete renderer state infrastructure
  - [x] 1.1 Write 2-6 focused tests for KeyboardRenderer state
    - Test KeyboardRenderer initialization with Layout
    - Test pressed key state tracking (add/remove pressed keys)
    - Test panel switching state updates
    - Test animation progress state (0.0 to 1.0 range validation)
    - Test toast queue management (add, dismiss, queue order)
  - [x] 1.2 Create `src/renderer/mod.rs` module structure
    - Create module file with public exports
    - Add `renderer` module to `src/lib.rs`
    - Define module documentation
  - [x] 1.3 Create `src/renderer/state.rs` with KeyboardRenderer struct
    - Fields: `layout: Layout`
    - Fields: `current_panel_id: String`
    - Fields: `pressed_keys: HashSet<String>` (key identifiers)
    - Fields: `sticky_keys_active: HashSet<String>`
    - Fields: `long_press_key: Option<String>` (key being long-pressed)
    - Fields: `long_press_start: Option<Instant>`
    - Fields: `animation_state: Option<PanelAnimation>`
    - Fields: `toast_queue: VecDeque<Toast>`
    - Fields: `current_toast: Option<(Toast, Instant)>`
  - [x] 1.4 Create PanelAnimation struct for transition state
    - Fields: `from_panel_id: String`
    - Fields: `to_panel_id: String`
    - Fields: `progress: f32` (0.0 to 1.0)
    - Fields: `start_time: Instant`
    - Constant: `ANIMATION_DURATION_MS: u64 = 250`
  - [x] 1.5 Create Toast struct for notifications
    - Fields: `message: String`
    - Fields: `severity: ToastSeverity` (Info, Warning, Error)
    - Constant: `TOAST_DURATION_MS: u64 = 3000`
  - [x] 1.6 Implement KeyboardRenderer constructor and basic methods
    - `new(layout: Layout) -> Self`
    - `current_panel(&self) -> Option<&Panel>`
    - `is_key_pressed(&self, identifier: &str) -> bool`
    - `is_sticky_active(&self, identifier: &str) -> bool`
  - [x] 1.7 Ensure renderer state tests pass
    - Run ONLY the tests written in 1.1
    - Verify struct creation and method behavior

**Acceptance Criteria:**
- KeyboardRenderer struct holds all necessary rendering state
- Panel animation and toast types are defined
- Basic state query methods work correctly
- Tests from 1.1 pass

---

### Sizing and Theme System

#### Task Group 2: Sizing Calculations and Theme Integration
**Dependencies:** Task Group 1

- [x] 2.0 Complete sizing and theme system
  - [x] 2.1 Write 2-6 focused tests for sizing calculations
    - Test relative sizing calculation (1.0 = base unit)
    - Test relative sizing with multiplier (1.5x, 2.0x)
    - Test pixel sizing parsing ("20px" format)
    - Test pixel sizing with HDPI scaling (1x, 1.5x, 2x factors)
    - Test base unit calculation from surface dimensions
  - [x] 2.2 Create `src/renderer/sizing.rs` module
    - Function: `calculate_base_unit(surface_width: f32, surface_height: f32, row_cell_count: usize) -> f32`
    - Function: `resolve_sizing(sizing: &Sizing, base_unit: f32, scale_factor: f32) -> f32`
    - Function: `parse_pixels(pixel_str: &str) -> Option<f32>` (parses "20px" format)
  - [x] 2.3 Create `src/renderer/theme.rs` module
    - Function: `key_background_color(theme: &cosmic::Theme) -> Color`
    - Function: `key_pressed_color(theme: &cosmic::Theme) -> Color`
    - Function: `key_text_color(theme: &cosmic::Theme) -> Color`
    - Function: `sticky_active_color(theme: &cosmic::Theme) -> Color`
    - Function: `toast_background_color(theme: &cosmic::Theme) -> Color`
  - [x] 2.4 Add scale factor retrieval utility
    - Use `cosmic::app::cosmic::scale_factor()` or equivalent API
    - Provide fallback of 1.0 if unavailable
  - [x] 2.5 Ensure sizing and theme tests pass
    - Run ONLY the tests written in 2.1
    - Verify calculations produce expected values

**Acceptance Criteria:**
- Relative sizing correctly calculates pixel dimensions from base units
- Pixel sizing correctly parses "Npx" format and applies HDPI scaling
- Theme colors are retrieved from COSMIC theme system
- Tests from 2.1 pass

---

### Basic Rendering

#### Task Group 3: Key and Panel Rendering
**Dependencies:** Task Groups 1, 2

- [x] 3.0 Complete basic key and panel rendering
  - [x] 3.1 Write 2-6 focused tests for rendering functions
    - Test single key rendering produces valid Element
    - Test row rendering with multiple keys
    - Test panel rendering with padding/margin
    - Test key label rendering (text vs icon detection)
    - Test Cell::Widget placeholder rendering
    - Test Cell::PanelRef button rendering
  - [x] 3.2 Create `src/renderer/key.rs` for key rendering
    - Function: `render_key(key: &Key, state: &KeyboardRenderer, base_unit: f32, scale: f32) -> Element<Message>`
    - Use `widget::button()` for key container
    - Apply width/height from sizing calculations
    - Set background based on pressed/sticky state
    - Center label text within key bounds
  - [x] 3.3 Implement key label rendering with icon detection
    - Function: `render_label(label: &str) -> Element<Message>`
    - Detect icon names: "backspace", "return", "shift", "tab", etc.
    - Use `widget::icon::from_name()` for icons
    - Use `widget::text::body()` for text labels
    - Support Unicode symbols directly
  - [x] 3.4 Create `src/renderer/row.rs` for row rendering
    - Function: `render_row(row: &Row, state: &KeyboardRenderer, base_unit: f32, scale: f32) -> Element<Message>`
    - Use `cosmic::widget::row()` for horizontal layout
    - Apply margin spacing between cells
    - Handle Key, Widget, and PanelRef cell types
  - [x] 3.5 Create `src/renderer/panel.rs` for panel rendering
    - Function: `render_panel(panel: &Panel, state: &KeyboardRenderer, surface_width: f32, surface_height: f32, scale: f32) -> Element<Message>`
    - Use `widget::container()` with panel padding
    - Use `cosmic::widget::column()` for vertical row layout
    - Calculate base unit from surface dimensions and max row width
  - [x] 3.6 Implement widget placeholder rendering
    - Function: `render_widget_placeholder(widget: &Widget, base_unit: f32, scale: f32) -> Element<Message>`
    - Render container with themed background
    - Display centered label: "Trackpad" or "Autocomplete"
    - Respect width/height sizing
  - [x] 3.7 Implement PanelRef button rendering
    - Function: `render_panel_ref_button(panel_ref: &PanelRef, base_unit: f32, scale: f32) -> Element<Message>`
    - Render as button with panel_id as label
    - Apply width/height sizing
    - Emit `Message::SwitchPanel(panel_id)` on click
  - [x] 3.8 Ensure rendering tests pass
    - Run ONLY the tests written in 3.1
    - Verify Element generation works correctly

**Acceptance Criteria:**
- Keys render with correct sizing and theme colors
- Labels display text or icons appropriately
- Rows arrange cells horizontally with margins
- Panels arrange rows vertically with padding
- Widget placeholders show type label
- PanelRef buttons trigger panel switch
- Tests from 3.1 pass

---

### Interactive Features

#### Task Group 4: Press States, Long Press, and Popups
**Dependencies:** Task Group 3

- [x] 4.0 Complete interactive key features
  - [x] 4.1 Write 2-6 focused tests for interactive features
    - Test key press state change (pressed -> released)
    - Test long press timer start after 300ms threshold
    - Test long press cancellation on early release
    - Test popup positioning (up/down/left/right directions)
    - Test popup dismiss on pointer leave
  - [x] 4.2 Add Message variants for key interactions
    - `KeyPressed(String)` - key identifier
    - `KeyReleased(String)` - key identifier
    - `LongPressTimerTick` - periodic timer check
    - `PopupDismiss` - close active popup
  - [x] 4.3 Implement key press state tracking in KeyboardRenderer
    - Method: `press_key(&mut self, identifier: &str)`
    - Method: `release_key(&mut self, identifier: &str)`
    - Method: `start_long_press_timer(&mut self, identifier: &str)`
    - Method: `cancel_long_press(&mut self)`
    - Method: `check_long_press_threshold(&mut self) -> bool` (returns true if 300ms elapsed)
  - [x] 4.4 Implement visual feedback in key rendering
    - Update `render_key()` to check `state.is_key_pressed()`
    - Apply accent color background when pressed
    - Apply sticky_active_color when sticky key is active
    - Instant state change (no animation)
  - [x] 4.5 Create `src/renderer/popup.rs` for swipe gesture popups
    - Function: `render_popup(key: &Key, alternatives: &HashMap<AlternativeKey, Action>, position: PopupPosition) -> Element<Message>`
    - Struct: `PopupPosition { anchor_x: f32, anchor_y: f32, direction: SwipeDirection }`
    - Render popup cells for each swipe direction alternative
    - Use semi-transparent overlay background
  - [x] 4.6 Implement popup positioning logic
    - Function: `calculate_popup_position(key_bounds: Rectangle, available_directions: &[SwipeDirection]) -> PopupPosition`
    - Position popup around key based on available alternatives
    - Avoid positioning off-screen
  - [x] 4.7 Add subscription for long press timer
    - Return timer subscription when `long_press_key.is_some()`
    - Emit `LongPressTimerTick` periodically (e.g., every 50ms)
    - Check threshold in update handler
  - [x] 4.8 Ensure interactive feature tests pass
    - Run ONLY the tests written in 4.1
    - Verify press states and popup behavior

**Acceptance Criteria:**
- Keys show visual feedback on press (instant color change)
- Long press detected after 300ms hold
- Popup appears when long press completes on keys with alternatives
- Popup dismisses on pointer leave or release
- Tests from 4.1 pass

---

### Panel Switching and Animation

#### Task Group 5: Panel Transitions
**Dependencies:** Task Groups 3, 4

- [x] 5.0 Complete panel switching with animation
  - [x] 5.1 Write 2-6 focused tests for panel transitions
    - Test panel switch to valid panel_id
    - Test panel switch to invalid panel_id (error handling)
    - Test animation progress interpolation (0.0 -> 1.0)
    - Test animation completion callback
    - Test rendering during animation (both panels visible)
  - [x] 5.2 Add Message variants for panel switching
    - `SwitchPanel(String)` - target panel_id
    - `AnimationTick` - animation frame update
    - `AnimationComplete` - animation finished
  - [x] 5.3 Implement panel switch validation
    - Method: `KeyboardRenderer::switch_panel(&mut self, panel_id: &str) -> Result<(), String>`
    - Validate panel exists in layout.panels
    - Return error message if panel not found
    - Start animation if valid
  - [x] 5.4 Implement panel slide animation
    - Method: `KeyboardRenderer::start_animation(&mut self, to_panel_id: String)`
    - Method: `KeyboardRenderer::update_animation(&mut self, delta_ms: u64)`
    - Method: `KeyboardRenderer::is_animating(&self) -> bool`
    - Calculate progress: `elapsed_ms / ANIMATION_DURATION_MS`
    - New panel slides in from right edge
  - [x] 5.5 Update panel rendering for animation
    - Function: `render_animated_panels(state: &KeyboardRenderer, ...) -> Element<Message>`
    - Render both old and new panels during transition
    - Apply horizontal offset transforms based on progress
    - Old panel: offset from 0 to -width (exits left)
    - New panel: offset from +width to 0 (enters from right)
  - [x] 5.6 Add subscription for animation frames
    - Return animation subscription when `is_animating()`
    - Emit `AnimationTick` at ~60fps (every ~16ms)
    - Stop subscription when animation completes
  - [x] 5.7 Handle animation completion
    - Update `current_panel_id` to new panel
    - Clear animation state
    - Emit `AnimationComplete` message
  - [x] 5.8 Ensure panel transition tests pass
    - Run ONLY the tests written in 5.1
    - Verify smooth transitions and error handling

**Acceptance Criteria:**
- Panel switch validates target panel exists
- Invalid panel switch shows error, stays on current panel
- Animation runs for 250ms with smooth interpolation
- Both panels render during transition with offset transforms
- Tests from 5.1 pass

---

### Toast Notifications

#### Task Group 6: Toast System and Error Display
**Dependencies:** Task Group 5

- [x] 6.0 Complete toast notification system
  - [x] 6.1 Write 2-4 focused tests for toast system
    - Test toast queue ordering (FIFO)
    - Test toast auto-dismiss after 3 seconds
    - Test multiple toasts queued (show one at a time)
    - Test toast display area positioning
  - [x] 6.2 Add Message variants for toasts
    - `ShowToast(String, ToastSeverity)` - display a toast
    - `DismissToast` - remove current toast
    - `ToastTimerTick` - check toast timeout
  - [x] 6.3 Implement toast queue management in KeyboardRenderer
    - Method: `queue_toast(&mut self, message: String, severity: ToastSeverity)`
    - Method: `show_next_toast(&mut self)` - pop from queue, set as current
    - Method: `dismiss_current_toast(&mut self)`
    - Method: `check_toast_timeout(&mut self) -> bool` - returns true if 3s elapsed
  - [x] 6.4 Create `src/renderer/toast.rs` for toast rendering
    - Function: `render_toast(toast: &Toast, theme: &cosmic::Theme) -> Element<Message>`
    - Use semi-transparent themed background
    - Display message text centered
    - Apply severity-based color accent
  - [x] 6.5 Integrate toast area into keyboard surface
    - Reserve small area at bottom of keyboard surface
    - Function: `render_keyboard_with_toast(panel: Element, toast: Option<Element>) -> Element<Message>`
    - Use column layout: keyboard panel + toast area
    - Toast area collapses when no toast active
  - [x] 6.6 Connect panel switch errors to toast system
    - On invalid panel switch: `queue_toast("Panel 'X' not found", ToastSeverity::Error)`
    - Update `SwitchPanel` handler to queue toast on error
  - [x] 6.7 Add subscription for toast timer
    - Return timer subscription when `current_toast.is_some()`
    - Emit `ToastTimerTick` periodically
    - Dismiss and show next toast on timeout
  - [x] 6.8 Ensure toast system tests pass
    - Run ONLY the tests written in 6.1
    - Verify queue behavior and auto-dismiss

**Acceptance Criteria:**
- Toasts display at bottom of keyboard surface
- Toasts auto-dismiss after 3 seconds
- Multiple toasts queue and display sequentially
- Panel switch errors trigger toast notification
- Tests from 6.1 pass

---

### Integration

#### Task Group 7: Applet Integration and Testing
**Dependencies:** Task Groups 1-6

- [x] 7.0 Complete integration with applet
  - [x] 7.1 Add KeyboardRenderer field to AppletModel
    - Field: `keyboard_renderer: Option<KeyboardRenderer>`
    - Initialize when layout is loaded
    - Clear on layout unload
  - [x] 7.2 Load layout on keyboard surface creation
    - In `Message::Show` handler: parse layout file
    - Create KeyboardRenderer with parsed layout
    - Handle parse errors with toast notification
  - [x] 7.3 Update view_window() to use renderer
    - Replace placeholder text with `render_panel()` output
    - Pass surface dimensions for sizing calculations
    - Include toast area in layout
  - [x] 7.4 Wire up renderer messages to applet update()
    - Handle `KeyPressed`, `KeyReleased` messages
    - Handle `SwitchPanel`, `AnimationTick` messages
    - Handle `ShowToast`, `DismissToast` messages
  - [x] 7.5 Update applet subscription() for renderer
    - Merge renderer subscriptions (timers, animation)
    - Maintain existing drag/resize subscription logic
    - Return combined subscription when needed
  - [x] 7.6 Test end-to-end keyboard rendering
    - Load example layout file
    - Verify keyboard renders on layer surface
    - Verify key press visual feedback
    - Verify panel switching works

**Acceptance Criteria:**
- Keyboard renders actual layout when surface opens
- Key press visual feedback works in real applet
- Panel switching works with animation
- Toast notifications display on errors
- Existing drag/resize functionality preserved

---

### Test Review

#### Task Group 8: Test Review and Gap Analysis
**Dependencies:** Task Groups 1-7

- [x] 8.0 Review existing tests and fill critical gaps only
  - [x] 8.1 Review tests from Task Groups 1-7
    - Review the 2-6 tests written in each task group
    - Document total test count (expected: ~25-35 tests)
    - Identify any critical user workflows lacking coverage
  - [x] 8.2 Analyze test coverage gaps for layout renderer feature
    - Focus on integration points between renderer and applet
    - Check coverage of error paths (invalid layout, missing panel)
    - Verify theme integration is tested
  - [x] 8.3 Write up to 10 additional strategic tests if needed
    - Integration test: Full render pipeline (layout -> Element)
    - Integration test: Resize recalculates sizing
    - Integration test: Theme change updates colors
    - Error handling test: Malformed layout file
    - Error handling test: Missing default panel
  - [x] 8.4 Run feature-specific tests only
    - Run all renderer module tests
    - Run applet integration tests related to rendering
    - Expected total: approximately 30-45 tests
    - Verify all tests pass

**Acceptance Criteria:**
- All feature-specific tests pass
- Critical user workflows are covered
- No more than 10 additional tests added
- Testing focused on layout renderer feature requirements

---

## Execution Order

Recommended implementation sequence:

1. **Task Group 1: Renderer State and Core Types** - Foundation for all rendering
2. **Task Group 2: Sizing and Theme Integration** - Required for visual rendering
3. **Task Group 3: Key and Panel Rendering** - Core visual output
4. **Task Group 4: Press States, Long Press, and Popups** - Interactive feedback
5. **Task Group 5: Panel Transitions** - Panel switching with animation
6. **Task Group 6: Toast Notifications** - Error display system
7. **Task Group 7: Applet Integration** - Connect renderer to existing applet
8. **Task Group 8: Test Review** - Verify coverage and fill gaps

## File Structure

New files to create:
```
src/renderer/
  mod.rs          - Module exports
  state.rs        - KeyboardRenderer struct and state management
  sizing.rs       - Sizing calculations (relative, pixel, HDPI)
  theme.rs        - COSMIC theme color helpers
  key.rs          - Key rendering with labels/icons
  row.rs          - Row rendering with margin spacing
  panel.rs        - Panel rendering with padding
  message.rs      - Renderer message types
  panel_ref.rs    - Panel reference button rendering
  widget_placeholder.rs - Widget placeholder rendering
  popup.rs        - Long press swipe gesture popups
  toast.rs        - Toast notification rendering
```

Files to modify:
```
src/lib.rs                - Add renderer module
src/applet/mod.rs         - Integrate KeyboardRenderer, update view_window()
```

## Notes

- **Out of Scope**: Key press event handling and input emission (deferred to "Basic Key Input" spec)
- **Out of Scope**: Actual trackpad/autocomplete functionality (placeholder widgets only)
- **Performance**: Renderer must not block - animations use subscriptions, not blocking loops
- **Accessibility**: Consider contrast ratios when selecting theme colors (future enhancement)
