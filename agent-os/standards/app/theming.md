# Theming

## Overview

This document defines theming patterns for COSMIC desktop applications. libcosmic uses `cosmic-theme` for consistent styling that automatically adapts to system preferences.

## Theme System

### Accessing the Theme

```rust
// Get active theme
let theme = cosmic::theme::active();

// Check theme type
let is_dark = cosmic::theme::is_dark();
let is_high_contrast = cosmic::theme::is_high_contrast();

// Get spacing values
let spacing = cosmic::theme::spacing();
```

### Theme Types

```rust
pub enum ThemeType {
    Dark,
    Light,
    HighContrastDark,
    HighContrastLight,
    Custom(Arc<CosmicTheme>),
    System {
        prefer_dark: Option<bool>,
        theme: Arc<CosmicTheme>,
    },
}
```

## Color System

### Semantic Colors

Access colors through the theme:

```rust
let theme = cosmic::theme::active();
let cosmic = theme.cosmic();

// Background colors
let bg_base = cosmic.background.base;
let bg_component = cosmic.background.component;

// Text colors
let text_primary = cosmic.text.primary;

// Accent color
let accent = cosmic.accent.base;

// Status colors
let success = cosmic.success.base;
let warning = cosmic.warning.base;
let destructive = cosmic.destructive.base;
```

### Color Components

Each color component has multiple variants:

```rust
// background.base - Primary background
// background.component - Component backgrounds
// background.divider - Divider lines

// text.primary - Primary text
// text.secondary - Secondary/muted text

// accent.base - Accent color
// accent.on - Text on accent background
```

## Spacing System

Use the standardized spacing values:

```rust
let spacing = cosmic::theme::spacing();

// Available spacing values
spacing.space_none    // 0
spacing.space_xxxs    // 2
spacing.space_xxs     // 4
spacing.space_xs      // 8
spacing.space_s       // 12
spacing.space_m       // 16
spacing.space_l       // 24
spacing.space_xl      // 32
spacing.space_xxl     // 48
spacing.space_xxxl    // 64
```

### Applying Spacing

```rust
cosmic::widget::column()
    .spacing(spacing.space_s)
    .padding(spacing.space_m)

cosmic::widget::container(content)
    .padding([
        spacing.space_s,  // top
        spacing.space_m,  // right
        spacing.space_s,  // bottom
        spacing.space_m,  // left
    ])
```

## Corner Radius

```rust
let cosmic = cosmic::theme::active().cosmic();

// Radius values
cosmic.radius_0()   // No radius (sharp corners)
cosmic.radius_xs()  // Extra small
cosmic.radius_s()   // Small
cosmic.radius_m()   // Medium
cosmic.radius_l()   // Large
cosmic.radius_xl()  // Extra large
```

## Widget Styling

### Container Classes

```rust
// Card style
cosmic::widget::container(content)
    .class(cosmic::theme::Container::Card)

// Window background
cosmic::widget::container(content)
    .class(cosmic::theme::Container::WindowBackground)

// Primary container
cosmic::widget::container(content)
    .class(cosmic::theme::Container::Primary)

// Custom styling
cosmic::widget::container(content)
    .class(cosmic::theme::Container::custom(|theme| {
        cosmic::widget::container::Style {
            background: Some(cosmic::iced::Background::Color(
                theme.cosmic().background.component.into(),
            )),
            border: cosmic::iced::Border {
                radius: theme.cosmic().radius_s().into(),
                width: 1.0,
                color: theme.cosmic().background.divider.into(),
            },
            ..Default::default()
        }
    }))
```

### Button Classes

```rust
// Default button
cosmic::widget::button::text("Click")

// Suggested (accent) button
cosmic::widget::button::suggested("Save")

// Destructive button
cosmic::widget::button::destructive("Delete")

// Standard button
cosmic::widget::button::standard("Cancel")

// Custom button class
cosmic::widget::button::text("Custom")
    .class(cosmic::theme::Button::custom(|theme, status| {
        // Return button::Style based on theme and status
    }))
```

### Text Classes

```rust
// Default text
cosmic::widget::text("Regular text")

// Accent text
cosmic::widget::text("Accent text")
    .class(cosmic::theme::Text::Accent)

// Muted text
cosmic::widget::text("Secondary text")
    .class(cosmic::theme::Text::Default)
```

## Dark/Light Mode

### Responding to Theme Changes

```rust
impl Application for App {
    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        // Theme changes are handled automatically by COSMIC
        // You can watch for config changes if you need to react
        self.core()
            .watch_config::<AppConfig>(Self::APP_ID)
            .map(Message::ConfigUpdated)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ConfigUpdated(config) => {
                // React to config/theme changes
                Task::none()
            }
            _ => Task::none(),
        }
    }
}
```

### Conditional Styling

```rust
fn view(&self) -> cosmic::Element<Message> {
    let is_dark = cosmic::theme::is_dark();

    let icon_name = if is_dark {
        "weather-clear-night-symbolic"
    } else {
        "weather-clear-symbolic"
    };

    cosmic::widget::icon::from_name(icon_name).into()
}
```

## Application Settings

### Initial Theme

```rust
let settings = cosmic::app::Settings::default()
    .theme(cosmic::theme::Theme::dark());  // Force dark
    // .theme(cosmic::theme::Theme::light()); // Force light
    // .theme(cosmic::theme::Theme::system()); // Follow system

cosmic::app::run::<App>(settings, ())?;
```

## High Contrast Mode

```rust
fn view(&self) -> cosmic::Element<Message> {
    let is_high_contrast = cosmic::theme::is_high_contrast();

    // Adjust UI for high contrast if needed
    let border_width = if is_high_contrast { 2.0 } else { 1.0 };

    cosmic::widget::container(content)
        .class(cosmic::theme::Container::custom(|theme| {
            cosmic::widget::container::Style {
                border: cosmic::iced::Border {
                    width: border_width,
                    color: theme.cosmic().accent.base.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }))
        .into()
}
```

## Common Patterns

### Themed Card

```rust
fn view_card(&self, title: &str, content: impl Into<cosmic::Element<Message>>)
    -> cosmic::Element<Message>
{
    let spacing = cosmic::theme::spacing();

    cosmic::widget::container(
        cosmic::widget::column()
            .push(cosmic::widget::text::heading(title))
            .push(content)
            .spacing(spacing.space_s)
    )
    .padding(spacing.space_m)
    .class(cosmic::theme::Container::Card)
    .into()
}
```

### Themed Divider Row

```rust
fn view_divider_with_label(&self, label: &str) -> cosmic::Element<Message> {
    let spacing = cosmic::theme::spacing();

    cosmic::widget::row()
        .push(cosmic::widget::divider::horizontal::default())
        .push(
            cosmic::widget::text(label)
                .class(cosmic::theme::Text::Default)
        )
        .push(cosmic::widget::divider::horizontal::default())
        .spacing(spacing.space_s)
        .align_y(cosmic::iced::Alignment::Center)
        .into()
}
```

### Themed Status Badge

```rust
fn view_status_badge(&self, status: Status) -> cosmic::Element<Message> {
    let (color_name, text) = match status {
        Status::Success => ("success", "Success"),
        Status::Warning => ("warning", "Warning"),
        Status::Error => ("destructive", "Error"),
    };

    cosmic::widget::container(
        cosmic::widget::text(text)
    )
    .padding([
        cosmic::theme::spacing().space_xxs,
        cosmic::theme::spacing().space_xs,
    ])
    .class(cosmic::theme::Container::custom(move |theme| {
        let color = match color_name {
            "success" => theme.cosmic().success.base,
            "warning" => theme.cosmic().warning.base,
            "destructive" => theme.cosmic().destructive.base,
            _ => theme.cosmic().accent.base,
        };

        cosmic::widget::container::Style {
            background: Some(cosmic::iced::Background::Color(color.into())),
            border: cosmic::iced::Border {
                radius: theme.cosmic().radius_s().into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }))
    .into()
}
```

## Best Practices

### Do

```rust
// Use semantic spacing
.spacing(cosmic::theme::spacing().space_m)

// Use theme-aware container classes
.class(cosmic::theme::Container::Card)

// Check theme type for conditional logic
if cosmic::theme::is_dark() { ... }

// Use standard button variants
cosmic::widget::button::suggested("Save")
cosmic::widget::button::destructive("Delete")
```

### Don't

```rust
// DON'T: Hardcode colors
.style(cosmic::iced::Color::from_rgb(0.2, 0.2, 0.2))

// DON'T: Hardcode spacing
.spacing(16)
.padding(8)

// DON'T: Hardcode border radius
border_radius: 4.0
```

## Best Practices Checklist

- [ ] Use `cosmic::theme::spacing()` for all spacing values
- [ ] Use standard container classes (Card, Primary, etc.)
- [ ] Use standard button variants (suggested, destructive)
- [ ] Never hardcode colors - use theme colors
- [ ] Test in both light and dark modes
- [ ] Test in high contrast mode
- [ ] Use theme radius values for borders

## References

- [cosmic-theme Documentation](https://pop-os.github.io/libcosmic/cosmic_theme/)
- [COSMIC Design Guidelines](https://github.com/pop-os/cosmic-design)
