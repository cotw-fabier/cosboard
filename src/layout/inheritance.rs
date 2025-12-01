// SPDX-License-Identifier: GPL-3.0-only

//! Layout inheritance resolution logic.
//!
//! This module handles resolving layout inheritance chains, merging parent
//! and child layouts, and detecting circular references.

use crate::layout::types::{Cell, Key, Layout, Panel, ParseError, Row};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Maximum inheritance depth allowed
const MAX_INHERITANCE_DEPTH: usize = 5;

/// Resolves inheritance for a layout.
///
/// If the layout has an `inherits` field, this function recursively loads
/// the parent layout, merges them, and returns the fully resolved layout.
/// If the layout has no inheritance, it returns the layout unchanged.
///
/// # Arguments
///
/// * `layout` - The layout to resolve inheritance for
/// * `layout_path` - Optional path to the layout file (used for resolving relative parent paths)
///
/// # Returns
///
/// Returns the fully resolved layout with all inheritance flattened,
/// or a ParseError if circular inheritance or max depth is exceeded.
///
/// # Example
///
/// ```rust,ignore
/// use cosboard::layout::inheritance::resolve_inheritance;
/// use cosboard::layout::types::Layout;
///
/// let layout = Layout {
///     inherits: Some("../parent.json".to_string()),
///     ..Layout::default()
/// };
///
/// match resolve_inheritance(layout, Some("/path/to/child.json")) {
///     Ok(resolved) => println!("Resolved layout: {}", resolved.name),
///     Err(e) => eprintln!("Inheritance error: {}", e),
/// }
/// ```
pub fn resolve_inheritance(
    layout: Layout,
    layout_path: Option<&str>,
) -> Result<Layout, ParseError> {
    let mut visited = HashSet::new();

    // Add current layout path to visited if provided
    if let Some(path) = layout_path {
        let path_buf = PathBuf::from(path);
        let canonical = path_buf.canonicalize().unwrap_or(path_buf);
        visited.insert(canonical);
    }

    resolve_inheritance_recursive(layout, layout_path, &mut visited, 0)
}

/// Internal recursive function for resolving inheritance.
fn resolve_inheritance_recursive(
    layout: Layout,
    layout_path: Option<&str>,
    visited: &mut HashSet<PathBuf>,
    depth: usize,
) -> Result<Layout, ParseError> {
    // Check max depth
    if depth > MAX_INHERITANCE_DEPTH {
        return Err(ParseError::max_depth_exceeded(
            "Inheritance chain too deep",
            MAX_INHERITANCE_DEPTH,
            depth,
        ));
    }

    // If no inheritance, return as-is
    let parent_path = match &layout.inherits {
        Some(path) => path,
        None => return Ok(layout),
    };

    // Resolve parent path (relative to current layout if layout_path is provided)
    let resolved_parent_path = if let Some(current_path) = layout_path {
        let current_dir = Path::new(current_path)
            .parent()
            .unwrap_or_else(|| Path::new("."));
        current_dir.join(parent_path)
    } else {
        PathBuf::from(parent_path)
    };

    // Use resolved path directly for cycle detection (canonicalization can fail unpredictably)
    let canonical_path = resolved_parent_path.clone();

    // Check for circular inheritance
    if visited.contains(&canonical_path) {
        let chain = visited
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" -> ");
        return Err(ParseError::circular_reference(
            format!(
                "Circular inheritance detected: layout inherits from '{}'",
                canonical_path.display()
            ),
            format!("{} -> {}", chain, canonical_path.display()),
        ));
    }

    visited.insert(canonical_path.clone());

    // Load parent layout (without resolving its inheritance yet)
    let parent_layout = load_parent_layout_raw(resolved_parent_path.to_str().unwrap())?;

    // Recursively resolve parent's inheritance
    let resolved_parent = resolve_inheritance_recursive(
        parent_layout,
        Some(resolved_parent_path.to_str().unwrap()),
        visited,
        depth + 1,
    )?;

    // Merge child into parent
    let merged = merge_layouts(layout, resolved_parent);

    Ok(merged)
}

/// Loads a parent layout file without resolving inheritance.
///
/// This is a helper to avoid circular dependency with the parser module.
fn load_parent_layout_raw(parent_path: &str) -> Result<Layout, ParseError> {
    // Read file from filesystem
    let json_str = fs::read_to_string(parent_path)
        .map_err(|e| ParseError::io_error_with_path(e, parent_path))?;

    // Parse JSON using serde_json
    let layout: Layout = serde_json::from_str(&json_str)
        .map_err(|e| ParseError::json_error_with_path(e, parent_path))?;

    Ok(layout)
}

/// Detects circular inheritance by checking if a path has already been visited.
///
/// This is handled within resolve_inheritance_recursive, but this function
/// is exported for testing purposes.
pub fn detect_circular_inheritance(
    layout_path: &str,
    visited: &HashSet<PathBuf>,
) -> Result<(), ParseError> {
    let path = PathBuf::from(layout_path);
    let canonical_path = path.canonicalize().unwrap_or(path);

    if visited.contains(&canonical_path) {
        return Err(ParseError::circular_reference(
            format!("Circular inheritance: layout already visited"),
            visited
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" -> "),
        ));
    }

    Ok(())
}

/// Merges a child layout with its parent layout.
///
/// The child layout takes precedence for metadata fields. Panels are merged
/// using `override_panels`, which matches panels by ID and overrides matching
/// panels while preserving non-matching ones from both child and parent.
///
/// # Arguments
///
/// * `child` - The child layout (takes precedence)
/// * `parent` - The parent layout (provides defaults)
///
/// # Returns
///
/// Returns a new Layout with merged content.
pub fn merge_layouts(child: Layout, parent: Layout) -> Layout {
    // Start with parent as base
    let mut merged = parent;

    // Override metadata from child (child takes precedence)
    merged.name = child.name;
    merged.description = child.description.or(merged.description);
    merged.author = child.author.or(merged.author);
    merged.language = child.language.or(merged.language);
    merged.locale = child.locale.or(merged.locale);
    merged.version = child.version;
    merged.default_panel_id = child.default_panel_id;

    // Clear inherits field in merged layout (inheritance is now resolved)
    merged.inherits = None;

    // Merge panels
    merged.panels = override_panels(child.panels, merged.panels);

    merged
}

/// Merges child panels with parent panels.
///
/// For each panel in the child:
/// - If a panel with the same ID exists in the parent, override it with the child panel
/// - If no matching parent panel exists, add the child panel
///
/// Panels from the parent that are not overridden are preserved.
///
/// # Arguments
///
/// * `child_panels` - Panels from the child layout
/// * `parent_panels` - Panels from the parent layout
///
/// # Returns
///
/// Returns merged panels map.
pub fn override_panels(
    child_panels: std::collections::HashMap<String, Panel>,
    mut parent_panels: std::collections::HashMap<String, Panel>,
) -> std::collections::HashMap<String, Panel> {
    // For each child panel
    for (panel_id, child_panel) in child_panels {
        if let Some(parent_panel) = parent_panels.get(&panel_id) {
            // Panel exists in both - merge rows by overriding keys/widgets
            let merged_panel = override_panel(child_panel, parent_panel.clone());
            parent_panels.insert(panel_id, merged_panel);
        } else {
            // Panel only exists in child - add it
            parent_panels.insert(panel_id, child_panel);
        }
    }

    parent_panels
}

/// Overrides a parent panel with a child panel, merging rows and cells.
///
/// The child panel structure takes precedence, but we attempt to merge
/// keys by identifier when possible.
fn override_panel(child: Panel, parent: Panel) -> Panel {
    let mut merged = child.clone();

    // If child has fewer rows than parent, we use child's structure
    // If child has more rows, we use child's structure
    // We only try to merge keys within matching row indices

    let min_rows = std::cmp::min(merged.rows.len(), parent.rows.len());

    for row_idx in 0..min_rows {
        merged.rows[row_idx] = override_row(
            merged.rows[row_idx].clone(),
            parent.rows[row_idx].clone(),
        );
    }

    merged
}

/// Overrides a parent row with a child row, merging cells.
fn override_row(child: Row, parent: Row) -> Row {
    let mut merged = child.clone();

    // Try to merge keys by identifier at the same position
    let min_cells = std::cmp::min(merged.cells.len(), parent.cells.len());

    for cell_idx in 0..min_cells {
        merged.cells[cell_idx] = override_cell(
            merged.cells[cell_idx].clone(),
            parent.cells[cell_idx].clone(),
        );
    }

    merged
}

/// Overrides a parent cell with a child cell.
///
/// Matching logic:
/// - Keys match by identifier (if present)
/// - Widgets match by widget_type at same position
/// - PanelRefs are replaced without merging
fn override_cell(child: Cell, parent: Cell) -> Cell {
    match (&child, &parent) {
        (Cell::Key(child_key), Cell::Key(parent_key)) => {
            // If both keys have identifiers and they match, merge them
            if let (Some(child_id), Some(parent_id)) =
                (&child_key.identifier, &parent_key.identifier)
            {
                if child_id == parent_id {
                    // Keys match by identifier - child takes precedence but merge alternatives
                    return Cell::Key(override_key(child_key.clone(), parent_key.clone()));
                }
            }
            // No identifier match - child replaces parent
            child
        }
        (Cell::Widget(child_widget), Cell::Widget(parent_widget)) => {
            // Widgets match by type at same position
            if child_widget.widget_type == parent_widget.widget_type {
                // Child widget takes precedence (no merging needed for widgets)
                child
            } else {
                // Different widget types - child replaces parent
                child
            }
        }
        _ => {
            // Different cell types or PanelRefs - child replaces parent
            child
        }
    }
}

/// Merges a child key with a parent key when they match by identifier.
///
/// Child key properties take precedence, but we merge alternatives maps.
fn override_key(child: Key, parent: Key) -> Key {
    let mut merged = child.clone();

    // Merge alternatives: start with parent's, then override with child's
    let mut merged_alternatives = parent.alternatives.clone();
    for (alt_key, alt_action) in child.alternatives {
        merged_alternatives.insert(alt_key, alt_action);
    }
    merged.alternatives = merged_alternatives;

    merged
}

/// Overrides widgets in a parent panel with widgets from a child panel.
///
/// Widgets are matched by widget_type at the same position in rows.
/// This is a helper function for testing purposes.
#[allow(dead_code)]
pub fn override_widgets(
    _child_panel: &Panel,
    _parent_panel: &Panel,
) -> Vec<Row> {
    // This functionality is integrated into override_panel/override_row/override_cell
    // Keeping this function for API compatibility but it's not used
    Vec::new()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::types::{Action, AlternativeKey, KeyCode, Modifier, Sizing};
    use std::collections::HashMap;
    use tempfile::TempDir;

    // ========================================================================
    // Task 5.1: Focused tests for inheritance (2-8 tests)
    // ========================================================================

    /// Test 1: Load parent layout and merge
    #[test]
    fn test_load_parent_layout_and_merge() {
        // Create temp directory for test files
        let temp_dir = TempDir::new().unwrap();
        let parent_path = temp_dir.path().join("parent.json");
        let child_path = temp_dir.path().join("child.json");

        // Create parent layout
        let parent_json = r#"{
            "name": "Parent Layout",
            "version": "1.0",
            "description": "Parent description",
            "author": "Parent Author",
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

        std::fs::write(&parent_path, parent_json).unwrap();

        // Create child layout that inherits from parent
        let child_json = format!(
            r#"{{
            "name": "Child Layout",
            "version": "2.0",
            "default_panel_id": "main",
            "inherits": "{}",
            "panels": {{
                "main": {{
                    "id": "main",
                    "rows": []
                }}
            }}
        }}"#,
            parent_path.file_name().unwrap().to_str().unwrap()
        );

        std::fs::write(&child_path, child_json).unwrap();

        // Parse child layout (without inheritance resolution)
        let json_str = std::fs::read_to_string(&child_path).unwrap();
        let child_layout: Layout = serde_json::from_str(&json_str).unwrap();

        // Resolve inheritance
        let resolved = resolve_inheritance(child_layout, Some(child_path.to_str().unwrap()))
            .expect("Should resolve inheritance");

        // Check merged result
        assert_eq!(resolved.name, "Child Layout");
        assert_eq!(resolved.version, "2.0");
        assert_eq!(resolved.description, Some("Parent description".to_string()));
        assert_eq!(resolved.author, Some("Parent Author".to_string()));
        assert!(resolved.inherits.is_none(), "Inherits should be cleared");
    }

    /// Test 2: Override key by identifier
    #[test]
    fn test_override_key_by_identifier() {
        let parent_key = Key {
            label: "a".to_string(),
            code: KeyCode::Unicode('a'),
            identifier: Some("key_a".to_string()),
            width: Sizing::Relative(1.0),
            sticky: false,
            alternatives: {
                let mut alts = HashMap::new();
                alts.insert(
                    AlternativeKey::SingleModifier(Modifier::Shift),
                    Action::Character('A'),
                );
                alts
            },
            ..Key::default()
        };

        let child_key = Key {
            label: "α".to_string(), // Different label
            code: KeyCode::Unicode('α'), // Different code
            identifier: Some("key_a".to_string()), // Same identifier
            width: Sizing::Relative(1.5),
            sticky: true,
            alternatives: {
                let mut alts = HashMap::new();
                alts.insert(
                    AlternativeKey::SingleModifier(Modifier::Ctrl),
                    Action::Character('@'),
                );
                alts
            },
            ..Key::default()
        };

        let merged = override_key(child_key, parent_key);

        // Child properties take precedence
        assert_eq!(merged.label, "α");
        assert_eq!(merged.code, KeyCode::Unicode('α'));
        assert_eq!(merged.width, Sizing::Relative(1.5));
        assert_eq!(merged.sticky, true);

        // Alternatives should be merged (both Shift and Ctrl)
        assert_eq!(merged.alternatives.len(), 2);
        assert!(merged
            .alternatives
            .contains_key(&AlternativeKey::SingleModifier(Modifier::Shift)));
        assert!(merged
            .alternatives
            .contains_key(&AlternativeKey::SingleModifier(Modifier::Ctrl)));
    }

    /// Test 3: Override panel by id
    #[test]
    fn test_override_panel_by_id() {
        let mut parent_panels = HashMap::new();
        parent_panels.insert(
            "main".to_string(),
            Panel {
                id: "main".to_string(),
                padding: Some(10.0),
                rows: vec![Row {
                    cells: vec![Cell::Key(Key {
                        label: "Old".to_string(),
                        ..Key::default()
                    })],
                }],
                ..Panel::default()
            },
        );
        parent_panels.insert(
            "numpad".to_string(),
            Panel {
                id: "numpad".to_string(),
                rows: vec![],
                ..Panel::default()
            },
        );

        let mut child_panels = HashMap::new();
        child_panels.insert(
            "main".to_string(),
            Panel {
                id: "main".to_string(),
                padding: Some(5.0), // Different padding
                rows: vec![Row {
                    cells: vec![Cell::Key(Key {
                        label: "New".to_string(),
                        ..Key::default()
                    })],
                }],
                ..Panel::default()
            },
        );
        child_panels.insert(
            "symbols".to_string(),
            Panel {
                id: "symbols".to_string(),
                rows: vec![],
                ..Panel::default()
            },
        );

        let merged = override_panels(child_panels, parent_panels);

        // Should have 3 panels: main (overridden), numpad (from parent), symbols (from child)
        assert_eq!(merged.len(), 3);
        assert!(merged.contains_key("main"));
        assert!(merged.contains_key("numpad"));
        assert!(merged.contains_key("symbols"));

        // Main panel should have child's properties
        let main = merged.get("main").unwrap();
        assert_eq!(main.padding, Some(5.0));
        assert_eq!(main.rows[0].cells.len(), 1);
        match &main.rows[0].cells[0] {
            Cell::Key(key) => assert_eq!(key.label, "New"),
            _ => panic!("Expected Key cell"),
        }
    }

    /// Test 4: Detect circular inheritance
    #[test]
    fn test_detect_circular_inheritance() {
        let temp_dir = TempDir::new().unwrap();
        let layout_a_path = temp_dir.path().join("a.json");
        let layout_b_path = temp_dir.path().join("b.json");

        // Layout A inherits from B
        let layout_a_json = format!(
            r#"{{
            "name": "Layout A",
            "version": "1.0",
            "default_panel_id": "main",
            "inherits": "b.json",
            "panels": {{ "main": {{ "id": "main", "rows": [] }} }}
        }}"#
        );

        // Layout B inherits from A (circular!)
        let layout_b_json = format!(
            r#"{{
            "name": "Layout B",
            "version": "1.0",
            "default_panel_id": "main",
            "inherits": "a.json",
            "panels": {{ "main": {{ "id": "main", "rows": [] }} }}
        }}"#
        );

        std::fs::write(&layout_a_path, layout_a_json).unwrap();
        std::fs::write(&layout_b_path, layout_b_json).unwrap();

        // Try to resolve inheritance from A
        let json_str = std::fs::read_to_string(&layout_a_path).unwrap();
        let layout_a: Layout = serde_json::from_str(&json_str).unwrap();

        let result = resolve_inheritance(layout_a, Some(layout_a_path.to_str().unwrap()));

        assert!(result.is_err(), "Should detect circular inheritance");
        let err = result.unwrap_err();
        let display_str = format!("{}", err);
        assert!(
            display_str.contains("Circular"),
            "Error should mention circular reference"
        );
    }

    /// Test 5: Max depth exceeded
    #[test]
    fn test_max_depth_exceeded() {
        let temp_dir = TempDir::new().unwrap();

        // Create a chain of 7 layouts (exceeds limit of 5)
        // Layout chain: 0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 6
        // This creates inheritance depth 0:0, 1:1, 2:2, 3:3, 4:4, 5:5, 6:6
        // But actually the depth is checked by the number of *steps* not files
        // Let's create 0 -> 1 -> 2 -> 3 -> 4 -> 5 (6 files = depth 5, which should fail)

        let mut paths = Vec::new();
        for i in 0..7 {
            let path = temp_dir.path().join(format!("layout_{}.json", i));
            paths.push(path);
        }

        // Create the deepest layout (no inheritance)
        let layout_6_json = r#"{
            "name": "Layout 6",
            "version": "1.0",
            "default_panel_id": "main",
            "panels": { "main": { "id": "main", "rows": [] } }
        }"#;
        std::fs::write(&paths[6], layout_6_json).unwrap();

        // Create layouts 5 down to 0, each inheriting from the next
        for i in (0..6).rev() {
            let layout_json = format!(
                r#"{{
                "name": "Layout {}",
                "version": "1.0",
                "default_panel_id": "main",
                "inherits": "layout_{}.json",
                "panels": {{ "main": {{ "id": "main", "rows": [] }} }}
            }}"#,
                i,
                i + 1
            );
            std::fs::write(&paths[i], layout_json).unwrap();
        }

        // Try to resolve from layout 0
        let json_str = std::fs::read_to_string(&paths[0]).unwrap();
        let layout_0: Layout = serde_json::from_str(&json_str).unwrap();

        let result = resolve_inheritance(layout_0, Some(paths[0].to_str().unwrap()));

        assert!(result.is_err(), "Should reject excessive depth");
        let err = result.unwrap_err();
        let display_str = format!("{}", err);
        assert!(display_str.contains("Maximum depth exceeded"));
        assert!(display_str.contains("limit: 5"));
    }

    /// Test 6: Merge layouts metadata
    #[test]
    fn test_merge_layouts_metadata() {
        let parent = Layout {
            name: "Parent".to_string(),
            description: Some("Parent description".to_string()),
            author: Some("Parent Author".to_string()),
            language: Some("en".to_string()),
            locale: Some("en_US".to_string()),
            version: "1.0".to_string(),
            default_panel_id: "main".to_string(),
            inherits: None,
            panels: HashMap::new(),
        };

        let child = Layout {
            name: "Child".to_string(),
            description: None, // Should inherit from parent
            author: Some("Child Author".to_string()), // Should override parent
            language: None, // Should inherit from parent
            locale: Some("en_GB".to_string()), // Should override parent
            version: "2.0".to_string(),
            default_panel_id: "child_main".to_string(),
            inherits: Some("parent.json".to_string()),
            panels: HashMap::new(),
        };

        let merged = merge_layouts(child, parent);

        assert_eq!(merged.name, "Child");
        assert_eq!(merged.description, Some("Parent description".to_string()));
        assert_eq!(merged.author, Some("Child Author".to_string()));
        assert_eq!(merged.language, Some("en".to_string()));
        assert_eq!(merged.locale, Some("en_GB".to_string()));
        assert_eq!(merged.version, "2.0");
        assert_eq!(merged.default_panel_id, "child_main");
        assert!(merged.inherits.is_none());
    }

    /// Test 7: No inheritance returns layout unchanged
    #[test]
    fn test_no_inheritance() {
        let layout = Layout {
            name: "Test".to_string(),
            version: "1.0".to_string(),
            default_panel_id: "main".to_string(),
            inherits: None, // No inheritance
            ..Layout::default()
        };

        let original_name = layout.name.clone();
        let resolved = resolve_inheritance(layout, None).unwrap();

        assert_eq!(resolved.name, original_name);
        assert!(resolved.inherits.is_none());
    }

    /// Test 8: Override widgets by type at same position
    #[test]
    fn test_override_widgets_by_type() {
        let parent_cell = Cell::Widget(crate::layout::types::Widget {
            widget_type: "trackpad".to_string(),
            width: Sizing::Relative(2.0),
            height: Sizing::Relative(1.5),
        });

        let child_cell_same_type = Cell::Widget(crate::layout::types::Widget {
            widget_type: "trackpad".to_string(),
            width: Sizing::Relative(3.0),
            height: Sizing::Relative(2.0),
        });

        let child_cell_different_type = Cell::Widget(crate::layout::types::Widget {
            widget_type: "autocomplete".to_string(),
            width: Sizing::Relative(1.0),
            height: Sizing::Relative(1.0),
        });

        // Same type - child should replace parent
        let merged_same = override_cell(child_cell_same_type.clone(), parent_cell.clone());
        match merged_same {
            Cell::Widget(widget) => {
                assert_eq!(widget.widget_type, "trackpad");
                assert_eq!(widget.width, Sizing::Relative(3.0));
            }
            _ => panic!("Expected Widget cell"),
        }

        // Different type - child should replace parent
        let merged_different = override_cell(child_cell_different_type, parent_cell);
        match merged_different {
            Cell::Widget(widget) => {
                assert_eq!(widget.widget_type, "autocomplete");
            }
            _ => panic!("Expected Widget cell"),
        }
    }
}
