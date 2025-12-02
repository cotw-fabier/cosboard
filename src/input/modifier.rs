// SPDX-License-Identifier: GPL-3.0-only

//! Modifier state management for keyboard input.
//!
//! This module provides functionality for tracking the state of modifier keys
//! (Shift, Ctrl, Alt, Super) during keyboard input. It supports three modifier
//! behaviors:
//!
//! - **One-shot (sticky release)**: Modifier is cleared after the next key press
//! - **Toggle**: Modifier stays active until explicitly deactivated
//! - **Hold**: Modifier is active only while the key is held down
//!
//! # Example
//!
//! ```rust,ignore
//! use cosboard::input::ModifierState;
//! use cosboard::layout::Modifier;
//!
//! let mut state = ModifierState::new();
//!
//! // Activate Shift as one-shot
//! state.activate(Modifier::Shift, true);
//!
//! // ... user presses another key ...
//!
//! // Clear one-shot modifiers
//! state.clear_sticky();
//! ```

use crate::layout::Modifier;
use std::collections::HashSet;

/// Tracks the state of modifier keys during keyboard input.
///
/// This struct maintains which modifiers are currently active and whether
/// they should be cleared after the next key press (one-shot behavior).
#[derive(Debug, Clone, Default)]
pub struct ModifierState {
    /// Set of currently active modifiers
    active: HashSet<Modifier>,

    /// Set of modifiers that should be cleared after the next key (one-shot)
    sticky: HashSet<Modifier>,
}

impl ModifierState {
    /// Creates a new `ModifierState` with no active modifiers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active: HashSet::new(),
            sticky: HashSet::new(),
        }
    }

    /// Activates a modifier.
    ///
    /// # Arguments
    ///
    /// * `modifier` - The modifier to activate
    /// * `stickyrelease` - If `true`, the modifier will be cleared after the next key press
    ///                     (one-shot behavior). If `false`, the modifier stays active until
    ///                     explicitly deactivated (toggle behavior).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cosboard::input::ModifierState;
    /// use cosboard::layout::Modifier;
    ///
    /// let mut state = ModifierState::new();
    ///
    /// // Activate Shift as one-shot
    /// state.activate(Modifier::Shift, true);
    ///
    /// // Activate Ctrl as toggle (will stay active)
    /// state.activate(Modifier::Ctrl, false);
    /// ```
    pub fn activate(&mut self, modifier: Modifier, stickyrelease: bool) {
        self.active.insert(modifier);

        if stickyrelease {
            self.sticky.insert(modifier);
        } else {
            // If activating as toggle, remove from sticky set
            self.sticky.remove(&modifier);
        }
    }

    /// Deactivates a modifier.
    ///
    /// This removes the modifier from both the active set and the sticky set.
    ///
    /// # Arguments
    ///
    /// * `modifier` - The modifier to deactivate
    pub fn deactivate(&mut self, modifier: Modifier) {
        self.active.remove(&modifier);
        self.sticky.remove(&modifier);
    }

    /// Toggles a modifier's state.
    ///
    /// If the modifier is active, it will be deactivated.
    /// If the modifier is inactive, it will be activated.
    ///
    /// # Arguments
    ///
    /// * `modifier` - The modifier to toggle
    /// * `stickyrelease` - If `true` and the modifier is being activated,
    ///                     it will be cleared after the next key press
    ///
    /// # Returns
    ///
    /// `true` if the modifier is now active, `false` if it is now inactive
    pub fn toggle(&mut self, modifier: Modifier, stickyrelease: bool) -> bool {
        if self.active.contains(&modifier) {
            self.deactivate(modifier);
            false
        } else {
            self.activate(modifier, stickyrelease);
            true
        }
    }

    /// Checks if a modifier is currently active.
    ///
    /// # Arguments
    ///
    /// * `modifier` - The modifier to check
    ///
    /// # Returns
    ///
    /// `true` if the modifier is active, `false` otherwise
    #[must_use]
    pub fn is_active(&self, modifier: Modifier) -> bool {
        self.active.contains(&modifier)
    }

    /// Returns a list of all currently active modifiers.
    ///
    /// The modifiers are returned in a consistent order (sorted by enum value).
    ///
    /// # Returns
    ///
    /// A `Vec` containing all active modifiers
    #[must_use]
    pub fn get_active_modifiers(&self) -> Vec<Modifier> {
        let mut modifiers: Vec<Modifier> = self.active.iter().copied().collect();
        modifiers.sort();
        modifiers
    }

    /// Clears all one-shot (sticky) modifiers.
    ///
    /// This should be called after a regular key is pressed to implement
    /// one-shot modifier behavior. Modifiers activated with `stickyrelease=true`
    /// will be deactivated. Modifiers activated with `stickyrelease=false`
    /// (toggle mode) will remain active.
    pub fn clear_sticky(&mut self) {
        // Remove all sticky modifiers from the active set
        for modifier in self.sticky.drain() {
            self.active.remove(&modifier);
        }
    }

    /// Checks if a modifier is in one-shot (sticky) mode.
    ///
    /// # Arguments
    ///
    /// * `modifier` - The modifier to check
    ///
    /// # Returns
    ///
    /// `true` if the modifier is active and will be cleared after the next key
    #[must_use]
    pub fn is_sticky(&self, modifier: Modifier) -> bool {
        self.sticky.contains(&modifier)
    }

    /// Clears all modifiers (both active and sticky).
    ///
    /// This can be used to reset the modifier state completely.
    pub fn clear_all(&mut self) {
        self.active.clear();
        self.sticky.clear();
    }

    /// Checks if any modifiers are currently active.
    ///
    /// # Returns
    ///
    /// `true` if at least one modifier is active, `false` if no modifiers are active
    #[must_use]
    pub fn has_active_modifiers(&self) -> bool {
        !self.active.is_empty()
    }

    /// Returns the number of currently active modifiers.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.active.len()
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test basic activation and deactivation
    #[test]
    fn test_activate_deactivate() {
        let mut state = ModifierState::new();

        state.activate(Modifier::Shift, true);
        assert!(state.is_active(Modifier::Shift));

        state.deactivate(Modifier::Shift);
        assert!(!state.is_active(Modifier::Shift));
    }

    /// Test toggle functionality
    #[test]
    fn test_toggle() {
        let mut state = ModifierState::new();

        // Toggle on
        let is_active = state.toggle(Modifier::Alt, false);
        assert!(is_active);
        assert!(state.is_active(Modifier::Alt));

        // Toggle off
        let is_active = state.toggle(Modifier::Alt, false);
        assert!(!is_active);
        assert!(!state.is_active(Modifier::Alt));
    }

    /// Test sticky vs non-sticky modes
    #[test]
    fn test_sticky_mode() {
        let mut state = ModifierState::new();

        // Activate as sticky
        state.activate(Modifier::Shift, true);
        assert!(state.is_sticky(Modifier::Shift));

        // Activate as non-sticky
        state.activate(Modifier::Ctrl, false);
        assert!(!state.is_sticky(Modifier::Ctrl));

        // Switch sticky to non-sticky
        state.activate(Modifier::Shift, false);
        assert!(!state.is_sticky(Modifier::Shift));
    }

    /// Test get_active_modifiers returns sorted list
    #[test]
    fn test_get_active_modifiers_sorted() {
        let mut state = ModifierState::new();

        // Activate in non-sorted order
        state.activate(Modifier::Super, false);
        state.activate(Modifier::Shift, false);
        state.activate(Modifier::Alt, false);

        let active = state.get_active_modifiers();
        assert_eq!(active.len(), 3);

        // Verify sorted (Shift < Ctrl < Alt < Super based on enum definition)
        let positions: Vec<usize> = active
            .iter()
            .map(|m| match m {
                Modifier::Shift => 0,
                Modifier::Ctrl => 1,
                Modifier::Alt => 2,
                Modifier::Super => 3,
            })
            .collect();

        for i in 1..positions.len() {
            assert!(
                positions[i] > positions[i - 1],
                "Modifiers should be sorted"
            );
        }
    }

    /// Test clear_all
    #[test]
    fn test_clear_all() {
        let mut state = ModifierState::new();

        state.activate(Modifier::Shift, true);
        state.activate(Modifier::Ctrl, false);
        state.activate(Modifier::Alt, true);

        assert_eq!(state.active_count(), 3);

        state.clear_all();

        assert!(!state.has_active_modifiers());
        assert_eq!(state.active_count(), 0);
    }

    /// Test Default trait implementation
    #[test]
    fn test_default() {
        let state = ModifierState::default();

        assert!(!state.has_active_modifiers());
        assert_eq!(state.active_count(), 0);
    }
}
