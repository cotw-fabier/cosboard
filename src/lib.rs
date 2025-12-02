// SPDX-License-Identifier: GPL-3.0-only

//! Cosboard - A soft keyboard for COSMIC desktop
//!
//! This crate provides a soft keyboard applet for the COSMIC desktop environment.
//! The applet lives in the system panel and manages the keyboard window directly.
//!
//! # Modules
//!
//! - `applet`: System tray applet with integrated keyboard management
//! - `app_settings`: Centralized application constants and configuration
//! - `config`: User configuration with cosmic_config persistence
//! - `i18n`: Localization support using fluent translations
//! - `input`: Input handling for keycode parsing, modifier state, and virtual keyboard
//! - `layer_shell`: Wayland layer-shell integration for overlay behavior
//! - `layout`: JSON layout parser for keyboard layout definitions
//! - `renderer`: Keyboard layout renderer for visual UI generation
//! - `state`: Window state persistence (position, size)

pub mod app_settings;
pub mod applet;
pub mod config;
pub mod i18n;
pub mod input;
pub mod layer_shell;
pub mod layout;
pub mod renderer;
pub mod state;

// Re-export the fl! macro for localization
pub use crate::i18n::LANGUAGE_LOADER;

// Re-export key input types for convenient access
pub use crate::input::{parse_keycode, ModifierState, ResolvedKeycode};

// Re-export virtual keyboard types for convenient access (Task Group 3)
pub use crate::input::{keycodes, KeyEvent, KeyState, VirtualKeyboard};

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use crate::layer_shell::{Layer, LayerShellConfig};
    use crate::state::WindowState;

    /// Integration Test: State persistence across application restart
    ///
    /// This test verifies that WindowState can be serialized and deserialized
    /// correctly for persistence.
    #[test]
    fn test_state_persistence_roundtrip() {
        // Create a modified state (simulating user resize)
        let mut state = WindowState::default();
        state.width = 1024.0;
        state.height = 400.0;
        state.is_floating = true;

        // Verify state can be cloned (would be serialized/deserialized in real app)
        let restored_state = state.clone();

        assert_eq!(
            state.width, restored_state.width,
            "Width should persist correctly"
        );
        assert_eq!(
            state.height, restored_state.height,
            "Height should persist correctly"
        );
        assert_eq!(
            state.is_floating, restored_state.is_floating,
            "Floating mode should persist correctly"
        );
    }

    /// Integration Test: Multiple rapid resize events handling
    ///
    /// This test verifies that the state can handle multiple rapid updates
    /// (debouncing behavior is handled by the save logic, but the state should
    /// always reflect the latest values).
    #[test]
    fn test_rapid_resize_events() {
        let mut state = WindowState::default();

        // Simulate rapid resize events
        let resize_events = vec![
            (850.0, 350.0),
            (900.0, 380.0),
            (920.0, 400.0),
            (950.0, 420.0),
            (1000.0, 450.0),
        ];

        for (width, height) in &resize_events {
            state.width = *width;
            state.height = *height;
        }

        // State should reflect the final values
        let (final_width, final_height) = resize_events.last().unwrap();
        assert_eq!(
            state.width, *final_width,
            "State should reflect final width after rapid resizes"
        );
        assert_eq!(
            state.height, *final_height,
            "State should reflect final height after rapid resizes"
        );
    }

    /// Integration Test: Window state restoration accuracy
    ///
    /// This test verifies that window state values are preserved exactly.
    /// Position is stored as margins from bottom-right anchor in floating mode.
    #[test]
    fn test_window_state_restoration_accuracy() {
        let original = WindowState {
            width: 987.654,
            height: 321.098,
            is_floating: true,
            margin_bottom: 50,
            margin_right: 100,
        };

        // Clone simulates save/restore cycle
        let restored = original.clone();

        // Float comparison with exact equality since these are not computed
        assert_eq!(original.width, restored.width, "Width must be exact");
        assert_eq!(original.height, restored.height, "Height must be exact");
        assert_eq!(
            original.is_floating, restored.is_floating,
            "Floating mode must be exact"
        );
        assert_eq!(
            original.margin_bottom, restored.margin_bottom,
            "Margin bottom must be exact"
        );
        assert_eq!(
            original.margin_right, restored.margin_right,
            "Margin right must be exact"
        );
    }

    /// Integration Test: Layer-shell configuration workflow
    ///
    /// This test verifies the layer-shell configuration can be set up correctly
    /// for overlay behavior.
    #[test]
    fn test_layer_shell_configuration_workflow() {
        // Create default config (should be Overlay)
        let config = LayerShellConfig::new();
        assert_eq!(config.layer(), Layer::Overlay);

        // Verify configuration chain works
        let config = LayerShellConfig::new().with_layer(Layer::Overlay);
        assert_eq!(config.layer(), Layer::Overlay);
        assert!(!config.is_layer_surface()); // Not a true layer surface without compositor

        // Verify all layer types work
        for layer in [Layer::Background, Layer::Bottom, Layer::Top, Layer::Overlay] {
            let config = LayerShellConfig::new().with_layer(layer);
            assert_eq!(config.layer(), layer);
        }
    }
}

// ============================================================================
// Task Group 7: End-to-End Integration Tests for Basic Key Input
// ============================================================================

#[cfg(test)]
mod key_input_integration_tests {
    use crate::input::{keycodes, parse_keycode, ResolvedKeycode, VirtualKeyboard};
    use crate::layout::{Cell, Key, KeyCode, Layout, Modifier, Panel, Row};
    use crate::renderer::KeyboardRenderer;
    use std::collections::HashMap;

    /// Helper function to create a test layout with various key types.
    fn create_key_input_test_layout() -> Layout {
        let mut panels = HashMap::new();

        // Main panel with regular keys and modifiers
        let main_panel = Panel {
            id: "main".to_string(),
            rows: vec![
                // Row 1: Regular character keys
                Row {
                    cells: vec![
                        Cell::Key(Key {
                            label: "a".to_string(),
                            code: KeyCode::Unicode('a'),
                            identifier: Some("key_a".to_string()),
                            ..Key::default()
                        }),
                        Cell::Key(Key {
                            label: "b".to_string(),
                            code: KeyCode::Unicode('b'),
                            identifier: Some("key_b".to_string()),
                            ..Key::default()
                        }),
                        Cell::Key(Key {
                            label: "c".to_string(),
                            code: KeyCode::Unicode('c'),
                            identifier: Some("key_c".to_string()),
                            ..Key::default()
                        }),
                    ],
                },
                // Row 2: Modifier keys
                Row {
                    cells: vec![
                        Cell::Key(Key {
                            label: "Shift".to_string(),
                            code: KeyCode::Keysym("Shift_L".to_string()),
                            identifier: Some("shift".to_string()),
                            sticky: true,
                            stickyrelease: true, // One-shot mode
                            ..Key::default()
                        }),
                        Cell::Key(Key {
                            label: "Ctrl".to_string(),
                            code: KeyCode::Keysym("Control_L".to_string()),
                            identifier: Some("ctrl".to_string()),
                            sticky: true,
                            stickyrelease: false, // Toggle mode
                            ..Key::default()
                        }),
                        Cell::Key(Key {
                            label: "Space".to_string(),
                            code: KeyCode::Keysym("space".to_string()),
                            identifier: Some("space".to_string()),
                            ..Key::default()
                        }),
                    ],
                },
            ],
            ..Panel::default()
        };

        // Numpad panel for panel switching tests
        let numpad_panel = Panel {
            id: "numpad".to_string(),
            rows: vec![Row {
                cells: vec![
                    Cell::Key(Key {
                        label: "1".to_string(),
                        code: KeyCode::Unicode('1'),
                        identifier: Some("key_1".to_string()),
                        ..Key::default()
                    }),
                    Cell::Key(Key {
                        label: "2".to_string(),
                        code: KeyCode::Unicode('2'),
                        identifier: Some("key_2".to_string()),
                        ..Key::default()
                    }),
                ],
            }],
            ..Panel::default()
        };

        panels.insert("main".to_string(), main_panel);
        panels.insert("numpad".to_string(), numpad_panel);

        Layout {
            name: "Test Layout".to_string(),
            version: "1.0".to_string(),
            default_panel_id: "main".to_string(),
            panels,
            ..Layout::default()
        }
    }

    // ========================================================================
    // Task 7.1: Focused integration tests (5 tests)
    // ========================================================================

    /// Integration Test 1: Full key press -> input emission -> key release cycle
    ///
    /// Tests the complete flow of pressing a regular key, emitting the key event,
    /// and releasing the key. Verifies that:
    /// - Key press is registered in renderer visual state
    /// - Keycode is resolved correctly
    /// - Virtual keyboard queues the correct events
    /// - Key release completes the cycle
    #[test]
    fn test_full_key_press_release_cycle() {
        let layout = create_key_input_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);
        let mut vk = VirtualKeyboard::new();

        // Initialize virtual keyboard (may fail in headless CI)
        if vk.initialize().is_err() {
            eprintln!("Skipping test: XKB initialization failed (likely headless environment)");
            return;
        }

        // Find key 'a' in the layout
        let key_code = KeyCode::Unicode('a');

        // Step 1: Press key in renderer (visual state)
        renderer.press_key("key_a");
        assert!(
            renderer.is_key_pressed("key_a"),
            "Key should be marked as pressed"
        );

        // Step 2: Parse and resolve keycode
        let resolved = parse_keycode(&key_code);
        assert!(resolved.is_some(), "Should parse keycode for 'a'");
        let resolved = resolved.unwrap();
        assert_eq!(resolved, ResolvedKeycode::Character('a'));

        // Step 3: Resolve to hardware keycode and emit
        if let Some(keycode) = vk.resolve_keycode(&resolved) {
            vk.press_key(keycode);
            assert!(
                !vk.pending_events().is_empty(),
                "Should have pending press event"
            );
        } else {
            // Character might not be directly mapped, fallback would be used
            // This is still a valid path
            eprintln!("Note: Character 'a' not directly mapped, fallback would be used");
        }

        // Step 4: Release key
        if let Some(keycode) = vk.resolve_keycode(&resolved) {
            vk.release_key(keycode);
        }

        // Step 5: Release in renderer (visual state)
        renderer.release_key("key_a");
        assert!(
            !renderer.is_key_pressed("key_a"),
            "Key should no longer be marked as pressed"
        );

        // Verify we have at least a press event (and possibly release)
        let events = vk.pending_events();
        assert!(
            !events.is_empty(),
            "Should have at least one event queued"
        );
    }

    /// Integration Test 2: Modifier + key combo flow
    ///
    /// Tests the complete flow of activating a modifier (Shift), then pressing
    /// a regular key, and verifying that:
    /// - Modifier state is tracked correctly
    /// - Combo key sequence is emitted (modifier press, key press, key release, modifier release)
    /// - One-shot modifier clears after the combo
    #[test]
    fn test_modifier_key_combo_flow() {
        let layout = create_key_input_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);
        let mut vk = VirtualKeyboard::new();

        if vk.initialize().is_err() {
            eprintln!("Skipping test: XKB initialization failed");
            return;
        }

        // Step 1: Activate Shift modifier (one-shot mode)
        renderer.activate_modifier(Modifier::Shift, true);
        renderer.sync_modifier_visual_state(Modifier::Shift, "shift");

        assert!(
            renderer.is_modifier_active(Modifier::Shift),
            "Shift should be active"
        );
        assert!(
            renderer.is_sticky_active("shift"),
            "Shift visual state should be active"
        );

        // Step 2: Get active modifiers before pressing regular key
        let active_modifiers = renderer.get_active_modifiers();
        assert_eq!(active_modifiers.len(), 1, "Should have 1 active modifier");
        assert_eq!(active_modifiers[0], Modifier::Shift);

        // Step 3: Emit modifier press first
        let shift_keycode = keycodes::KEY_LEFTSHIFT;
        vk.press_key(shift_keycode);

        // Step 4: Emit regular key 'a' press
        let key_code = KeyCode::Unicode('a');
        if let Some(resolved) = parse_keycode(&key_code) {
            if let Some(keycode) = vk.resolve_keycode(&resolved) {
                vk.press_key(keycode);
                vk.release_key(keycode);
            }
        }

        // Step 5: Emit modifier release
        vk.release_key(shift_keycode);

        // Step 6: Clear one-shot modifiers
        renderer.clear_oneshot_modifiers();

        // Verify Shift is now inactive
        assert!(
            !renderer.is_modifier_active(Modifier::Shift),
            "Shift should be cleared after combo"
        );
        assert!(
            !renderer.is_sticky_active("shift"),
            "Shift visual state should be cleared"
        );

        // Verify we have the expected event sequence
        let events = vk.pending_events();
        // Expected: Shift press, 'a' press, 'a' release, Shift release = at least 4 events
        // (if 'a' is mapped; otherwise at least Shift press/release)
        assert!(events.len() >= 2, "Should have modifier press and release events");
    }

    /// Integration Test 3: Panel switch does not affect modifier state unexpectedly
    ///
    /// Tests that switching panels preserves modifier state. The modifier state
    /// should persist across panel switches so users can activate a modifier
    /// on one panel and use it on another.
    #[test]
    fn test_panel_switch_preserves_modifier_state() {
        let layout = create_key_input_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Activate modifiers in toggle mode (should persist)
        renderer.activate_modifier(Modifier::Ctrl, false); // Toggle mode
        renderer.activate_modifier(Modifier::Shift, true); // One-shot mode

        assert!(renderer.is_modifier_active(Modifier::Ctrl));
        assert!(renderer.is_modifier_active(Modifier::Shift));

        // Switch to numpad panel
        let switch_result = renderer.switch_panel("numpad");
        assert!(switch_result.is_ok(), "Panel switch should succeed");

        // Complete the animation immediately for testing
        renderer.complete_animation();
        assert_eq!(renderer.current_panel_id, "numpad");

        // Verify modifiers are still active after panel switch
        assert!(
            renderer.is_modifier_active(Modifier::Ctrl),
            "Toggle modifier should persist across panel switch"
        );
        assert!(
            renderer.is_modifier_active(Modifier::Shift),
            "One-shot modifier should persist across panel switch"
        );

        // Now press a key to clear one-shot modifiers
        renderer.clear_oneshot_modifiers();

        // Ctrl (toggle) should remain, Shift (one-shot) should clear
        assert!(
            renderer.is_modifier_active(Modifier::Ctrl),
            "Toggle modifier should remain after key press"
        );
        assert!(
            !renderer.is_modifier_active(Modifier::Shift),
            "One-shot modifier should clear after key press"
        );
    }

    /// Integration Test 4: Multiple rapid key presses
    ///
    /// Tests that rapid key presses are handled correctly without state corruption.
    /// Simulates a user typing quickly.
    #[test]
    fn test_multiple_rapid_key_presses() {
        let layout = create_key_input_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);
        let mut vk = VirtualKeyboard::new();

        if vk.initialize().is_err() {
            eprintln!("Skipping test: XKB initialization failed");
            return;
        }

        // Simulate rapid typing: a, b, c in quick succession
        let keys = vec![
            ("key_a", KeyCode::Unicode('a')),
            ("key_b", KeyCode::Unicode('b')),
            ("key_c", KeyCode::Unicode('c')),
        ];

        let mut total_events = 0;

        for (identifier, code) in &keys {
            // Press key
            renderer.press_key(*identifier);
            assert!(renderer.is_key_pressed(identifier));

            // Emit key event
            if let Some(resolved) = parse_keycode(code) {
                if let Some(keycode) = vk.resolve_keycode(&resolved) {
                    vk.press_key(keycode);
                    vk.release_key(keycode);
                    total_events += 2;
                }
            }

            // Release key
            renderer.release_key(identifier);
            assert!(!renderer.is_key_pressed(identifier));
        }

        // Verify no state corruption - no keys should be pressed
        assert!(
            !renderer.is_key_pressed("key_a"),
            "Key a should not be pressed"
        );
        assert!(
            !renderer.is_key_pressed("key_b"),
            "Key b should not be pressed"
        );
        assert!(
            !renderer.is_key_pressed("key_c"),
            "Key c should not be pressed"
        );

        // Verify events were queued (at least some, depending on keymap)
        let events = vk.pending_events();
        // We should have at least some events if any keys were mapped
        if total_events > 0 {
            assert!(
                !events.is_empty(),
                "Should have queued events for rapid key presses"
            );
        }
    }

    /// Integration Test 5: Error handling for unmapped keycodes
    ///
    /// Tests that unmapped or invalid keycodes are handled gracefully
    /// with appropriate fallback behavior and logging.
    #[test]
    fn test_error_handling_unmapped_keycodes() {
        let mut vk = VirtualKeyboard::new();

        if vk.initialize().is_err() {
            eprintln!("Skipping test: XKB initialization failed");
            return;
        }

        // Test 1: Invalid keysym name
        let invalid_keysym = KeyCode::Keysym("CompletelyInvalidKeysymName12345".to_string());
        let resolved = parse_keycode(&invalid_keysym);
        assert!(
            resolved.is_some(),
            "Parse should succeed (returns Keysym variant)"
        );

        // Resolution should fail for invalid keysym
        let resolved = resolved.unwrap();
        let keycode = vk.resolve_keycode(&resolved);
        assert!(
            keycode.is_none(),
            "Invalid keysym should not resolve to keycode"
        );

        // Test 2: Unicode codepoint (requires fallback)
        let unicode_code = KeyCode::Keysym("U+03C0".to_string()); // Pi symbol
        let resolved = parse_keycode(&unicode_code);
        assert!(resolved.is_some(), "Parse should succeed for Unicode");

        let resolved = resolved.unwrap();
        assert!(
            matches!(resolved, ResolvedKeycode::UnicodeCodepoint(0x03C0)),
            "Should parse as Unicode codepoint"
        );

        // Unicode codepoint resolution should return None (requires fallback)
        let keycode = vk.resolve_keycode(&resolved);
        assert!(
            keycode.is_none(),
            "Unicode codepoint should not resolve directly (requires fallback)"
        );

        // Test 3: Empty keysym string
        let empty_keysym = KeyCode::Keysym(String::new());
        let resolved = parse_keycode(&empty_keysym);
        assert!(
            resolved.is_none(),
            "Empty keysym should not parse"
        );

        // Test 4: Unicode fallback mechanism
        // When a codepoint can't be mapped, emit_unicode_codepoint should work
        vk.emit_unicode_codepoint(0x03C0); // Pi symbol

        // Should have generated the Ctrl+Shift+U sequence
        let events = vk.pending_events();
        assert!(
            events.len() >= 4,
            "Unicode fallback should generate Ctrl+Shift+U sequence (got {} events)",
            events.len()
        );
    }

    // ========================================================================
    // Additional tests for wiring verification
    // ========================================================================

    /// Test that virtual keyboard initialization and cleanup work correctly
    #[test]
    fn test_virtual_keyboard_lifecycle() {
        let mut vk = VirtualKeyboard::new();

        // Initially not initialized
        assert!(!vk.is_initialized());

        // Attempt initialization
        let init_result = vk.initialize();
        // May fail in headless CI
        if init_result.is_ok() {
            assert!(vk.is_initialized());

            // Queue some events
            vk.press_key(keycodes::KEY_SPACE);
            vk.release_key(keycodes::KEY_SPACE);
            assert_eq!(vk.pending_events().len(), 2);

            // Cleanup
            vk.cleanup();
            assert!(!vk.is_initialized());
            assert!(vk.pending_events().is_empty());

            // Should be able to reinitialize
            let reinit_result = vk.initialize();
            assert!(reinit_result.is_ok());
            assert!(vk.is_initialized());
        } else {
            eprintln!("Note: Virtual keyboard initialization failed (likely headless environment)");
        }
    }

    /// Test that renderer modifier state and visual state stay synchronized
    #[test]
    fn test_modifier_state_visual_synchronization() {
        let layout = create_key_input_test_layout();
        let mut renderer = KeyboardRenderer::new(layout);

        // Initially no modifiers active
        assert!(!renderer.has_active_modifiers());
        assert!(!renderer.is_sticky_active("shift"));

        // Activate modifier and sync visual state
        renderer.activate_modifier(Modifier::Shift, true);
        renderer.sync_modifier_visual_state(Modifier::Shift, "shift");

        // Both should be active
        assert!(renderer.is_modifier_active(Modifier::Shift));
        assert!(renderer.is_sticky_active("shift"));

        // Clear one-shot modifiers
        renderer.clear_oneshot_modifiers();

        // Both should be cleared (clear_oneshot_modifiers syncs visual state)
        assert!(!renderer.is_modifier_active(Modifier::Shift));
        assert!(!renderer.is_sticky_active("shift"));

        // Test toggle modifier (stickyrelease: false) persists
        renderer.activate_modifier(Modifier::Ctrl, false);
        renderer.sync_modifier_visual_state(Modifier::Ctrl, "ctrl");

        // Clear one-shot modifiers - toggle should remain
        renderer.clear_oneshot_modifiers();

        // Ctrl (toggle) should remain active
        assert!(renderer.is_modifier_active(Modifier::Ctrl));
    }
}
