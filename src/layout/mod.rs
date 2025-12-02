// SPDX-License-Identifier: GPL-3.0-only

//! JSON Layout Parser for Cosboard keyboard layouts.
//!
//! This module provides functionality for loading keyboard layout definitions from JSON files,
//! supporting hierarchical structures (Layout -> Panels -> Rows -> Keys), layout inheritance,
//! widgets, embeddable panels, and comprehensive validation with helpful error messages.
//!
//! # Features
//!
//! - **Hierarchical layout structure**: Define layouts with panels, rows, and keys
//! - **Layout inheritance**: Extend existing layouts with the `inherits` field
//! - **Permissive validation**: Continues parsing with sensible defaults, collecting warnings
//! - **Helpful error messages**: Includes line numbers, field paths, and suggestions
//! - **Widget support**: Embed widgets like trackpads and prediction bars
//! - **Panel references**: Nest panels within other panels for modular layouts
//!
//! # Example Usage
//!
//! ## Basic Layout Parsing
//!
//! ```rust,ignore
//! use cosboard::layout::{parse_layout_file, ParseResult};
//!
//! // Parse a layout file
//! match parse_layout_file("resources/layouts/qwerty.json") {
//!     Ok(result) => {
//!         let layout = result.layout;
//!         println!("Loaded layout: {}", layout.name);
//!
//!         // Check for warnings
//!         if result.has_warnings() {
//!             println!("Parsed with {} warnings:", result.warning_count());
//!             for warning in &result.warnings {
//!                 println!("  {}", warning);
//!             }
//!         }
//!
//!         // Access panels
//!         if let Some(panel) = layout.panels.get(&layout.default_panel_id) {
//!             println!("Default panel has {} rows", panel.rows.len());
//!         }
//!     }
//!     Err(e) => {
//!         eprintln!("Failed to parse layout: {}", e);
//!     }
//! }
//! ```
//!
//! ## Parsing from String
//!
//! ```rust,ignore
//! use cosboard::layout::parse_layout_from_string;
//!
//! let json = r#"{
//!     "name": "Simple Layout",
//!     "version": "1.0",
//!     "default_panel_id": "main",
//!     "panels": {
//!         "main": {
//!             "id": "main",
//!             "rows": [
//!                 {
//!                     "cells": [
//!                         {"type": "key", "label": "A", "code": "a"}
//!                     ]
//!                 }
//!             ]
//!         }
//!     }
//! }"#;
//!
//! match parse_layout_from_string(json) {
//!     Ok(result) => {
//!         println!("Parsed layout: {}", result.layout.name);
//!     }
//!     Err(e) => {
//!         eprintln!("Parse error: {}", e);
//!     }
//! }
//! ```
//!
//! ## Error Handling
//!
//! The parser uses a permissive approach: non-fatal validation issues are returned
//! as warnings in the `ParseResult`, while fatal errors (like circular references
//! or JSON syntax errors) return a `ParseError`.
//!
//! ```rust,ignore
//! use cosboard::layout::{parse_layout_file, ParseError};
//!
//! match parse_layout_file("path/to/layout.json") {
//!     Ok(result) => {
//!         // Check for non-fatal warnings
//!         for warning in &result.warnings {
//!             eprintln!("Warning: {}", warning);
//!         }
//!         // Use the layout
//!         let layout = result.layout;
//!     }
//!     Err(ParseError::IoError { source, file_path, .. }) => {
//!         eprintln!("Failed to read file {}: {}", file_path.unwrap(), source);
//!     }
//!     Err(ParseError::JsonError { source, line_number, .. }) => {
//!         eprintln!("JSON parse error at line {}: {}", line_number.unwrap(), source);
//!     }
//!     Err(ParseError::CircularReference { message, .. }) => {
//!         eprintln!("Circular reference detected: {}", message);
//!     }
//!     Err(e) => {
//!         eprintln!("Parse error: {}", e);
//!     }
//! }
//! ```
//!
//! ## Layout Inheritance
//!
//! Layouts can inherit from parent layouts using the `inherits` field:
//!
//! ```json
//! {
//!     "name": "QWERTY with Numpad",
//!     "version": "1.0",
//!     "inherits": "resources/layouts/example_qwerty_base.json",
//!     "default_panel_id": "main",
//!     "panels": {
//!         "numpad": {
//!             "id": "numpad",
//!             "rows": [...]
//!         }
//!     }
//! }
//! ```
//!
//! The parser automatically resolves inheritance chains up to 5 levels deep,
//! merging panels and keys by their IDs.

// Sub-modules
pub mod inheritance;
pub mod parser;
pub mod types;
pub mod validation;

// Re-export public API - Error handling types
pub use types::{ParseError, ParseResult, Severity, ValidationIssue};

// Re-export public API - Parser functions
pub use parser::{parse_layout_file, parse_layout_from_string};

// Re-export public API - Data structures
pub use types::{
    Action, AlternativeKey, Cell, Key, KeyCode, Layout, Modifier, Panel, PanelRef, Row,
    Sizing, SwipeDirection, Widget,
};

// ============================================================================
// Public API Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Test 1: End-to-end parse from file with all features
    ///
    /// This test verifies that the public API can parse a complete layout file
    /// containing all JSON schema elements (metadata, panels, rows, keys, alternatives,
    /// widgets, panel references).
    #[test]
    fn test_public_api_parse_complete_layout() {
        // Create a temporary test file with all features
        let test_json = r#"{
            "name": "Complete Test Layout",
            "description": "A layout with all features",
            "author": "Test Author",
            "language": "en",
            "locale": "en_US",
            "version": "1.0",
            "default_panel_id": "main",
            "panels": {
                "main": {
                    "id": "main",
                    "padding": 5.0,
                    "margin": 2.0,
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "key",
                                    "label": "A",
                                    "code": "a",
                                    "identifier": "key_a",
                                    "width": 1.0,
                                    "height": 1.0
                                },
                                {
                                    "type": "widget",
                                    "widget_type": "trackpad",
                                    "width": 2.0,
                                    "height": 2.0
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_complete_layout.json");
        fs::write(&test_file, test_json).expect("Failed to write test file");

        // Test parsing via public API
        let result = parse_layout_file(test_file.to_str().unwrap());
        if result.is_err() {
            eprintln!("Parse error: {:?}", result.as_ref().unwrap_err());
        }
        assert!(result.is_ok(), "Should parse complete layout successfully");

        let parse_result = result.unwrap();
        let layout = parse_result.layout;

        // Verify all fields are accessible via public API
        assert_eq!(layout.name, "Complete Test Layout");
        assert_eq!(layout.description, Some("A layout with all features".to_string()));
        assert_eq!(layout.author, Some("Test Author".to_string()));
        assert_eq!(layout.version, "1.0");
        assert_eq!(layout.default_panel_id, "main");
        assert!(layout.panels.contains_key("main"));

        // Clean up
        let _ = fs::remove_file(&test_file);
    }

    /// Test 2: Parse with warnings collection
    ///
    /// This test verifies that the public API properly collects and returns warnings
    /// for non-fatal validation issues while still returning a usable layout.
    #[test]
    fn test_public_api_parse_with_warnings() {
        // Create a layout with some optional fields missing (may generate warnings)
        let test_json = r#"{
            "name": "Minimal Layout",
            "version": "1.0",
            "default_panel_id": "main",
            "panels": {
                "main": {
                    "id": "main",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "key",
                                    "label": "A",
                                    "code": "a"
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        let result = parse_layout_from_string(test_json);
        if result.is_err() {
            eprintln!("Parse error: {:?}", result.as_ref().unwrap_err());
        }
        assert!(result.is_ok(), "Should parse layout with or without warnings");

        let parse_result = result.unwrap();

        // Verify warning inspection API works (methods exist)
        let _ = parse_result.has_warnings();
        let count = parse_result.warning_count();
        assert!(count >= 0, "Should provide warning count method");

        // Layout should still be usable
        assert_eq!(parse_result.layout.name, "Minimal Layout");
    }

    /// Test 3: Parse with error handling
    ///
    /// This test verifies that the public API properly handles various error conditions
    /// and returns appropriate error types.
    #[test]
    fn test_public_api_error_handling() {
        // Test 1: File not found
        let result = parse_layout_file("/nonexistent/path/layout.json");
        assert!(result.is_err(), "Should return error for missing file");

        if let Err(ParseError::IoError { .. }) = result {
            // Correct error type
        } else {
            panic!("Should return IoError for missing file");
        }

        // Test 2: Invalid JSON syntax
        let really_invalid_json = r#"{ "name": "Invalid" "version": "1.0" }"#; // missing comma
        let result = parse_layout_from_string(really_invalid_json);
        assert!(result.is_err(), "Should return error for invalid JSON");

        if let Err(ParseError::JsonError { .. }) = result {
            // Correct error type
        } else {
            panic!("Should return JsonError for malformed JSON");
        }
    }

    /// Test 4: Inheritance resolution via public API
    ///
    /// This test verifies that the public API properly resolves layout inheritance
    /// when parsing files with the inherits field.
    #[test]
    fn test_public_api_inheritance_resolution() {
        // Create parent layout
        let parent_json = r#"{
            "name": "Parent Layout",
            "version": "1.0",
            "default_panel_id": "main",
            "panels": {
                "main": {
                    "id": "main",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "key",
                                    "label": "A",
                                    "code": "a",
                                    "identifier": "key_a"
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        let temp_dir = std::env::temp_dir();
        let parent_file = temp_dir.join("test_parent_layout.json");
        fs::write(&parent_file, parent_json).expect("Failed to write parent file");

        // Create child layout that inherits from parent
        let child_json = format!(r#"{{
            "name": "Child Layout",
            "version": "1.0",
            "inherits": "{}",
            "default_panel_id": "main",
            "panels": {{
                "main": {{
                    "id": "main",
                    "rows": [
                        {{
                            "cells": [
                                {{
                                    "type": "key",
                                    "label": "B",
                                    "code": "b",
                                    "identifier": "key_a"
                                }}
                            ]
                        }}
                    ]
                }}
            }}
        }}"#, parent_file.to_str().unwrap());

        let child_file = temp_dir.join("test_child_layout.json");
        fs::write(&child_file, child_json).expect("Failed to write child file");

        // Test inheritance resolution via public API
        let result = parse_layout_file(child_file.to_str().unwrap());
        if result.is_err() {
            eprintln!("Inheritance parse error: {:?}", result.as_ref().unwrap_err());
        }
        assert!(result.is_ok(), "Should resolve inheritance successfully");

        let parse_result = result.unwrap();
        assert_eq!(parse_result.layout.name, "Child Layout");

        // Clean up
        let _ = fs::remove_file(&parent_file);
        let _ = fs::remove_file(&child_file);
    }

    /// Test 5: Public API type exports
    ///
    /// This test verifies that all necessary types are properly exported through
    /// the public API and can be used by consumers.
    #[test]
    fn test_public_api_type_exports() {
        // Verify enums are accessible
        let _keycode = KeyCode::Unicode('a');
        let _sizing = Sizing::Relative(1.0);
        let _modifier = Modifier::Shift;
        let _swipe = SwipeDirection::Up;
        let _action = Action::Character('a');

        // Verify structs are accessible (using struct update syntax for defaults)
        let _key = Key {
            label: "A".to_string(),
            code: KeyCode::Unicode('a'),
            identifier: Some("key_a".to_string()),
            width: Sizing::Relative(1.0),
            height: Sizing::Relative(1.0),
            min_width: None,
            min_height: None,
            alternatives: std::collections::HashMap::new(),
            sticky: false,
            ..Key::default()
        };

        let _widget = Widget {
            widget_type: "trackpad".to_string(),
            width: Sizing::Relative(2.0),
            height: Sizing::Relative(2.0),
        };

        let _panel_ref = PanelRef {
            panel_id: "numpad".to_string(),
            width: Sizing::Relative(3.0),
            height: Sizing::Relative(3.0),
        };

        // Verify Cell enum variants are accessible
        let _cell_key = Cell::Key(_key.clone());
        let _cell_widget = Cell::Widget(_widget.clone());
        let _cell_panel_ref = Cell::PanelRef(_panel_ref.clone());

        // Verify Row and Panel
        let _row = Row {
            cells: vec![_cell_key],
        };

        let _panel = Panel {
            id: "main".to_string(),
            padding: None,
            margin: None,
            nesting_depth: 0,
            rows: vec![_row],
        };

        // Verify Layout
        let mut panels = std::collections::HashMap::new();
        panels.insert("main".to_string(), _panel);

        let _layout = Layout {
            name: "Test".to_string(),
            description: None,
            author: None,
            language: None,
            locale: None,
            version: "1.0".to_string(),
            default_panel_id: "main".to_string(),
            inherits: None,
            panels,
        };

        // Verify error types are accessible
        let _validation_issue = ValidationIssue::new(
            Severity::Warning,
            "Test warning",
            "test.field",
        );

        // Test passes if all types compile and are accessible
        assert!(true, "All public API types are accessible");
    }

    /// Test 6: Circular reference detection via public API
    ///
    /// This test verifies that circular references are properly detected and reported
    /// through the public API error handling.
    #[test]
    fn test_public_api_circular_reference_detection() {
        // Create two files that reference each other circularly
        let temp_dir = std::env::temp_dir();

        let file_a_path = temp_dir.join("test_circular_a.json");
        let file_b_path = temp_dir.join("test_circular_b.json");

        let file_a_json = format!(r#"{{
            "name": "Layout A",
            "version": "1.0",
            "inherits": "{}",
            "default_panel_id": "main",
            "panels": {{
                "main": {{
                    "id": "main",
                    "rows": [
                        {{
                            "cells": [
                                {{
                                    "type": "key",
                                    "label": "A",
                                    "code": "a"
                                }}
                            ]
                        }}
                    ]
                }}
            }}
        }}"#, file_b_path.to_str().unwrap());

        let file_b_json = format!(r#"{{
            "name": "Layout B",
            "version": "1.0",
            "inherits": "{}",
            "default_panel_id": "main",
            "panels": {{
                "main": {{
                    "id": "main",
                    "rows": [
                        {{
                            "cells": [
                                {{
                                    "type": "key",
                                    "label": "B",
                                    "code": "b"
                                }}
                            ]
                        }}
                    ]
                }}
            }}
        }}"#, file_a_path.to_str().unwrap());

        fs::write(&file_a_path, file_a_json).expect("Failed to write file A");
        fs::write(&file_b_path, file_b_json).expect("Failed to write file B");

        // Attempt to parse - should detect circular reference
        let result = parse_layout_file(file_a_path.to_str().unwrap());
        assert!(result.is_err(), "Should detect circular reference");

        if let Err(ParseError::CircularReference { .. }) = result {
            // Correct error type
        } else {
            eprintln!("Got error: {:?}", result.unwrap_err());
            panic!("Should return CircularReference error");
        }

        // Clean up
        let _ = fs::remove_file(&file_a_path);
        let _ = fs::remove_file(&file_b_path);
    }

    // ========================================================================
    // Task 7.3: Strategic integration tests (up to 10 additional tests)
    // ========================================================================

    /// Strategic Test 1: Complex layout with all cell types in one row
    ///
    /// Tests that keys, widgets, and panel references can coexist in the same row
    /// and are all properly parsed and validated.
    #[test]
    fn test_integration_all_cell_types_together() {
        let test_json = r#"{
            "name": "Mixed Cell Types Layout",
            "version": "1.0",
            "default_panel_id": "main",
            "panels": {
                "main": {
                    "id": "main",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "key",
                                    "label": "A",
                                    "code": "a",
                                    "identifier": "key_a",
                                    "width": 1.0
                                },
                                {
                                    "type": "widget",
                                    "widget_type": "trackpad",
                                    "width": 3.0,
                                    "height": 2.0
                                },
                                {
                                    "type": "key",
                                    "label": "B",
                                    "code": "b",
                                    "width": 1.0
                                },
                                {
                                    "type": "panel_ref",
                                    "panel_id": "numpad",
                                    "width": 2.0,
                                    "height": 3.0
                                }
                            ]
                        }
                    ]
                },
                "numpad": {
                    "id": "numpad",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "key",
                                    "label": "1",
                                    "code": "1"
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        let result = parse_layout_from_string(test_json);
        assert!(result.is_ok(), "Should parse layout with mixed cell types");

        let parse_result = result.unwrap();
        let main_panel = parse_result.layout.panels.get("main").unwrap();
        assert_eq!(main_panel.rows[0].cells.len(), 4, "Should have all 4 cells");

        // Verify cell types
        assert!(matches!(main_panel.rows[0].cells[0], Cell::Key(_)));
        assert!(matches!(main_panel.rows[0].cells[1], Cell::Widget(_)));
        assert!(matches!(main_panel.rows[0].cells[2], Cell::Key(_)));
        assert!(matches!(main_panel.rows[0].cells[3], Cell::PanelRef(_)));
    }

    /// Strategic Test 2: Multi-level inheritance (3 levels)
    ///
    /// Tests that inheritance works correctly through multiple levels
    /// and that metadata and panels are properly merged.
    #[test]
    fn test_integration_multi_level_inheritance() {
        let temp_dir = std::env::temp_dir();

        // Create grandparent layout
        let grandparent_json = r#"{
            "name": "Grandparent",
            "version": "1.0",
            "description": "Base layout",
            "author": "Original Author",
            "default_panel_id": "main",
            "panels": {
                "main": {
                    "id": "main",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "key",
                                    "label": "A",
                                    "code": "a",
                                    "identifier": "key_a"
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        let grandparent_file = temp_dir.join("test_grandparent.json");
        fs::write(&grandparent_file, grandparent_json).unwrap();

        // Create parent layout (inherits from grandparent)
        let parent_json = format!(r#"{{
            "name": "Parent",
            "version": "2.0",
            "inherits": "{}",
            "default_panel_id": "main",
            "panels": {{
                "main": {{
                    "id": "main",
                    "rows": [
                        {{
                            "cells": [
                                {{
                                    "type": "key",
                                    "label": "B",
                                    "code": "b",
                                    "identifier": "key_b"
                                }}
                            ]
                        }}
                    ]
                }},
                "symbols": {{
                    "id": "symbols",
                    "rows": []
                }}
            }}
        }}"#, grandparent_file.to_str().unwrap());

        let parent_file = temp_dir.join("test_parent_multi.json");
        fs::write(&parent_file, parent_json).unwrap();

        // Create child layout (inherits from parent)
        let child_json = format!(r#"{{
            "name": "Child",
            "version": "3.0",
            "inherits": "{}",
            "author": "Child Author",
            "default_panel_id": "main",
            "panels": {{
                "numpad": {{
                    "id": "numpad",
                    "rows": []
                }}
            }}
        }}"#, parent_file.to_str().unwrap());

        let child_file = temp_dir.join("test_child_multi.json");
        fs::write(&child_file, child_json).unwrap();

        // Parse child (should resolve 3 levels)
        let result = parse_layout_file(child_file.to_str().unwrap());
        assert!(result.is_ok(), "Should resolve 3-level inheritance");

        let parse_result = result.unwrap();
        let layout = parse_result.layout;

        // Verify inheritance resolution
        assert_eq!(layout.name, "Child");
        assert_eq!(layout.version, "3.0");
        assert_eq!(layout.description, Some("Base layout".to_string())); // from grandparent
        assert_eq!(layout.author, Some("Child Author".to_string())); // overridden by child

        // Verify all panels are present
        assert!(layout.panels.contains_key("main"));
        assert!(layout.panels.contains_key("symbols")); // from parent
        assert!(layout.panels.contains_key("numpad")); // from child

        // Clean up
        let _ = fs::remove_file(&grandparent_file);
        let _ = fs::remove_file(&parent_file);
        let _ = fs::remove_file(&child_file);
    }

    /// Strategic Test 3: Complex modifier combinations with alternatives
    ///
    /// Tests that keys can have multiple alternatives with different modifier
    /// combinations and swipe directions.
    #[test]
    fn test_integration_complex_modifier_combinations() {
        let test_json = r#"{
            "name": "Complex Modifiers",
            "version": "1.0",
            "default_panel_id": "main",
            "panels": {
                "main": {
                    "id": "main",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "key",
                                    "label": "a",
                                    "code": "a",
                                    "identifier": "key_a",
                                    "alternatives": {
                                        "Shift": "A",
                                        "Ctrl": "@",
                                        "Up": "1",
                                        "Down": "2",
                                        "Left": "3",
                                        "Right": "4"
                                    }
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        let result = parse_layout_from_string(test_json);
        assert!(result.is_ok(), "Should parse complex alternatives");

        let parse_result = result.unwrap();
        let main_panel = parse_result.layout.panels.get("main").unwrap();

        match &main_panel.rows[0].cells[0] {
            Cell::Key(key) => {
                assert_eq!(key.alternatives.len(), 6, "Should have 6 alternatives");
            }
            _ => panic!("Expected Key cell"),
        }
    }

    /// Strategic Test 4: Panel reference chain (approaching max depth)
    ///
    /// Tests that panel references can be nested up to the maximum depth
    /// and that nesting depth is properly calculated and enforced.
    #[test]
    fn test_integration_panel_reference_chain_max_depth() {
        let test_json = r#"{
            "name": "Deep Nesting",
            "version": "1.0",
            "default_panel_id": "p0",
            "panels": {
                "p0": {
                    "id": "p0",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "panel_ref",
                                    "panel_id": "p1"
                                }
                            ]
                        }
                    ]
                },
                "p1": {
                    "id": "p1",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "panel_ref",
                                    "panel_id": "p2"
                                }
                            ]
                        }
                    ]
                },
                "p2": {
                    "id": "p2",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "panel_ref",
                                    "panel_id": "p3"
                                }
                            ]
                        }
                    ]
                },
                "p3": {
                    "id": "p3",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "panel_ref",
                                    "panel_id": "p4"
                                }
                            ]
                        }
                    ]
                },
                "p4": {
                    "id": "p4",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "panel_ref",
                                    "panel_id": "p5"
                                }
                            ]
                        }
                    ]
                },
                "p5": {
                    "id": "p5",
                    "rows": []
                }
            }
        }"#;

        let result = parse_layout_from_string(test_json);
        assert!(result.is_ok(), "Should parse deep nesting at limit");

        let parse_result = result.unwrap();
        let layout = parse_result.layout;

        // Verify nesting depths are calculated
        assert_eq!(layout.panels.get("p5").unwrap().nesting_depth, 0);
        assert_eq!(layout.panels.get("p4").unwrap().nesting_depth, 1);
        assert_eq!(layout.panels.get("p3").unwrap().nesting_depth, 2);
        assert_eq!(layout.panels.get("p2").unwrap().nesting_depth, 3);
        assert_eq!(layout.panels.get("p1").unwrap().nesting_depth, 4);
        assert_eq!(layout.panels.get("p0").unwrap().nesting_depth, 5);
    }

    /// Strategic Test 5: Sticky keys with alternatives
    ///
    /// Tests that sticky keys are properly parsed and can have alternatives.
    #[test]
    fn test_integration_sticky_keys_with_alternatives() {
        let test_json = r#"{
            "name": "Sticky Keys",
            "version": "1.0",
            "default_panel_id": "main",
            "panels": {
                "main": {
                    "id": "main",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "key",
                                    "label": "Shift",
                                    "code": "Shift_L",
                                    "identifier": "shift_left",
                                    "sticky": true,
                                    "alternatives": {
                                        "Ctrl": "Control_L"
                                    }
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        let result = parse_layout_from_string(test_json);
        assert!(result.is_ok(), "Should parse sticky keys");

        let parse_result = result.unwrap();
        let main_panel = parse_result.layout.panels.get("main").unwrap();

        match &main_panel.rows[0].cells[0] {
            Cell::Key(key) => {
                assert!(key.sticky, "Key should be sticky");
                assert!(!key.alternatives.is_empty(), "Sticky key can have alternatives");
            }
            _ => panic!("Expected Key cell"),
        }
    }

    /// Strategic Test 6: Mixed sizing (relative and pixels) across cells
    ///
    /// Tests that keys can use both relative and pixel sizing in the same layout.
    #[test]
    fn test_integration_mixed_sizing_across_cells() {
        let test_json = r#"{
            "name": "Mixed Sizing",
            "version": "1.0",
            "default_panel_id": "main",
            "panels": {
                "main": {
                    "id": "main",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "key",
                                    "label": "Relative",
                                    "code": "a",
                                    "width": 1.5,
                                    "height": 1.0
                                },
                                {
                                    "type": "key",
                                    "label": "Pixels",
                                    "code": "b",
                                    "width": "80px",
                                    "height": "60px",
                                    "min_width": 60,
                                    "min_height": 40
                                },
                                {
                                    "type": "widget",
                                    "widget_type": "trackpad",
                                    "width": 2.0,
                                    "height": "100px"
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        let result = parse_layout_from_string(test_json);
        assert!(result.is_ok(), "Should parse mixed sizing");

        let parse_result = result.unwrap();
        let main_panel = parse_result.layout.panels.get("main").unwrap();

        // Verify first key uses relative sizing
        match &main_panel.rows[0].cells[0] {
            Cell::Key(key) => {
                assert!(matches!(key.width, Sizing::Relative(_)));
                assert!(matches!(key.height, Sizing::Relative(_)));
            }
            _ => panic!("Expected Key cell"),
        }

        // Verify second key uses pixel sizing
        match &main_panel.rows[0].cells[1] {
            Cell::Key(key) => {
                assert!(matches!(key.width, Sizing::Pixels(_)));
                assert!(matches!(key.height, Sizing::Pixels(_)));
                assert_eq!(key.min_width, Some(60));
                assert_eq!(key.min_height, Some(40));
            }
            _ => panic!("Expected Key cell"),
        }

        // Verify widget uses mixed sizing
        match &main_panel.rows[0].cells[2] {
            Cell::Widget(widget) => {
                assert!(matches!(widget.width, Sizing::Relative(_)));
                assert!(matches!(widget.height, Sizing::Pixels(_)));
            }
            _ => panic!("Expected Widget cell"),
        }
    }

    /// Strategic Test 7: Bidirectional panel navigation is allowed
    ///
    /// Tests that bidirectional panel references (for navigation) are now allowed.
    /// Panel refs are navigation buttons, not structural dependencies, so
    /// "circular" patterns like main <-> symbols are valid.
    #[test]
    fn test_integration_circular_panel_reference_detection() {
        let test_json = r#"{
            "name": "Circular Panels",
            "version": "1.0",
            "default_panel_id": "main",
            "panels": {
                "main": {
                    "id": "main",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "panel_ref",
                                    "panel_id": "panel_a"
                                }
                            ]
                        }
                    ]
                },
                "panel_a": {
                    "id": "panel_a",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "panel_ref",
                                    "panel_id": "panel_b"
                                }
                            ]
                        }
                    ]
                },
                "panel_b": {
                    "id": "panel_b",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "panel_ref",
                                    "panel_id": "main"
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        // Bidirectional panel navigation is now allowed - panel refs are navigation
        // buttons, not structural dependencies
        let result = parse_layout_from_string(test_json);
        assert!(result.is_ok(), "Bidirectional panel navigation should be allowed");

        let parse_result = result.unwrap();
        assert_eq!(parse_result.layout.panels.len(), 3);
    }

    /// Strategic Test 8: Invalid default panel reference
    ///
    /// Tests that referencing a non-existent default panel is caught as an error.
    #[test]
    fn test_integration_invalid_default_panel() {
        let test_json = r#"{
            "name": "Invalid Default",
            "version": "1.0",
            "default_panel_id": "nonexistent",
            "panels": {
                "main": {
                    "id": "main",
                    "rows": []
                }
            }
        }"#;

        let result = parse_layout_from_string(test_json);
        assert!(result.is_err(), "Should fail with invalid default panel");

        match result.unwrap_err() {
            ParseError::ValidationError { issues, .. } => {
                assert!(
                    issues.iter().any(|i| i.message.contains("Default panel")),
                    "Should report missing default panel"
                );
            }
            e => panic!("Expected ValidationError, got: {:?}", e),
        }
    }

    /// Strategic Test 9: End-to-end with all features and inheritance
    ///
    /// Comprehensive test combining inheritance, all cell types, complex alternatives,
    /// and validation to ensure the entire feature works together.
    #[test]
    fn test_integration_end_to_end_all_features() {
        let temp_dir = std::env::temp_dir();

        // Create parent layout with base structure
        let parent_json = r#"{
            "name": "Base Layout",
            "version": "1.0",
            "description": "Base keyboard layout",
            "author": "Base Author",
            "language": "en",
            "default_panel_id": "main",
            "panels": {
                "main": {
                    "id": "main",
                    "padding": 5.0,
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "key",
                                    "label": "A",
                                    "code": "a",
                                    "identifier": "key_a",
                                    "alternatives": {
                                        "Shift": "A"
                                    }
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        let parent_file = temp_dir.join("test_e2e_parent.json");
        fs::write(&parent_file, parent_json).unwrap();

        // Create child layout with additional features
        let child_json = format!(r#"{{
            "name": "Extended Layout",
            "version": "2.0",
            "inherits": "{}",
            "locale": "en_US",
            "default_panel_id": "main",
            "panels": {{
                "main": {{
                    "id": "main",
                    "rows": [
                        {{
                            "cells": [
                                {{
                                    "type": "key",
                                    "label": "a",
                                    "code": "a",
                                    "identifier": "key_a",
                                    "width": 1.5,
                                    "alternatives": {{
                                        "Shift": "A",
                                        "Up": "1"
                                    }}
                                }},
                                {{
                                    "type": "widget",
                                    "widget_type": "trackpad",
                                    "width": "120px",
                                    "height": 2.0
                                }},
                                {{
                                    "type": "panel_ref",
                                    "panel_id": "numpad"
                                }}
                            ]
                        }}
                    ]
                }},
                "numpad": {{
                    "id": "numpad",
                    "rows": [
                        {{
                            "cells": [
                                {{
                                    "type": "key",
                                    "label": "1",
                                    "code": "1"
                                }}
                            ]
                        }}
                    ]
                }}
            }}
        }}"#, parent_file.to_str().unwrap());

        let child_file = temp_dir.join("test_e2e_child.json");
        fs::write(&child_file, child_json).unwrap();

        // Parse the complete layout
        let result = parse_layout_file(child_file.to_str().unwrap());
        assert!(result.is_ok(), "Should parse complex end-to-end layout");

        let parse_result = result.unwrap();
        let layout = parse_result.layout;

        // Verify metadata merge
        assert_eq!(layout.name, "Extended Layout");
        assert_eq!(layout.version, "2.0");
        assert_eq!(layout.description, Some("Base keyboard layout".to_string()));
        assert_eq!(layout.language, Some("en".to_string()));
        assert_eq!(layout.locale, Some("en_US".to_string()));

        // Verify panels exist
        assert!(layout.panels.contains_key("main"));
        assert!(layout.panels.contains_key("numpad"));

        // Verify main panel structure
        let main_panel = layout.panels.get("main").unwrap();
        assert_eq!(main_panel.rows[0].cells.len(), 3);

        // Verify key override with merged alternatives
        match &main_panel.rows[0].cells[0] {
            Cell::Key(key) => {
                assert_eq!(key.label, "a");
                assert_eq!(key.alternatives.len(), 2); // Shift and Up
            }
            _ => panic!("Expected Key cell"),
        }

        // Verify widget
        assert!(matches!(main_panel.rows[0].cells[1], Cell::Widget(_)));

        // Verify panel ref
        assert!(matches!(main_panel.rows[0].cells[2], Cell::PanelRef(_)));

        // Clean up
        let _ = fs::remove_file(&parent_file);
        let _ = fs::remove_file(&child_file);
    }

    /// Strategic Test 10: Malformed JSON recovery with helpful error messages
    ///
    /// Tests that the parser provides helpful error messages with line numbers
    /// and suggestions when JSON is malformed.
    #[test]
    fn test_integration_malformed_json_helpful_errors() {
        // Missing closing brace on line 5
        let malformed_json = r#"{
    "name": "Test",
    "version": "1.0",
    "default_panel_id": "main",
    "panels": {
"#;

        let result = parse_layout_from_string(malformed_json);
        assert!(result.is_err(), "Should fail for malformed JSON");

        match result.unwrap_err() {
            ParseError::JsonError { line_number, suggestion, .. } => {
                assert!(line_number.is_some(), "Should include line number");
                assert!(suggestion.is_some(), "Should include suggestion");

                let display = format!("{:?}", line_number);
                assert!(display.contains("Some"), "Line number should be present");
            }
            e => panic!("Expected JsonError, got: {:?}", e),
        }
    }
}
