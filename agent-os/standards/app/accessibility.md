# Accessibility

## Overview

This document defines accessibility (a11y) patterns for COSMIC desktop applications. Building accessible applications ensures all users can effectively use your software, including those using assistive technologies.

## Core Principles

1. **Perceivable** - Users can perceive all content
2. **Operable** - Users can operate all controls
3. **Understandable** - Users can understand the interface
4. **Robust** - Works with assistive technologies

## Keyboard Navigation

### Focus Management

Ensure all interactive elements are keyboard accessible:

```rust
// Buttons are automatically focusable
cosmic::widget::button::text("Click Me")
    .on_press(Message::Clicked)

// Text inputs handle focus automatically
cosmic::widget::text_input("Enter text...", &self.value)
    .on_input(Message::InputChanged)
```

### Tab Order

Widgets receive focus in the order they appear in the view:

```rust
fn view(&self) -> cosmic::Element<Message> {
    cosmic::widget::column()
        // First in tab order
        .push(cosmic::widget::text_input("Name", &self.name)
            .on_input(Message::NameChanged))
        // Second in tab order
        .push(cosmic::widget::text_input("Email", &self.email)
            .on_input(Message::EmailChanged))
        // Third in tab order
        .push(cosmic::widget::button::suggested("Submit")
            .on_press(Message::Submit))
        .into()
}
```

### Keyboard Shortcuts

Implement common keyboard shortcuts:

```rust
// Handle keyboard events in subscriptions or update
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::KeyPressed(key) => {
            match key {
                Key::Named(Named::Escape) => {
                    // Close dialog or cancel action
                    self.close_dialog();
                }
                _ => {}
            }
            Task::none()
        }
        _ => Task::none(),
    }
}
```

## Screen Reader Support

### Descriptive Labels

Provide meaningful text for all interactive elements:

```rust
// GOOD: Descriptive button text
cosmic::widget::button::text("Save Document")
    .on_press(Message::Save)

// BAD: Unclear button text
cosmic::widget::button::text("OK")
    .on_press(Message::Save)
```

### Icon Buttons with Labels

Icon-only buttons should have tooltips for screen readers:

```rust
// Add tooltip for icon buttons
cosmic::widget::tooltip(
    cosmic::widget::button::icon(
        cosmic::widget::icon::from_name("edit-delete-symbolic")
    )
    .on_press(Message::Delete),
    cosmic::widget::text("Delete item"),
    cosmic::widget::tooltip::Position::Bottom,
)
```

### Meaningful Text Hierarchy

```rust
fn view(&self) -> cosmic::Element<Message> {
    cosmic::widget::column()
        // Heading for screen reader structure
        .push(cosmic::widget::text::heading("Settings"))
        // Section description
        .push(cosmic::widget::text("Configure your application preferences"))
        // Content
        .push(self.view_settings_content())
        .into()
}
```

## Visual Accessibility

### Color Contrast

Use theme colors which meet contrast requirements:

```rust
// GOOD: Use semantic theme colors
cosmic::widget::text("Important message")
    .class(cosmic::theme::Text::Accent)

// Theme colors are designed to meet WCAG contrast ratios
```

### Don't Rely on Color Alone

Convey information through multiple channels:

```rust
fn view_status(&self, status: Status) -> cosmic::Element<Message> {
    let (icon_name, status_text) = match status {
        Status::Success => ("emblem-ok-symbolic", "Success"),
        Status::Warning => ("dialog-warning-symbolic", "Warning"),
        Status::Error => ("dialog-error-symbolic", "Error"),
    };

    // Use both icon AND text, not just color
    cosmic::widget::row()
        .push(cosmic::widget::icon::from_name(icon_name))
        .push(cosmic::widget::text(status_text))
        .spacing(cosmic::theme::spacing().space_xs)
        .into()
}
```

### High Contrast Mode

Support high contrast themes:

```rust
fn view(&self) -> cosmic::Element<Message> {
    let is_high_contrast = cosmic::theme::is_high_contrast();

    // Optionally adjust for high contrast
    let border_style = if is_high_contrast {
        // Thicker borders in high contrast
        cosmic::iced::Border {
            width: 2.0,
            ..Default::default()
        }
    } else {
        cosmic::iced::Border::default()
    };

    // Theme handles most high contrast automatically
    cosmic::widget::container(content)
        .class(cosmic::theme::Container::Card)
        .into()
}
```

### Text Sizing

Support system font size preferences:

```rust
let settings = cosmic::app::Settings::default()
    .default_text_size(14.0);  // Base size, scales with system

// Don't use fixed pixel sizes for text
// GOOD: Relative sizing
cosmic::widget::text::heading("Title")  // Uses theme heading size

// BAD: Fixed size that won't scale
cosmic::widget::text("Text").size(12)  // Avoid when possible
```

## Form Accessibility

### Labels for Inputs

```rust
fn view_form_field(
    &self,
    label: &str,
    placeholder: &str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'static,
) -> cosmic::Element<Message> {
    cosmic::widget::column()
        // Label above input
        .push(cosmic::widget::text(label))
        .push(
            cosmic::widget::text_input(placeholder, value)
                .on_input(on_change)
        )
        .spacing(cosmic::theme::spacing().space_xxs)
        .into()
}
```

### Error Messages

```rust
fn view_form_field_with_error(
    &self,
    label: &str,
    value: &str,
    error: Option<&str>,
) -> cosmic::Element<Message> {
    let mut column = cosmic::widget::column()
        .push(cosmic::widget::text(label))
        .push(
            cosmic::widget::text_input("", value)
                .on_input(Message::InputChanged)
        );

    // Show error message if present
    if let Some(error_text) = error {
        column = column.push(
            cosmic::widget::text(error_text)
                .class(cosmic::theme::Text::Accent)  // Destructive color
                .size(12)
        );
    }

    column.spacing(cosmic::theme::spacing().space_xxs).into()
}
```

### Required Fields

```rust
fn view_required_field(&self, label: &str, value: &str) -> cosmic::Element<Message> {
    cosmic::widget::column()
        .push(
            cosmic::widget::row()
                .push(cosmic::widget::text(label))
                .push(cosmic::widget::text("*").class(cosmic::theme::Text::Accent))
        )
        .push(
            cosmic::widget::text_input("Required", value)
                .on_input(Message::InputChanged)
        )
        .spacing(cosmic::theme::spacing().space_xxs)
        .into()
}
```

## Loading States

Communicate loading states clearly:

```rust
fn view_loading(&self) -> cosmic::Element<Message> {
    cosmic::widget::container(
        cosmic::widget::column()
            .push(cosmic::widget::text("Loading..."))
            // Progress indicator for visual feedback
            .push(cosmic::widget::text("Please wait while data is loading"))
            .spacing(cosmic::theme::spacing().space_s)
            .align_x(cosmic::iced::Alignment::Center)
    )
    .center(cosmic::iced::Length::Fill)
    .into()
}
```

## Dialog Accessibility

### Modal Dialogs

```rust
// Ensure dialogs are clearly separated from main content
fn view_dialog(&self) -> cosmic::Element<Message> {
    cosmic::widget::container(
        cosmic::widget::column()
            // Clear dialog title
            .push(cosmic::widget::text::heading("Confirm Delete"))
            // Clear description
            .push(cosmic::widget::text(
                "Are you sure you want to delete this item? This action cannot be undone."
            ))
            // Clear action buttons
            .push(
                cosmic::widget::row()
                    .push(
                        cosmic::widget::button::standard("Cancel")
                            .on_press(Message::CancelDelete)
                    )
                    .push(
                        cosmic::widget::button::destructive("Delete")
                            .on_press(Message::ConfirmDelete)
                    )
                    .spacing(cosmic::theme::spacing().space_s)
            )
            .spacing(cosmic::theme::spacing().space_m)
    )
    .padding(cosmic::theme::spacing().space_l)
    .class(cosmic::theme::Container::Card)
    .into()
}
```

## Navigation Accessibility

### Clear Navigation Structure

```rust
fn init_nav_model(&mut self) {
    // Use clear, descriptive labels
    self.nav_model.insert()
        .text("Home")  // Clear label
        .icon(cosmic::widget::icon::from_name("go-home-symbolic"))
        .data(Page::Home);

    self.nav_model.insert()
        .text("Documents")
        .icon(cosmic::widget::icon::from_name("folder-documents-symbolic"))
        .data(Page::Documents);

    // Not just icons - always include text labels
}
```

### Breadcrumbs for Deep Navigation

```rust
fn view_breadcrumbs(&self) -> cosmic::Element<Message> {
    let mut row = cosmic::widget::row();

    for (i, crumb) in self.breadcrumbs.iter().enumerate() {
        if i > 0 {
            row = row.push(cosmic::widget::text(" > "));
        }

        if i == self.breadcrumbs.len() - 1 {
            // Current page (not a link)
            row = row.push(cosmic::widget::text(crumb));
        } else {
            // Previous pages (links)
            row = row.push(
                cosmic::widget::button::link(crumb)
                    .on_press(Message::NavigateTo(i))
            );
        }
    }

    row.into()
}
```

## Best Practices Checklist

**Keyboard:**
- [ ] All interactive elements are keyboard accessible
- [ ] Tab order follows logical reading order
- [ ] Focus is visible on all focusable elements
- [ ] Escape closes dialogs/popups

**Screen Readers:**
- [ ] All images/icons have text alternatives
- [ ] Icon buttons have tooltips
- [ ] Headings create logical structure
- [ ] Form fields have labels

**Visual:**
- [ ] Don't rely on color alone
- [ ] Support high contrast mode
- [ ] Text scales with system preferences
- [ ] Sufficient color contrast

**Forms:**
- [ ] All inputs have visible labels
- [ ] Required fields are indicated
- [ ] Error messages are clear and associated
- [ ] Success/error states are communicated

## Testing

### Manual Testing

1. Navigate entire app using only keyboard
2. Test with system high contrast mode enabled
3. Test with large font sizes
4. Verify all buttons have clear purpose

### Screen Reader Testing

Test with common screen readers:
- Linux: Orca
- macOS: VoiceOver
- Windows: NVDA, JAWS

## References

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [GNOME Accessibility Guidelines](https://developer.gnome.org/hig/guidelines/accessibility.html)
- [Iced Accessibility](https://docs.iced.rs/iced/)
