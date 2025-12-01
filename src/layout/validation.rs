// SPDX-License-Identifier: GPL-3.0-only

//! Validation rules for keyboard layout definitions.
//!
//! This module implements permissive validation that collects warnings
//! and provides sensible defaults for missing or invalid values.

use crate::layout::types::{
    Action, AlternativeKey, Cell, Key, Layout, Panel, ParseError, ParseResult,
    Severity, Sizing, ValidationIssue,
};
use std::collections::{HashMap, HashSet};

/// Maximum allowed nesting depth for panel references
const MAX_NESTING_DEPTH: u8 = 5;

/// Validates a layout and returns it with warnings.
///
/// This function performs comprehensive validation on a layout, collecting
/// warnings for non-fatal issues and returning errors only for fatal problems
/// like circular references or excessive nesting depth.
pub fn validate_layout(mut layout: Layout) -> Result<ParseResult<Layout>, ParseError> {
    let mut warnings = Vec::new();

    // Validate required fields
    validate_required_fields(&layout, &mut warnings);

    // Validate sizing across all keys
    validate_all_sizing(&layout, &mut warnings);

    // Validate modifier combinations
    validate_all_modifier_combinations(&layout, &mut warnings);

    // Validate panel references (this can add warnings)
    validate_panel_references(&layout, &mut warnings)?;

    // Detect circular references (fatal error if found)
    detect_circular_references(&layout)?;

    // Enforce max nesting depth and update panel depths
    enforce_max_nesting_depth(&mut layout)?;

    // Return the validated layout with collected warnings
    Ok(collect_warnings(layout, warnings))
}

/// Validates that required fields are present and provides defaults where appropriate.
pub fn validate_required_fields(layout: &Layout, warnings: &mut Vec<ValidationIssue>) {
    // Check Layout required fields
    if layout.name.is_empty() {
        warnings.push(
            ValidationIssue::new(
                Severity::Warning,
                "Layout name is empty",
                "name",
            )
            .with_suggestion("Provide a descriptive name for the layout"),
        );
    }

    if layout.version.is_empty() {
        warnings.push(
            ValidationIssue::new(
                Severity::Warning,
                "Layout version is empty",
                "version",
            )
            .with_suggestion("Use semantic versioning (e.g., '1.0', '1.0.0')"),
        );
    }

    if layout.default_panel_id.is_empty() {
        warnings.push(
            ValidationIssue::new(
                Severity::Warning,
                "Default panel ID is empty",
                "default_panel_id",
            )
            .with_suggestion("Specify which panel should be shown by default"),
        );
    }

    // Warn about missing optional metadata
    if layout.description.is_none() {
        warnings.push(ValidationIssue::new(
            Severity::Warning,
            "Missing layout description",
            "description",
        ));
    }

    if layout.author.is_none() {
        warnings.push(ValidationIssue::new(
            Severity::Warning,
            "Missing layout author",
            "author",
        ));
    }

    // Validate panels
    for (panel_id, panel) in &layout.panels {
        let panel_path = format!("panels[{}]", panel_id);

        if panel.id.is_empty() {
            warnings.push(
                ValidationIssue::new(
                    Severity::Warning,
                    "Panel ID is empty",
                    format!("{}.id", panel_path),
                )
                .with_suggestion("Provide a unique identifier for the panel"),
            );
        }

        // Validate keys in panel rows
        for (row_idx, row) in panel.rows.iter().enumerate() {
            for (cell_idx, cell) in row.cells.iter().enumerate() {
                if let Cell::Key(key) = cell {
                    let key_path = format!("{}.rows[{}].cells[{}]", panel_path, row_idx, cell_idx);
                    validate_key_required_fields(key, &key_path, warnings);
                }
            }
        }
    }
}

/// Validates required fields for a single key.
fn validate_key_required_fields(key: &Key, key_path: &str, warnings: &mut Vec<ValidationIssue>) {
    if key.label.is_empty() {
        warnings.push(
            ValidationIssue::new(
                Severity::Warning,
                "Key label is empty",
                format!("{}.label", key_path),
            )
            .with_suggestion("Provide a display label for the key"),
        );
    }

    // Note: code has a default value (Unicode(' ')), so we don't need to check for empty
}

/// Validates sizing values across all keys and widgets.
pub fn validate_sizing(sizing: &Sizing, field_path: &str, warnings: &mut Vec<ValidationIssue>) {
    match sizing {
        Sizing::Relative(value) => {
            if *value <= 0.0 {
                warnings.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        format!("Relative size {} is not positive", value),
                        field_path,
                    )
                    .with_suggestion("Use a positive number (e.g., 1.0 for standard size)"),
                );
            }

            // Warn about unusually large values
            if *value > 10.0 {
                warnings.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        format!("Relative size {} is unusually large", value),
                        field_path,
                    )
                    .with_suggestion("Consider using a smaller value (typical range: 0.5-5.0)"),
                );
            }
        }
        Sizing::Pixels(px_str) => {
            // Validate pixel string format: must match r"^\d+px$"
            if !px_str.ends_with("px") {
                warnings.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        format!("Invalid pixel format: '{}'", px_str),
                        field_path,
                    )
                    .with_suggestion("Use format like '20px' (number followed by 'px')"),
                );
            } else {
                // Check if the numeric part is valid
                let num_part = &px_str[..px_str.len() - 2];
                if num_part.parse::<u32>().is_err() {
                    warnings.push(
                        ValidationIssue::new(
                            Severity::Warning,
                            format!("Invalid pixel value: '{}'", px_str),
                            field_path,
                        )
                        .with_suggestion("Use a positive integer followed by 'px' (e.g., '20px')"),
                    );
                }
            }
        }
    }
}

/// Validates sizing for all keys and widgets in the layout.
fn validate_all_sizing(layout: &Layout, warnings: &mut Vec<ValidationIssue>) {
    for (panel_id, panel) in &layout.panels {
        let panel_path = format!("panels[{}]", panel_id);

        for (row_idx, row) in panel.rows.iter().enumerate() {
            for (cell_idx, cell) in row.cells.iter().enumerate() {
                let cell_path = format!("{}.rows[{}].cells[{}]", panel_path, row_idx, cell_idx);

                match cell {
                    Cell::Key(key) => {
                        validate_sizing(&key.width, &format!("{}.width", cell_path), warnings);
                        validate_sizing(&key.height, &format!("{}.height", cell_path), warnings);

                        // Check for unusually large keys (width or height)
                        if let Sizing::Relative(w) = &key.width {
                            if *w > 10.0 {
                                warnings.push(
                                    ValidationIssue::new(
                                        Severity::Warning,
                                        format!("Key width {} is unusually large", w),
                                        format!("{}.width", cell_path),
                                    )
                                    .with_suggestion("Typical key widths are between 1.0 and 5.0"),
                                );
                            }
                        }

                        if let Sizing::Relative(h) = &key.height {
                            if *h > 5.0 {
                                warnings.push(
                                    ValidationIssue::new(
                                        Severity::Warning,
                                        format!("Key height {} is unusually large", h),
                                        format!("{}.height", cell_path),
                                    )
                                    .with_suggestion("Typical key heights are between 1.0 and 3.0"),
                                );
                            }
                        }
                    }
                    Cell::Widget(widget) => {
                        validate_sizing(&widget.width, &format!("{}.width", cell_path), warnings);
                        validate_sizing(&widget.height, &format!("{}.height", cell_path), warnings);
                    }
                    Cell::PanelRef(panel_ref) => {
                        validate_sizing(&panel_ref.width, &format!("{}.width", cell_path), warnings);
                        validate_sizing(&panel_ref.height, &format!("{}.height", cell_path), warnings);
                    }
                }
            }
        }
    }
}

/// Validates modifier combinations in key alternatives.
pub fn validate_modifier_combinations(
    alternatives: &HashMap<AlternativeKey, Action>,
    key_path: &str,
    warnings: &mut Vec<ValidationIssue>,
) {
    for (alt_key, _) in alternatives {
        match alt_key {
            AlternativeKey::ModifierCombo(modifiers) => {
                // Check for empty combinations
                if modifiers.is_empty() {
                    warnings.push(
                        ValidationIssue::new(
                            Severity::Warning,
                            "Modifier combination is empty",
                            format!("{}.alternatives", key_path),
                        )
                        .with_suggestion("Remove empty modifier combinations or add modifiers"),
                    );
                }

                // Check for duplicates
                let mut seen = HashSet::new();
                for modifier in modifiers {
                    if !seen.insert(modifier) {
                        warnings.push(
                            ValidationIssue::new(
                                Severity::Warning,
                                format!("Duplicate modifier {:?} in combination", modifier),
                                format!("{}.alternatives", key_path),
                            )
                            .with_suggestion("Remove duplicate modifiers from the combination"),
                        );
                    }
                }

                // Warn about unusual combinations (all four modifiers)
                if modifiers.len() == 4 {
                    warnings.push(
                        ValidationIssue::new(
                            Severity::Warning,
                            "Modifier combination uses all four modifiers",
                            format!("{}.alternatives", key_path),
                        )
                        .with_suggestion("This combination may be difficult for users to trigger"),
                    );
                }

                // Check if modifiers are sorted (they should be for consistent matching)
                let mut sorted_modifiers = modifiers.clone();
                sorted_modifiers.sort();
                if modifiers != &sorted_modifiers {
                    warnings.push(
                        ValidationIssue::new(
                            Severity::Warning,
                            "Modifiers are not in canonical order",
                            format!("{}.alternatives", key_path),
                        )
                        .with_suggestion("Sort modifiers for consistent matching"),
                    );
                }
            }
            _ => {
                // Single modifiers and swipes don't need validation
            }
        }
    }
}

/// Validates modifier combinations for all keys in the layout.
fn validate_all_modifier_combinations(layout: &Layout, warnings: &mut Vec<ValidationIssue>) {
    for (panel_id, panel) in &layout.panels {
        let panel_path = format!("panels[{}]", panel_id);

        for (row_idx, row) in panel.rows.iter().enumerate() {
            for (cell_idx, cell) in row.cells.iter().enumerate() {
                if let Cell::Key(key) = cell {
                    let key_path = format!("{}.rows[{}].cells[{}]", panel_path, row_idx, cell_idx);
                    validate_modifier_combinations(&key.alternatives, &key_path, warnings);
                }
            }
        }
    }
}

/// Detects circular references in panel references.
///
/// Uses depth-first search to detect cycles in the panel dependency graph.
/// Returns a CircularReference error if a cycle is found.
pub fn detect_circular_references(layout: &Layout) -> Result<(), ParseError> {
    // Build a set of all panel IDs that are referenced
    let mut visited = HashSet::new();
    let mut recursion_stack = Vec::new();

    for panel_id in layout.panels.keys() {
        if !visited.contains(panel_id) {
            detect_circular_references_dfs(
                layout,
                panel_id,
                &mut visited,
                &mut recursion_stack,
            )?;
        }
    }

    Ok(())
}

/// Helper function for depth-first search to detect circular references.
fn detect_circular_references_dfs(
    layout: &Layout,
    panel_id: &str,
    visited: &mut HashSet<String>,
    recursion_stack: &mut Vec<String>,
) -> Result<(), ParseError> {
    visited.insert(panel_id.to_string());
    recursion_stack.push(panel_id.to_string());

    // Get the panel
    if let Some(panel) = layout.panels.get(panel_id) {
        // Check all panel references in this panel
        for row in &panel.rows {
            for cell in &row.cells {
                if let Cell::PanelRef(panel_ref) = cell {
                    let ref_id = &panel_ref.panel_id;

                    // Check if this panel is already in the recursion stack (cycle detected)
                    if recursion_stack.contains(ref_id) {
                        // Build the cycle chain
                        let cycle_start = recursion_stack
                            .iter()
                            .position(|id| id == ref_id)
                            .unwrap();
                        let mut chain = recursion_stack[cycle_start..].to_vec();
                        chain.push(ref_id.clone());

                        return Err(ParseError::circular_reference(
                            format!("Panel '{}' creates a circular reference", ref_id),
                            chain.join(" -> "),
                        ));
                    }

                    // Recursively check the referenced panel
                    if !visited.contains(ref_id) {
                        detect_circular_references_dfs(
                            layout,
                            ref_id,
                            visited,
                            recursion_stack,
                        )?;
                    }
                }
            }
        }
    }

    recursion_stack.pop();
    Ok(())
}

/// Enforces maximum nesting depth for panel references.
///
/// Updates the nesting_depth field on each panel and returns an error
/// if any panel exceeds the maximum depth.
pub fn enforce_max_nesting_depth(layout: &mut Layout) -> Result<(), ParseError> {
    // Calculate depth for all panels reachable from the default
    let mut depths: HashMap<String, u8> = HashMap::new();

    // First, calculate depths for all panels
    for panel_id in layout.panels.keys() {
        let depth = calculate_panel_depth(layout, panel_id, &mut HashMap::new())?;
        depths.insert(panel_id.clone(), depth);
    }

    // Update the nesting_depth field for each panel
    for (panel_id, depth) in depths {
        if let Some(panel) = layout.panels.get_mut(&panel_id) {
            panel.nesting_depth = depth;
        }
    }

    Ok(())
}

/// Calculates the nesting depth for a panel recursively.
fn calculate_panel_depth(
    layout: &Layout,
    panel_id: &str,
    memo: &mut HashMap<String, u8>,
) -> Result<u8, ParseError> {
    // Check memo first
    if let Some(&depth) = memo.get(panel_id) {
        return Ok(depth);
    }

    let panel = match layout.panels.get(panel_id) {
        Some(p) => p,
        None => return Ok(0), // Panel doesn't exist, depth is 0
    };

    let mut max_child_depth = 0;

    // Find the maximum depth of any referenced panel
    for row in &panel.rows {
        for cell in &row.cells {
            if let Cell::PanelRef(panel_ref) = cell {
                let child_depth = calculate_panel_depth(layout, &panel_ref.panel_id, memo)?;
                max_child_depth = max_child_depth.max(child_depth);
            }
        }
    }


    // Actually, let's fix this properly:
    // - A panel with no PanelRef children has depth 0
    // - A panel with PanelRef children has depth = 1 + max(child depths)
    let has_refs = has_panel_refs(panel);
    let depth = if has_refs {
        1 + max_child_depth
    } else {
        0
    };

    // Check if depth exceeds maximum
    if depth > MAX_NESTING_DEPTH {
        return Err(ParseError::max_depth_exceeded(
            format!("Panel '{}' nesting depth too deep", panel_id),
            MAX_NESTING_DEPTH as usize,
            depth as usize,
        ));
    }

    memo.insert(panel_id.to_string(), depth);
    Ok(depth)
}

/// Checks if a panel has any PanelRef cells.
fn has_panel_refs(panel: &Panel) -> bool {
    for row in &panel.rows {
        for cell in &row.cells {
            if matches!(cell, Cell::PanelRef(_)) {
                return true;
            }
        }
    }
    false
}

/// Validates panel references and checks that all referenced panels exist.
pub fn validate_panel_references(
    layout: &Layout,
    warnings: &mut Vec<ValidationIssue>,
) -> Result<(), ParseError> {
    // Check that default_panel_id references an existing panel
    if !layout.panels.contains_key(&layout.default_panel_id) {
        // This is a fatal error
        return Err(ParseError::validation_error(vec![ValidationIssue::new(
            Severity::Error,
            format!(
                "Default panel '{}' does not exist",
                layout.default_panel_id
            ),
            "default_panel_id",
        )
        .with_suggestion(format!(
            "Use one of the existing panel IDs: {}",
            layout
                .panels
                .keys()
                .map(|s| format!("'{}'", s))
                .collect::<Vec<_>>()
                .join(", ")
        ))]));
    }

    // Track which panels are referenced
    let mut referenced_panels = HashSet::new();
    referenced_panels.insert(layout.default_panel_id.clone());

    // Check all panel references
    for (panel_id, panel) in &layout.panels {
        let panel_path = format!("panels[{}]", panel_id);

        for (row_idx, row) in panel.rows.iter().enumerate() {
            for (cell_idx, cell) in row.cells.iter().enumerate() {
                if let Cell::PanelRef(panel_ref) = cell {
                    let ref_path =
                        format!("{}.rows[{}].cells[{}].panel_id", panel_path, row_idx, cell_idx);
                    referenced_panels.insert(panel_ref.panel_id.clone());

                    // Check if the referenced panel exists
                    if !layout.panels.contains_key(&panel_ref.panel_id) {
                        // Provide suggestions for typos
                        let suggestion = if let Some(similar) =
                            find_similar_panel_name(&panel_ref.panel_id, &layout.panels)
                        {
                            format!("Did you mean '{}'?", similar)
                        } else {
                            format!(
                                "Available panels: {}",
                                layout
                                    .panels
                                    .keys()
                                    .map(|s| format!("'{}'", s))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        };

                        warnings.push(
                            ValidationIssue::new(
                                Severity::Warning,
                                format!("Panel '{}' does not exist", panel_ref.panel_id),
                                ref_path,
                            )
                            .with_suggestion(suggestion),
                        );
                    }
                }
            }
        }
    }

    // Warn about unreferenced panels
    for panel_id in layout.panels.keys() {
        if !referenced_panels.contains(panel_id) {
            warnings.push(ValidationIssue::new(
                Severity::Warning,
                format!("Panel '{}' is never referenced", panel_id),
                format!("panels[{}]", panel_id),
            ));
        }
    }

    Ok(())
}

/// Finds a panel name similar to the given name (for typo suggestions).
fn find_similar_panel_name(target: &str, panels: &HashMap<String, Panel>) -> Option<String> {
    // Simple similarity check: find panels with similar length and overlapping characters
    let target_lower = target.to_lowercase();

    for panel_id in panels.keys() {
        let panel_lower = panel_id.to_lowercase();

        // Check if names are similar length
        if (panel_lower.len() as i32 - target_lower.len() as i32).abs() <= 2 {
            // Check if they share most characters
            let shared_chars = target_lower
                .chars()
                .filter(|c| panel_lower.contains(*c))
                .count();

            if shared_chars >= target_lower.len() - 2 {
                return Some(panel_id.clone());
            }
        }

        // Check if target is a substring
        if panel_lower.contains(&target_lower) || target_lower.contains(&panel_lower) {
            return Some(panel_id.clone());
        }
    }

    None
}

/// Collects validation warnings and returns a ParseResult.
pub fn collect_warnings(layout: Layout, mut warnings: Vec<ValidationIssue>) -> ParseResult<Layout> {
    // Sort warnings by severity (errors first) and then by field path
    warnings.sort_by(|a, b| {
        match (a.severity, b.severity) {
            (Severity::Error, Severity::Warning) => std::cmp::Ordering::Less,
            (Severity::Warning, Severity::Error) => std::cmp::Ordering::Greater,
            _ => a.field_path.cmp(&b.field_path),
        }
    });

    ParseResult::with_warnings(layout, warnings)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::types::{KeyCode, Modifier, PanelRef, Row};

    // ========================================================================
    // Task 4.1: Focused tests for validation (2-8 tests)
    // ========================================================================

    /// Test 1: Circular reference detection
    #[test]
    fn test_detect_circular_references() {
        let mut layout = Layout::default();

        // Create three panels: main -> panel_a -> panel_b -> main (circular)
        let mut main_panel = Panel {
            id: "main".to_string(),
            ..Panel::default()
        };
        main_panel.rows.push(Row {
            cells: vec![Cell::PanelRef(PanelRef {
                panel_id: "panel_a".to_string(),
                width: Sizing::default(),
                height: Sizing::default(),
            })],
        });

        let mut panel_a = Panel {
            id: "panel_a".to_string(),
            ..Panel::default()
        };
        panel_a.rows.push(Row {
            cells: vec![Cell::PanelRef(PanelRef {
                panel_id: "panel_b".to_string(),
                width: Sizing::default(),
                height: Sizing::default(),
            })],
        });

        let mut panel_b = Panel {
            id: "panel_b".to_string(),
            ..Panel::default()
        };
        panel_b.rows.push(Row {
            cells: vec![Cell::PanelRef(PanelRef {
                panel_id: "main".to_string(),
                width: Sizing::default(),
                height: Sizing::default(),
            })],
        });

        layout.panels.insert("main".to_string(), main_panel);
        layout.panels.insert("panel_a".to_string(), panel_a);
        layout.panels.insert("panel_b".to_string(), panel_b);

        let result = detect_circular_references(&layout);
        assert!(result.is_err(), "Should detect circular reference");

        let err = result.unwrap_err();
        let display_str = format!("{}", err);
        assert!(display_str.contains("Circular reference"));
        assert!(display_str.contains("->"), "Should show dependency chain");
    }

    /// Test 2: Max depth enforcement
    #[test]
    fn test_enforce_max_nesting_depth() {
        let mut layout = Layout::default();

        // Create a chain of panels that exceeds max depth
        // p0 -> p1 -> p2 -> p3 -> p4 -> p5 -> p6
        // Depths should be: p6=0, p5=1, p4=2, p3=3, p2=4, p1=5, p0=6 (exceeds limit of 5)
        for i in 0..=6 {
            let mut panel = Panel {
                id: format!("p{}", i),
                ..Panel::default()
            };

            if i < 6 {
                panel.rows.push(Row {
                    cells: vec![Cell::PanelRef(PanelRef {
                        panel_id: format!("p{}", i + 1),
                        width: Sizing::default(),
                        height: Sizing::default(),
                    })],
                });
            }

            layout.panels.insert(format!("p{}", i), panel);
        }

        layout.default_panel_id = "p0".to_string();

        let result = enforce_max_nesting_depth(&mut layout);
        assert!(result.is_err(), "Should reject excessive nesting depth");

        let err = result.unwrap_err();
        let display_str = format!("{}", err);
        assert!(display_str.contains("Maximum depth exceeded"));
        assert!(display_str.contains("limit: 5"));
    }

    /// Test 3: Required field validation
    #[test]
    fn test_validate_required_fields() {
        let layout = Layout {
            name: String::new(),
            version: String::new(),
            default_panel_id: String::new(),
            description: None,
            author: None,
            ..Layout::default()
        };

        let mut warnings = Vec::new();
        validate_required_fields(&layout, &mut warnings);

        assert!(warnings.len() >= 3, "Should warn about empty required fields");

        // Check that we have warnings about missing fields
        let warning_messages: Vec<String> = warnings.iter().map(|w| w.message.clone()).collect();
        assert!(
            warning_messages
                .iter()
                .any(|m| m.contains("name")),
            "Should warn about empty name"
        );
        assert!(
            warning_messages
                .iter()
                .any(|m| m.contains("version")),
            "Should warn about empty version"
        );
        assert!(
            warning_messages
                .iter()
                .any(|m| m.contains("description")),
            "Should warn about missing description"
        );
    }

    /// Test 4: Warning collection and sorting
    #[test]
    fn test_collect_warnings() {
        let layout = Layout::default();

        let warnings = vec![
            ValidationIssue::new(Severity::Warning, "Warning 1", "field_z"),
            ValidationIssue::new(Severity::Error, "Error 1", "field_m"),
            ValidationIssue::new(Severity::Warning, "Warning 2", "field_a"),
        ];

        let result = collect_warnings(layout, warnings);

        assert_eq!(result.warnings.len(), 3);
        assert!(result.has_warnings());
        assert_eq!(result.warning_count(), 3);

        // Errors should come first
        assert_eq!(result.warnings[0].severity, Severity::Error);
        assert_eq!(result.warnings[0].message, "Error 1");

        // Warnings should be sorted by field path
        assert_eq!(result.warnings[1].field_path, "field_a");
        assert_eq!(result.warnings[2].field_path, "field_z");
    }

    /// Test 5: Sizing validation
    #[test]
    fn test_validate_sizing() {
        let mut warnings = Vec::new();

        // Test negative relative size
        validate_sizing(&Sizing::Relative(-1.0), "test.width", &mut warnings);
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("not positive")),
            "Should warn about negative size"
        );

        warnings.clear();

        // Test unusually large relative size
        validate_sizing(&Sizing::Relative(15.0), "test.width", &mut warnings);
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("unusually large")),
            "Should warn about unusually large size"
        );

        warnings.clear();

        // Test invalid pixel format
        validate_sizing(&Sizing::Pixels("20".to_string()), "test.width", &mut warnings);
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("Invalid pixel format")),
            "Should warn about invalid pixel format"
        );

        warnings.clear();

        // Test invalid pixel value
        validate_sizing(&Sizing::Pixels("abcpx".to_string()), "test.width", &mut warnings);
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("Invalid pixel value")),
            "Should warn about invalid pixel value"
        );

        warnings.clear();

        // Test valid sizing
        validate_sizing(&Sizing::Relative(1.0), "test.width", &mut warnings);
        validate_sizing(&Sizing::Pixels("20px".to_string()), "test.height", &mut warnings);
        assert_eq!(warnings.len(), 0, "Should not warn about valid sizing");
    }

    /// Test 6: Modifier combination validation
    #[test]
    fn test_validate_modifier_combinations() {
        let mut warnings = Vec::new();

        // Test empty modifier combo
        let mut alternatives = HashMap::new();
        alternatives.insert(
            AlternativeKey::ModifierCombo(vec![]),
            Action::Character('x'),
        );
        validate_modifier_combinations(&alternatives, "test_key", &mut warnings);
        assert!(
            warnings.iter().any(|w| w.message.contains("empty")),
            "Should warn about empty modifier combination"
        );

        warnings.clear();

        // Test duplicate modifiers
        alternatives.clear();
        alternatives.insert(
            AlternativeKey::ModifierCombo(vec![Modifier::Shift, Modifier::Shift]),
            Action::Character('x'),
        );
        validate_modifier_combinations(&alternatives, "test_key", &mut warnings);
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("Duplicate modifier")),
            "Should warn about duplicate modifiers"
        );

        warnings.clear();

        // Test all four modifiers
        alternatives.clear();
        alternatives.insert(
            AlternativeKey::ModifierCombo(vec![
                Modifier::Shift,
                Modifier::Ctrl,
                Modifier::Alt,
                Modifier::Super,
            ]),
            Action::Character('x'),
        );
        validate_modifier_combinations(&alternatives, "test_key", &mut warnings);
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("all four modifiers")),
            "Should warn about using all four modifiers"
        );

        warnings.clear();

        // Test sorted modifiers - [Shift, Ctrl] is sorted
        alternatives.clear();
        alternatives.insert(
            AlternativeKey::ModifierCombo(vec![Modifier::Shift, Modifier::Ctrl]),
            Action::Character('x'),
        );
        validate_modifier_combinations(&alternatives, "test_key", &mut warnings);
        // This should NOT warn because [Shift, Ctrl] is already sorted
        assert!(
            !warnings
                .iter()
                .any(|w| w.message.contains("canonical order")),
            "Should NOT warn about sorted modifiers"
        );
    }

    /// Test 7: Panel reference validation
    #[test]
    fn test_validate_panel_references() {
        let mut layout = Layout::default();

        // Add a panel that references a non-existent panel
        let mut main_panel = Panel {
            id: "main".to_string(),
            ..Panel::default()
        };
        main_panel.rows.push(Row {
            cells: vec![Cell::PanelRef(PanelRef {
                panel_id: "nonexistent".to_string(),
                width: Sizing::default(),
                height: Sizing::default(),
            })],
        });

        layout.panels.insert("main".to_string(), main_panel);
        layout.default_panel_id = "main".to_string();

        let mut warnings = Vec::new();
        let result = validate_panel_references(&layout, &mut warnings);

        assert!(result.is_ok(), "Non-existent panel ref should be a warning");
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("does not exist")),
            "Should warn about non-existent panel"
        );
    }

    /// Test 8: Full validation integration
    #[test]
    fn test_validate_layout_integration() {
        let mut layout = Layout {
            name: "Test Layout".to_string(),
            version: "1.0".to_string(),
            default_panel_id: "main".to_string(),
            ..Layout::default()
        };

        // Add a valid key with a sizing issue
        let mut main_panel = Panel {
            id: "main".to_string(),
            ..Panel::default()
        };
        main_panel.rows.push(Row {
            cells: vec![Cell::Key(Key {
                label: "A".to_string(),
                code: KeyCode::Unicode('a'),
                width: Sizing::Relative(-1.0), // Invalid: negative
                ..Key::default()
            })],
        });

        layout.panels.insert("main".to_string(), main_panel);

        let result = validate_layout(layout);
        assert!(result.is_ok(), "Should succeed with warnings");

        let parse_result = result.unwrap();
        assert!(parse_result.has_warnings(), "Should have warnings");

        let warning_messages: Vec<String> = parse_result.warnings.iter().map(|w| w.message.clone()).collect();
        assert!(
            warning_messages.iter().any(|m| m.contains("not positive")),
            "Should warn about negative sizing"
        );
    }
}
