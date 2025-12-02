# Spec Requirements: Layout Renderer

## Initial Description

Transform parsed JSON layouts into visual keyboard UI using libcosmic/Iced widgets. This is for the Cosboard project - a native soft keyboard for the COSMIC desktop environment.

## Requirements Discussion

### First Round Questions

**Q1:** How should keys be visually styled - should they use COSMIC theme colors, or have custom colors defined in the JSON layout?
**Answer:** Keys use COSMIC theme colors (background, accent, text)

**Q2:** What should be the default key sizing behavior - fixed pixel sizes, or proportional to keyboard dimensions?
**Answer:** Default sizing scales proportionally with keyboard dimensions

**Q3:** Should the renderer support custom pixel amounts for sizing?
**Answer:** Yes, custom pixel amounts supported via "px" suffix (e.g., "20px") with HDPI scaling

**Q4:** How should HDPI scaling be handled?
**Answer:** HDPI scaling uses COSMIC Desktop's scaling factor (1x, 1.5x, 2x, etc.)

**Q5:** Should the renderer respect panel padding and margin settings?
**Answer:** Yes, respect panel padding and margin settings

**Q6:** What content can keys display?
**Answer:** Keys have a label property that can include icons

**Q7:** What visual feedback should be provided on key press?
**Answer:** No animation for press state - instant state change

**Q8:** What is the long press threshold?
**Answer:** Long press threshold: 300ms

**Q9:** What happens on long press?
**Answer:** Long press reveals swipe gesture popups positioned around the key (up/down/left/right)

**Q10:** How should PanelRef cells behave?
**Answer:** PanelRef cells act as "switch to panel" buttons. Clicking triggers rebuild with new referenced panel.

**Q11:** What animation should be used for panel switching?
**Answer:** Slide animation: consistent direction (new panel slides in from right), 250ms duration

**Q12:** What happens if a referenced panel doesn't exist?
**Answer:** Display error, stay on current panel

**Q13:** How should errors and messages be displayed?
**Answer:** Small message area at bottom of keyboard surface. Toast-style notifications that auto-dismiss. Used for panel switch errors and potentially other keyboard messages.

**Q14:** How should widget placeholders be handled?
**Answer:** Implement placeholder widgets for "trackpad" and "autocomplete" with themed backgrounds and type labels

**Q15:** What is the scope boundary for this spec?
**Answer:** Focus on static rendering (appearance, sizing, arrangement). Key press handling and input emission deferred to "Basic Key Input" spec.

### Existing Code to Reference

**Similar Features Identified:**
- Feature: JSON Layout Parser - Path: `src/layout/` (recently completed, provides the parsed layout data structures)
- Feature: Applet Shell - Path: `src/applet/mod.rs` (existing keyboard applet structure)
- Feature: Layer Shell - Path: `src/layer_shell.rs` (layer-shell configuration for overlay behavior)

### Follow-up Questions

No follow-up questions were needed.

## Visual Assets

### Files Provided:

No visual assets provided.

### Visual Insights:

N/A - No visuals to analyze.

## Requirements Summary

### Functional Requirements

**Core Rendering:**
- Render keyboard layouts from parsed JSON definitions using libcosmic/Iced widgets
- Keys use COSMIC theme colors (background, accent, text)
- Default sizing scales proportionally with keyboard dimensions
- Support custom pixel amounts via "px" suffix (e.g., "20px")
- Apply HDPI scaling using COSMIC Desktop's scaling factor (1x, 1.5x, 2x, etc.)
- Respect panel padding and margin settings from layout definitions
- Keys display labels that can include icons

**Key Press Feedback:**
- Instant state change on press (no animation)
- Long press threshold: 300ms
- Long press reveals swipe gesture popups positioned around the key (up/down/left/right)

**Panel Switching:**
- PanelRef cells act as "switch to panel" buttons
- Clicking PanelRef triggers rebuild with new referenced panel
- Slide animation for panel transitions: new panel slides in from right, 250ms duration
- If referenced panel doesn't exist: display error, stay on current panel

**Error/Message Display:**
- Small message area at bottom of keyboard surface
- Toast-style notifications that auto-dismiss
- Used for panel switch errors and potentially other keyboard messages

**Widget Placeholders:**
- Implement placeholder widgets for "trackpad" and "autocomplete"
- Themed backgrounds with type labels

### Reusability Opportunities

- JSON Layout Parser (`src/layout/`): Provides `Layout`, `Panel`, `Row`, `Key` data structures to render
- Applet Shell (`src/applet/mod.rs`): Existing `cosmic::Application` implementation to integrate with
- Layer Shell (`src/layer_shell.rs`): Layer surface management for keyboard positioning
- COSMIC theming system for consistent visual styling
- libcosmic widget primitives (Button, Container, Row, Column, Text)

### Scope Boundaries

**In Scope:**
- Visual rendering of keyboard layouts from parsed JSON
- Key styling with COSMIC theme integration
- Proportional and pixel-based sizing with HDPI support
- Panel padding and margins
- Key labels with icon support
- Visual press state feedback (instant state change)
- Long press detection and swipe popup display
- Panel switching via PanelRef cells with slide animation
- Toast-style error/message notifications
- Placeholder widgets for trackpad and autocomplete

**Out of Scope (Deferred to "Basic Key Input" spec):**
- Key press event handling
- Input emission to system
- Actual key code/character generation
- Modifier key state management

**Out of Scope (Future phases):**
- Rhai scripting integration
- Word prediction/autocomplete functionality
- Actual trackpad functionality
- Speech-to-text integration

### Technical Considerations

- Integration with existing `cosmic::Application` trait implementation in `src/applet/mod.rs`
- Must work with layer-shell surface management in `src/layer_shell.rs`
- Consume parsed layout data structures from `src/layout/` module
- Use libcosmic/Iced widget primitives for rendering
- COSMIC theme integration for colors and styling
- HDPI scaling via COSMIC Desktop's scaling factor API
- Animation system needed for panel slide transitions (250ms duration)
- Timer/timeout handling for long press detection (300ms threshold)
- Toast notification system with auto-dismiss behavior
