# Task Breakdown: JSON Layout Parser

## Overview
Total Tasks: 32 organized into 6 task groups

This implementation creates a permissive JSON layout parser with support for hierarchical structures (Layout → Panels → Rows → Keys), layout inheritance, widgets, embeddable panels, and comprehensive validation with helpful error messages.

## Task List

### Core Data Structures

#### Task Group 1: Foundation Types and Error Handling
**Dependencies:** None

- [x] 1.0 Complete foundation types and error handling
  - [x] 1.1 Write 2-8 focused tests for error handling
    - Limit to 2-8 highly focused tests maximum
    - Test only critical error scenarios (e.g., JSON parse error with line number, validation error with suggestion, circular reference detection)
    - Skip exhaustive coverage of all error types
  - [x] 1.2 Create src/layout/ directory structure
    - Create src/layout/mod.rs for public API exports
    - Create src/layout/types.rs for data structures
    - Create src/layout/parser.rs for parsing logic
    - Create src/layout/validation.rs for validation rules
    - Create src/layout/inheritance.rs for inheritance resolution
  - [x] 1.3 Define ParseError enum in types.rs
    - Variants: IoError(std::io::Error), JsonError(serde_json::Error), ValidationError(Vec<ValidationIssue>), CircularReference(String), MaxDepthExceeded(String)
    - Include context fields: file_path (Option<String>), line_number (Option<usize>), suggestion (Option<String>)
    - Implement Display and Error traits with helpful messages
    - Reuse pattern from: cosmic::Application error handling
  - [x] 1.4 Define ValidationIssue struct in types.rs
    - Fields: severity (enum: Error, Warning), message (String), line_number (Option<usize>), field_path (String), suggestion (Option<String>)
    - Implement Display trait for user-friendly output
  - [x] 1.5 Define ParseResult struct in types.rs
    - Fields: layout (Layout), warnings (Vec<ValidationIssue>)
    - Provides access to parsed layout and non-fatal validation issues
    - Methods: has_warnings(), warning_count()
  - [x] 1.6 Ensure foundation types tests pass
    - Run ONLY the 2-8 tests written in 1.1
    - Verify error types can be constructed and displayed
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 1.1 pass
- ParseError enum properly implements Display and Error traits
- ValidationIssue struct formats messages clearly
- ParseResult provides access to warnings

#### Task Group 2: Layout Data Structures
**Dependencies:** Task Group 1

- [x] 2.0 Complete layout data structures
  - [x] 2.1 Write 2-8 focused tests for data structures
    - Limit to 2-8 highly focused tests maximum
    - Test only critical struct behaviors (e.g., Layout creation with default panel, Panel with rows, Key with alternatives, sizing enum variants)
    - Skip exhaustive coverage of all struct methods
  - [x] 2.2 Define KeyCode enum in types.rs
    - Variants: Unicode(char), Keysym(String)
    - Use Unicode for regular characters, Keysym for modifiers
    - Implement Display trait for debugging
  - [x] 2.3 Define Sizing enum in types.rs
    - Variants: Relative(f32), Pixels(String)
    - Relative uses float for relative units (1.0 = standard size)
    - Pixels stores string like "20px" for DPI-aware rendering
    - Validation: pixel strings must match pattern r"^\d+px$"
  - [x] 2.4 Define Modifier enum and SwipeDirection enum in types.rs
    - Modifier variants: Shift, Ctrl, Alt, Super
    - SwipeDirection variants: Up, Down, Left, Right
    - Both derive Hash, Eq for use as HashMap keys
  - [x] 2.5 Define AlternativeKey enum in types.rs
    - Variants: SingleModifier(Modifier), ModifierCombo(Vec<Modifier>), Swipe(SwipeDirection)
    - ModifierCombo uses sorted Vec for consistent matching
    - Implement Hash and Eq for HashMap keys
  - [x] 2.6 Define Action enum in types.rs
    - Variants: Character(char), KeyCode(KeyCode), Script(String), PanelSwitch(String)
    - Script variant stores "script:custom_macro" format
    - PanelSwitch variant stores "panel(panel_name)" format
  - [x] 2.7 Define Key struct in types.rs
    - Fields: label (String), code (KeyCode), identifier (Option<String>), width (Sizing), height (Sizing), min_width (Option<u32>), min_height (Option<u32>), alternatives (HashMap<AlternativeKey, Action>), sticky (bool)
    - Derive Serialize, Deserialize with serde
    - Use #[serde(default)] for optional fields with sensible defaults
    - Defaults: width = Relative(1.0), height = Relative(1.0), sticky = false, alternatives = empty HashMap
  - [x] 2.8 Define Widget and PanelRef structs in types.rs
    - Widget fields: widget_type (String), width (Sizing), height (Sizing)
    - PanelRef fields: panel_id (String), width (Sizing), height (Sizing)
    - Both derive Serialize, Deserialize
  - [x] 2.9 Define Cell enum in types.rs
    - Variants: Key(Key), Widget(Widget), PanelRef(PanelRef)
    - Derive Serialize, Deserialize with serde tag
  - [x] 2.10 Define Row struct in types.rs
    - Fields: cells (Vec<Cell>)
    - Derive Serialize, Deserialize
  - [x] 2.11 Define Panel struct in types.rs
    - Fields: id (String), padding (Option<f32>), margin (Option<f32>), nesting_depth (u8), rows (Vec<Row>)
    - Derive Serialize, Deserialize
    - Use #[serde(default)] for optional positioning hints
  - [x] 2.12 Define Layout struct in types.rs
    - Fields: name (String), description (Option<String>), author (Option<String>), language (Option<String>), locale (Option<String>), version (String), default_panel_id (String), inherits (Option<String>), panels (HashMap<String, Panel>)
    - Derive Serialize, Deserialize, Clone
    - Consider CosmicConfigEntry for future persistence
    - Use #[serde(default)] for optional metadata
  - [x] 2.13 Ensure layout data structure tests pass
    - Run ONLY the 2-8 tests written in 2.1
    - Verify structs can be constructed with default values
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 2.1 pass
- All structs properly derive Serialize and Deserialize
- Enums implement required traits (Hash, Eq, Display)
- Default values work correctly via serde attributes

### JSON Parsing and Deserialization

#### Task Group 3: Core Parser Implementation
**Dependencies:** Task Groups 1-2

- [x] 3.0 Complete core parser implementation
  - [x] 3.1 Write 2-8 focused tests for parser
    - Limit to 2-8 highly focused tests maximum
    - Test only critical parser behaviors (e.g., parse valid JSON file, handle missing file, parse layout with panels and keys, parse alternatives)
    - Skip exhaustive coverage of all JSON schema variations
  - [x] 3.2 Implement parse_layout_file() in parser.rs
    - Function signature: pub fn parse_layout_file(path: &str) -> Result<ParseResult, ParseError>
    - Read file from filesystem, handle IoError with context
    - Distinguish between I/O errors and JSON parse errors
    - Use serde_json::from_str with line number tracking
  - [x] 3.3 Implement parse_layout_from_string() in parser.rs
    - Function signature: pub fn parse_layout_from_string(json: &str) -> Result<ParseResult, ParseError>
    - Parse pre-loaded JSON string
    - Use serde_json::from_str for deserialization
    - Return JsonError with line numbers on parse failure
  - [x] 3.4 Add JSON line number extraction to error handling
    - Use serde_json::Error::line() method for line numbers
    - Populate ParseError.line_number field
    - Include line number in error messages
  - [x] 3.5 Implement serde field attributes for permissive parsing
    - Add #[serde(default)] to optional fields across all structs
    - Add #[serde(rename)] where JSON keys differ from Rust field names
    - Use #[serde(skip_serializing_if)] for Option fields
    - Follow pattern from: Config struct in config.rs
  - [x] 3.6 Ensure core parser tests pass
    - Run ONLY the 2-8 tests written in 3.1
    - Verify parser can load valid JSON files
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 3.1 pass
- Parser successfully loads valid JSON files
- I/O errors and JSON parse errors are distinguished
- Line numbers are included in error messages

### Validation System

#### Task Group 4: Permissive Validation with Warnings
**Dependencies:** Task Groups 1-3

- [x] 4.0 Complete validation system
  - [x] 4.1 Write 2-8 focused tests for validation
    - Limit to 2-8 highly focused tests maximum
    - Test only critical validation behaviors (e.g., circular reference detection, max depth enforcement, required field validation, warning collection)
    - Skip exhaustive testing of all validation rules
  - [x] 4.2 Implement validate_required_fields() in validation.rs
    - Check Layout has: name, version, default_panel_id
    - Check Panel has: id
    - Check Key has: label, code
    - Add warnings for missing optional fields (description, author)
    - Provide default values: code = Unicode(' '), width/height = Relative(1.0)
  - [x] 4.3 Implement validate_sizing() in validation.rs
    - Check Relative values are positive (> 0.0)
    - Check Pixels strings match pattern r"^\d+px$"
    - Warn if width > 10 or height > 5 (unusually large keys)
    - Provide suggestions for invalid sizing values
  - [x] 4.4 Implement validate_modifier_combinations() in validation.rs
    - Check ModifierCombo variants are non-empty
    - Check for duplicate modifiers in combinations
    - Warn about unusual combinations (e.g., all four modifiers)
    - Sort modifiers in canonical order for consistent matching
  - [x] 4.5 Implement detect_circular_references() in validation.rs
    - Build dependency graph of panel references
    - Use depth-first search to detect cycles
    - Return CircularReference error if cycle found
    - Include panel chain in error message
  - [x] 4.6 Implement enforce_max_nesting_depth() in validation.rs
    - Recursively traverse panel references
    - Track nesting depth for each panel
    - Return MaxDepthExceeded error if depth > 5
    - Include panel path in error message
  - [x] 4.7 Implement validate_panel_references() in validation.rs
    - Check all PanelRef.panel_id values reference existing panels
    - Check default_panel_id references existing panel
    - Add warnings for unreferenced panels
    - Provide suggestions for typos in panel names
  - [x] 4.8 Implement collect_warnings() in validation.rs
    - Aggregate validation issues into Vec<ValidationIssue>
    - Separate errors (fatal) from warnings (non-fatal)
    - Sort warnings by severity and line number
    - Return ParseResult with layout and warnings
  - [x] 4.9 Ensure validation system tests pass
    - Run ONLY the 2-8 tests written in 4.1
    - Verify circular references are detected
    - Verify max depth is enforced
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 4.1 pass
- Circular reference detection works correctly
- Max nesting depth of 5 is enforced
- Warnings are collected and returned with ParseResult
- Validation is permissive (continues with defaults)

### Layout Inheritance

#### Task Group 5: Inheritance Resolution
**Dependencies:** Task Groups 1-4 (COMPLETED)

- [x] 5.0 Complete inheritance resolution
  - [x] 5.1 Write 2-8 focused tests for inheritance
    - Limit to 2-8 highly focused tests maximum
    - Test only critical inheritance behaviors (e.g., load parent layout, override key by identifier, override panel by id, detect circular inheritance)
    - Skip exhaustive testing of all override scenarios
  - [x] 5.2 Implement load_parent_layout() in inheritance.rs
    - Read parent layout path from Layout.inherits field
    - Recursively load parent layout using parse_layout_file()
    - Track inheritance depth (max 5 levels)
    - Return MaxDepthExceeded error if depth > 5
  - [x] 5.3 Implement detect_circular_inheritance() in inheritance.rs
    - Track visited layout file paths during recursive loading
    - Return CircularReference error if same file loaded twice
    - Include inheritance chain in error message
  - [x] 5.4 Implement merge_layouts() in inheritance.rs
    - Function signature: fn merge_layouts(child: Layout, parent: Layout) -> Layout
    - Copy parent layout as base
    - Override metadata fields from child (name, description, author, version)
    - Merge panels using override_panels()
  - [x] 5.5 Implement override_panels() in inheritance.rs
    - Iterate child panels, check for matching id in parent
    - If match found, override parent panel with child panel
    - If no match, add child panel to result
    - Preserve parent panels not overridden by child
  - [x] 5.6 Implement override_keys() in inheritance.rs
    - Within panel rows, iterate child keys
    - Match keys by identifier (if present)
    - Override matching parent keys with child keys
    - Preserve key position in row during override
  - [x] 5.7 Implement override_widgets() in inheritance.rs
    - Within panel rows, iterate child widgets
    - Match widgets by widget_type at same position in row
    - Override matching parent widgets with child widgets
  - [x] 5.8 Implement resolve_inheritance() in inheritance.rs
    - Public function: pub fn resolve_inheritance(layout: Layout) -> Result<Layout, ParseError>
    - Check if layout.inherits is Some
    - Load parent layout recursively
    - Merge layouts using merge_layouts()
    - Return fully resolved layout
  - [x] 5.9 Integrate resolve_inheritance() into parser.rs
    - Call resolve_inheritance() after initial JSON parsing
    - Handle inheritance errors and propagate to caller
    - Update parse_layout_file() to include inheritance resolution
  - [x] 5.10 Ensure inheritance resolution tests pass
    - Run ONLY the 2-8 tests written in 5.1
    - Verify parent layouts are loaded correctly
    - Verify overrides work by matching IDs
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 5.1 pass
- Parent layouts load recursively up to 5 levels
- Circular inheritance is detected and rejected
- Keys, panels, and widgets override correctly by ID matching
- Resolved layout has all inheritance flattened

### Integration and Module Exports

#### Task Group 6: Public API and Integration
**Dependencies:** Task Groups 1-5

- [x] 6.0 Complete public API and integration
  - [x] 6.1 Write 2-8 focused tests for public API
    - Limit to 2-8 highly focused tests maximum
    - Test only critical API behaviors (e.g., end-to-end parse from file, parse with warnings, inheritance resolution via public API)
    - Skip exhaustive integration scenarios
  - [x] 6.2 Implement public API re-exports in layout/mod.rs
    - Re-export parse_layout_file(), parse_layout_from_string()
    - Re-export Layout, Panel, Row, Key, Cell structs
    - Re-export ParseResult, ParseError, ValidationIssue
    - Re-export all enums: KeyCode, Sizing, Modifier, SwipeDirection, AlternativeKey, Action
    - Follow pattern from: src/lib.rs module structure
  - [x] 6.3 Add layout module to src/lib.rs
    - Add `pub mod layout;` declaration
    - Update lib.rs documentation with layout module description
  - [x] 6.4 Create resources/layouts/ directory
    - Create directory for built-in layouts
    - Add .gitkeep or example layout file
    - Mirror pattern from: resources/icons/
  - [x] 6.5 Create example layout JSON file
    - Create resources/layouts/example_qwerty.json
    - Include all JSON schema elements: metadata, panels, rows, keys, alternatives
    - Demonstrate width/height sizing (relative and pixels)
    - Demonstrate modifiers, swipes, sticky keys
    - Demonstrate widgets and panel references
    - Include comments in separate documentation file
  - [x] 6.6 Create example layout with inheritance
    - Create resources/layouts/example_qwerty_base.json (parent)
    - Create resources/layouts/example_qwerty_with_numpad.json (child)
    - Demonstrate inherits field usage
    - Demonstrate panel override by id
    - Demonstrate key override by identifier
  - [x] 6.7 Add integration documentation in layout/mod.rs
    - Module-level doc comments explaining usage
    - Example code showing parse_layout_file() usage
    - Example code showing error handling
    - Example code showing warning inspection
    - Link to JSON schema documentation
  - [x] 6.8 Ensure public API tests pass
    - Run ONLY the 2-8 tests written in 6.1
    - Verify public API functions are accessible
    - Verify example layouts parse successfully
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 6.1 pass
- Public API is properly exported through layout/mod.rs
- Layout module is integrated into src/lib.rs
- Example layout files parse successfully
- Documentation explains usage clearly

### Testing

#### Task Group 7: Test Review & Gap Analysis
**Dependencies:** Task Groups 1-6

- [x] 7.0 Review existing tests and fill critical gaps only
  - [x] 7.1 Review tests from Task Groups 1-6
    - Review the 2-8 tests written for foundation types (Task 1.1)
    - Review the 2-8 tests written for data structures (Task 2.1)
    - Review the 2-8 tests written for parser (Task 3.1)
    - Review the 2-8 tests written for validation (Task 4.1)
    - Review the 2-8 tests written for inheritance (Task 5.1)
    - Review the 2-8 tests written for public API (Task 6.1)
    - Total existing tests: 46 tests (8 + 8 + 8 + 8 + 8 + 6)
  - [x] 7.2 Analyze test coverage gaps for JSON Layout Parser feature only
    - Identify critical user workflows that lack test coverage
    - Focus ONLY on gaps related to this spec's feature requirements
    - Do NOT assess entire application test coverage
    - Prioritize end-to-end workflows over unit test gaps
    - Example critical workflows: parse complex layout with all features, handle malformed JSON gracefully, resolve multi-level inheritance
  - [x] 7.3 Write up to 10 additional strategic tests maximum
    - Add maximum of 10 new tests to fill identified critical gaps
    - Focus on integration points and end-to-end workflows
    - Do NOT write comprehensive coverage for all scenarios
    - Skip edge cases, performance tests unless business-critical
    - Example tests: parse layout with all cell types, deeply nested panels, complex modifier combinations
    - Added 10 strategic integration tests covering:
      1. Complex layout with all cell types in one row
      2. Multi-level inheritance (3 levels)
      3. Complex modifier combinations with alternatives
      4. Panel reference chain (approaching max depth)
      5. Sticky keys with alternatives
      6. Mixed sizing (relative and pixels) across cells
      7. Panel with circular panel reference detection
      8. Invalid default panel reference
      9. End-to-end with all features and inheritance
      10. Malformed JSON recovery with helpful error messages
  - [x] 7.4 Run feature-specific tests only
    - Run ONLY tests related to this spec's feature (tests from 1.1, 2.1, 3.1, 4.1, 5.1, 6.1, and 7.3)
    - Expected total: approximately 22-58 tests maximum
    - Do NOT run the entire application test suite
    - Verify critical workflows pass
    - Result: 56 tests passed successfully

**Acceptance Criteria:**
- All feature-specific tests pass (56 tests total)
- Critical user workflows for this feature are covered
- No more than 10 additional tests added when filling in testing gaps
- Testing focused exclusively on this spec's feature requirements

## Execution Order

Recommended implementation sequence:
1. Core Data Structures - Foundation Types (Task Group 1)
2. Core Data Structures - Layout Data Structures (Task Group 2)
3. JSON Parsing and Deserialization (Task Group 3)
4. Validation System (Task Group 4)
5. Layout Inheritance (Task Group 5)
6. Integration and Module Exports (Task Group 6)
7. Test Review & Gap Analysis (Task Group 7)

## Implementation Notes

### Key Architectural Decisions

**Permissive Validation Philosophy:**
- Parser continues with sensible defaults rather than failing on minor issues
- Warnings are collected and returned to caller for inspection
- Only fatal errors (circular references, max depth exceeded) cause parse failure

**Inheritance Resolution Strategy:**
- Inheritance is resolved recursively during parsing (not lazily)
- Final Layout struct contains fully flattened configuration
- ID-based matching allows flexible overrides without position dependency

**DPI-Aware Pixel Handling:**
- Parser stores pixel strings as-is ("20px")
- Renderer applies DPI scaling at runtime
- Parser validates format but does not convert to numeric pixels

**Module Organization:**
- Small, focused modules following Microsoft M-SMALLER-CRATES guideline
- Clear separation: types.rs (data), parser.rs (I/O), validation.rs (rules), inheritance.rs (resolution)
- Public API surface minimized through layout/mod.rs re-exports

### Integration Considerations

**Resources Directory:**
- Built-in layouts stored in resources/layouts/
- Follow pattern from resources/icons/ for consistency
- Consider use of include_str! for embedding layouts in binary

**Future Extensions:**
- Layout struct derives Clone for potential caching layer
- Consider CosmicConfigEntry for persisting user's selected layout preference
- Parser designed to support runtime hot-reloading from filesystem

**Error Message Quality:**
- All errors include line numbers when available
- Validation errors include suggestions for fixes
- Circular reference errors include full dependency chain

### Testing Strategy

**Focused Test-Driven Approach:**
- Each task group starts with writing 2-8 focused tests
- Tests verify only critical behaviors, not exhaustive coverage
- Final verification runs only newly written tests, not entire suite

**Test Organization:**
- Unit tests inline with modules (#[cfg(test)] mod tests)
- Integration tests in dedicated test group (Task Group 7)
- Example layouts serve as integration test fixtures

### Performance Considerations

**Parse-Time Optimizations:**
- Use HashMap for panel lookups (O(1) by id)
- Circular reference detection uses visited set (O(n))
- Max depth enforcement via single recursive traversal

**Memory Efficiency:**
- Inheritance resolution creates new Layout (no COW optimization in initial implementation)
- Consider Arc<Panel> for shared panel references in future optimization
- String interning for repeated panel_id values could reduce memory in large layouts

### Documentation Requirements

**Module-Level Documentation:**
- Explain JSON schema structure with examples
- Document inheritance resolution algorithm
- Provide error handling best practices

**Struct-Level Documentation:**
- Each struct includes doc comments with field descriptions
- Enums document all variants with usage examples
- Public functions include example code

### Future Enhancement Hooks

**Layout Caching:**
- parse_layout_file() returns ParseResult which can be cached
- Consider LRU cache keyed by file path hash

**Schema Versioning:**
- Layout.version field enables future format migrations
- Parser can route to different deserializers based on version

**Extended Validation:**
- Validation module designed for easy addition of new rules
- ValidationIssue severity enum extensible to Info level

**Script System Integration:**
- Action::Script variant stores opaque string for future resolution
- No script validation in parser (delegated to script engine)
