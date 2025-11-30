# Spec Initialization

## Date Created
2025-11-30

## Initial Description
It is time to build out the initial skeleton for this app. Use ~/Documents/libraries/cosmic-app-template for the base template and ~/Documents/libraries/libcosmic for detailed information on how to utilize libcosmic. We need to build out the framework for the keyboard. This should enable a resizable window which floats above other windows. It should be aware of its width and height. It should have a menubar icon which contains a menu for displaying the keyboard and closing the soft keyboard. I think this will be enough to get us started, we will target additional functionality in the future.

## Context
- This is a soft keyboard application (cosboard)
- Uses libcosmic library (COSMIC desktop environment UI toolkit)
- Base template at ~/Documents/libraries/cosmic-app-template
- Reference library at ~/Documents/libraries/libcosmic

## Finalized Requirements

### Window Architecture
- **Window Type**: Standard `cosmic::Application` (independent floating window)
- **Architecture**: Standalone `cosmic::Application` window + separate system tray applet (two processes or combined approach - Option A)

### Window Behavior
- **Always-on-Top**: Required - use Wayland layer-shell (`zwlr_layer_shell_v1`) for always-on-top behavior
- **Layer-Shell Integration**: Required in initial spec (not deferred)
- **Window Decorations**: Borderless/chromeless with resize handles only
- **Resizable**: Yes, with resize handles

### System Tray
- **Implementation**: Separate applet component
- **Left-click**: Shows keyboard
- **Right-click**: Opens context menu

### Dimensions
- **Default Size**: 800x300 pixels (configurable via centralized app_settings.rs)
- **Size Constraints**: Minimum constraints, customizable via keyboard JSON file

### State Persistence
- **Persistence Method**: Save window position/size to cosmic_config state files
- **Update Behavior**: Save settings as the window is moved and resized (runtime persistence)

### Base Resources
- **Template**: ~/Documents/libraries/cosmic-app-template
- **Library Reference**: ~/Documents/libraries/libcosmic

### Out of Scope
- Keyboard layouts
- Key rendering
- Input handling
- All functionality beyond the skeleton window framework

## Status
Requirements gathering complete. Ready for specification creation.
