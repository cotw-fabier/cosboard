// SPDX-License-Identifier: GPL-3.0-only

//! Renderer message types for key and panel interactions.
//!
//! This module defines the messages emitted by the keyboard renderer
//! for handling key presses, panel switching, toast notifications,
//! and other interactions.

use crate::renderer::state::ToastSeverity;

/// Messages emitted by the keyboard renderer.
///
/// These messages are used by the rendering functions to communicate
/// user interactions back to the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RendererMessage {
    /// A key was pressed. Contains the key identifier.
    KeyPressed(String),

    /// A key was released. Contains the key identifier.
    KeyReleased(String),

    /// Switch to a different panel. Contains the panel ID.
    SwitchPanel(String),

    /// Animation frame tick for panel transitions.
    ///
    /// This message is emitted at ~60fps (every ~16ms) during panel
    /// slide animations to update the animation progress.
    AnimationTick,

    /// Animation has completed.
    ///
    /// This message is emitted when a panel slide animation finishes,
    /// signaling that the transition is complete and the new panel
    /// is now the current panel.
    AnimationComplete,

    /// Periodic timer tick for long press detection.
    ///
    /// This message is emitted periodically (e.g., every 50ms) when a key
    /// is being held down, allowing the renderer to check if the long press
    /// threshold (300ms) has been exceeded.
    LongPressTimerTick,

    /// Dismiss the active popup.
    ///
    /// This message is emitted when the user releases the key or moves
    /// the pointer away from the popup area.
    PopupDismiss,

    // ========================================================================
    // Toast Messages (Task 6.2)
    // ========================================================================

    /// Display a toast notification.
    ///
    /// This message queues a toast notification with the given message and
    /// severity level. The toast will be displayed at the bottom of the
    /// keyboard surface.
    ///
    /// # Arguments
    ///
    /// * `String` - The message text to display
    /// * `ToastSeverity` - The severity level (Info, Warning, Error)
    ShowToast(String, ToastSeverity),

    /// Dismiss the current toast notification.
    ///
    /// This message removes the currently displayed toast. If there are
    /// more toasts in the queue, the next one will be displayed.
    DismissToast,

    /// Periodic timer tick for toast timeout checking.
    ///
    /// This message is emitted periodically when a toast is displayed,
    /// allowing the renderer to check if the 3-second timeout has elapsed
    /// and auto-dismiss the toast.
    ToastTimerTick,

    /// No-op message (used for placeholder elements).
    Noop,
}

impl Default for RendererMessage {
    fn default() -> Self {
        Self::Noop
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_variants() {
        let key_pressed = RendererMessage::KeyPressed("key_a".to_string());
        let key_released = RendererMessage::KeyReleased("key_a".to_string());
        let switch_panel = RendererMessage::SwitchPanel("numpad".to_string());
        let animation_tick = RendererMessage::AnimationTick;
        let animation_complete = RendererMessage::AnimationComplete;
        let long_press_tick = RendererMessage::LongPressTimerTick;
        let popup_dismiss = RendererMessage::PopupDismiss;
        let show_toast = RendererMessage::ShowToast("Error".to_string(), ToastSeverity::Error);
        let dismiss_toast = RendererMessage::DismissToast;
        let toast_timer_tick = RendererMessage::ToastTimerTick;
        let noop = RendererMessage::Noop;

        assert!(matches!(key_pressed, RendererMessage::KeyPressed(_)));
        assert!(matches!(key_released, RendererMessage::KeyReleased(_)));
        assert!(matches!(switch_panel, RendererMessage::SwitchPanel(_)));
        assert!(matches!(animation_tick, RendererMessage::AnimationTick));
        assert!(matches!(
            animation_complete,
            RendererMessage::AnimationComplete
        ));
        assert!(matches!(long_press_tick, RendererMessage::LongPressTimerTick));
        assert!(matches!(popup_dismiss, RendererMessage::PopupDismiss));
        assert!(matches!(show_toast, RendererMessage::ShowToast(_, _)));
        assert!(matches!(dismiss_toast, RendererMessage::DismissToast));
        assert!(matches!(toast_timer_tick, RendererMessage::ToastTimerTick));
        assert!(matches!(noop, RendererMessage::Noop));
    }

    #[test]
    fn test_message_default() {
        let default = RendererMessage::default();
        assert_eq!(default, RendererMessage::Noop);
    }

    #[test]
    fn test_message_clone_and_eq() {
        let msg1 = RendererMessage::KeyPressed("key_a".to_string());
        let msg2 = msg1.clone();
        assert_eq!(msg1, msg2);

        let msg3 = RendererMessage::KeyPressed("key_b".to_string());
        assert_ne!(msg1, msg3);
    }

    #[test]
    fn test_long_press_timer_tick_eq() {
        let tick1 = RendererMessage::LongPressTimerTick;
        let tick2 = RendererMessage::LongPressTimerTick;
        assert_eq!(tick1, tick2);
    }

    #[test]
    fn test_popup_dismiss_eq() {
        let dismiss1 = RendererMessage::PopupDismiss;
        let dismiss2 = RendererMessage::PopupDismiss;
        assert_eq!(dismiss1, dismiss2);
    }

    #[test]
    fn test_animation_tick_eq() {
        let tick1 = RendererMessage::AnimationTick;
        let tick2 = RendererMessage::AnimationTick;
        assert_eq!(tick1, tick2);
    }

    #[test]
    fn test_animation_complete_eq() {
        let complete1 = RendererMessage::AnimationComplete;
        let complete2 = RendererMessage::AnimationComplete;
        assert_eq!(complete1, complete2);
    }

    // ========================================================================
    // Toast Message Tests (Task 6.2)
    // ========================================================================

    #[test]
    fn test_show_toast_message() {
        let msg1 = RendererMessage::ShowToast("Error occurred".to_string(), ToastSeverity::Error);
        let msg2 = RendererMessage::ShowToast("Error occurred".to_string(), ToastSeverity::Error);
        assert_eq!(msg1, msg2);

        let msg3 = RendererMessage::ShowToast("Warning".to_string(), ToastSeverity::Warning);
        assert_ne!(msg1, msg3);

        // Different severity, same message
        let msg4 = RendererMessage::ShowToast("Error occurred".to_string(), ToastSeverity::Warning);
        assert_ne!(msg1, msg4);
    }

    #[test]
    fn test_dismiss_toast_message() {
        let dismiss1 = RendererMessage::DismissToast;
        let dismiss2 = RendererMessage::DismissToast;
        assert_eq!(dismiss1, dismiss2);
    }

    #[test]
    fn test_toast_timer_tick_message() {
        let tick1 = RendererMessage::ToastTimerTick;
        let tick2 = RendererMessage::ToastTimerTick;
        assert_eq!(tick1, tick2);
    }

    #[test]
    fn test_toast_severity_levels() {
        let info = RendererMessage::ShowToast("Info".to_string(), ToastSeverity::Info);
        let warning = RendererMessage::ShowToast("Warning".to_string(), ToastSeverity::Warning);
        let error = RendererMessage::ShowToast("Error".to_string(), ToastSeverity::Error);

        // All should be different
        assert_ne!(info, warning);
        assert_ne!(warning, error);
        assert_ne!(info, error);
    }
}
