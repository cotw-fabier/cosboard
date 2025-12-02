# Specification: Layout Renderer

## Goal
Transform parsed JSON keyboard layouts into visual UI using libcosmic/Iced widgets, enabling users to see and interact with the rendered keyboard on the layer-shell surface.

## User Stories
- As a user, I want to see a visually rendered keyboard on screen so that I can use it for text input
- As a user, I want the keyboard to match my system's COSMIC theme so that it feels integrated with my desktop

## Specific Requirements

**Keyboard Layout Rendering**
- Consume `Layout` data structure from `src/layout/` module to render keyboard panels
- Render the `default_panel_id` panel on initial display
- Build widget tree using libcosmic Row, Column, and Button primitives
- Integrate rendered keyboard into existing `view_window()` method in applet

**Key Styling with COSMIC Theme**
- Use `cosmic::theme::active().cosmic()` to access theme colors
- Key backgrounds use theme's button/component background color
- Key text uses theme's text color for legibility
- Pressed state uses theme's accent color for visual feedback
- Sticky keys in active state show distinct accent-colored background

**Proportional Sizing System**
- Calculate base unit size from keyboard surface dimensions divided by row cell count
- `Sizing::Relative(1.0)` equals one base unit
- `Sizing::Relative(1.5)` equals 1.5 base units (e.g., Shift key)
- Recalculate sizes when keyboard surface is resized

**Pixel Sizing with HDPI Support**
- Parse `Sizing::Pixels("20px")` format from layout definitions
- Multiply pixel values by COSMIC scaling factor (1x, 1.5x, 2x)
- Access scale factor via `cosmic::app::cosmic::scale_factor()` or window API
- Apply min_width/min_height constraints after scaling

**Panel Padding and Margins**
- Apply `panel.padding` as inner spacing around the entire key grid
- Apply `panel.margin` as spacing between individual keys
- Use libcosmic Container with padding for panel-level spacing
- Use Row/Column spacing for inter-key margins

**Key Labels with Icons**
- Render key labels using `widget::text::body()` for text content
- Detect icon names (e.g., backspace, return symbols) and use `widget::icon::from_name()`
- Center labels both horizontally and vertically within key bounds
- Support Unicode symbols directly in labels (e.g., arrows, special characters)

**Key Press Visual Feedback**
- Track pressed state per-key in renderer state
- Instant background color change on press (no animation)
- Use theme accent color for pressed background
- Return to normal background on release

**Long Press Detection**
- Start 300ms timer on initial key press
- Cancel timer if key is released before threshold
- Set `long_press_active` flag when timer completes
- Long press state persists until finger/pointer is released

**Swipe Gesture Popups**
- Display popup overlay when long press is detected on keys with alternatives
- Position popup around the key: up/down/left/right based on available swipe directions
- Each popup cell shows the alternative action label
- Use semi-transparent overlay background behind popups
- Dismiss popup when pointer leaves the popup area or on release

**Panel Switching via PanelRef**
- Render `Cell::PanelRef` as a button with the referenced panel's ID as label
- On click, trigger panel switch to the referenced `panel_id`
- Validate panel exists before switching; if not, show error toast
- Do not switch panels if referenced panel is missing

**Panel Slide Animation**
- Animate panel transitions with 250ms duration
- New panel slides in from the right edge
- Use libcosmic/Iced animation primitives or manual interpolation
- Track animation progress in renderer state (0.0 to 1.0)
- Render both old and new panels during transition with offset transforms

**Toast Notification System**
- Reserve small area at bottom of keyboard surface for messages
- Display toast with semi-transparent themed background
- Auto-dismiss toasts after 3 seconds
- Queue multiple toasts if needed, showing one at a time
- Use for: panel switch errors, future keyboard status messages

**Widget Placeholders**
- Render `Cell::Widget` with `widget_type: "trackpad"` as placeholder container
- Render `Cell::Widget` with `widget_type: "autocomplete"` as placeholder container
- Use themed background color matching key background
- Display centered label showing widget type (e.g., "Trackpad", "Autocomplete")
- Respect width/height sizing from layout definition

**Renderer State Management**
- Create `KeyboardRenderer` struct to hold rendering state
- Store current panel ID, loaded layout, pressed key states
- Store animation state for panel transitions
- Store toast queue and current toast display state
- Integrate with `AppletModel` as a field

## Visual Design

No visual mockups provided. Refer to COSMIC desktop design language for styling guidance.

## Existing Code to Leverage

**JSON Layout Parser (`src/layout/`)**
- Provides `Layout`, `Panel`, `Row`, `Cell`, `Key`, `Widget`, `PanelRef` types
- Use `parse_layout_file()` or `parse_layout_from_string()` to load layouts
- `Sizing` enum already handles relative vs pixel values
- `AlternativeKey` enum defines swipe directions for popup alternatives

**Applet Shell (`src/applet/mod.rs`)**
- Existing `view_window()` method renders keyboard surface content
- Replace placeholder text with actual keyboard renderer output
- Use existing layer surface ID tracking (`keyboard_surface`)
- Follow existing pattern for widget composition with Row/Column

**Layer Shell (`src/layer_shell.rs`)**
- Surface sizing already managed by applet
- Keyboard dimensions available from `window_state.width` and `window_state.height`
- Use these dimensions for base unit calculations

**libcosmic Widget Primitives**
- `widget::button()` for key rendering with press handling
- `widget::container()` for padding and backgrounds
- `widget::text::body()` for key labels
- `cosmic::widget::{row, column}` for layout composition
- `cosmic::style::Container::custom()` for custom themed backgrounds

**COSMIC Theming System**
- `cosmic::theme::active()` returns current theme
- `.cosmic().accent_color()` for accent/highlight color
- Theme provides background, text, and component colors
- Use `cosmic::style::Button::custom()` for styled key buttons

## Out of Scope
- Key press event handling and input emission (deferred to "Basic Key Input" spec)
- Actual key code/character generation to system
- Modifier key state management and combination handling
- Rhai scripting integration for custom key actions
- Word prediction/autocomplete functionality (placeholder only)
- Actual trackpad touch/gesture functionality (placeholder only)
- Speech-to-text integration
- Layout editor or customization UI
- Keyboard shortcuts or hotkey configuration
- Sound feedback on key press
