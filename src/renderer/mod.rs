// SPDX-License-Identifier: GPL-3.0-only

//! Keyboard Layout Renderer for Cosboard.
//!
//! This module transforms parsed JSON keyboard layouts into visual UI using
//! libcosmic/Iced widgets. It consumes `Layout` data structures from `src/layout/`
//! and renders them on the keyboard layer-shell surface.
//!
//! # Architecture
//!
//! The renderer is organized into several sub-modules:
//!
//! - **state**: Core renderer state including `KeyboardRenderer`, `PanelAnimation`,
//!   and `Toast` types for tracking pressed keys, panel transitions, and notifications.
//! - **sizing**: Size calculations for relative and pixel-based sizing with HDPI support.
//! - **theme**: COSMIC theme integration for consistent keyboard styling.
//! - **key**: Individual key rendering with label/icon detection.
//! - **row**: Horizontal row layout for keyboard cells.
//! - **panel**: Full panel rendering with rows, padding, and animation support.
//! - **message**: Renderer message types for interactions.
//! - **widget_placeholder**: Placeholder rendering for trackpad/autocomplete widgets.
//! - **panel_ref**: Panel reference button rendering for panel switching.
//! - **popup**: Long press popup rendering for swipe gesture alternatives.
//! - **toast**: Toast notification rendering for error messages and status updates.
//!
//! # Usage
//!
//! ```rust,ignore
//! use cosboard::layout::parse_layout_file;
//! use cosboard::renderer::{KeyboardRenderer, render_current_panel, render_animated_panels};
//!
//! // Load a keyboard layout
//! let result = parse_layout_file("resources/layouts/qwerty.json").unwrap();
//! let layout = result.layout;
//!
//! // Create the renderer
//! let mut renderer = KeyboardRenderer::new(layout);
//!
//! // Render the current panel (or animated transition if animating)
//! let element = render_animated_panels(&renderer, 800.0, 300.0, 1.0);
//!
//! // Query state
//! if let Some(panel) = renderer.current_panel() {
//!     println!("Current panel: {}", panel.id);
//! }
//!
//! // Track key presses
//! renderer.press_key("key_a");
//! assert!(renderer.is_key_pressed("key_a"));
//! renderer.release_key("key_a");
//!
//! // Switch panels (with animation)
//! if renderer.switch_panel("numpad").is_ok() {
//!     while renderer.is_animating() {
//!         renderer.update_animation();
//!         // Render frame using render_animated_panels()...
//!     }
//! }
//!
//! // Show toast notifications
//! use cosboard::renderer::ToastSeverity;
//! renderer.queue_toast("Panel not found", ToastSeverity::Error);
//! ```
//!
//! # Panel Animation
//!
//! Panel transitions are animated with a smooth slide effect:
//!
//! ```rust,ignore
//! use cosboard::renderer::{KeyboardRenderer, render_animated_panels, ANIMATION_FRAME_INTERVAL_MS};
//!
//! let mut renderer = KeyboardRenderer::new(layout);
//!
//! // Start a panel switch animation
//! renderer.switch_panel("numpad").unwrap();
//!
//! // In your render loop, use render_animated_panels() which handles:
//! // - Rendering both panels during transition
//! // - Applying horizontal offset transforms
//! // - Using eased progress for smooth visual effect
//! let element = render_animated_panels(&renderer, 800.0, 300.0, 1.0);
//!
//! // Update animation progress (call from AnimationTick subscription)
//! if renderer.update_animation() {
//!     // Animation completed - new panel is now current
//! }
//! ```
//!
//! # Toast Notifications
//!
//! Toast notifications are displayed at the bottom of the keyboard surface:
//!
//! ```rust,ignore
//! use cosboard::renderer::{KeyboardRenderer, ToastSeverity, TOAST_TIMER_INTERVAL_MS};
//! use cosboard::renderer::toast::{render_toast, render_keyboard_with_toast};
//!
//! let mut renderer = KeyboardRenderer::new(layout);
//!
//! // Queue a toast notification
//! renderer.queue_toast("Panel 'symbols' not found", ToastSeverity::Error);
//!
//! // Render keyboard with toast
//! let panel_element = render_animated_panels(&renderer, 800.0, 300.0, 1.0);
//! let toast_element = render_current_toast(&renderer, &theme);
//! let combined = render_keyboard_with_toast(panel_element, toast_element, 300.0);
//!
//! // Check for toast timeout in subscription handler (or use handle_toast_timer_tick)
//! if renderer.handle_toast_timer_tick() {
//!     // Toast was dismissed, next toast (if any) is now showing
//! }
//! ```
//!
//! # Sizing System
//!
//! The renderer uses a proportional sizing system based on a calculated base unit:
//!
//! ```rust,ignore
//! use cosboard::renderer::sizing::{calculate_base_unit, resolve_sizing};
//! use cosboard::layout::Sizing;
//!
//! // Calculate base unit from surface dimensions (width, height, max_row_width, total_height_units)
//! // Uses min of width-based and height-based unit to ensure everything fits
//! let base_unit = calculate_base_unit(800.0, 320.0, 10, 4.0);  // 80px (both constraints equal)
//!
//! // Resolve relative sizing
//! let width = resolve_sizing(&Sizing::Relative(1.5), base_unit, 1.0);  // 120px
//!
//! // Resolve pixel sizing with HDPI
//! let height = resolve_sizing(&Sizing::Pixels("20px".to_string()), base_unit, 2.0);  // 40px
//! ```
//!
//! # Theme Integration
//!
//! Colors are retrieved from the COSMIC theme system:
//!
//! ```rust,ignore
//! use cosboard::renderer::theme;
//! use cosmic::Theme;
//!
//! let theme = Theme::dark_default();
//! let key_bg = theme::key_background_color(&theme);
//! let key_pressed = theme::key_pressed_color(&theme);
//! ```
//!
//! # Visual Modifier State Indication
//!
//! Modifier keys (Shift, Ctrl, Alt, Super) show visual feedback when active:
//!
//! ```rust,ignore
//! use cosboard::renderer::{KeyboardRenderer, should_show_modifier_active};
//! use cosboard::layout::{Key, Modifier};
//!
//! let mut renderer = KeyboardRenderer::new(layout);
//!
//! // Activate a modifier and sync visual state
//! renderer.activate_modifier(Modifier::Shift, true); // One-shot mode
//! renderer.sync_modifier_visual_state(Modifier::Shift, "shift");
//!
//! // The key will now show sticky_active_color styling
//! let shift_key = /* get shift key from layout */;
//! assert!(should_show_modifier_active(&shift_key, &renderer, "shift"));
//! ```
//!
//! # Features
//!
//! - **Panel Management**: Track current panel, switch between panels with animations
//! - **Key State Tracking**: Monitor pressed and sticky key states for visual feedback
//! - **Modifier Visual Indication**: Active modifiers display with distinct styling
//! - **Long Press Detection**: Detect long presses (300ms) for popup alternatives
//! - **Animation Support**: Smooth panel slide transitions with 250ms duration and easing
//! - **Toast Notifications**: Queue-based notification system with auto-dismiss (3 seconds)
//! - **Proportional Sizing**: Base unit system for consistent key scaling
//! - **HDPI Support**: Pixel values are scaled for high-resolution displays
//! - **Theme Integration**: Colors adapt to the user's COSMIC theme

// Core modules (Task Groups 1-2)
pub mod sizing;
pub mod state;
pub mod theme;

// Rendering modules (Task Group 3)
pub mod key;
pub mod message;
pub mod panel;
pub mod panel_ref;
pub mod row;
pub mod widget_placeholder;

// Interactive modules (Task Group 4)
pub mod popup;

// Toast notification module (Task Group 6)
pub mod toast;

// Re-export public API from state
pub use state::{
    KeyboardRenderer, PanelAnimation, Toast, ToastSeverity, ANIMATION_DURATION_MS,
    ANIMATION_FRAME_INTERVAL_MS, LONG_PRESS_THRESHOLD_MS, LONG_PRESS_TIMER_INTERVAL_MS,
    TOAST_DURATION_MS, TOAST_TIMER_INTERVAL_MS,
};

// Re-export sizing functions for convenience
pub use sizing::{
    calculate_base_unit, calculate_total_height_units, get_scale_factor, parse_pixels,
    resolve_sizing,
};

// Re-export theme functions for convenience
pub use theme::{
    key_background_color, key_pressed_border_color, key_pressed_color, key_text_color,
    keyboard_background_color, sticky_active_color, toast_background_color, toast_text_color,
};

// Re-export message types
pub use message::RendererMessage;

// Re-export rendering functions
pub use key::{is_icon_name, key_identifier, render_key, render_label, should_show_modifier_active};
pub use panel::{render_animated_panels, render_current_panel, render_panel};
pub use panel_ref::render_panel_ref_button;
pub use row::{calculate_row_width, render_cell, render_row};
pub use widget_placeholder::render_widget_placeholder;

// Re-export popup functions and constants
pub use popup::{
    adjust_popup_position, calculate_popup_position, has_swipe_alternatives, render_popup,
    PopupPosition, Rectangle, POPUP_CELL_SIZE, POPUP_CELL_SPACING,
};

// Re-export toast functions and constants (Task Group 6)
pub use toast::{
    render_current_toast, render_keyboard_with_toast, render_toast, TOAST_HEIGHT,
};
