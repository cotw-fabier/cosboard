# Specification: JSON Layout Parser

## Goal
Create a JSON layout parser that loads keyboard layout definitions from JSON files, supporting hierarchical structures (Layout → Panels → Rows → Keys), layout inheritance, widgets, embeddable panels, and comprehensive validation with helpful error messages.

## User Stories
- As a keyboard layout designer, I want to define custom keyboard layouts in JSON so that I can create layouts for different languages and use cases
- As a developer, I want to extend existing layouts through inheritance so that I can create layout variations without duplicating configuration
- As a user, I want the parser to provide helpful error messages with line numbers so that I can quickly fix layout definition issues

## Specific Requirements

**JSON Schema Structure**
- Root object contains metadata (name, description, author, language/locale, version, default_panel) and panels array
- Panel objects contain id, optional positioning hints (padding, margins), and rows array
- Row objects contain cells array where each cell is either a key, widget, or panel reference
- Key objects contain display label, key code (Unicode or keysym), optional identifier, width/height sizing, alternatives for modifiers and swipes, and optional actions
- Width/height use relative units (integer) or pixel overrides (string "20px") with DPI-aware scaling
- Support minWidth/minHeight constraints in pixels

**Key Definition and Alternatives**
- Keys support display label (what user sees), key code (Unicode preferred, keysyms for modifiers), and identifier for script references
- Alternatives object supports single modifiers (shift, ctrl, alt, super) and combinations (ctrl+shift, ctrl+alt+shift)
- Swipe alternatives (swipe_up, swipe_down, swipe_left, swipe_right) support characters, key codes, and script actions
- Sticky keys use toggle mode (tap to hold, tap again to release) with "sticky": true property
- Alternative values can be characters, key codes, script references ("script:custom_macro"), or panel switches ("panel(panel_name)")

**Widget and Panel Reference Support**
- Widgets have distinct JSON structure: {"type": "widget", "widget_type": "trackpad", "width": 3, "height": 2}
- Panel references have distinct JSON structure: {"type": "panel_ref", "panel_id": "numpad", "width": 2, "height": 3}
- Both widgets and panel references can appear in row cells where keys normally go
- Widget types are extensible strings (trackpad, autocomplete, prediction_bar, etc.)

**Layout Inheritance Mechanism**
- Root layout object supports optional "inherits" field with path to parent layout file
- Child layouts override keys, widgets, and panels by matching on IDs (identifiers)
- Parser resolves inheritance chain recursively up to maximum 5 levels deep
- Circular reference detection prevents infinite loops during inheritance resolution
- Override matching: keys match on identifier, panels match on id, widgets match on widget_type within same position

**Panel Configuration and Nesting**
- Panels require id/name field for referencing and switching
- Positioning hints (padding, margins) are suggestions to renderer, not strict requirements
- Support embeddable panels (panels within panels) through panel_ref cells
- Maximum nesting depth of 5 levels enforced during parsing
- Circular reference detection at parse time prevents panels from embedding themselves directly or indirectly
- Smart defaults for simple row-based structures when positioning hints omitted

**Permissive Validation and Error Handling**
- Parser operates in permissive mode: warns about issues but continues with sensible defaults
- Collect and return multiple errors simultaneously (not just first error encountered)
- Error messages include JSON line numbers and suggestions for fixes
- Validation checks: required fields present, width/height within valid ranges, modifier combinations valid, nesting depth under limit, no circular references
- Default values: width=1, height=1 for keys; empty alternatives object; Unicode space character for missing key codes
- Warning categories: missing optional fields, deprecated syntax, performance hints for large layouts

**Data Structure Output**
- Parser returns Result with Layout struct or Vec of ParseError
- Layout struct contains metadata fields, HashMap of Panel structs indexed by id, and default_panel_id
- Panel struct contains id, positioning hints (optional), nesting_depth tracking, and Vec of Row structs
- Row struct contains Vec of Cell enum variants (Key, Widget, PanelRef)
- Key struct contains label, code (enum: Unicode(char) or Keysym(String)), identifier (Option), sizing, alternatives HashMap, sticky bool
- Alternatives HashMap uses enum key (Modifier combination or SwipeDirection) and Action enum value (Character, KeyCode, Script, PanelSwitch)

**Integration Points**
- Parser module lives in src/layout/parser.rs with public parse_layout_file() function
- Uses serde and serde_json for JSON deserialization (already in Cargo.toml dependencies)
- Layout files stored in resources/layouts/ directory with .json extension
- Parser provides validation summary with warnings accessible through ParseResult struct
- Integration with cosmic_config for potential layout preferences in future
- Output Layout struct designed for consumption by future Layout Renderer component

**DPI-Aware Pixel Scaling**
- Parser stores pixel override strings as-is in Layout struct (e.g., "20px")
- Renderer responsible for applying DPI scaling factor at runtime
- Parser validates pixel string format (number followed by "px") but does not convert to pixels
- Sizing struct contains enum: Relative(f32) for relative units, Pixels(String) for override strings
- Documentation notes that pixel overrides use DPI-aware scaling when rendered

**Runtime Layout Loading**
- Support loading layouts at runtime from filesystem paths
- Parser function accepts file path or pre-loaded JSON string
- Enable hot-reloading of layouts during development without restart
- Layout cache mechanism recommended but not required in initial implementation
- Error handling distinguishes between file I/O errors and JSON parsing errors

## Visual Design

No visual mockups provided.

## Existing Code to Leverage

**CosmicConfigEntry Pattern (src/config.rs, src/state.rs)**
- Use #[derive(Debug, Clone, CosmicConfigEntry)] pattern for Layout struct if persisting user's selected layout to config
- Version annotation (#[version = 1]) enables schema evolution for layout format
- Demonstrates serde integration with cosmic_config system
- Consider CosmicConfigEntry for LayoutPreferences struct (selected layout name, last used panel)

**Serde Deserialization Pattern (Cargo.toml)**
- Project already includes serde with derive features enabled
- Use #[derive(Serialize, Deserialize)] on Layout, Panel, Row, Key structs
- Use serde field attributes for optional fields, defaults, and validation (#[serde(default)], #[serde(rename)])
- Pattern: permissive deserialization matches existing Config struct approach

**Error Handling Pattern (src/applet/mod.rs)**
- Follow cosmic::Application Task and Result patterns for async operations
- Use custom error enums with Display and Error trait implementations
- Structure: ParseError enum with variants for different error categories (IoError, JsonError, ValidationError, CircularReference)
- Include context in errors (file path, line number, field name)

**Module Organization (src/lib.rs)**
- Create src/layout/ directory for layout-related modules
- Split into parser.rs (parsing logic), types.rs (Layout/Panel/Key structs), validation.rs (validation rules), inheritance.rs (inheritance resolution)
- Public API exposed through src/layout/mod.rs re-exports
- Follow existing pattern of small, focused modules

**Resource Management (resources/ directory)**
- Store built-in layouts in resources/layouts/ directory
- Mirror resources/icons/ pattern for organizing embedded assets
- Use include_str! or similar for embedding default layouts in binary
- Allow external layout loading from XDG_DATA_HOME or system paths for user customization

## Out of Scope
- Layout rendering implementation (separate Layout Renderer spec)
- Key input handling and event dispatching (separate Basic Key Input spec)
- Gesture detection for long-press and swipes (Phase 2 feature)
- Rhai scripting engine integration and script execution (Phase 3 feature)
- Specific widget implementations like trackpad or autocomplete (separate widget specs)
- DPI calculation or detection logic (provided by libcosmic/Iced at runtime)
- Layout editor UI for visual layout creation (future enhancement)
- Layout validation beyond parse-time checks (runtime validation in renderer)
- Performance optimization like layout caching or lazy loading (can be added later)
- Migration tools for converting layouts between format versions (future utility)
