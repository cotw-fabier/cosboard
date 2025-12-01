# Product Roadmap

## Current Status (2025-12-01)

**Phase 1 Progress:** 3/7 items complete (43%)

**Recently Completed:**
- ✅ JSON Layout Parser with full spec implementation
  - Hierarchical layout structure (Layout → Panels → Rows → Keys)
  - Layout inheritance with 5-level depth support
  - Widget and embeddable panel support
  - Comprehensive validation with helpful error messages
  - 56 tests, all passing

**Immediate Next Steps:**
1. **Layout Renderer** - Transform parsed JSON layouts into visual keyboard UI using libcosmic/Iced widgets
2. **Basic Key Input** - Connect rendered keys to system input events
3. **Default Layout Bundle** - Create production-ready QWERTY layout

---

## Phase 1: MVP - Core Keyboard Applet

1. [x] Keyboard Applet Shell — Create the base Cosmic applet structure with proper window management, positioning, and desktop integration `M`
2. [x] JSON Layout Parser — Implement a JSON schema for defining keyboard layouts including key positions, sizes, labels, and basic key codes `M`
3. [ ] Layout Renderer — Render keyboard layouts from parsed JSON definitions with proper key sizing and spacing `M` **← NEXT**
4. [ ] Basic Key Input — Handle key press events and emit corresponding key codes to the system input layer `S`
5. [ ] Default Layout Bundle — Create standard QWERTY layout with shift states, numbers row, and common punctuation `S`
6. [ ] Layout Switching — Enable switching between multiple loaded layouts (e.g., letters, numbers, symbols) `S`
7. [x] Applet Toggle — Implement show/hide functionality with proper focus handling `S`

## Phase 2: Enhanced Key Actions

8. [ ] Long-Press Detection — Detect and handle long-press gestures on keys with configurable timing thresholds `S`
9. [ ] Long-Press Actions — Support alternate characters or actions triggered by long-press (e.g., accented characters) `S`
10. [ ] Swipe Gesture Detection — Detect swipe direction on keys (up, down, left, right) `M`
11. [ ] Swipe Actions — Map swipe gestures to configurable actions in the JSON layout schema `S`
12. [ ] Key Repeat — Implement key repeat behavior for held keys with configurable delay and rate `XS`
13. [ ] Haptic/Audio Feedback — Add optional feedback on key press (audio click, if system supports haptic) `XS`

## Phase 3: Scripting and Advanced Behavior

14. [ ] Rhai Engine Integration — Integrate the Rhai scripting engine for custom key action scripting `M`
15. [ ] Script-Bound Keys — Allow JSON layouts to bind keys to Rhai scripts for custom behavior `S`
16. [ ] Script API — Expose keyboard state, text manipulation, and system commands to Rhai scripts `M`
17. [ ] Floating Applet Mode — Implement draggable, resizable floating keyboard mode as alternative to docked `M`
18. [ ] Applet Size Presets — Support configurable size presets (compact, standard, large) `S`
19. [ ] Transparency and Theming — Support opacity settings and Cosmic theme integration `S`

## Phase 4: Prediction and Dictionary

20. [ ] Word Prediction Engine — Implement basic word prediction based on input context and frequency `L`
21. [ ] Prediction UI — Display word suggestions above the keyboard with tap-to-complete functionality `M`
22. [ ] Autocomplete Integration — Integrate prediction with typing flow for seamless completion `S`
23. [ ] User Dictionary — Allow users to add custom words to a personal dictionary `S`
24. [ ] Dictionary Sync — Support import/export of user dictionaries `XS`
25. [ ] Learning Mode — Optionally learn from user typing patterns to improve predictions `M`

## Phase 5: Speech-to-Text

26. [ ] ONNX Runtime Integration — Integrate ONNX runtime for model inference (CPU-based) `M`
27. [ ] Parakeet v3 Model Loading — Load and initialize Parakeet v3 STT model `M`
28. [ ] Audio Capture — Capture microphone audio input with proper permissions handling `S`
29. [ ] Real-Time Transcription — Process audio and generate text transcription in real-time `L`
30. [ ] STT UI Integration — Add microphone button to keyboard with visual feedback during recording `S`
31. [ ] STT Text Insertion — Insert transcribed text at cursor position with proper formatting `S`

## Phase 6: Touch Input Emulation

32. [ ] Virtual Mouse Mode — Implement a mode for controlling mouse cursor via touch gestures `M`
33. [ ] Gesture Mapping — Map touch gestures to mouse actions (tap=click, drag=move, two-finger=scroll) `M`
34. [ ] Touchpad Overlay — Provide a touchpad-like overlay for precise cursor control `M`
35. [ ] Per-App Profiles — Allow different input modes for specific applications `S`
36. [ ] Quick Mode Toggle — Easy switching between keyboard and mouse emulation modes `XS`

> Notes
> - Order items by technical dependencies and product architecture
> - Each item should represent an end-to-end functional and testable feature
> - MVP (Phase 1) focuses on core keyboard functionality with JSON layouts
> - Scripting architecture should be considered during MVP for proper extensibility
> - STT and mouse emulation are independent tracks that can proceed in parallel once core is stable
