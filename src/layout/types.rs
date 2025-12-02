// SPDX-License-Identifier: GPL-3.0-only

//! Core data types for the JSON Layout Parser.
//!
//! This module defines the fundamental types for parsing keyboard layout definitions
//! from JSON files, including error types, validation structures, and data models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ============================================================================
// Error Handling Types
// ============================================================================

/// Severity level for validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Fatal error that prevents layout from being used
    Error,
    /// Non-fatal issue that should be addressed
    Warning,
}

/// A validation issue discovered during layout parsing.
///
/// Contains detailed information about problems found in the layout definition,
/// including severity, location, and suggestions for fixes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    /// Severity level (Error or Warning)
    pub severity: Severity,
    /// Human-readable description of the issue
    pub message: String,
    /// Line number in the JSON file where the issue was found (if available)
    pub line_number: Option<usize>,
    /// Path to the field that caused the issue (e.g., "panels[0].rows[1].cells[2]")
    pub field_path: String,
    /// Optional suggestion for how to fix the issue
    pub suggestion: Option<String>,
}

impl ValidationIssue {
    /// Creates a new validation issue.
    pub fn new(
        severity: Severity,
        message: impl Into<String>,
        field_path: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            message: message.into(),
            line_number: None,
            field_path: field_path.into(),
            suggestion: None,
        }
    }

    /// Adds a line number to the validation issue.
    pub fn with_line_number(mut self, line_number: usize) -> Self {
        self.line_number = Some(line_number);
        self
    }

    /// Adds a suggestion to the validation issue.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity_str = match self.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARNING",
        };

        write!(f, "[{}] {}: {}", severity_str, self.field_path, self.message)?;

        if let Some(line) = self.line_number {
            write!(f, " (line {})", line)?;
        }

        if let Some(suggestion) = &self.suggestion {
            write!(f, "\n  Suggestion: {}", suggestion)?;
        }

        Ok(())
    }
}

/// Error type for layout parsing operations.
///
/// This error type follows the canonical error struct pattern with context fields
/// for helpful error messages. It wraps different error sources and provides
/// line number information when available.
#[derive(Debug)]
pub enum ParseError {
    /// I/O error occurred while reading layout file
    IoError {
        /// The underlying I/O error
        source: std::io::Error,
        /// Optional file path that caused the error
        file_path: Option<String>,
        /// Optional suggestion for fixing the error
        suggestion: Option<String>,
    },

    /// JSON parsing error
    JsonError {
        /// The underlying JSON parsing error
        source: serde_json::Error,
        /// Optional file path being parsed
        file_path: Option<String>,
        /// Line number where the error occurred (from serde_json)
        line_number: Option<usize>,
        /// Optional suggestion for fixing the error
        suggestion: Option<String>,
    },

    /// Validation errors found during parsing
    ValidationError {
        /// List of validation issues found
        issues: Vec<ValidationIssue>,
        /// Optional file path being validated
        file_path: Option<String>,
    },

    /// Circular reference detected in panel references or inheritance
    CircularReference {
        /// Description of the circular dependency
        message: String,
        /// Chain of references forming the cycle (e.g., "panel_a -> panel_b -> panel_a")
        chain: String,
        /// Optional file path where cycle was detected
        file_path: Option<String>,
        /// Optional suggestion for breaking the cycle
        suggestion: Option<String>,
    },

    /// Maximum nesting or inheritance depth exceeded
    MaxDepthExceeded {
        /// Description of what exceeded the depth limit
        message: String,
        /// The depth limit that was exceeded
        max_depth: usize,
        /// The actual depth reached
        actual_depth: usize,
        /// Optional file path where depth was exceeded
        file_path: Option<String>,
        /// Optional suggestion for reducing depth
        suggestion: Option<String>,
    },
}

impl ParseError {
    /// Creates an I/O error with context.
    pub fn io_error(source: std::io::Error) -> Self {
        Self::IoError {
            source,
            file_path: None,
            suggestion: None,
        }
    }

    /// Creates an I/O error with file path.
    pub fn io_error_with_path(source: std::io::Error, file_path: impl Into<String>) -> Self {
        Self::IoError {
            source,
            file_path: Some(file_path.into()),
            suggestion: Some("Check that the file exists and you have read permissions".into()),
        }
    }

    /// Creates a JSON parsing error with context.
    pub fn json_error(source: serde_json::Error) -> Self {
        let line_number = source.line().into();
        Self::JsonError {
            source,
            file_path: None,
            line_number,
            suggestion: Some("Check the JSON syntax at the indicated line".into()),
        }
    }

    /// Creates a JSON parsing error with file path.
    pub fn json_error_with_path(
        source: serde_json::Error,
        file_path: impl Into<String>,
    ) -> Self {
        let line_number = source.line().into();
        Self::JsonError {
            source,
            file_path: Some(file_path.into()),
            line_number,
            suggestion: Some("Check the JSON syntax at the indicated line".into()),
        }
    }

    /// Creates a validation error from a list of issues.
    pub fn validation_error(issues: Vec<ValidationIssue>) -> Self {
        Self::ValidationError {
            issues,
            file_path: None,
        }
    }

    /// Creates a validation error with file path.
    pub fn validation_error_with_path(
        issues: Vec<ValidationIssue>,
        file_path: impl Into<String>,
    ) -> Self {
        Self::ValidationError {
            issues,
            file_path: Some(file_path.into()),
        }
    }

    /// Creates a circular reference error.
    pub fn circular_reference(message: impl Into<String>, chain: impl Into<String>) -> Self {
        Self::CircularReference {
            message: message.into(),
            chain: chain.into(),
            file_path: None,
            suggestion: Some("Remove or break the circular dependency".into()),
        }
    }

    /// Creates a max depth exceeded error.
    pub fn max_depth_exceeded(
        message: impl Into<String>,
        max_depth: usize,
        actual_depth: usize,
    ) -> Self {
        Self::MaxDepthExceeded {
            message: message.into(),
            max_depth,
            actual_depth,
            file_path: None,
            suggestion: Some(format!(
                "Reduce nesting depth to {} or less",
                max_depth
            )),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::IoError {
                source,
                file_path,
                suggestion,
            } => {
                write!(f, "I/O error")?;
                if let Some(path) = file_path {
                    write!(f, " reading file '{}'", path)?;
                }
                write!(f, ": {}", source)?;
                if let Some(hint) = suggestion {
                    write!(f, "\n  Suggestion: {}", hint)?;
                }
            }
            ParseError::JsonError {
                source,
                file_path,
                line_number,
                suggestion,
            } => {
                write!(f, "JSON parsing error")?;
                if let Some(path) = file_path {
                    write!(f, " in file '{}'", path)?;
                }
                if let Some(line) = line_number {
                    write!(f, " at line {}", line)?;
                }
                write!(f, ": {}", source)?;
                if let Some(hint) = suggestion {
                    write!(f, "\n  Suggestion: {}", hint)?;
                }
            }
            ParseError::ValidationError { issues, file_path } => {
                write!(f, "Validation failed")?;
                if let Some(path) = file_path {
                    write!(f, " for file '{}'", path)?;
                }
                write!(f, " with {} issue(s):\n", issues.len())?;
                for (i, issue) in issues.iter().enumerate() {
                    write!(f, "  {}. {}", i + 1, issue)?;
                    if i < issues.len() - 1 {
                        writeln!(f)?;
                    }
                }
            }
            ParseError::CircularReference {
                message,
                chain,
                file_path,
                suggestion,
            } => {
                write!(f, "Circular reference detected")?;
                if let Some(path) = file_path {
                    write!(f, " in file '{}'", path)?;
                }
                write!(f, ": {}", message)?;
                write!(f, "\n  Dependency chain: {}", chain)?;
                if let Some(hint) = suggestion {
                    write!(f, "\n  Suggestion: {}", hint)?;
                }
            }
            ParseError::MaxDepthExceeded {
                message,
                max_depth,
                actual_depth,
                file_path,
                suggestion,
            } => {
                write!(f, "Maximum depth exceeded")?;
                if let Some(path) = file_path {
                    write!(f, " in file '{}'", path)?;
                }
                write!(
                    f,
                    ": {} (limit: {}, actual: {})",
                    message, max_depth, actual_depth
                )?;
                if let Some(hint) = suggestion {
                    write!(f, "\n  Suggestion: {}", hint)?;
                }
            }
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::IoError { source, .. } => Some(source),
            ParseError::JsonError { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        Self::io_error(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        Self::json_error(err)
    }
}

// ============================================================================
// ParseResult Type
// ============================================================================

/// Result of successfully parsing a layout with optional warnings.
///
/// This struct allows the parser to operate in permissive mode, returning
/// a valid layout even when non-fatal validation issues are found.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseResult<T> {
    /// The successfully parsed layout
    pub layout: T,
    /// Non-fatal validation warnings
    pub warnings: Vec<ValidationIssue>,
}

impl<T> ParseResult<T> {
    /// Creates a new parse result with no warnings.
    pub fn new(layout: T) -> Self {
        Self {
            layout,
            warnings: Vec::new(),
        }
    }

    /// Creates a new parse result with warnings.
    pub fn with_warnings(layout: T, warnings: Vec<ValidationIssue>) -> Self {
        Self { layout, warnings }
    }

    /// Returns true if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Returns the number of warnings.
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Consumes the result and returns the layout, discarding warnings.
    pub fn into_layout(self) -> T {
        self.layout
    }
}

// ============================================================================
// Layout Data Structures (Task Group 2)
// ============================================================================

/// Key code representation for keyboard keys.
///
/// Keys can emit either Unicode characters or system keysyms (like modifiers).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeyCode {
    /// Regular character key (e.g., 'a', '1', ' ')
    Unicode(char),
    /// System keysym for modifiers and special keys (e.g., "Shift_L", "Control_L")
    Keysym(String),
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyCode::Unicode(c) => write!(f, "'{}'", c),
            KeyCode::Keysym(s) => write!(f, "Keysym({})", s),
        }
    }
}

impl Default for KeyCode {
    fn default() -> Self {
        KeyCode::Unicode(' ')
    }
}

/// Sizing specification for keys and widgets.
///
/// Supports both relative sizing (multiples of standard size) and
/// DPI-aware pixel overrides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Sizing {
    /// Relative size multiplier (1.0 = standard size)
    Relative(f32),
    /// Pixel override with DPI-aware scaling (format: "20px")
    Pixels(String),
}

impl Default for Sizing {
    fn default() -> Self {
        Sizing::Relative(1.0)
    }
}

impl Sizing {
    /// Returns the relative value for layout calculations.
    /// For Relative sizing, returns the multiplier directly.
    /// For Pixels sizing, returns 1.0 as a default unit for layout calculations.
    pub fn as_relative(&self) -> f32 {
        match self {
            Sizing::Relative(r) => *r,
            Sizing::Pixels(_) => 1.0,
        }
    }
}

/// Keyboard modifier keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Modifier {
    /// Shift modifier
    Shift,
    /// Control modifier
    Ctrl,
    /// Alt modifier
    Alt,
    /// Super/Windows/Meta modifier
    Super,
}

/// Swipe direction for gesture alternatives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SwipeDirection {
    /// Swipe up
    Up,
    /// Swipe down
    Down,
    /// Swipe left
    Left,
    /// Swipe right
    Right,
}

/// Alternative key activation method.
///
/// Used as HashMap key for key alternatives, supporting modifier combinations
/// and swipe gestures.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AlternativeKey {
    /// Single modifier key
    SingleModifier(Modifier),
    /// Combination of multiple modifiers (sorted for consistency)
    ModifierCombo(Vec<Modifier>),
    /// Swipe gesture
    Swipe(SwipeDirection),
}

impl AlternativeKey {
    /// Creates a modifier combination with sorted modifiers for consistent matching.
    pub fn modifier_combo(mut modifiers: Vec<Modifier>) -> Self {
        modifiers.sort();
        AlternativeKey::ModifierCombo(modifiers)
    }
}

/// Action to perform when a key or alternative is activated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Action {
    /// Emit a single character
    Character(char),
    /// Emit a key code
    KeyCode(KeyCode),
    /// Execute a script (format: "script:custom_macro")
    Script(String),
    /// Switch to a different panel (format: "panel(panel_name)")
    PanelSwitch(String),
}

/// Default value for `stickyrelease` field.
///
/// Returns `true` because the default behavior for sticky keys is one-shot mode,
/// where the modifier releases automatically after the next key press.
fn default_stickyrelease() -> bool {
    true
}

/// A keyboard key definition.
///
/// Contains the display label, key code, sizing, and alternative actions
/// for modifiers and gestures.
///
/// # Sticky Key Behavior
///
/// The `sticky` and `stickyrelease` fields control modifier key behavior:
///
/// - `sticky: false` (default): Key must be held down to keep modifier active.
/// - `sticky: true, stickyrelease: true` (default): One-shot mode. Modifier activates
///   on tap and automatically releases after the next key press.
/// - `sticky: true, stickyrelease: false`: Toggle mode. Modifier activates on tap
///   and stays active until the same key is tapped again to deactivate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Key {
    /// Display label shown on the key
    pub label: String,

    /// Key code emitted when pressed
    #[serde(default)]
    pub code: KeyCode,

    /// Optional identifier for script references and overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,

    /// Width sizing
    #[serde(default)]
    pub width: Sizing,

    /// Height sizing
    #[serde(default)]
    pub height: Sizing,

    /// Minimum width in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<u32>,

    /// Minimum height in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<u32>,

    /// Alternative actions for modifiers and swipes
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub alternatives: HashMap<AlternativeKey, Action>,

    /// Whether this is a sticky key (toggle mode).
    ///
    /// When `true`, the key can be tapped to toggle its state rather than
    /// requiring it to be held down. Used primarily for modifier keys.
    #[serde(default)]
    pub sticky: bool,

    /// Whether the sticky key should release after the next key press.
    ///
    /// Only relevant when `sticky` is `true`:
    /// - `true` (default): One-shot behavior. The sticky modifier releases
    ///   automatically after emitting a combo with the next key.
    /// - `false`: Toggle behavior. The sticky modifier stays active until
    ///   the user taps the modifier key again to deactivate it.
    #[serde(default = "default_stickyrelease")]
    pub stickyrelease: bool,
}

impl Default for Key {
    fn default() -> Self {
        Self {
            label: String::new(),
            code: KeyCode::default(),
            identifier: None,
            width: Sizing::default(),
            height: Sizing::default(),
            min_width: None,
            min_height: None,
            alternatives: HashMap::new(),
            sticky: false,
            stickyrelease: true, // Default to one-shot behavior
        }
    }
}

/// A widget embedded in the keyboard layout.
///
/// Widgets are specialized UI components like trackpads or autocomplete bars.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Widget {
    /// Widget type identifier (e.g., "trackpad", "autocomplete")
    pub widget_type: String,

    /// Width sizing
    #[serde(default)]
    pub width: Sizing,

    /// Height sizing
    #[serde(default)]
    pub height: Sizing,
}

/// A reference to another panel for embedding.
///
/// Allows panels to be nested within other panels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanelRef {
    /// ID of the panel to embed
    pub panel_id: String,

    /// Width sizing
    #[serde(default)]
    pub width: Sizing,

    /// Height sizing
    #[serde(default)]
    pub height: Sizing,
}

/// A cell in a keyboard row.
///
/// Can contain a key, widget, or panel reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Cell {
    /// A keyboard key
    Key(Key),
    /// An embedded widget
    Widget(Widget),
    /// A reference to another panel
    PanelRef(PanelRef),
}

/// A row of cells in a panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Row {
    /// Cells in this row
    pub cells: Vec<Cell>,
}

impl Default for Row {
    fn default() -> Self {
        Self { cells: Vec::new() }
    }
}

/// A keyboard panel containing rows of keys.
///
/// Panels are the main organizational unit and can be switched between.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Panel {
    /// Unique identifier for this panel
    pub id: String,

    /// Optional padding suggestion (in pixels)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<f32>,

    /// Optional margin suggestion (in pixels)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<f32>,

    /// Nesting depth (for tracking embedded panels)
    #[serde(default)]
    pub nesting_depth: u8,

    /// Rows of cells in this panel
    #[serde(default)]
    pub rows: Vec<Row>,
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            id: String::new(),
            padding: None,
            margin: None,
            nesting_depth: 0,
            rows: Vec::new(),
        }
    }
}

/// A complete keyboard layout definition.
///
/// Contains metadata and a collection of panels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    /// Layout name
    pub name: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Optional language
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Optional locale
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    /// Layout version
    pub version: String,

    /// ID of the default panel to show
    pub default_panel_id: String,

    /// Optional path to parent layout for inheritance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherits: Option<String>,

    /// Panels indexed by ID
    #[serde(default)]
    pub panels: HashMap<String, Panel>,
}

impl Default for Layout {
    fn default() -> Self {
        let mut panels = HashMap::new();
        panels.insert(
            "main".to_string(),
            Panel {
                id: "main".to_string(),
                ..Panel::default()
            },
        );

        Self {
            name: String::new(),
            description: None,
            author: None,
            language: None,
            locale: None,
            version: "1.0".to_string(),
            default_panel_id: "main".to_string(),
            inherits: None,
            panels,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Task 1.1: Focused tests for error handling (2-8 tests)
    // ========================================================================

    /// Test 1: JSON parse error includes line number
    #[test]
    fn test_json_error_includes_line_number() {
        // Create invalid JSON that will fail on line 3
        let invalid_json = r#"{
  "name": "test",
  "invalid":
}"#;

        let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
        let json_err = result.unwrap_err();

        let parse_err = ParseError::json_error_with_path(json_err, "test.json");

        let display_str = format!("{}", parse_err);
        assert!(
            display_str.contains("line"),
            "Error message should include line number"
        );
        assert!(
            display_str.contains("test.json"),
            "Error message should include file path"
        );
        assert!(
            display_str.contains("Suggestion"),
            "Error message should include suggestion"
        );
    }

    /// Test 2: Validation error with suggestion
    #[test]
    fn test_validation_error_with_suggestion() {
        let issue = ValidationIssue::new(
            Severity::Error,
            "Width must be positive",
            "panels[0].rows[1].cells[2].width",
        )
        .with_line_number(42)
        .with_suggestion("Use a positive number like 1 or 1.5");

        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.message, "Width must be positive");
        assert_eq!(issue.line_number, Some(42));
        assert_eq!(issue.field_path, "panels[0].rows[1].cells[2].width");

        let display_str = format!("{}", issue);
        assert!(display_str.contains("ERROR"));
        assert!(display_str.contains("line 42"));
        assert!(display_str.contains("Suggestion"));
        assert!(display_str.contains("positive number"));
    }

    /// Test 3: Circular reference detection
    #[test]
    fn test_circular_reference_error() {
        let err = ParseError::circular_reference(
            "Panel 'main' references itself",
            "main -> numpad -> symbols -> main",
        );

        let display_str = format!("{}", err);
        assert!(display_str.contains("Circular reference"));
        assert!(display_str.contains("main -> numpad -> symbols -> main"));
        assert!(display_str.contains("Suggestion"));
    }

    /// Test 4: Max depth exceeded error
    #[test]
    fn test_max_depth_exceeded_error() {
        let err = ParseError::max_depth_exceeded("Panel nesting too deep", 5, 7);

        let display_str = format!("{}", err);
        assert!(display_str.contains("Maximum depth exceeded"));
        assert!(display_str.contains("limit: 5"));
        assert!(display_str.contains("actual: 7"));
        assert!(display_str.contains("Reduce nesting depth"));
    }

    /// Test 5: ParseResult with warnings
    #[test]
    fn test_parse_result_with_warnings() {
        let layout = "test_layout";
        let warnings = vec![
            ValidationIssue::new(Severity::Warning, "Missing description", "description"),
            ValidationIssue::new(Severity::Warning, "Missing author", "author"),
        ];

        let result = ParseResult::with_warnings(layout, warnings.clone());

        assert_eq!(result.layout, "test_layout");
        assert!(result.has_warnings());
        assert_eq!(result.warning_count(), 2);
        assert_eq!(result.warnings.len(), 2);
    }

    /// Test 6: ParseResult without warnings
    #[test]
    fn test_parse_result_without_warnings() {
        let layout = "test_layout";
        let result = ParseResult::new(layout);

        assert_eq!(result.layout, "test_layout");
        assert!(!result.has_warnings());
        assert_eq!(result.warning_count(), 0);

        let extracted_layout = result.into_layout();
        assert_eq!(extracted_layout, "test_layout");
    }

    /// Test 7: ValidationIssue Display format
    #[test]
    fn test_validation_issue_display() {
        let warning = ValidationIssue::new(
            Severity::Warning,
            "Key is unusually large",
            "panels[0].rows[0].cells[0]",
        )
        .with_line_number(15)
        .with_suggestion("Consider using width <= 10");

        let display_str = format!("{}", warning);

        // Check that all components are present
        assert!(display_str.contains("WARNING"));
        assert!(display_str.contains("panels[0].rows[0].cells[0]"));
        assert!(display_str.contains("Key is unusually large"));
        assert!(display_str.contains("line 15"));
        assert!(display_str.contains("Suggestion: Consider using width <= 10"));
    }

    /// Test 8: I/O error with file path context
    #[test]
    fn test_io_error_with_context() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let parse_err = ParseError::io_error_with_path(io_err, "/path/to/layout.json");

        let display_str = format!("{}", parse_err);
        assert!(display_str.contains("I/O error"));
        assert!(display_str.contains("/path/to/layout.json"));
        assert!(display_str.contains("file not found"));
        assert!(display_str.contains("Suggestion"));
    }

    // ========================================================================
    // Task 2.1: Focused tests for data structures (2-8 tests)
    // ========================================================================

    /// Test 1: Layout creation with default panel
    #[test]
    fn test_layout_with_default_panel() {
        let layout = Layout::default();

        assert_eq!(layout.name, "");
        assert_eq!(layout.version, "1.0");
        assert_eq!(layout.default_panel_id, "main");
        assert!(layout.panels.contains_key("main"));

        let main_panel = &layout.panels["main"];
        assert_eq!(main_panel.id, "main");
        assert_eq!(main_panel.nesting_depth, 0);
    }

    /// Test 2: Panel with rows
    #[test]
    fn test_panel_with_rows() {
        let mut panel = Panel {
            id: "test_panel".to_string(),
            rows: vec![Row::default(), Row::default()],
            ..Panel::default()
        };

        assert_eq!(panel.id, "test_panel");
        assert_eq!(panel.rows.len(), 2);
        assert_eq!(panel.nesting_depth, 0);

        panel.nesting_depth = 2;
        assert_eq!(panel.nesting_depth, 2);
    }

    /// Test 3: Key with alternatives
    #[test]
    fn test_key_with_alternatives() {
        let mut key = Key {
            label: "a".to_string(),
            code: KeyCode::Unicode('a'),
            identifier: Some("key_a".to_string()),
            ..Key::default()
        };

        // Add shift alternative
        key.alternatives.insert(
            AlternativeKey::SingleModifier(Modifier::Shift),
            Action::Character('A'),
        );

        // Add swipe alternative
        key.alternatives.insert(
            AlternativeKey::Swipe(SwipeDirection::Up),
            Action::Character('@'),
        );

        assert_eq!(key.label, "a");
        assert_eq!(key.code, KeyCode::Unicode('a'));
        assert_eq!(key.alternatives.len(), 2);
        assert!(!key.sticky);
        assert!(key.stickyrelease); // Default should be true
    }

    /// Test 4: Sizing enum variants
    #[test]
    fn test_sizing_variants() {
        let relative = Sizing::Relative(1.5);
        let pixels = Sizing::Pixels("20px".to_string());

        match relative {
            Sizing::Relative(val) => assert_eq!(val, 1.5),
            _ => panic!("Expected Relative variant"),
        }

        match pixels {
            Sizing::Pixels(val) => assert_eq!(val, "20px"),
            _ => panic!("Expected Pixels variant"),
        }

        // Test default
        let default_sizing = Sizing::default();
        match default_sizing {
            Sizing::Relative(val) => assert_eq!(val, 1.0),
            _ => panic!("Expected default to be Relative(1.0)"),
        }
    }

    /// Test 5: KeyCode Display implementation
    #[test]
    fn test_keycode_display() {
        let unicode = KeyCode::Unicode('x');
        let keysym = KeyCode::Keysym("Shift_L".to_string());

        assert_eq!(format!("{}", unicode), "'x'");
        assert!(format!("{}", keysym).contains("Shift_L"));
    }

    /// Test 6: AlternativeKey modifier combo with sorting
    #[test]
    fn test_alternative_key_modifier_combo() {
        // Create combo with unsorted modifiers
        let combo = AlternativeKey::modifier_combo(vec![Modifier::Alt, Modifier::Ctrl]);

        match combo {
            AlternativeKey::ModifierCombo(mods) => {
                // Should be sorted: Ctrl comes before Alt
                assert_eq!(mods.len(), 2);
                // We know Ctrl < Alt based on enum definition order
            }
            _ => panic!("Expected ModifierCombo variant"),
        }
    }

    /// Test 7: Cell enum variants
    #[test]
    fn test_cell_variants() {
        let key_cell = Cell::Key(Key {
            label: "Space".to_string(),
            code: KeyCode::Unicode(' '),
            ..Key::default()
        });

        let widget_cell = Cell::Widget(Widget {
            widget_type: "trackpad".to_string(),
            width: Sizing::Relative(3.0),
            height: Sizing::Relative(2.0),
        });

        let panel_ref_cell = Cell::PanelRef(PanelRef {
            panel_id: "numpad".to_string(),
            width: Sizing::Relative(2.0),
            height: Sizing::Relative(3.0),
        });

        // Verify variants exist and can be constructed
        match key_cell {
            Cell::Key(k) => assert_eq!(k.label, "Space"),
            _ => panic!("Expected Key variant"),
        }

        match widget_cell {
            Cell::Widget(w) => assert_eq!(w.widget_type, "trackpad"),
            _ => panic!("Expected Widget variant"),
        }

        match panel_ref_cell {
            Cell::PanelRef(p) => assert_eq!(p.panel_id, "numpad"),
            _ => panic!("Expected PanelRef variant"),
        }
    }

    /// Test 8: Action enum variants
    #[test]
    fn test_action_variants() {
        let char_action = Action::Character('x');
        let keycode_action = Action::KeyCode(KeyCode::Keysym("Return".to_string()));
        let script_action = Action::Script("script:my_macro".to_string());
        let panel_action = Action::PanelSwitch("panel(numpad)".to_string());

        match char_action {
            Action::Character(c) => assert_eq!(c, 'x'),
            _ => panic!("Expected Character variant"),
        }

        match keycode_action {
            Action::KeyCode(KeyCode::Keysym(s)) => assert_eq!(s, "Return"),
            _ => panic!("Expected KeyCode variant"),
        }

        match script_action {
            Action::Script(s) => assert!(s.starts_with("script:")),
            _ => panic!("Expected Script variant"),
        }

        match panel_action {
            Action::PanelSwitch(s) => assert!(s.starts_with("panel(")),
            _ => panic!("Expected PanelSwitch variant"),
        }
    }

    // ========================================================================
    // Task Group 2 - Task 2.1: Focused tests for stickyrelease field (3-4 tests)
    // ========================================================================

    /// Test 1: stickyrelease defaults to true when field is omitted
    #[test]
    fn test_stickyrelease_default_value() {
        // Test using Key::default()
        let key = Key::default();
        assert!(
            key.stickyrelease,
            "stickyrelease should default to true (one-shot behavior)"
        );

        // Test JSON deserialization without the field
        let json = r#"{
            "type": "key",
            "label": "Shift",
            "code": "Shift_L",
            "sticky": true
        }"#;
        let cell: Cell = serde_json::from_str(json).expect("Should parse key without stickyrelease");
        match cell {
            Cell::Key(key) => {
                assert!(key.sticky, "sticky should be true");
                assert!(
                    key.stickyrelease,
                    "stickyrelease should default to true when omitted from JSON"
                );
            }
            _ => panic!("Expected Key variant"),
        }
    }

    /// Test 2: Explicit false value for stickyrelease is preserved
    #[test]
    fn test_stickyrelease_explicit_false() {
        let json = r#"{
            "type": "key",
            "label": "Ctrl",
            "code": "Control_L",
            "sticky": true,
            "stickyrelease": false
        }"#;
        let cell: Cell = serde_json::from_str(json).expect("Should parse key with stickyrelease: false");
        match cell {
            Cell::Key(key) => {
                assert!(key.sticky, "sticky should be true");
                assert!(
                    !key.stickyrelease,
                    "stickyrelease should be false when explicitly set to false"
                );
            }
            _ => panic!("Expected Key variant"),
        }
    }

    /// Test 3: JSON deserialization with and without stickyrelease field
    #[test]
    fn test_stickyrelease_json_deserialization() {
        // Key with stickyrelease: true (explicit)
        let json_true = r#"{
            "type": "key",
            "label": "Shift",
            "code": "Shift_L",
            "sticky": true,
            "stickyrelease": true
        }"#;
        let cell_true: Cell = serde_json::from_str(json_true).expect("Should parse");
        match cell_true {
            Cell::Key(key) => {
                assert!(key.stickyrelease, "Explicit true should be preserved");
            }
            _ => panic!("Expected Key variant"),
        }

        // Key with stickyrelease omitted (should default to true)
        let json_omitted = r#"{
            "type": "key",
            "label": "Alt",
            "code": "Alt_L",
            "sticky": true
        }"#;
        let cell_omitted: Cell = serde_json::from_str(json_omitted).expect("Should parse");
        match cell_omitted {
            Cell::Key(key) => {
                assert!(key.stickyrelease, "Omitted should default to true");
            }
            _ => panic!("Expected Key variant"),
        }

        // Non-sticky key (stickyrelease is still present but not relevant)
        let json_non_sticky = r#"{
            "type": "key",
            "label": "A",
            "code": "a"
        }"#;
        let cell_non_sticky: Cell = serde_json::from_str(json_non_sticky).expect("Should parse");
        match cell_non_sticky {
            Cell::Key(key) => {
                assert!(!key.sticky, "Non-sticky key should have sticky = false");
                assert!(
                    key.stickyrelease,
                    "stickyrelease defaults to true even for non-sticky keys"
                );
            }
            _ => panic!("Expected Key variant"),
        }
    }

    /// Test 4: sticky + stickyrelease behavior combinations
    #[test]
    fn test_sticky_stickyrelease_combinations() {
        // Combination 1: sticky=false (hold behavior - stickyrelease is irrelevant)
        let hold_key = Key {
            label: "Shift".to_string(),
            code: KeyCode::Keysym("Shift_L".to_string()),
            sticky: false,
            stickyrelease: true, // Irrelevant when sticky=false
            ..Key::default()
        };
        assert!(!hold_key.sticky, "Hold mode: sticky should be false");

        // Combination 2: sticky=true, stickyrelease=true (one-shot behavior)
        let oneshot_key = Key {
            label: "Shift".to_string(),
            code: KeyCode::Keysym("Shift_L".to_string()),
            sticky: true,
            stickyrelease: true, // One-shot: releases after next key
            ..Key::default()
        };
        assert!(oneshot_key.sticky, "One-shot mode: sticky should be true");
        assert!(
            oneshot_key.stickyrelease,
            "One-shot mode: stickyrelease should be true"
        );

        // Combination 3: sticky=true, stickyrelease=false (toggle behavior)
        let toggle_key = Key {
            label: "Ctrl".to_string(),
            code: KeyCode::Keysym("Control_L".to_string()),
            sticky: true,
            stickyrelease: false, // Toggle: stays until manually toggled off
            ..Key::default()
        };
        assert!(toggle_key.sticky, "Toggle mode: sticky should be true");
        assert!(
            !toggle_key.stickyrelease,
            "Toggle mode: stickyrelease should be false"
        );

        // Verify serialization roundtrip preserves values
        let json = serde_json::to_string(&Cell::Key(toggle_key.clone())).expect("Should serialize");
        let parsed: Cell = serde_json::from_str(&json).expect("Should deserialize");
        match parsed {
            Cell::Key(key) => {
                assert!(key.sticky, "Roundtrip: sticky should be preserved");
                assert!(
                    !key.stickyrelease,
                    "Roundtrip: stickyrelease=false should be preserved"
                );
            }
            _ => panic!("Expected Key variant"),
        }
    }
}
