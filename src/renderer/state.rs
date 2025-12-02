// SPDX-License-Identifier: GPL-3.0-only

//! Renderer state management for the keyboard layout renderer.
//!
//! This module provides the core state structures for tracking keyboard rendering,
//! including pressed keys, sticky keys, panel animations, and toast notifications.

use std::collections::{HashSet, VecDeque};
use std::time::Instant;

use crate::input::ModifierState;
use crate::layout::{Layout, Modifier, Panel};

// ============================================================================
// Animation Constants
// ============================================================================

/// Duration of panel slide animations in milliseconds.
pub const ANIMATION_DURATION_MS: u64 = 250;

/// Animation frame interval for smooth 60fps animations in milliseconds.
pub const ANIMATION_FRAME_INTERVAL_MS: u64 = 16;

/// Duration of toast notifications in milliseconds.
pub const TOAST_DURATION_MS: u64 = 3000;

/// Timer tick interval for toast timeout checking in milliseconds.
///
/// The toast timer emits ticks at this interval to check if the
/// 3-second timeout has elapsed.
pub const TOAST_TIMER_INTERVAL_MS: u64 = 100;

/// Long press detection threshold in milliseconds.
///
/// A key press that exceeds this duration triggers long press behavior,
/// showing the swipe alternatives popup.
pub const LONG_PRESS_THRESHOLD_MS: u64 = 300;

/// Timer tick interval for long press detection in milliseconds.
///
/// The long press timer emits ticks at this interval to check if the
/// threshold has been exceeded.
pub const LONG_PRESS_TIMER_INTERVAL_MS: u64 = 50;

// ============================================================================
// Toast Types
// ============================================================================

/// Severity level for toast notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastSeverity {
    /// Informational message
    Info,
    /// Warning message
    Warning,
    /// Error message
    Error,
}

/// A toast notification message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    /// The message to display
    pub message: String,
    /// Severity level affecting visual styling
    pub severity: ToastSeverity,
}

impl Toast {
    /// Creates a new toast notification.
    pub fn new(message: impl Into<String>, severity: ToastSeverity) -> Self {
        Self {
            message: message.into(),
            severity,
        }
    }

    /// Creates an info toast.
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, ToastSeverity::Info)
    }

    /// Creates a warning toast.
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, ToastSeverity::Warning)
    }

    /// Creates an error toast.
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, ToastSeverity::Error)
    }
}

// ============================================================================
// Panel Animation Types
// ============================================================================

/// State for panel slide animations.
#[derive(Debug, Clone)]
pub struct PanelAnimation {
    /// ID of the panel being animated from
    pub from_panel_id: String,
    /// ID of the panel being animated to
    pub to_panel_id: String,
    /// Animation progress from 0.0 (start) to 1.0 (complete)
    pub progress: f32,
    /// When the animation started
    pub start_time: Instant,
}

impl PanelAnimation {
    /// Creates a new panel animation.
    pub fn new(from_panel_id: impl Into<String>, to_panel_id: impl Into<String>) -> Self {
        Self {
            from_panel_id: from_panel_id.into(),
            to_panel_id: to_panel_id.into(),
            progress: 0.0,
            start_time: Instant::now(),
        }
    }

    /// Updates the animation progress based on elapsed time.
    ///
    /// Returns `true` if the animation is complete.
    pub fn update(&mut self) -> bool {
        let elapsed_ms = self.start_time.elapsed().as_millis() as u64;
        self.progress = (elapsed_ms as f32 / ANIMATION_DURATION_MS as f32).min(1.0);
        self.progress >= 1.0
    }

    /// Returns `true` if the animation is complete.
    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }

    /// Applies an easing function to the progress for smoother animation.
    ///
    /// Uses ease-out-cubic for a natural deceleration effect.
    pub fn eased_progress(&self) -> f32 {
        // Ease-out-cubic: 1 - (1 - x)^3
        let x = self.progress;
        1.0 - (1.0 - x).powi(3)
    }
}

// ============================================================================
// Keyboard Renderer State
// ============================================================================

/// Main state struct for keyboard rendering.
///
/// Holds all state necessary to render the keyboard UI, including the current
/// layout, panel state, pressed keys, and animation state.
#[derive(Debug)]
pub struct KeyboardRenderer {
    /// The loaded keyboard layout
    pub layout: Layout,

    /// ID of the currently displayed panel
    pub current_panel_id: String,

    /// Set of key identifiers that are currently pressed
    pub pressed_keys: HashSet<String>,

    /// Set of sticky key identifiers that are currently active (for visual state)
    pub sticky_keys_active: HashSet<String>,

    /// Modifier state for tracking active modifiers (for input emission)
    ///
    /// This delegates to `ModifierState` from `src/input/modifier.rs` to avoid
    /// duplicating modifier tracking logic. The `sticky_keys_active` HashSet
    /// tracks visual state by key identifier, while this tracks logical
    /// modifier state by `Modifier` enum.
    modifier_state: ModifierState,

    /// Key identifier for the key being long-pressed (if any)
    pub long_press_key: Option<String>,

    /// When the long press started (if any)
    pub long_press_start: Option<Instant>,

    /// Whether a long press has been detected and popup is active
    pub long_press_active: bool,

    /// Current panel animation state (if animating)
    pub animation_state: Option<PanelAnimation>,

    /// Queue of pending toast notifications
    pub toast_queue: VecDeque<Toast>,

    /// Currently displayed toast with its display start time
    pub current_toast: Option<(Toast, Instant)>,
}

impl KeyboardRenderer {
    /// Creates a new keyboard renderer with the given layout.
    ///
    /// The renderer initializes to the layout's default panel.
    pub fn new(layout: Layout) -> Self {
        let current_panel_id = layout.default_panel_id.clone();
        Self {
            layout,
            current_panel_id,
            pressed_keys: HashSet::new(),
            sticky_keys_active: HashSet::new(),
            modifier_state: ModifierState::new(),
            long_press_key: None,
            long_press_start: None,
            long_press_active: false,
            animation_state: None,
            toast_queue: VecDeque::new(),
            current_toast: None,
        }
    }

    /// Returns a reference to the current panel.
    ///
    /// Returns `None` if the current panel ID does not exist in the layout.
    pub fn current_panel(&self) -> Option<&Panel> {
        self.layout.panels.get(&self.current_panel_id)
    }

    /// Returns a reference to a panel by ID.
    ///
    /// Returns `None` if the panel ID does not exist in the layout.
    pub fn get_panel(&self, panel_id: &str) -> Option<&Panel> {
        self.layout.panels.get(panel_id)
    }

    /// Returns `true` if the key with the given identifier is currently pressed.
    pub fn is_key_pressed(&self, identifier: &str) -> bool {
        self.pressed_keys.contains(identifier)
    }

    /// Returns `true` if the sticky key with the given identifier is currently active.
    pub fn is_sticky_active(&self, identifier: &str) -> bool {
        self.sticky_keys_active.contains(identifier)
    }

    // ========================================================================
    // Key Press State Tracking (Task 4.3)
    // ========================================================================

    /// Marks a key as pressed and starts the long press timer.
    ///
    /// This method:
    /// 1. Adds the key to the pressed keys set
    /// 2. Starts the long press timer for the key
    pub fn press_key(&mut self, identifier: impl Into<String>) {
        let id = identifier.into();
        self.pressed_keys.insert(id.clone());
        self.start_long_press_timer(&id);
    }

    /// Marks a key as released and cancels any long press timer.
    ///
    /// This method:
    /// 1. Removes the key from the pressed keys set
    /// 2. Cancels the long press timer if this key was being long-pressed
    /// 3. Resets the long press active state
    pub fn release_key(&mut self, identifier: &str) {
        self.pressed_keys.remove(identifier);

        // Cancel long press if this was the key being long-pressed
        if self.long_press_key.as_deref() == Some(identifier) {
            self.cancel_long_press();
        }
    }

    /// Starts the long press timer for a key.
    ///
    /// Records the key identifier and the current time so that
    /// `check_long_press_threshold` can determine if 300ms has elapsed.
    pub fn start_long_press_timer(&mut self, identifier: &str) {
        self.long_press_key = Some(identifier.to_string());
        self.long_press_start = Some(Instant::now());
        self.long_press_active = false;
    }

    /// Cancels the current long press timer.
    ///
    /// Clears the long press key, start time, and active flag.
    pub fn cancel_long_press(&mut self) {
        self.long_press_key = None;
        self.long_press_start = None;
        self.long_press_active = false;
    }

    /// Checks if the long press threshold has been exceeded.
    ///
    /// Returns `true` if:
    /// - A long press timer is active
    /// - At least `LONG_PRESS_THRESHOLD_MS` (300ms) has elapsed since the press
    /// - The long press has not already been activated
    ///
    /// When this returns `true`, it also sets `long_press_active` to `true`.
    pub fn check_long_press_threshold(&mut self) -> bool {
        if self.long_press_active {
            // Already activated, don't trigger again
            return false;
        }

        if let Some(start_time) = self.long_press_start {
            let elapsed_ms = start_time.elapsed().as_millis() as u64;
            if elapsed_ms >= LONG_PRESS_THRESHOLD_MS {
                self.long_press_active = true;
                return true;
            }
        }

        false
    }

    /// Returns `true` if a long press is currently active.
    pub fn is_long_press_active(&self) -> bool {
        self.long_press_active
    }

    /// Returns the identifier of the key currently being long-pressed (if any).
    pub fn long_press_key_identifier(&self) -> Option<&str> {
        self.long_press_key.as_deref()
    }

    /// Returns `true` if a long press timer is running.
    ///
    /// This is used to determine if the subscription should emit timer ticks.
    pub fn has_pending_long_press(&self) -> bool {
        self.long_press_key.is_some() && self.long_press_start.is_some() && !self.long_press_active
    }

    // ========================================================================
    // Sticky Key Management
    // ========================================================================

    /// Toggles a sticky key's active state.
    pub fn toggle_sticky(&mut self, identifier: impl Into<String>) {
        let id = identifier.into();
        if self.sticky_keys_active.contains(&id) {
            self.sticky_keys_active.remove(&id);
        } else {
            self.sticky_keys_active.insert(id);
        }
    }

    // ========================================================================
    // Modifier State Management (Task Group 4)
    // ========================================================================

    /// Activates a modifier key.
    ///
    /// This method handles all three modifier behaviors based on `stickyrelease`:
    ///
    /// - **One-shot** (`stickyrelease: true`): Modifier activates and will be
    ///   cleared after the next regular key is pressed via `clear_oneshot_modifiers()`.
    /// - **Toggle** (`stickyrelease: false`): Modifier activates and stays active
    ///   until explicitly deactivated by calling `deactivate_modifier()` or
    ///   toggling again.
    ///
    /// For hold behavior (non-sticky keys), this method is not called; instead,
    /// the modifier is tracked while the key is physically held down.
    ///
    /// # Arguments
    ///
    /// * `modifier` - The modifier to activate (Shift, Ctrl, Alt, Super)
    /// * `stickyrelease` - If `true`, the modifier will be cleared after the next key
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // One-shot Shift (clears after next key)
    /// renderer.activate_modifier(Modifier::Shift, true);
    ///
    /// // Toggle Ctrl (stays until manually deactivated)
    /// renderer.activate_modifier(Modifier::Ctrl, false);
    /// ```
    pub fn activate_modifier(&mut self, modifier: Modifier, stickyrelease: bool) {
        self.modifier_state.activate(modifier, stickyrelease);
    }

    /// Deactivates a modifier key.
    ///
    /// This removes the modifier from both the active set and the one-shot set.
    /// Used for:
    /// - Toggling off a toggle-mode modifier
    /// - Releasing a held (non-sticky) modifier
    ///
    /// # Arguments
    ///
    /// * `modifier` - The modifier to deactivate
    pub fn deactivate_modifier(&mut self, modifier: Modifier) {
        self.modifier_state.deactivate(modifier);
    }

    /// Checks if a modifier is currently active.
    ///
    /// Returns `true` if the modifier is active, regardless of whether it's
    /// in one-shot mode, toggle mode, or held.
    ///
    /// # Arguments
    ///
    /// * `modifier` - The modifier to check
    ///
    /// # Returns
    ///
    /// `true` if the modifier is active, `false` otherwise
    pub fn is_modifier_active(&self, modifier: Modifier) -> bool {
        self.modifier_state.is_active(modifier)
    }

    /// Returns a list of all currently active modifiers.
    ///
    /// The modifiers are returned in a consistent order (Shift, Ctrl, Alt, Super).
    /// This is useful for emitting combo key events.
    ///
    /// # Returns
    ///
    /// A `Vec` containing all active modifiers, sorted by enum order
    pub fn get_active_modifiers(&self) -> Vec<Modifier> {
        self.modifier_state.get_active_modifiers()
    }

    /// Clears all one-shot (sticky release) modifiers.
    ///
    /// This should be called after a regular key is pressed to implement
    /// one-shot modifier behavior. Modifiers activated with `stickyrelease=true`
    /// will be deactivated. Modifiers activated with `stickyrelease=false`
    /// (toggle mode) will remain active.
    ///
    /// Also updates the visual sticky key state (`sticky_keys_active`) to stay
    /// in sync with the logical modifier state.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // User taps Shift (one-shot), then types 'a'
    /// renderer.activate_modifier(Modifier::Shift, true);
    /// // ... emit Shift+A ...
    /// renderer.clear_oneshot_modifiers();
    /// // Now Shift is no longer active
    /// ```
    pub fn clear_oneshot_modifiers(&mut self) {
        // Get one-shot modifiers before clearing
        let oneshot_modifiers: Vec<Modifier> = [Modifier::Shift, Modifier::Ctrl, Modifier::Alt, Modifier::Super]
            .iter()
            .filter(|&&m| self.modifier_state.is_sticky(m))
            .copied()
            .collect();

        // Clear from logical modifier state
        self.modifier_state.clear_sticky();

        // Also remove from visual sticky keys state to keep in sync
        for modifier in oneshot_modifiers {
            let identifier = modifier_to_identifier(modifier);
            self.sticky_keys_active.remove(identifier);
        }
    }

    /// Returns `true` if any modifiers are currently active.
    ///
    /// Useful for determining if a combo key sequence needs to be emitted.
    pub fn has_active_modifiers(&self) -> bool {
        self.modifier_state.has_active_modifiers()
    }

    /// Returns the number of currently active modifiers.
    pub fn active_modifier_count(&self) -> usize {
        self.modifier_state.active_count()
    }

    /// Synchronizes visual sticky key state with logical modifier state.
    ///
    /// This method updates the `sticky_keys_active` HashSet to match the
    /// current `modifier_state`. Call this after activating/deactivating
    /// modifiers to ensure visual state matches logical state.
    ///
    /// # Arguments
    ///
    /// * `modifier` - The modifier that was changed
    /// * `key_identifier` - The key identifier for visual tracking
    pub fn sync_modifier_visual_state(&mut self, modifier: Modifier, key_identifier: &str) {
        if self.modifier_state.is_active(modifier) {
            self.sticky_keys_active.insert(key_identifier.to_string());
        } else {
            self.sticky_keys_active.remove(key_identifier);
        }
    }

    // ========================================================================
    // Panel Switching (Task 5.3, 5.4)
    // ========================================================================

    /// Switches to a different panel by ID.
    ///
    /// Returns `Ok(())` if the panel exists, or `Err` with an error message if not.
    /// This initiates a panel animation if the panel exists.
    ///
    /// # Errors
    ///
    /// Returns an error message if the target panel does not exist in the layout.
    pub fn switch_panel(&mut self, panel_id: &str) -> Result<(), String> {
        if !self.layout.panels.contains_key(panel_id) {
            return Err(format!("Panel '{}' not found", panel_id));
        }

        // Don't animate if we're already on this panel
        if self.current_panel_id == panel_id {
            return Ok(());
        }

        // Start animation from current panel to target panel
        self.start_animation(panel_id.to_string());

        Ok(())
    }

    /// Switches to a different panel by ID, queuing a toast on error.
    ///
    /// This is a convenience method that combines `switch_panel()` with
    /// automatic toast notification on failure. If the panel doesn't exist,
    /// an error toast is queued with the error message.
    ///
    /// # Arguments
    ///
    /// * `panel_id` - The ID of the panel to switch to
    ///
    /// # Returns
    ///
    /// `true` if the switch was successful, `false` if the panel was not found
    /// (and a toast was queued).
    pub fn switch_panel_with_toast(&mut self, panel_id: &str) -> bool {
        match self.switch_panel(panel_id) {
            Ok(()) => true,
            Err(error_message) => {
                self.queue_toast(error_message, ToastSeverity::Error);
                false
            }
        }
    }

    /// Starts a panel slide animation to the target panel.
    ///
    /// This method creates a new `PanelAnimation` from the current panel
    /// to the specified target panel. The animation will slide the new
    /// panel in from the right edge.
    ///
    /// # Arguments
    ///
    /// * `to_panel_id` - The ID of the panel to animate to
    pub fn start_animation(&mut self, to_panel_id: String) {
        let animation = PanelAnimation::new(&self.current_panel_id, to_panel_id);
        self.animation_state = Some(animation);
    }

    /// Returns `true` if a panel animation is currently in progress.
    pub fn is_animating(&self) -> bool {
        self.animation_state.is_some()
    }

    /// Updates the panel animation progress.
    ///
    /// Returns `true` if the animation completed during this update.
    /// When the animation completes, the `current_panel_id` is updated
    /// to the target panel and the animation state is cleared.
    pub fn update_animation(&mut self) -> bool {
        if let Some(ref mut animation) = self.animation_state {
            if animation.update() {
                // Animation complete - switch to the new panel
                self.current_panel_id = animation.to_panel_id.clone();
                self.animation_state = None;
                return true;
            }
        }
        false
    }

    /// Completes the current animation immediately.
    ///
    /// This is useful for skipping animations or handling edge cases.
    /// If no animation is in progress, this method does nothing.
    pub fn complete_animation(&mut self) {
        if let Some(animation) = self.animation_state.take() {
            self.current_panel_id = animation.to_panel_id;
        }
    }

    /// Returns the current animation progress (0.0 to 1.0), or `None` if not animating.
    pub fn animation_progress(&self) -> Option<f32> {
        self.animation_state.as_ref().map(|a| a.progress)
    }

    /// Returns the eased animation progress (0.0 to 1.0), or `None` if not animating.
    ///
    /// The eased progress uses ease-out-cubic for smoother visual transitions.
    pub fn eased_animation_progress(&self) -> Option<f32> {
        self.animation_state.as_ref().map(|a| a.eased_progress())
    }

    /// Returns the animation state if currently animating.
    pub fn animation(&self) -> Option<&PanelAnimation> {
        self.animation_state.as_ref()
    }

    // ========================================================================
    // Toast Management (Task 6.3, 6.6, 6.7)
    // ========================================================================

    /// Queues a toast notification.
    ///
    /// If no toast is currently displayed, the new toast becomes the current
    /// toast immediately. Otherwise, it is added to the queue and will be
    /// displayed after the current toast is dismissed.
    ///
    /// # Arguments
    ///
    /// * `message` - The message text to display
    /// * `severity` - The severity level (Info, Warning, Error)
    pub fn queue_toast(&mut self, message: impl Into<String>, severity: ToastSeverity) {
        let toast = Toast::new(message, severity);
        self.toast_queue.push_back(toast);

        // If no toast is currently displayed, show this one
        if self.current_toast.is_none() {
            self.show_next_toast();
        }
    }

    /// Shows the next toast from the queue.
    ///
    /// Pops the front toast from the queue and sets it as the current toast
    /// with the current time as the display start time. If the queue is empty,
    /// the current toast remains `None`.
    pub fn show_next_toast(&mut self) {
        if let Some(toast) = self.toast_queue.pop_front() {
            self.current_toast = Some((toast, Instant::now()));
        }
    }

    /// Dismisses the current toast.
    ///
    /// Sets the current toast to `None`. Call `show_next_toast()` after this
    /// to display the next toast in the queue (if any).
    pub fn dismiss_current_toast(&mut self) {
        self.current_toast = None;
    }

    /// Checks if the current toast has timed out.
    ///
    /// Returns `true` if a toast is currently displayed and the 3-second
    /// timeout (`TOAST_DURATION_MS`) has elapsed since it was displayed.
    ///
    /// # Returns
    ///
    /// `true` if the toast should be dismissed, `false` otherwise.
    pub fn check_toast_timeout(&self) -> bool {
        if let Some((_, start_time)) = &self.current_toast {
            start_time.elapsed().as_millis() as u64 >= TOAST_DURATION_MS
        } else {
            false
        }
    }

    /// Returns `true` if a toast is currently being displayed.
    ///
    /// This is used to determine if the toast timer subscription should be
    /// active. When a toast is active, the subscription emits periodic
    /// `ToastTimerTick` messages to check for timeout.
    pub fn has_active_toast(&self) -> bool {
        self.current_toast.is_some()
    }

    /// Handles the toast timer tick by checking timeout and advancing queue.
    ///
    /// This is a convenience method that combines `check_toast_timeout()`,
    /// `dismiss_current_toast()`, and `show_next_toast()` into a single call.
    ///
    /// # Returns
    ///
    /// `true` if a toast was dismissed (and possibly replaced by the next one),
    /// `false` if no action was taken (toast still displaying or no toast active).
    pub fn handle_toast_timer_tick(&mut self) -> bool {
        if self.check_toast_timeout() {
            self.dismiss_current_toast();
            self.show_next_toast();
            true
        } else {
            false
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Converts a `Modifier` enum to a standard key identifier string.
///
/// This is used to synchronize visual sticky key state with logical modifier state.
fn modifier_to_identifier(modifier: Modifier) -> &'static str {
    match modifier {
        Modifier::Shift => "shift",
        Modifier::Ctrl => "ctrl",
        Modifier::Alt => "alt",
        Modifier::Super => "super",
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, Key, KeyCode, Panel, Row, Sizing};
    use std::collections::HashMap;
    use std::thread::sleep;
    use std::time::Duration;

    /// Helper function to create a test layout with two panels.
    fn create_test_layout() -> Layout {
        let mut panels = HashMap::new();

        // Main panel with a key
        let main_panel = Panel {
            id: "main".to_string(),
            padding: Some(5.0),
            margin: Some(2.0),
            nesting_depth: 0,
            rows: vec![Row {
                cells: vec![Cell::Key(Key {
                    label: "A".to_string(),
                    code: KeyCode::Unicode('a'),
                    identifier: Some("key_a".to_string()),
                    width: Sizing::Relative(1.0),
                    height: Sizing::Relative(1.0),
                    min_width: None,
                    min_height: None,
                    alternatives: HashMap::new(),
                    sticky: false,
                    stickyrelease: true,
                })],
            }],
        };

        // Numpad panel
        let numpad_panel = Panel {
            id: "numpad".to_string(),
            padding: Some(5.0),
            margin: Some(2.0),
            nesting_depth: 0,
            rows: vec![Row {
                cells: vec![Cell::Key(Key {
                    label: "1".to_string(),
                    code: KeyCode::Unicode('1'),
                    identifier: Some("key_1".to_string()),
                    width: Sizing::Relative(1.0),
                    height: Sizing::Relative(1.0),
                    min_width: None,
                    min_height: None,
                    alternatives: HashMap::new(),
                    sticky: false,
                    stickyrelease: true,
                })],
            }],
        };

        // Symbols panel for additional testing
        let symbols_panel = Panel {
            id: "symbols".to_string(),
            padding: Some(5.0),
            margin: Some(2.0),
            nesting_depth: 0,
            rows: vec![Row {
                cells: vec![Cell::Key(Key {
                    label: "!".to_string(),
                    code: KeyCode::Unicode('!'),
                    identifier: Some("key_exclaim".to_string()),
                    width: Sizing::Relative(1.0),
                    height: Sizing::Relative(1.0),
                    min_width: None,
                    min_height: None,
                    alternatives: HashMap::new(),
                    sticky: false,
                    stickyrelease: true,
                })],
            }],
        };

        panels.insert("main".to_string(), main_panel);
        panels.insert("numpad".to_string(), numpad_panel);
        panels.insert("symbols".to_string(), symbols_panel);

        Layout {
            name: "Test Layout".to_string(),
            description: Some("A test layout".to_string()),
            author: Some("Test Author".to_string()),
            language: Some("en".to_string()),
            locale: None,
            version: "1.0".to_string(),
            default_panel_id: "main".to_string(),
            inherits: None,
            panels,
        }
    }

    // ========================================================================
    // Task 1.1: Focused tests for KeyboardRenderer state (2-6 tests)
    // ========================================================================

    /// Test 1: KeyboardRenderer initialization with Layout
    ///
    /// Verifies that the renderer initializes correctly with a layout,
    /// setting the current panel to the default panel.
    #[test]
    fn test_keyboard_renderer_initialization() {
        let layout = create_test_layout();
        let renderer = KeyboardRenderer::new(layout);

        // Verify initial state
        assert_eq!(renderer.current_panel_id, "main");
        assert!(renderer.pressed_keys.is_empty());
        assert!(renderer.sticky_keys_active.is_empty());
        assert!(renderer.long_press_key.is_none());
        assert!(renderer.long_press_start.is_none());
        assert!(!renderer.long_press_active);
        assert!(renderer.animation_state.is_none());
        assert!(renderer.toast_queue.is_empty());
        assert!(renderer.current_toast.is_none());

        // Verify current_panel() works
        let panel = renderer.current_panel();
        assert!(panel.is_some());
        assert_eq!(panel.unwrap().id, "main");
    }

    /// Test 2: Pressed key state tracking (add/remove pressed keys)
    ///
    /// Verifies that keys can be pressed and released, and the state
    /// is tracked correctly.
    #[test]
    fn test_pressed_key_state_tracking() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initially no keys pressed
        assert!(!renderer.is_key_pressed("key_a"));
        assert!(!renderer.is_key_pressed("key_b"));

        // Press a key
        renderer.press_key("key_a");
        assert!(renderer.is_key_pressed("key_a"));
        assert!(!renderer.is_key_pressed("key_b"));

        // Press another key
        renderer.press_key("key_b");
        assert!(renderer.is_key_pressed("key_a"));
        assert!(renderer.is_key_pressed("key_b"));

        // Release first key
        renderer.release_key("key_a");
        assert!(!renderer.is_key_pressed("key_a"));
        assert!(renderer.is_key_pressed("key_b"));

        // Release second key
        renderer.release_key("key_b");
        assert!(!renderer.is_key_pressed("key_a"));
        assert!(!renderer.is_key_pressed("key_b"));

        // Releasing non-existent key should not panic
        renderer.release_key("nonexistent");
    }

    /// Test 3: Panel switching state updates
    ///
    /// Verifies that panel switching validates the target panel and
    /// updates state appropriately.
    #[test]
    fn test_panel_switching_state() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Verify initial panel
        assert_eq!(renderer.current_panel_id, "main");

        // Switch to valid panel - should start animation
        let result = renderer.switch_panel("numpad");
        assert!(result.is_ok());
        assert!(renderer.is_animating());

        // Animation should be from main to numpad
        let animation = renderer.animation_state.as_ref().unwrap();
        assert_eq!(animation.from_panel_id, "main");
        assert_eq!(animation.to_panel_id, "numpad");
        assert_eq!(animation.progress, 0.0);

        // Note: current_panel_id doesn't change until animation completes
        assert_eq!(renderer.current_panel_id, "main");

        // Switch to invalid panel - should fail
        let mut renderer2 = KeyboardRenderer::new(create_test_layout());
        let result = renderer2.switch_panel("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
        assert!(!renderer2.is_animating());
        assert_eq!(renderer2.current_panel_id, "main");
    }

    /// Test 4: Animation progress state (0.0 to 1.0 range validation)
    ///
    /// Verifies that animation progress updates correctly and stays
    /// within the 0.0 to 1.0 range.
    #[test]
    fn test_animation_progress_state() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Start animation
        renderer.switch_panel("numpad").unwrap();

        // Initial progress should be 0.0
        let progress = renderer.animation_progress();
        assert!(progress.is_some());
        assert!(progress.unwrap() >= 0.0);
        assert!(progress.unwrap() <= 1.0);

        // Create a manual animation to test progress bounds
        let mut animation = PanelAnimation::new("main", "numpad");
        animation.progress = 0.5;
        assert!(!animation.is_complete());

        animation.progress = 1.0;
        assert!(animation.is_complete());

        // Progress should be clamped to 1.0 max
        animation.progress = 1.5;
        animation.progress = animation.progress.min(1.0);
        assert_eq!(animation.progress, 1.0);

        // Test animation completion triggers panel switch
        // We need to wait for the animation to complete
        let mut renderer2 = KeyboardRenderer::new(create_test_layout());
        renderer2.switch_panel("numpad").unwrap();

        // Manually set progress to complete
        if let Some(ref mut anim) = renderer2.animation_state {
            anim.progress = 1.0;
        }

        // Note: update_animation recalculates progress from elapsed time,
        // so we use a separate assertion
        let _result = renderer2.update_animation();

        // Let's test PanelAnimation directly
        let mut anim = PanelAnimation::new("main", "numpad");

        // Initially not complete
        assert!(!anim.is_complete());

        // Simulate time passing by setting start_time in the past
        // This is a workaround since we can't easily mock time
        // For now, just verify the bounds checking works
        assert!(anim.progress >= 0.0);
        assert!(anim.progress <= 1.0);

        // Test with actual time delay (if this test is slow, it confirms animation works)
        sleep(Duration::from_millis(ANIMATION_DURATION_MS + 50));
        anim.update();
        assert!(anim.is_complete());
        assert_eq!(anim.progress, 1.0); // Clamped to max
    }

    /// Test 5: Toast queue management (add, dismiss, queue order)
    ///
    /// Verifies that toasts are queued in FIFO order and can be
    /// dismissed correctly.
    #[test]
    fn test_toast_queue_management() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initially no toasts
        assert!(renderer.toast_queue.is_empty());
        assert!(renderer.current_toast.is_none());

        // Queue first toast - should become current immediately
        renderer.queue_toast("First message", ToastSeverity::Info);
        assert!(renderer.toast_queue.is_empty()); // Moved to current
        assert!(renderer.current_toast.is_some());
        let (toast, _) = renderer.current_toast.as_ref().unwrap();
        assert_eq!(toast.message, "First message");
        assert_eq!(toast.severity, ToastSeverity::Info);

        // Queue more toasts - should stay in queue
        renderer.queue_toast("Second message", ToastSeverity::Warning);
        renderer.queue_toast("Third message", ToastSeverity::Error);
        assert_eq!(renderer.toast_queue.len(), 2);

        // Dismiss current toast
        renderer.dismiss_current_toast();
        assert!(renderer.current_toast.is_none());

        // Show next toast
        renderer.show_next_toast();
        assert!(renderer.current_toast.is_some());
        let (toast, _) = renderer.current_toast.as_ref().unwrap();
        assert_eq!(toast.message, "Second message");
        assert_eq!(toast.severity, ToastSeverity::Warning);

        // Queue should now have one toast
        assert_eq!(renderer.toast_queue.len(), 1);

        // Continue to third toast
        renderer.dismiss_current_toast();
        renderer.show_next_toast();
        let (toast, _) = renderer.current_toast.as_ref().unwrap();
        assert_eq!(toast.message, "Third message");
        assert_eq!(toast.severity, ToastSeverity::Error);

        // Queue should be empty
        assert!(renderer.toast_queue.is_empty());

        // Showing next toast when queue is empty should result in no current toast
        renderer.dismiss_current_toast();
        renderer.show_next_toast();
        assert!(renderer.current_toast.is_none());
    }

    /// Test 6: Sticky key state management
    ///
    /// Verifies that sticky keys can be toggled on and off correctly.
    #[test]
    fn test_sticky_key_state_management() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initially no sticky keys active
        assert!(!renderer.is_sticky_active("shift"));
        assert!(!renderer.is_sticky_active("ctrl"));

        // Toggle shift on
        renderer.toggle_sticky("shift");
        assert!(renderer.is_sticky_active("shift"));
        assert!(!renderer.is_sticky_active("ctrl"));

        // Toggle ctrl on
        renderer.toggle_sticky("ctrl");
        assert!(renderer.is_sticky_active("shift"));
        assert!(renderer.is_sticky_active("ctrl"));

        // Toggle shift off
        renderer.toggle_sticky("shift");
        assert!(!renderer.is_sticky_active("shift"));
        assert!(renderer.is_sticky_active("ctrl"));

        // Toggle ctrl off
        renderer.toggle_sticky("ctrl");
        assert!(!renderer.is_sticky_active("shift"));
        assert!(!renderer.is_sticky_active("ctrl"));
    }

    // ========================================================================
    // Task 4.1: Focused tests for interactive features (2-6 tests)
    // ========================================================================

    /// Test 1: Key press state change (pressed -> released)
    ///
    /// Verifies that pressing and releasing a key correctly updates
    /// the pressed state and triggers/cancels long press timer.
    #[test]
    fn test_key_press_state_change() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initially no keys pressed
        assert!(!renderer.is_key_pressed("key_a"));
        assert!(renderer.long_press_key.is_none());

        // Press key_a
        renderer.press_key("key_a");
        assert!(renderer.is_key_pressed("key_a"));
        assert_eq!(renderer.long_press_key, Some("key_a".to_string()));
        assert!(renderer.long_press_start.is_some());
        assert!(!renderer.long_press_active);

        // Release key_a
        renderer.release_key("key_a");
        assert!(!renderer.is_key_pressed("key_a"));
        assert!(renderer.long_press_key.is_none());
        assert!(renderer.long_press_start.is_none());
        assert!(!renderer.long_press_active);
    }

    /// Test 2: Long press timer start after 300ms threshold
    ///
    /// Verifies that the long press is detected after holding for 300ms.
    #[test]
    fn test_long_press_timer_threshold() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Press key
        renderer.press_key("key_a");
        assert!(renderer.has_pending_long_press());
        assert!(!renderer.is_long_press_active());

        // Immediately check - should not be triggered yet
        assert!(!renderer.check_long_press_threshold());
        assert!(!renderer.is_long_press_active());

        // Wait for threshold to be exceeded
        sleep(Duration::from_millis(LONG_PRESS_THRESHOLD_MS + 50));

        // Now check - should trigger
        assert!(renderer.check_long_press_threshold());
        assert!(renderer.is_long_press_active());

        // Second check should return false (already active)
        assert!(!renderer.check_long_press_threshold());
    }

    /// Test 3: Long press cancellation on early release
    ///
    /// Verifies that releasing a key before 300ms cancels the long press.
    #[test]
    fn test_long_press_cancellation_on_early_release() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Press key
        renderer.press_key("key_a");
        assert!(renderer.has_pending_long_press());

        // Wait less than threshold
        sleep(Duration::from_millis(100));

        // Release before threshold
        renderer.release_key("key_a");

        // Long press should be cancelled
        assert!(!renderer.has_pending_long_press());
        assert!(!renderer.is_long_press_active());
        assert!(renderer.long_press_key.is_none());
        assert!(renderer.long_press_start.is_none());
    }

    /// Test 4: Long press key identifier tracking
    ///
    /// Verifies that the long press key identifier is correctly tracked.
    #[test]
    fn test_long_press_key_identifier() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initially no long press key
        assert!(renderer.long_press_key_identifier().is_none());

        // Press key
        renderer.press_key("key_a");
        assert_eq!(renderer.long_press_key_identifier(), Some("key_a"));

        // Press different key (overwrites previous)
        renderer.press_key("key_b");
        assert_eq!(renderer.long_press_key_identifier(), Some("key_b"));

        // Cancel long press
        renderer.cancel_long_press();
        assert!(renderer.long_press_key_identifier().is_none());
    }

    /// Test 5: Multiple key presses don't interfere
    ///
    /// Verifies that pressing multiple keys correctly tracks each key's state.
    #[test]
    fn test_multiple_key_presses() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Press key_a
        renderer.press_key("key_a");
        assert!(renderer.is_key_pressed("key_a"));
        assert_eq!(renderer.long_press_key_identifier(), Some("key_a"));

        // Press key_b (long press timer moves to key_b)
        renderer.press_key("key_b");
        assert!(renderer.is_key_pressed("key_a"));
        assert!(renderer.is_key_pressed("key_b"));
        assert_eq!(renderer.long_press_key_identifier(), Some("key_b"));

        // Release key_a (doesn't affect key_b's long press)
        renderer.release_key("key_a");
        assert!(!renderer.is_key_pressed("key_a"));
        assert!(renderer.is_key_pressed("key_b"));
        // key_b is still in pressed_keys but long press was for key_b
        assert_eq!(renderer.long_press_key_identifier(), Some("key_b"));

        // Release key_b (cancels long press for key_b)
        renderer.release_key("key_b");
        assert!(!renderer.is_key_pressed("key_b"));
        assert!(renderer.long_press_key_identifier().is_none());
    }

    // ========================================================================
    // Task 5.1: Focused tests for panel transitions (2-6 tests)
    // ========================================================================

    /// Test 1: Panel switch to valid panel_id
    ///
    /// Verifies that switching to a valid panel starts the animation
    /// and correctly tracks the from/to panels.
    #[test]
    fn test_panel_switch_to_valid_panel() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initial state
        assert_eq!(renderer.current_panel_id, "main");
        assert!(!renderer.is_animating());

        // Switch to numpad
        let result = renderer.switch_panel("numpad");
        assert!(result.is_ok());
        assert!(renderer.is_animating());

        // Verify animation state
        let anim = renderer.animation().unwrap();
        assert_eq!(anim.from_panel_id, "main");
        assert_eq!(anim.to_panel_id, "numpad");
        assert_eq!(anim.progress, 0.0);

        // Current panel should still be main until animation completes
        assert_eq!(renderer.current_panel_id, "main");

        // Can get both panels during animation
        assert!(renderer.get_panel("main").is_some());
        assert!(renderer.get_panel("numpad").is_some());
    }

    /// Test 2: Panel switch to invalid panel_id (error handling)
    ///
    /// Verifies that switching to a non-existent panel returns an error
    /// and does not start any animation.
    #[test]
    fn test_panel_switch_to_invalid_panel() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initial state
        assert_eq!(renderer.current_panel_id, "main");
        assert!(!renderer.is_animating());

        // Try to switch to nonexistent panel
        let result = renderer.switch_panel("nonexistent");
        assert!(result.is_err());

        // Verify error message
        let err = result.unwrap_err();
        assert!(err.contains("nonexistent"));
        assert!(err.contains("not found"));

        // Should not be animating
        assert!(!renderer.is_animating());
        assert!(renderer.animation_state.is_none());

        // Current panel should be unchanged
        assert_eq!(renderer.current_panel_id, "main");
    }

    /// Test 3: Animation progress interpolation (0.0 -> 1.0)
    ///
    /// Verifies that animation progress correctly interpolates from 0.0 to 1.0
    /// based on elapsed time.
    #[test]
    fn test_animation_progress_interpolation() {
        // Test direct PanelAnimation progress
        let mut anim = PanelAnimation::new("main", "numpad");

        // Initial progress is 0.0
        assert_eq!(anim.progress, 0.0);
        assert!(!anim.is_complete());

        // After a short delay, progress should increase
        sleep(Duration::from_millis(50));
        anim.update();
        assert!(anim.progress > 0.0);
        assert!(anim.progress < 1.0);

        // Progress should be bounded between 0.0 and 1.0
        let progress = anim.progress;
        assert!(progress >= 0.0);
        assert!(progress <= 1.0);

        // After full animation duration, progress should be 1.0
        sleep(Duration::from_millis(ANIMATION_DURATION_MS));
        anim.update();
        assert_eq!(anim.progress, 1.0);
        assert!(anim.is_complete());

        // Test eased progress is also in valid range
        let eased = anim.eased_progress();
        assert!(eased >= 0.0);
        assert!(eased <= 1.0);
    }

    /// Test 4: Animation completion callback
    ///
    /// Verifies that update_animation returns true when animation completes
    /// and updates the current_panel_id accordingly.
    #[test]
    fn test_animation_completion_callback() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Start animation
        renderer.switch_panel("numpad").unwrap();
        assert!(renderer.is_animating());
        assert_eq!(renderer.current_panel_id, "main");

        // Update animation - should return false while in progress
        let completed = renderer.update_animation();
        assert!(!completed); // Still animating

        // Wait for animation to complete
        sleep(Duration::from_millis(ANIMATION_DURATION_MS + 50));

        // Now update should return true
        let completed = renderer.update_animation();
        assert!(completed);

        // Verify state after completion
        assert!(!renderer.is_animating());
        assert!(renderer.animation_state.is_none());
        assert_eq!(renderer.current_panel_id, "numpad");

        // Current panel should now return numpad
        let panel = renderer.current_panel().unwrap();
        assert_eq!(panel.id, "numpad");
    }

    /// Test 5: Rendering during animation (both panels accessible)
    ///
    /// Verifies that during animation, both the source and target panels
    /// are accessible for rendering.
    #[test]
    fn test_rendering_during_animation() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Start animation from main to numpad
        renderer.switch_panel("numpad").unwrap();
        assert!(renderer.is_animating());

        // Both panels should be accessible via get_panel()
        let from_panel = renderer.get_panel("main");
        let to_panel = renderer.get_panel("numpad");
        assert!(from_panel.is_some());
        assert!(to_panel.is_some());
        assert_eq!(from_panel.unwrap().id, "main");
        assert_eq!(to_panel.unwrap().id, "numpad");

        // Animation state should provide panel IDs
        let anim = renderer.animation().unwrap();
        assert_eq!(anim.from_panel_id, "main");
        assert_eq!(anim.to_panel_id, "numpad");

        // Progress should be available for offset calculations
        let progress = renderer.animation_progress();
        assert!(progress.is_some());

        // Eased progress should also be available
        let eased = renderer.eased_animation_progress();
        assert!(eased.is_some());

        // Verify we can complete animation immediately if needed
        renderer.complete_animation();
        assert!(!renderer.is_animating());
        assert_eq!(renderer.current_panel_id, "numpad");
    }

    /// Test 6: Switch to same panel does not animate
    ///
    /// Verifies that switching to the current panel does not start an animation.
    #[test]
    fn test_switch_to_same_panel_no_animation() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initial state
        assert_eq!(renderer.current_panel_id, "main");
        assert!(!renderer.is_animating());

        // Switch to current panel
        let result = renderer.switch_panel("main");
        assert!(result.is_ok());

        // Should not be animating
        assert!(!renderer.is_animating());
        assert!(renderer.animation_state.is_none());
    }

    // ========================================================================
    // Task 6.6: Panel switch with toast on error
    // ========================================================================

    /// Test: switch_panel_with_toast queues error toast on invalid panel
    #[test]
    fn test_switch_panel_with_toast_error() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initially no toasts
        assert!(renderer.current_toast.is_none());

        // Try to switch to nonexistent panel with toast
        let success = renderer.switch_panel_with_toast("nonexistent");
        assert!(!success);

        // Should have queued an error toast
        assert!(renderer.current_toast.is_some());
        let (toast, _) = renderer.current_toast.as_ref().unwrap();
        assert!(toast.message.contains("nonexistent"));
        assert!(toast.message.contains("not found"));
        assert_eq!(toast.severity, ToastSeverity::Error);

        // Should not be animating
        assert!(!renderer.is_animating());
    }

    /// Test: switch_panel_with_toast succeeds without toast
    #[test]
    fn test_switch_panel_with_toast_success() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initially no toasts
        assert!(renderer.current_toast.is_none());

        // Switch to valid panel with toast
        let success = renderer.switch_panel_with_toast("numpad");
        assert!(success);

        // Should NOT have queued a toast
        assert!(renderer.current_toast.is_none());

        // Should be animating
        assert!(renderer.is_animating());
    }

    // ========================================================================
    // Task 6.7: Toast timer helpers
    // ========================================================================

    /// Test: has_active_toast returns correct state
    #[test]
    fn test_has_active_toast() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initially no active toast
        assert!(!renderer.has_active_toast());

        // Queue a toast
        renderer.queue_toast("Test toast", ToastSeverity::Info);
        assert!(renderer.has_active_toast());

        // Dismiss the toast
        renderer.dismiss_current_toast();
        assert!(!renderer.has_active_toast());
    }

    /// Test: handle_toast_timer_tick advances queue
    #[test]
    fn test_handle_toast_timer_tick() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Queue two toasts
        renderer.queue_toast("First", ToastSeverity::Info);
        renderer.queue_toast("Second", ToastSeverity::Warning);

        // First toast should be current
        let (toast, _) = renderer.current_toast.as_ref().unwrap();
        assert_eq!(toast.message, "First");

        // Immediate tick should not dismiss (timeout not reached)
        let dismissed = renderer.handle_toast_timer_tick();
        assert!(!dismissed);
        let (toast, _) = renderer.current_toast.as_ref().unwrap();
        assert_eq!(toast.message, "First");

        // Wait for timeout
        sleep(Duration::from_millis(TOAST_DURATION_MS + 100));

        // Now tick should dismiss and show next
        let dismissed = renderer.handle_toast_timer_tick();
        assert!(dismissed);
        let (toast, _) = renderer.current_toast.as_ref().unwrap();
        assert_eq!(toast.message, "Second");
    }

    // ========================================================================
    // Task Group 4: Modifier State Tracking in Renderer (4-5 tests)
    // ========================================================================

    /// Test 1: One-shot modifier behavior (sticky: true, stickyrelease: true)
    ///
    /// Verifies that a modifier activated with stickyrelease=true is cleared
    /// after calling clear_oneshot_modifiers().
    #[test]
    fn test_oneshot_modifier_behavior() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initially no modifiers active
        assert!(!renderer.is_modifier_active(Modifier::Shift));
        assert!(!renderer.has_active_modifiers());

        // Activate Shift as one-shot (stickyrelease: true)
        renderer.activate_modifier(Modifier::Shift, true);
        assert!(renderer.is_modifier_active(Modifier::Shift));
        assert!(renderer.has_active_modifiers());
        assert_eq!(renderer.active_modifier_count(), 1);

        // Verify get_active_modifiers includes Shift
        let active = renderer.get_active_modifiers();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0], Modifier::Shift);

        // Simulate pressing a regular key by clearing one-shot modifiers
        renderer.clear_oneshot_modifiers();

        // Shift should now be inactive
        assert!(!renderer.is_modifier_active(Modifier::Shift));
        assert!(!renderer.has_active_modifiers());
        assert_eq!(renderer.active_modifier_count(), 0);
    }

    /// Test 2: Toggle modifier behavior (sticky: true, stickyrelease: false)
    ///
    /// Verifies that a modifier activated with stickyrelease=false stays
    /// active after clear_oneshot_modifiers() is called.
    #[test]
    fn test_toggle_modifier_behavior() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initially no modifiers active
        assert!(!renderer.is_modifier_active(Modifier::Ctrl));

        // Activate Ctrl as toggle (stickyrelease: false)
        renderer.activate_modifier(Modifier::Ctrl, false);
        assert!(renderer.is_modifier_active(Modifier::Ctrl));

        // Clear one-shot modifiers (should NOT affect toggle modifiers)
        renderer.clear_oneshot_modifiers();
        assert!(
            renderer.is_modifier_active(Modifier::Ctrl),
            "Toggle modifier should persist after clear_oneshot_modifiers()"
        );

        // Toggle modifier must be explicitly deactivated
        renderer.deactivate_modifier(Modifier::Ctrl);
        assert!(!renderer.is_modifier_active(Modifier::Ctrl));
    }

    /// Test 3: Hold modifier behavior (sticky: false)
    ///
    /// Verifies that hold behavior works by activating/deactivating
    /// the modifier directly when the key is pressed/released.
    #[test]
    fn test_hold_modifier_behavior() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // For hold behavior, we activate when key is pressed and deactivate when released
        // The stickyrelease parameter doesn't matter for hold behavior since we
        // manually deactivate on key release

        // Initially no modifiers active
        assert!(!renderer.is_modifier_active(Modifier::Alt));

        // User presses Alt key (hold mode - activate modifier)
        renderer.activate_modifier(Modifier::Alt, false);
        assert!(renderer.is_modifier_active(Modifier::Alt));

        // While held, user presses another key - modifier should still be active
        // (for hold mode, we don't clear after combo key)
        assert!(renderer.is_modifier_active(Modifier::Alt));

        // User releases Alt key (deactivate modifier)
        renderer.deactivate_modifier(Modifier::Alt);
        assert!(!renderer.is_modifier_active(Modifier::Alt));
    }

    /// Test 4: Multiple simultaneous modifiers
    ///
    /// Verifies that multiple modifiers can be active at the same time.
    #[test]
    fn test_multiple_simultaneous_modifiers() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Activate Ctrl (one-shot) and Shift (toggle)
        renderer.activate_modifier(Modifier::Ctrl, true);
        renderer.activate_modifier(Modifier::Shift, false);

        // Both should be active
        assert!(renderer.is_modifier_active(Modifier::Ctrl));
        assert!(renderer.is_modifier_active(Modifier::Shift));
        assert_eq!(renderer.active_modifier_count(), 2);

        // get_active_modifiers should return both, sorted
        let active = renderer.get_active_modifiers();
        assert_eq!(active.len(), 2);
        // Shift should come before Ctrl based on Modifier enum order
        assert_eq!(active[0], Modifier::Shift);
        assert_eq!(active[1], Modifier::Ctrl);

        // Clear one-shot modifiers
        renderer.clear_oneshot_modifiers();

        // Only Shift (toggle) should remain
        assert!(!renderer.is_modifier_active(Modifier::Ctrl));
        assert!(renderer.is_modifier_active(Modifier::Shift));
        assert_eq!(renderer.active_modifier_count(), 1);
    }

    /// Test 5: Modifier clearing after combo key emission
    ///
    /// Verifies the complete workflow: activate modifier, "emit" combo key,
    /// clear one-shot modifiers, and verify visual state stays in sync.
    #[test]
    fn test_modifier_clearing_after_combo_key() {
        let layout = create_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Simulate: User taps Shift (one-shot mode)
        renderer.activate_modifier(Modifier::Shift, true);
        renderer.sync_modifier_visual_state(Modifier::Shift, "shift");
        assert!(renderer.is_modifier_active(Modifier::Shift));
        assert!(renderer.is_sticky_active("shift"));

        // User taps 'a' key - emit Shift+A combo
        // After emitting, clear one-shot modifiers
        let active_before = renderer.get_active_modifiers();
        assert_eq!(active_before, vec![Modifier::Shift]);

        // Simulate combo key emission and clearing
        renderer.clear_oneshot_modifiers();

        // Shift should be cleared from both logical and visual state
        assert!(!renderer.is_modifier_active(Modifier::Shift));
        assert!(
            !renderer.is_sticky_active("shift"),
            "Visual sticky state should be cleared by clear_oneshot_modifiers"
        );

        // No more active modifiers
        let active_after = renderer.get_active_modifiers();
        assert!(active_after.is_empty());
    }

    /// Test: Renderer initialization includes empty modifier state
    #[test]
    fn test_renderer_init_empty_modifier_state() {
        let layout = create_test_layout();
        let renderer = KeyboardRenderer::new(layout);

        // Verify modifier state is empty on initialization
        assert!(!renderer.has_active_modifiers());
        assert_eq!(renderer.active_modifier_count(), 0);
        assert!(!renderer.is_modifier_active(Modifier::Shift));
        assert!(!renderer.is_modifier_active(Modifier::Ctrl));
        assert!(!renderer.is_modifier_active(Modifier::Alt));
        assert!(!renderer.is_modifier_active(Modifier::Super));
    }
}
