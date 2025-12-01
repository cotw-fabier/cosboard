// SPDX-License-Identifier: GPL-3.0-only

//! Layout parsing logic for loading JSON layout definitions.
//!
//! This module provides functions for parsing keyboard layout definitions from
//! JSON files and strings, with support for inheritance resolution and validation.

use crate::layout::inheritance::resolve_inheritance;
use crate::layout::types::{Layout, ParseError, ParseResult};
use crate::layout::validation::validate_layout;
use std::fs;

/// Parses a keyboard layout from a JSON file.
///
/// This function reads a layout file from the filesystem and parses it,
/// distinguishing between I/O errors (file not found, permission denied, etc.)
/// and JSON parsing errors (malformed JSON, missing required fields, etc.).
///
/// After parsing, it resolves any inheritance chain and validates the layout.
///
/// # Arguments
///
/// * `path` - Path to the JSON layout file
///
/// # Returns
///
/// Returns a `ParseResult` containing the parsed layout and any warnings,
/// or a `ParseError` if parsing fails.
///
/// # Example
///
/// ```rust,ignore
/// use cosboard::layout::parser::parse_layout_file;
///
/// match parse_layout_file("resources/layouts/qwerty.json") {
///     Ok(result) => {
///         println!("Loaded layout: {}", result.layout.name);
///         if result.has_warnings() {
///             println!("Warnings: {}", result.warning_count());
///         }
///     }
///     Err(e) => eprintln!("Failed to parse layout: {}", e),
/// }
/// ```
pub fn parse_layout_file(path: &str) -> Result<ParseResult<Layout>, ParseError> {
    // Read file from filesystem
    let json_str = fs::read_to_string(path)
        .map_err(|e| ParseError::io_error_with_path(e, path))?;

    // Parse JSON using serde_json
    let layout: Layout = serde_json::from_str(&json_str)
        .map_err(|e| ParseError::json_error_with_path(e, path))?;

    // Resolve inheritance if present
    let resolved_layout = resolve_inheritance(layout, Some(path))?;

    // Validate the layout and collect warnings
    validate_layout(resolved_layout)
        .map_err(|e| {
            // Add file path context to validation errors if not already present
            match e {
                ParseError::ValidationError { issues, file_path: None } => {
                    ParseError::ValidationError {
                        issues,
                        file_path: Some(path.to_string()),
                    }
                }
                other => other,
            }
        })
}

/// Parses a keyboard layout from a JSON string.
///
/// This function parses a pre-loaded JSON string into a layout structure.
/// Use this when you already have the JSON content in memory, or for testing.
///
/// The parser validates the layout after parsing, collecting warnings for
/// non-fatal issues and returning errors only for fatal problems like circular
/// references or excessive nesting depth.
///
/// Note: This function does NOT resolve inheritance, as it has no file path
/// context for loading parent layouts. Use `parse_layout_file` for layouts
/// with inheritance.
///
/// # Arguments
///
/// * `json` - JSON string containing the layout definition
///
/// # Returns
///
/// Returns a `ParseResult` containing the parsed layout and any warnings,
/// or a `ParseError` if parsing fails.
///
/// # Example
///
/// ```rust,ignore
/// use cosboard::layout::parser::parse_layout_from_string;
///
/// let json = r#"{
///     "name": "Test Layout",
///     "version": "1.0",
///     "default_panel_id": "main",
///     "panels": {}
/// }"#;
///
/// match parse_layout_from_string(json) {
///     Ok(result) => println!("Parsed layout: {}", result.layout.name),
///     Err(e) => eprintln!("Parse error: {}", e),
/// }
/// ```
pub fn parse_layout_from_string(json: &str) -> Result<ParseResult<Layout>, ParseError> {
    // Parse JSON using serde_json
    let layout: Layout = serde_json::from_str(json)
        .map_err(ParseError::json_error)?;

    // NOTE: We don't resolve inheritance here because we have no file path
    // context for loading parent layouts. If the layout has an inherits field,
    // it will remain unresolved (but validation will still work).
    // For full inheritance support, use parse_layout_file instead.

    // Validate the layout and collect warnings
    validate_layout(layout)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::types::{Cell, Sizing};
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ========================================================================
    // Task 3.1: Focused tests for parser (2-8 tests)
    // ========================================================================

    /// Test 1: Parse valid JSON string
    #[test]
    fn test_parse_valid_json_string() {
        let json = r#"{
            "name": "Test Layout",
            "version": "1.0",
            "default_panel_id": "main",
            "panels": {
                "main": {
                    "id": "main",
                    "rows": []
                }
            }
        }"#;

        let result = parse_layout_from_string(json);
        assert!(result.is_ok(), "Should parse valid JSON");

        let parse_result = result.unwrap();
        assert_eq!(parse_result.layout.name, "Test Layout");
        assert_eq!(parse_result.layout.version, "1.0");
        assert_eq!(parse_result.layout.default_panel_id, "main");
        assert!(parse_result.layout.panels.contains_key("main"));
    }

    /// Test 2: Handle missing file with I/O error
    #[test]
    fn test_parse_missing_file() {
        let result = parse_layout_file("/nonexistent/path/to/layout.json");

        assert!(result.is_err(), "Should fail for missing file");

        let err = result.unwrap_err();
        let display_str = format!("{}", err);

        match &err {
            ParseError::IoError { file_path, suggestion, .. } => {
                assert!(file_path.is_some(), "Error should include file path");
                assert!(suggestion.is_some(), "Error should include suggestion");
                assert!(display_str.contains("I/O error"));
                assert!(display_str.contains("/nonexistent/path/to/layout.json"));
            }
            _ => panic!("Expected IoError variant"),
        }
    }

    /// Test 3: Parse layout with panels and keys
    #[test]
    fn test_parse_layout_with_panels_and_keys() {
        let json = r#"{
            "name": "QWERTY Layout",
            "version": "1.0",
            "default_panel_id": "letters",
            "description": "Standard QWERTY keyboard",
            "author": "Test Author",
            "language": "en",
            "locale": "en_US",
            "panels": {
                "letters": {
                    "id": "letters",
                    "padding": 5.0,
                    "margin": 10.0,
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "key",
                                    "label": "Q",
                                    "code": "q",
                                    "identifier": "key_q",
                                    "width": 1.0,
                                    "height": 1.0
                                },
                                {
                                    "type": "key",
                                    "label": "W",
                                    "code": "w"
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        let result = parse_layout_from_string(json);
        assert!(result.is_ok(), "Should parse layout with panels and keys");

        let parse_result = result.unwrap();
        let layout = parse_result.layout;

        assert_eq!(layout.name, "QWERTY Layout");
        assert_eq!(layout.description, Some("Standard QWERTY keyboard".to_string()));
        assert_eq!(layout.author, Some("Test Author".to_string()));
        assert_eq!(layout.language, Some("en".to_string()));
        assert_eq!(layout.locale, Some("en_US".to_string()));

        let panel = layout.panels.get("letters").expect("Should have letters panel");
        assert_eq!(panel.id, "letters");
        assert_eq!(panel.padding, Some(5.0));
        assert_eq!(panel.margin, Some(10.0));
        assert_eq!(panel.rows.len(), 1);

        let row = &panel.rows[0];
        assert_eq!(row.cells.len(), 2);

        // Check first key
        match &row.cells[0] {
            Cell::Key(key) => {
                assert_eq!(key.label, "Q");
                assert_eq!(key.identifier, Some("key_q".to_string()));
            }
            _ => panic!("Expected Key cell"),
        }
    }

    /// Test 4: Parse alternatives (modifiers and swipes)
    #[test]
    fn test_parse_alternatives() {
        let json = r#"{
            "name": "Test Layout",
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
                                    "sticky": true,
                                    "alternatives": {
                                        "Shift": "A",
                                        "Up": "@"
                                    }
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        let result = parse_layout_from_string(json);
        assert!(result.is_ok(), "Should parse alternatives");

        let parse_result = result.unwrap();
        let panel = parse_result.layout.panels.get("main").unwrap();

        match &panel.rows[0].cells[0] {
            Cell::Key(key) => {
                assert_eq!(key.label, "a");
                assert!(key.sticky, "Key should be sticky");
                assert!(!key.alternatives.is_empty(), "Should have alternatives");
            }
            _ => panic!("Expected Key cell"),
        }
    }

    /// Test 5: Parse layout with widgets and panel refs
    #[test]
    fn test_parse_widgets_and_panel_refs() {
        let json = r#"{
            "name": "Test Layout",
            "version": "1.0",
            "default_panel_id": "main",
            "panels": {
                "main": {
                    "id": "main",
                    "rows": [
                        {
                            "cells": [
                                {
                                    "type": "widget",
                                    "widget_type": "trackpad",
                                    "width": 3.0,
                                    "height": 2.0
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
                    "rows": []
                }
            }
        }"#;

        let result = parse_layout_from_string(json);
        assert!(result.is_ok(), "Should parse widgets and panel refs");

        let parse_result = result.unwrap();
        let panel = parse_result.layout.panels.get("main").unwrap();

        assert_eq!(panel.rows[0].cells.len(), 2);

        // Check widget
        match &panel.rows[0].cells[0] {
            Cell::Widget(widget) => {
                assert_eq!(widget.widget_type, "trackpad");
            }
            _ => panic!("Expected Widget cell"),
        }

        // Check panel ref
        match &panel.rows[0].cells[1] {
            Cell::PanelRef(panel_ref) => {
                assert_eq!(panel_ref.panel_id, "numpad");
            }
            _ => panic!("Expected PanelRef cell"),
        }
    }

    /// Test 6: Handle malformed JSON with line number
    #[test]
    fn test_malformed_json_with_line_number() {
        let json = r#"{
            "name": "Test",
            "version": "1.0",
            "invalid_syntax":
        }"#;

        let result = parse_layout_from_string(json);
        assert!(result.is_err(), "Should fail for malformed JSON");

        let err = result.unwrap_err();
        let display_str = format!("{}", err);

        match &err {
            ParseError::JsonError { line_number, suggestion, .. } => {
                assert!(line_number.is_some(), "Should include line number");
                assert!(suggestion.is_some(), "Should include suggestion");
                assert!(display_str.contains("line"));
                assert!(display_str.contains("Suggestion"));
            }
            _ => panic!("Expected JsonError variant"),
        }
    }

    /// Test 7: Parse from file with valid content
    #[test]
    fn test_parse_layout_file_valid() {
        let json = r#"{
            "name": "File Layout",
            "version": "1.0",
            "default_panel_id": "main",
            "panels": {
                "main": {
                    "id": "main",
                    "rows": []
                }
            }
        }"#;

        // Create temporary file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(json.as_bytes()).expect("Failed to write temp file");
        let path = temp_file.path().to_str().unwrap();

        let result = parse_layout_file(path);
        assert!(result.is_ok(), "Should parse valid file");

        let parse_result = result.unwrap();
        assert_eq!(parse_result.layout.name, "File Layout");
    }

    /// Test 8: Parse with pixel sizing
    #[test]
    fn test_parse_pixel_sizing() {
        let json = r#"{
            "name": "Test Layout",
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
                                    "label": "Wide",
                                    "width": "100px",
                                    "height": "50px",
                                    "min_width": 80,
                                    "min_height": 40
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        let result = parse_layout_from_string(json);
        assert!(result.is_ok(), "Should parse pixel sizing");

        let parse_result = result.unwrap();
        let panel = parse_result.layout.panels.get("main").unwrap();

        match &panel.rows[0].cells[0] {
            Cell::Key(key) => {
                match &key.width {
                    Sizing::Pixels(px) => assert_eq!(px, "100px"),
                    _ => panic!("Expected Pixels sizing for width"),
                }
                match &key.height {
                    Sizing::Pixels(px) => assert_eq!(px, "50px"),
                    _ => panic!("Expected Pixels sizing for height"),
                }
                assert_eq!(key.min_width, Some(80));
                assert_eq!(key.min_height, Some(40));
            }
            _ => panic!("Expected Key cell"),
        }
    }
}
