# Spec Requirements: Cosboard Skeleton

## Initial Description
It is time to build out the initial skeleton for this app. Use ~/Documents/libraries/cosmic-app-template for the base template and ~/Documents/libraries/libcosmic for detailed information on how to utilize libcosmic. We need to build out the framework for the keyboard. This should enable a resizable window which floats above other windows. It should be aware of its width and height. It should have a menubar icon which contains a menu for displaying the keyboard and closing the soft keyboard. I think this will be enough to get us started, we will target additional functionality in the future.

## Requirements Discussion

### First Round Questions

**Q1:** Window type - Should the keyboard be a standard `cosmic::Application` window or a `cosmic::Applet` embedded in the panel?
**Answer:** Standard `cosmic::Application` (independent floating window)

**Q2:** System tray architecture - Should we use a standalone application with separate system tray applet (two processes), or a combined approach?
**Answer:** Option A - Standalone `cosmic::Application` window + separate system tray applet (two processes or combined approach)

**Q3:** Should we implement layer-shell integration for always-on-top behavior in this initial spec, or defer to a future spec?
**Answer:** Layer-shell integration is required in this initial spec. The window should be resizable AND always-on-top right out of the gate. This is important for proper function of a keyboard app.

**Q4:** What should be the default window size and should size constraints be configurable?
**Answer:** Default size of 800x300 pixels (configurable via centralized app_settings.rs). Minimum constraints should be customizable via keyboard JSON file.

**Q5:** How should window state (position/size) be persisted?
**Answer:** Save window position/size to cosmic_config state files. Update settings as the window is moved and resized (runtime position/size remembering). Users will move the keyboard around, so save settings as changes occur.

**Q6:** What window decorations are needed?
**Answer:** Borderless/chromeless with resize handles only.

**Q7:** System tray behavior - what actions for left-click and right-click?
**Answer:** Left-click shows keyboard, right-click opens context menu.

**Q8:** What functionality should be explicitly out of scope for this skeleton spec?
**Answer:** Keyboard layouts, key rendering, input handling (future specs). This spec focuses only on the window framework skeleton.

### Existing Code to Reference

**Similar Features Identified:**
- Template: `~/Documents/libraries/cosmic-app-template` - Base cosmic application template
- Library: `~/Documents/libraries/libcosmic` - Reference for libcosmic usage patterns

No similar features in the existing cosboard codebase were identified (this is the initial skeleton).

### Follow-up Questions

**Follow-up 1:** System tray architecture preference - Option A (standalone + separate applet) or Option B (applet-only)?
**Answer:** Option A - Standalone `cosmic::Application` window + separate system tray applet

**Follow-up 2:** Layer-shell implementation timing - Include in initial spec or defer?
**Answer:** Required in initial spec. Always-on-top behavior is essential for proper keyboard app function.

**Follow-up 3:** State persistence method preference?
**Answer:** Runtime position/size remembering using state files. Save settings as the window is moved and resized.

## Visual Assets

### Files Provided:
No visual assets provided.

### Visual Insights:
N/A - No visual files were found in the visuals folder.

## Requirements Summary

### Functional Requirements
- Create a floating keyboard window using `cosmic::Application`
- Implement Wayland layer-shell (`zwlr_layer_shell_v1`) for always-on-top behavior
- Window should be borderless/chromeless with resize handles only
- Window must be resizable and position-aware
- Create separate system tray applet component
- System tray left-click shows keyboard
- System tray right-click opens context menu
- Default window size: 800x300 pixels
- Persist window position and size to cosmic_config state files
- Update state files in real-time as window is moved/resized
- Size constraints configurable via JSON file

### Reusability Opportunities
- Base template at `~/Documents/libraries/cosmic-app-template`
- Reference implementation patterns from `~/Documents/libraries/libcosmic`
- Centralized settings in `app_settings.rs` for configuration management

### Scope Boundaries

**In Scope:**
- Window framework using `cosmic::Application`
- Layer-shell integration for always-on-top behavior
- Borderless/chromeless window with resize handles
- System tray applet with show/hide and context menu
- State persistence for window position and size
- Default dimensions and size constraints
- Basic application structure

**Out of Scope:**
- Keyboard layouts and key definitions
- Key rendering and visual keyboard display
- Input handling and key event emission
- Long-press, swipe, or gesture detection
- Word prediction or speech-to-text
- Any functionality beyond the skeleton framework

### Technical Considerations
- Must use Wayland layer-shell protocol (`zwlr_layer_shell_v1`) for always-on-top
- Two-component architecture: main application window + system tray applet
- Configuration centralized in `app_settings.rs`
- State persistence via cosmic_config
- Reference cosmic-app-template for base structure
- Reference libcosmic for implementation patterns
