# Spec Requirements: JSON Layout Parser

## Initial Description

Create a JSON layout parser to handle keyboard layouts. The parser needs to support:
- Panels
- Rows
- Keys with width/height parameters
- Key codes
- Alternative codes (for modifier combinations)
- Overriding functions for future scripting extensibility

The spec should define the JSON file format/schema and implement a parser to handle it.

## Requirements Discussion

### First Round Questions

**Q1:** I assume you want a hierarchical structure like Layout → Panels → Rows → Keys. Is that correct?

**Answer:** Hierarchical structure (Layout → Panels → Rows → Keys) works well ✓

**Q2:** For width/height parameters, I'm thinking relative units (like flexbox) where width: 1 is a standard key and width: 2 is double-wide. Should we support both relative units and optional pixel overrides?

**Answer:** Use relative units (width: 1) with optional pixel override (width: "20px"). Support optional minWidth/minHeight in pixels to prevent keys from getting too small ✓

**Q3:** For key representation, should we support: display label (what user sees), key code (what gets sent), and an optional key name/identifier for referencing in scripts?

**Answer:** Support display labels, key codes, and key names/identifiers ✓

**Q4:** For alternative codes with modifiers, I assume you want support for single modifiers (shift, ctrl, alt) and combinations (ctrl+shift). Should alternatives also support swipe directions (up, down, left, right)?

**Answer:** Support single modifiers AND combinations (e.g., "ctrl+shift"). Add sticky key support (holds key down when tapped). Alternatives should support swipe directions (up, down, left, right). Include "super" as a modifier ✓

**Q5:** For overriding functions/scripting, should we use simple string references like "script:custom_macro" that the future scripting system can resolve, or do you want a more structured approach?

**Answer:** Simple string references like "script:custom_macro". Also support "panel(panel_name)" for panel switching ✓

**Q6:** Should panels have properties beyond their contents, like positioning hints, IDs for switching, or styling attributes?

**Answer:** Panel properties: ID/name, positioning hints ✓

**Q7:** For the layout root object, should we include metadata like layout name, description, author, target language/locale, default panel to show, and version?

**Answer:** All layout metadata (name, description, author, language/locale, default panel, version) ✓

### Additional Requirements from Initial Discussion

**Widgets and Embeddable Panels:**
- Support **widgets** in place of keys (for custom displays like trackpad, autocomplete)
- Support **embeddable panels** (panels within panels) for flexible layouts
  - Example: "keyboard" panel can contain both "keys" panel and "numpad" panel
- Widgets and embeddable panels work the same way - they can appear where a key would normally go

### Existing Code to Reference

No similar existing features identified for reference. This is a new component for Phase 1 (MVP) of the product roadmap.

### Follow-up Questions

**Follow-up 1:** Should pixel overrides be absolute values or DPI-aware (scaled for high-density displays)?

**Answer:** Pixel overrides should use DPI-aware scaling ✓

**Follow-up 2:** For sticky keys - should they timeout after a period, or toggle mode (tap to hold, tap again to release)?

**Answer:** Sticky keys use toggle mode (tap to hold, tap again to release) ✓

**Follow-up 3:** For swipe alternatives, I'm thinking a structure like:
```json
"alternatives": {
  "swipe_up": "!",
  "swipe_down": "?",
  "swipe_left": "@",
  "swipe_right": "#",
  "shift": "A"
}
```
Does this work? Should swipe alternatives only support characters or also support key codes and script actions?

**Answer:** Swipe alternatives structure as proposed is correct. Should support key codes and script actions (not just characters) ✓

**Follow-up 4:** For panel embedding - what's a reasonable maximum nesting depth? Should we detect and reject circular references at parse time?

**Answer:** Max nesting depth: 5 levels. Detect and reject circular references at parse time ✓

**Follow-up 5:** For widget JSON structure, I'm thinking:
```json
{
  "type": "widget",
  "widget_type": "trackpad",
  "width": 3,
  "height": 2
}
```
Does this match your vision?

**Answer:** Widget JSON structure as proposed is correct ✓

**Follow-up 6:** For key codes - should we use Linux keysyms (like XKB_KEY_a), Unicode code points, or support both? What's your preference?

**Answer:** Support both Linux keysyms and Unicode code points. Unicode is preferred. For modifier keys, use best available solution (likely keysyms) ✓

**Follow-up 7:** For parser validation - should it be strict (reject layouts with any issues) or permissive (warn about issues but continue with defaults)?

**Answer:** Parser should be permissive (warn about issues but continue with defaults) ✓

**Follow-up 8:** For error messages - should they include line numbers and suggestions for fixes? Should parsing return multiple errors at once or stop at first error?

**Answer:** Error messages should include line numbers and suggestions. Parsing should return multiple errors at once ✓

**Follow-up 9:** Should the parser support layout composition/inheritance? For example, a "dvorak_with_numpad" layout that inherits from "dvorak" and adds a numpad panel?

**Answer:** IMPORTANT: Support layout inheritance! Inherited layouts override keys, widgets, and panels by matching on IDs. This should be included from the start ✓

**Follow-up 10:** For positioning hints on panels - are these strict requirements or suggestions to the layout renderer? Should we include smart defaults for simple row structures? Should we support padding and margins for spacing?

**Answer:** Positioning hints are suggestions (not strict requirements). Include smart defaults for simple row structures. Support padding and margins for spacing ✓

**Follow-up 11:** Should panel references (when embedding) and widgets have distinct JSON structures, or should they both use the same pattern?

**Answer:** Panel references and widgets should have distinct JSON structures (they serve different purposes) ✓

## Visual Assets

### Files Provided:

No visual files found.

### Visual Insights:

No visual assets provided.

## Requirements Summary

### Functional Requirements

**Core Parser Features:**
- Parse JSON layout files into structured data
- Support hierarchical structure: Layout → Panels → Rows → Keys
- Handle relative sizing units (width: 1) with optional pixel overrides (width: "20px")
- Pixel overrides use DPI-aware scaling
- Support minWidth/minHeight constraints in pixels
- Support display labels, key codes, and key identifiers
- Permissive validation: warn about issues but continue with defaults
- Error messages include line numbers and suggestions
- Return multiple errors at once (not just first error)

**Key Features:**
- Single and combined modifiers (shift, ctrl, alt, super, ctrl+shift, etc.)
- Sticky key support using toggle mode (tap to hold, tap again to release)
- Swipe direction alternatives (up, down, left, right) supporting characters, key codes, and script actions
- Script references ("script:custom_macro")
- Panel switching actions ("panel(panel_name)")
- Support both Linux keysyms and Unicode code points (Unicode preferred, keysyms for modifiers)

**Panel Features:**
- Panel IDs/names for referencing
- Positioning hints (suggestions, not strict requirements)
- Smart defaults for simple row structures
- Support padding and margins for spacing
- Embeddable panels (panels within panels)
- Maximum nesting depth: 5 levels
- Circular reference detection at parse time
- Panel references use distinct JSON structure from widgets

**Widget Support:**
- Widgets can appear in place of keys
- Support custom display widgets (trackpad, autocomplete, etc.)
- Widget JSON structure includes type, widget_type, width, height
- Widgets have distinct JSON structure from panel references

**Layout Metadata:**
- Name, description, author
- Language/locale targeting
- Default panel specification
- Version information

**Layout Inheritance:**
- Support layout composition/inheritance
- Child layouts can inherit from parent layouts
- Inherited layouts override keys, widgets, and panels by matching on IDs
- This is a core feature to be included from the start

### Reusability Opportunities

This is a foundational MVP feature. Future phases may reference:
- Phase 2: Long-press and swipe gesture detection will use the alternative codes defined in JSON
- Phase 3: Rhai scripting engine will resolve script references
- Phase 4: Widget system may use embeddable panels for prediction UI

### Scope Boundaries

**In Scope:**
- JSON schema definition
- Parser implementation with permissive validation
- Data structure for representing parsed layouts
- Validation of layout structure with helpful error messages
- DPI-aware scaling for pixel overrides
- Layout inheritance mechanism
- Circular reference detection
- Maximum nesting depth enforcement
- Support for widgets and panel references with distinct structures

**Out of Scope:**
- Rendering of layouts (separate feature: Layout Renderer)
- Key input handling (separate feature: Basic Key Input)
- Gesture detection implementation (Phase 2)
- Scripting engine integration (Phase 3)
- Specific widget implementations (trackpad, autocomplete, etc.)
- DPI calculation/detection (will be provided by the system/renderer)

### Technical Considerations

- Uses Rust with serde/serde_json for JSON parsing
- Must integrate with libcosmic/Iced UI framework
- Parser output should be usable by the Layout Renderer component
- Should support runtime layout loading
- Schema validation for user-created layouts (permissive mode)
- Error handling for malformed JSON files (collect and return multiple errors)
- Circular reference detection algorithm for panel embedding
- Nesting depth tracking during parse
- ID-based matching for layout inheritance
- Support both Linux keysyms and Unicode code points in key definitions
- DPI-aware pixel scaling (implementation will receive DPI from system)
