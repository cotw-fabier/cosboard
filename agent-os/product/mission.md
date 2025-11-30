# Product Mission

## Cosboard

**Tagline:** A powerful, customizable soft keyboard for the Cosmic desktop

## Mission Statement

Cosboard delivers the first native soft keyboard solution for the Cosmic desktop environment, empowering touch keyboard users with a fully customizable, JSON-driven layout system that prioritizes accessibility, extensibility, and ease of use.

## Vision

To become the definitive soft keyboard for Cosmic desktop, offering unmatched customization through declarative JSON layouts while expanding into speech-to-text, intelligent predictions, and advanced input methods that serve users of all abilities and technical backgrounds.

## Target Users

### Primary Customers

- **Touch Keyboard Users on Cosmic Desktop:** Users operating Cosmic on touch-enabled devices (tablets, 2-in-1 laptops, touchscreen monitors) who require a software keyboard for text input
- **Accessibility-Focused Users:** Individuals who rely on alternative input methods due to physical limitations or preferences, requiring customizable layouts and enhanced accessibility features
- **Power Users and Developers:** Technical users who want fine-grained control over their keyboard behavior, custom key actions, and scripting capabilities
- **Multi-Language Users:** Users who work across multiple languages and need easy layout switching and custom character support

### User Personas

**Touch Device User** (25-55)
- **Role:** Professional or student using a Cosmic-powered tablet or 2-in-1 device
- **Context:** Needs reliable text input for daily productivity tasks
- **Pain Points:** No native soft keyboard exists for Cosmic; third-party solutions lack integration
- **Goals:** Seamless, responsive typing experience that feels native to the desktop environment

**Accessibility User** (All ages)
- **Role:** User with motor impairments or other accessibility needs
- **Context:** Requires customized input methods that standard hardware keyboards cannot provide
- **Pain Points:** Generic keyboards lack customization; accessibility features are afterthoughts
- **Goals:** Fully customizable layout with adjustable key sizes, positions, and behaviors

**Developer/Power User** (20-45)
- **Role:** Software developer, system administrator, or technical enthusiast
- **Context:** Wants to extend keyboard functionality with custom actions and scripts
- **Pain Points:** Existing soft keyboards are closed systems with no extensibility
- **Goals:** Programmable keyboard with scripting support and custom key behaviors

## Value Proposition

Cosboard is the only soft keyboard purpose-built for Cosmic desktop that combines:

1. **Native Integration:** Built with libcosmic/Iced for seamless desktop integration
2. **Declarative Customization:** JSON-based layout definitions enable users to create, share, and modify keyboard layouts without coding
3. **Extensibility:** From simple key remapping to full scripting with Rhai, users control every aspect of their input experience
4. **Future-Ready Architecture:** Designed from the ground up to support advanced features including speech-to-text, word prediction, and touch input emulation

## Core Problems Solved

### No Native Soft Keyboard for Cosmic

libcosmic currently lacks a soft keyboard implementation, leaving touch device users without a native input method. Cosboard fills this critical gap with a first-class, native solution.

**Our Solution:** A purpose-built soft keyboard applet that integrates directly with the Cosmic desktop environment.

### Limited Customization in Existing Solutions

Traditional soft keyboards offer minimal layout customization, forcing users to adapt to the keyboard rather than the keyboard adapting to them.

**Our Solution:** A JSON-based layout system that allows complete control over keyboard appearance, key arrangement, and behavior without requiring programming knowledge.

### Lack of Accessibility Options

Many soft keyboards treat accessibility as an afterthought, offering few options for users with specific input needs.

**Our Solution:** A flexible architecture that supports custom layouts, adjustable key sizes, and alternative input methods including planned speech-to-text integration.

### No Advanced Input Methods

Power users lack the ability to extend keyboard functionality beyond basic character input.

**Our Solution:** Progressive feature rollout including custom key actions (press, long-press, swipe), Rhai scripting engine, word prediction, and speech-to-text capabilities.

### Touch Input Limitations in Desktop Applications

Many desktop applications do not support touch input natively, limiting usability on touchscreen devices.

**Our Solution:** Planned mouse/touchpad emulation features that translate touch gestures into mouse events, enabling touch control in applications that lack native touch support.
