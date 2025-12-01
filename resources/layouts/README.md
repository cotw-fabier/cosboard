# Keyboard Layout JSON Schema Documentation

This directory contains keyboard layout definitions for Cosboard. Layouts are defined in JSON format and support a rich set of features including panels, keys, widgets, alternatives (modifiers and swipes), and inheritance.

## Schema Overview

A keyboard layout JSON file has the following top-level structure:

```json
{
  "name": "Layout Name",
  "description": "Optional description",
  "author": "Optional author name",
  "language": "en",
  "locale": "en_US",
  "version": "1.0",
  "default_panel_id": "main",
  "inherits": "path/to/parent/layout.json",
  "panels": [ /* array of panel objects */ ]
}
```

### Root Object Fields

- **name** (required, string): Human-readable name of the layout
- **version** (required, string): Layout format version (currently "1.0")
- **default_panel_id** (required, string): ID of the panel to show by default
- **description** (optional, string): Brief description of the layout
- **author** (optional, string): Layout author name
- **language** (optional, string): ISO 639 language code (e.g., "en", "fr", "de")
- **locale** (optional, string): Full locale identifier (e.g., "en_US", "fr_FR")
- **inherits** (optional, string): Path to parent layout file for inheritance
- **panels** (required, array): Array of panel objects

## Panel Structure

Panels are container objects that group rows of keys/widgets:

```json
{
  "id": "main",
  "padding": 8.0,
  "margin": 4.0,
  "rows": [ /* array of row objects */ ]
}
```

### Panel Fields

- **id** (required, string): Unique identifier for the panel (used for panel switching and inheritance)
- **padding** (optional, number): Inner padding in pixels (DPI-aware)
- **margin** (optional, number): Outer margin in pixels (DPI-aware)
- **rows** (required, array): Array of row objects containing cells

## Row Structure

Rows contain cells, where each cell can be a key, widget, or panel reference:

```json
{
  "cells": [
    {"type": "key", ...},
    {"type": "widget", ...},
    {"type": "panel_ref", ...}
  ]
}
```

## Cell Types

### Key Cell

Keys are the primary interactive elements:

```json
{
  "type": "key",
  "label": "A",
  "code": {"Unicode": "a"},
  "identifier": "key_a",
  "width": {"Relative": 1.0},
  "height": {"Relative": 1.0},
  "min_width": 50,
  "min_height": 40,
  "sticky": false,
  "alternatives": {
    "SingleModifier": ["Shift", {"Character": "A"}],
    "Swipe": ["Up", {"Character": "1"}]
  }
}
```

#### Key Fields

- **type** (required): Must be "key"
- **label** (required, string): Display text shown on the key
- **code** (required, object): Key code to send when pressed
  - `{"Unicode": "a"}` for regular characters
  - `{"Keysym": "Shift_L"}` for special keys (modifiers, function keys)
- **identifier** (optional, string): Unique ID for inheritance and scripting
- **width** (optional, object): Key width sizing
  - `{"Relative": 1.0}` for relative sizing (default: 1.0)
  - `{"Pixels": "100px"}` for fixed pixel width (DPI-aware)
- **height** (optional, object): Key height sizing (same format as width)
- **min_width** (optional, number): Minimum width in pixels
- **min_height** (optional, number): Minimum height in pixels
- **sticky** (optional, boolean): If true, key acts as toggle (tap to hold, tap again to release)
- **alternatives** (optional, object): Alternative actions for modifiers and swipes

#### Alternatives

Alternatives define what happens when a key is pressed with modifiers or swiped:

```json
"alternatives": {
  "SingleModifier": ["Shift", {"Character": "A"}],
  "ModifierCombo": [["Ctrl", "Shift"], {"KeyCode": {"Keysym": "Tab"}}],
  "Swipe": ["Up", {"Character": "@"}],
  "Swipe": ["Down", {"PanelSwitch": "symbols"}],
  "Swipe": ["Left", {"Script": "custom_macro"}]
}
```

**Modifier types:**
- `SingleModifier`: One modifier key (Shift, Ctrl, Alt, Super)
- `ModifierCombo`: Array of modifiers pressed together

**Swipe directions:** Up, Down, Left, Right

**Action types:**
- `{"Character": "A"}`: Send a single character
- `{"KeyCode": {"Unicode": "a"}}`: Send a key code
- `{"KeyCode": {"Keysym": "Return"}}`: Send a keysym
- `{"PanelSwitch": "panel_id"}`: Switch to another panel
- `{"Script": "script_name"}`: Execute a script (future feature)

### Widget Cell

Widgets are special UI elements like trackpads or prediction bars:

```json
{
  "type": "widget",
  "widget_type": "trackpad",
  "width": {"Relative": 2.0},
  "height": {"Relative": 2.0}
}
```

#### Widget Fields

- **type** (required): Must be "widget"
- **widget_type** (required, string): Type of widget
  - "trackpad": Touch-sensitive cursor control area
  - "prediction_bar": Text prediction/autocomplete bar
  - Additional types may be added in future
- **width** (required, object): Widget width sizing
- **height** (required, object): Widget height sizing

### Panel Reference Cell

Panel references allow embedding one panel within another:

```json
{
  "type": "panel_ref",
  "panel_id": "numpad",
  "width": {"Relative": 3.0},
  "height": {"Relative": 3.0}
}
```

#### Panel Reference Fields

- **type** (required): Must be "panel_ref"
- **panel_id** (required, string): ID of the panel to embed
- **width** (required, object): Reference width sizing
- **height** (required, object): Reference height sizing

**Note:** Panel embedding has a maximum nesting depth of 5 levels. Circular references are detected and rejected.

## Layout Inheritance

Layouts can extend existing layouts using the `inherits` field:

```json
{
  "name": "Extended Layout",
  "version": "1.0",
  "inherits": "resources/layouts/base_layout.json",
  "default_panel_id": "main",
  "panels": [ /* panels that override or extend parent */ ]
}
```

### Inheritance Rules

1. **Metadata Override**: Child layout's name, description, author, version override parent
2. **Panel Override**: Panels with matching `id` replace parent panels completely
3. **Key Override**: Within panels, keys with matching `identifier` override parent keys
4. **Panel Addition**: Panels in child not in parent are added to the result
5. **Maximum Depth**: Inheritance chains can be up to 5 levels deep
6. **Circular Detection**: Circular inheritance references are detected and rejected

### Example: Inheritance Workflow

**Parent layout (base_qwerty.json):**
```json
{
  "name": "Base QWERTY",
  "version": "1.0",
  "default_panel_id": "main",
  "panels": [
    {
      "id": "main",
      "rows": [
        {
          "cells": [
            {"type": "key", "label": "Q", "code": {"Unicode": "q"}, "identifier": "key_q"}
          ]
        }
      ]
    }
  ]
}
```

**Child layout (extended_qwerty.json):**
```json
{
  "name": "Extended QWERTY",
  "version": "1.0",
  "inherits": "resources/layouts/base_qwerty.json",
  "default_panel_id": "main",
  "panels": [
    {
      "id": "main",
      "rows": [
        {
          "cells": [
            {
              "type": "key",
              "label": "Q",
              "code": {"Unicode": "q"},
              "identifier": "key_q",
              "alternatives": {
                "Swipe": ["Up", {"Character": "1"}]
              }
            }
          ]
        }
      ]
    },
    {
      "id": "numpad",
      "rows": [ /* new panel added by child */ ]
    }
  ]
}
```

The resolved layout will have:
- Name: "Extended QWERTY" (overridden)
- Panel "main" from child (with swipe alternative on Q key)
- Panel "numpad" from child (newly added)

## Validation and Error Handling

The parser uses a permissive validation approach:

- **Fatal Errors**: Cause parsing to fail
  - Invalid JSON syntax
  - Missing required fields (name, version, default_panel_id, panel id)
  - Circular references (panels or inheritance)
  - Maximum nesting depth exceeded (> 5 levels)

- **Warnings**: Returned with the parsed layout
  - Missing optional fields (description, author)
  - Unusual sizing values (width > 10, height > 5)
  - Unreferenced panels

### Error Messages

All error messages include:
- Line numbers (for JSON syntax errors)
- Field paths (e.g., "panels[0].rows[1].cells[2]")
- Suggestions for fixes

Example error:
```
[ERROR] panels[0].rows[0].cells[0]: Missing required field "label"
  Suggestion: Add a "label" field with the text to display on the key
```

## Example Layouts

See the example layout files in this directory:

- **example_qwerty.json**: Comprehensive layout demonstrating all features
- **example_qwerty_base.json**: Simple parent layout for inheritance demonstration
- **example_qwerty_with_numpad.json**: Child layout extending the base with a numpad panel

## Usage in Code

To parse a layout in Rust code:

```rust
use cosboard::layout::{parse_layout_file, ParseResult};

// Parse a layout file
match parse_layout_file("resources/layouts/example_qwerty.json") {
    Ok(result) => {
        let layout = result.layout;
        println!("Loaded: {}", layout.name);

        // Check for warnings
        if result.has_warnings() {
            for warning in &result.warnings {
                eprintln!("Warning: {}", warning);
            }
        }

        // Access panels
        if let Some(panel) = layout.panels.get(&layout.default_panel_id) {
            println!("Default panel has {} rows", panel.rows.len());
        }
    }
    Err(e) => {
        eprintln!("Failed to parse: {}", e);
    }
}
```

## Best Practices

1. **Use Identifiers**: Assign unique `identifier` values to keys for inheritance and scripting
2. **Relative Sizing**: Prefer relative sizing over pixel sizing for better scaling across devices
3. **Panel Organization**: Use multiple panels for different keyboard modes (letters, symbols, numbers)
4. **Inheritance**: Use inheritance to create layout variations without duplicating common keys
5. **Alternatives**: Use shift alternatives for uppercase, swipes for numbers/symbols
6. **Sticky Keys**: Use sticky mode for modifier keys (Shift, Ctrl, Alt) in touch keyboards

## Future Extensions

- Scripting integration via `{"Script": "name"}` actions
- Long-press alternatives for additional characters
- Dynamic panel switching based on input context
- Layout hot-reloading during development
- Visual layout editor UI
